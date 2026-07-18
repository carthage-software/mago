use mago_allocator::Arena;
use mago_allocator::collections::HashMap;
use mago_allocator::collections::HashSet;
use mago_allocator::vec::Vec;
use mago_hir::ir::item::member::MemberItem;
use mago_hir::ir::item::member::MemberItemKind;
use mago_hir::ir::item::member::trait_use::TraitUseAdaptation;
use mago_hir::ir::item::modifier::ModifierKind;

use crate::id::SymbolId;
use crate::id::SymbolIdBuildHasher;
use crate::linker::ClassLikeRef;
use crate::linker::descendants::for_each_parent;
use crate::linker::index::sorted_offsets;
use crate::linker::index::sorted_unique_ids;
use crate::path::Path;
use crate::symbol::Symbol;
use crate::symbol::SymbolTable;
use crate::symbol::class_like::ClassLikeSymbol;
use crate::symbol::class_like::part::constant::ClassLikeConstantMemberList;
use crate::symbol::class_like::part::inheritance::InheritedType;
use crate::symbol::class_like::part::inheritance::InheritedTypeList;
use crate::symbol::class_like::part::inheritance::Provenance;
use crate::symbol::class_like::part::method::MethodFlag;
use crate::symbol::class_like::part::method::MethodMember;
use crate::symbol::class_like::part::method::MethodMemberList;
use crate::symbol::class_like::part::method::MethodOverride;
use crate::symbol::class_like::part::property::PropertyMemberList;
use crate::symbol::class_like::part::property::PropertyOverride;
use crate::symbol::class_like::part::visibility::Visibility;

type Members<'arena, St, Ex> = [MemberItem<'arena, SymbolId, St, Ex>];

/// Flattens inherited members onto every class-like in `class_likes`, parents
/// first. After this each symbol's method/property/constant lists hold its own
/// members plus every non-private member reachable through its parents, each
/// carrying the `defining_symbol` of the ancestor that declares it, plus override
/// edges. Trait-use adaptations (`as` / `insteadof`) are read straight from the IR.
///
/// Class-likes already in the table but absent from `class_likes` - the symbols
/// seeded from a base table - are skipped: they are already flattened, and their
/// own members do not live in this slice's IR.
///
/// Every working buffer lives on `scratch`; only the flattened slices reach the
/// output arena.
pub(crate) fn resolve<'arena, S, A, St, Ex>(
    arena: &'arena A,
    scratch: &S,
    table: &mut SymbolTable<'arena, A>,
    class_likes: &HashMap<'_, SymbolId, ClassLikeRef<'_, 'arena, St, Ex>, S, SymbolIdBuildHasher>,
) where
    S: Arena,
    A: Arena,
    St: Copy,
    Ex: Copy,
{
    for id in topological_order(scratch, table) {
        if !class_likes.contains_key(&id) {
            continue;
        }

        let Some(symbol) = table.get_class_like(id) else {
            continue;
        };

        let members = class_likes.get(&id).map_or(&[][..], ClassLikeRef::members);
        let flattened = flatten(arena, scratch, table, symbol, members);
        table.class_likes.insert(id, flattened);
    }
}

/// Orders every class-like so a parent always precedes its children. Back-edges
/// (only reachable through a cycle the parser would have rejected) are dropped.
fn topological_order<'scratch, S, A>(scratch: &'scratch S, table: &SymbolTable<'_, A>) -> Vec<'scratch, SymbolId, S>
where
    S: Arena,
    A: Arena,
{
    let mut order = Vec::new_in(scratch);
    let mut visited = HashSet::with_hasher_in(SymbolIdBuildHasher, scratch);
    let mut in_progress = HashSet::with_hasher_in(SymbolIdBuildHasher, scratch);

    let mut roots = Vec::new_in(scratch);
    roots.extend(table.class_likes.keys().copied());
    for root in roots {
        visit(scratch, table, root, &mut visited, &mut in_progress, &mut order);
    }

    order
}

fn visit<'scratch, S, A>(
    scratch: &'scratch S,
    table: &SymbolTable<'_, A>,
    id: SymbolId,
    visited: &mut HashSet<'scratch, SymbolId, S, SymbolIdBuildHasher>,
    in_progress: &mut HashSet<'scratch, SymbolId, S, SymbolIdBuildHasher>,
    order: &mut Vec<'scratch, SymbolId, S>,
) where
    S: Arena,
    A: Arena,
{
    if visited.contains(&id) || !in_progress.insert(id) {
        return;
    }

    if let Some(symbol) = table.get_class_like(id) {
        let mut parents = Vec::new_in(scratch);
        for_each_parent(&symbol, |parent| parents.push(parent));
        for parent in parents {
            visit(scratch, table, parent, visited, in_progress, order);
        }
    }

    in_progress.remove(&id);
    if visited.insert(id) {
        order.push(id);
    }
}

/// Rebuilds `symbol` with its member lists flattened against its parents.
fn flatten<'arena, S, A, St, Ex>(
    arena: &'arena A,
    scratch: &S,
    table: &SymbolTable<'arena, A>,
    symbol: ClassLikeSymbol<'arena>,
    members: &Members<'arena, St, Ex>,
) -> ClassLikeSymbol<'arena>
where
    S: Arena,
    A: Arena,
{
    let mut parent_symbols = Vec::new_in(scratch);
    for_each_parent(&symbol, |parent| {
        if let Some(parent) = table.get_class_like(parent) {
            parent_symbols.push(parent);
        }
    });

    // A trait's `@require-extends` / `@require-implements` targets are guaranteed
    // to be present on `$this` at every use site, so their members are callable
    // from inside the trait. Fold them in like parents (for member resolution
    // only - this is not real inheritance, so it never affects the descendant
    // index, which is built from `for_each_parent`).
    if let ClassLikeSymbol::Trait(r#trait) = &symbol {
        for required in r#trait.require_extends.iter().chain(r#trait.require_implements.iter()) {
            if let Some(required) = table.get_class_like(required.target.id) {
                parent_symbols.push(required);
            }
        }
    }

    let class_name = symbol.path().segments.first().map_or(&[][..], |segment| segment.as_bytes());

    let methods = flatten_methods(arena, scratch, class_name, symbol.methods(), &parent_symbols, members);
    let constants = flatten_constants(arena, scratch, class_name, symbol.constants(), &parent_symbols);

    match symbol {
        ClassLikeSymbol::Class(class) => {
            let properties = flatten_properties(arena, scratch, class_name, class.properties, &parent_symbols);
            let mut updated = *class;
            updated.methods = ensure_default_constructor(arena, scratch, class_name, class.origin, class.span, methods);
            updated.properties = properties;
            updated.constants = constants;
            updated.implements = flatten_implemented_interfaces(arena, scratch, class.implements, &parent_symbols);
            ClassLikeSymbol::Class(arena.alloc(updated))
        }
        ClassLikeSymbol::Interface(interface) => {
            let properties = flatten_properties(arena, scratch, class_name, interface.properties, &parent_symbols);
            let mut updated = *interface;
            updated.methods = methods;
            updated.properties = properties;
            updated.constants = constants;
            updated.extends = flatten_extended_interfaces(arena, scratch, interface.extends, &parent_symbols);
            ClassLikeSymbol::Interface(arena.alloc(updated))
        }
        ClassLikeSymbol::Trait(r#trait) => {
            let properties = flatten_properties(arena, scratch, class_name, r#trait.properties, &parent_symbols);
            let mut updated = *r#trait;
            updated.methods = methods;
            updated.properties = properties;
            updated.constants = constants;
            ClassLikeSymbol::Trait(arena.alloc(updated))
        }
        ClassLikeSymbol::Enum(r#enum) => {
            let mut updated = *r#enum;
            updated.methods = methods;
            updated.constants = constants;
            updated.implements = flatten_implemented_interfaces(arena, scratch, r#enum.implements, &parent_symbols);
            ClassLikeSymbol::Enum(arena.alloc(updated))
        }
        ClassLikeSymbol::AnonymousClass(anonymous_class) => {
            let properties =
                flatten_properties(arena, scratch, class_name, anonymous_class.properties, &parent_symbols);
            let mut updated = *anonymous_class;
            updated.methods = ensure_default_constructor(
                arena,
                scratch,
                class_name,
                anonymous_class.origin,
                anonymous_class.span,
                methods,
            );
            updated.properties = properties;
            updated.constants = constants;
            updated.implements =
                flatten_implemented_interfaces(arena, scratch, anonymous_class.implements, &parent_symbols);
            ClassLikeSymbol::AnonymousClass(arena.alloc(updated))
        }
    }
}

/// Flattens a class-like's `implements` list to the full transitive set of
/// interfaces: the interfaces declared directly, those each declared interface
/// extends, and those a parent class already implements. Inherited edges carry
/// `Provenance::Inherited` recording the immediate parent they came through.
fn flatten_implemented_interfaces<'arena, S, A>(
    arena: &'arena A,
    scratch: &S,
    own: InheritedTypeList<'arena>,
    parents: &[ClassLikeSymbol<'arena>],
) -> InheritedTypeList<'arena>
where
    S: Arena,
    A: Arena,
{
    merge_inherited(arena, scratch, own, parents, |parent| match parent {
        ClassLikeSymbol::Interface(interface) => Some(interface.extends),
        ClassLikeSymbol::Class(class) => Some(class.implements),
        ClassLikeSymbol::Enum(r#enum) => Some(r#enum.implements),
        _ => None,
    })
}

/// Flattens an interface's `extends` list to the full transitive set of
/// extended interfaces, tagging inherited edges with their immediate parent.
fn flatten_extended_interfaces<'arena, S, A>(
    arena: &'arena A,
    scratch: &S,
    own: InheritedTypeList<'arena>,
    parents: &[ClassLikeSymbol<'arena>],
) -> InheritedTypeList<'arena>
where
    S: Arena,
    A: Arena,
{
    merge_inherited(arena, scratch, own, parents, |parent| match parent {
        ClassLikeSymbol::Interface(interface) => Some(interface.extends),
        _ => None,
    })
}

/// Merges `own` with edges drawn from each parent's `source` list, skipping
/// targets already present and tagging every added edge as inherited through
/// that parent. Source order is preserved so the rebuilt `target.id` index is
/// sound. Returns `own` unchanged when nothing new is inherited.
fn merge_inherited<'arena, S, A>(
    arena: &'arena A,
    scratch: &S,
    own: InheritedTypeList<'arena>,
    parents: &[ClassLikeSymbol<'arena>],
    source: impl Fn(&ClassLikeSymbol<'arena>) -> Option<InheritedTypeList<'arena>>,
) -> InheritedTypeList<'arena>
where
    S: Arena,
    A: Arena,
{
    let mut edges = Vec::new_in(scratch);
    edges.extend(own.edges.iter().copied());
    let mut seen = HashSet::with_hasher_in(SymbolIdBuildHasher, scratch);
    seen.extend(own.edges.iter().map(|edge| edge.target.id));

    for parent in parents {
        let Some(list) = source(parent) else {
            continue;
        };
        let via = parent.path();
        for edge in list.edges {
            if seen.insert(edge.target.id) {
                edges.push(InheritedType {
                    span: edge.span,
                    target: edge.target,
                    provenance: Provenance::Inherited { via },
                    arguments: edge.arguments,
                });
            }
        }
    }

    if edges.len() == own.edges.len() {
        return own;
    }

    let edges = arena.alloc_slice_fill_iter(edges);
    InheritedTypeList { index: sorted_offsets(arena, edges, |edge| edge.target.id), edges }
}

fn flatten_methods<'arena, S, A, St, Ex>(
    arena: &'arena A,
    scratch: &S,
    class_name: &'arena [u8],
    own: MethodMemberList<'arena>,
    parents: &[ClassLikeSymbol<'arena>],
    members: &Members<'arena, St, Ex>,
) -> MethodMemberList<'arena>
where
    S: Arena,
    A: Arena,
{
    let mut flattened = Vec::new_in(scratch);
    flattened.extend(own.members.iter().copied());
    let mut seen = HashSet::with_hasher_in(SymbolIdBuildHasher, scratch);
    seen.extend(own.members.iter().map(|member| member.name.id));

    let mut overrides = Vec::new_in(scratch);
    for (offset, member) in own.members.iter().enumerate() {
        let mut overridden = Vec::new_in(scratch);
        for parent in parents {
            for inherited in parent.methods().members {
                if SymbolId::method(class_name, last_segment(inherited.name)) == member.name.id {
                    overridden.push(inherited.defining_symbol);
                }
            }
        }

        if !overridden.is_empty() {
            overrides.push(MethodOverride { member: offset as u32, overrides: sorted_unique_ids(arena, overridden) });
        }
    }

    for parent in parents {
        let parent_id = class_id_of(parent);
        for inherited in parent.methods().members {
            // Inherited private methods are kept: `method_exists()` and reflection
            // report them on the child even though it cannot call them. Visibility
            // is preserved on the member for callability checks downstream.
            let short = last_segment(inherited.name);
            if method_excluded(members, parent_id, short) {
                continue;
            }

            let path = Path::method(arena, class_name, short);
            if seen.insert(path.id) {
                let mut member = *inherited;
                member.name = path;
                flattened.push(member);
            }
        }
    }

    apply_aliases(arena, class_name, parents, members, &mut flattened, &mut seen);

    let members = arena.alloc_slice_fill_iter(flattened);

    MethodMemberList {
        index: sorted_offsets(arena, members, |member| member.name.id),
        members,
        overrides: arena.alloc_slice_fill_iter(overrides),
    }
}

/// Whether `method` from `source_trait` is excluded from the using class by an
/// `A::method insteadof B` precedence rule. Read directly from the IR.
fn method_excluded<St, Ex>(members: &Members<'_, St, Ex>, source_trait: SymbolId, method: &[u8]) -> bool {
    members.iter().any(|member| {
        let MemberItemKind::TraitUse(trait_use) = &member.kind else {
            return false;
        };
        let Some(adaptations) = trait_use.adaptations else {
            return false;
        };

        adaptations.as_slice().iter().any(|adaptation| {
            let TraitUseAdaptation::Precedence(precedence) = adaptation else {
                return false;
            };

            precedence.method.value == method
                && precedence.instead_of.iter().any(|excluded| SymbolId::class_like(excluded.value) == source_trait)
        })
    })
}

/// Applies `foo as bar` / `foo as protected` trait-use aliases, read straight
/// from the IR: a renaming alias adds a copy of the source method under the new
/// name, while a visibility-only alias (alias name equals method name) rewrites
/// the inherited method's visibility in place.
fn apply_aliases<'arena, S, A, St, Ex>(
    arena: &'arena A,
    class_name: &'arena [u8],
    parents: &[ClassLikeSymbol<'arena>],
    members: &Members<'arena, St, Ex>,
    flattened: &mut Vec<'_, MethodMember<'arena>, S>,
    seen: &mut HashSet<'_, SymbolId, S, SymbolIdBuildHasher>,
) where
    S: Arena,
    A: Arena,
{
    for member in members {
        let MemberItemKind::TraitUse(trait_use) = &member.kind else {
            continue;
        };
        let Some(adaptations) = trait_use.adaptations else {
            continue;
        };

        for adaptation in adaptations.as_slice() {
            let TraitUseAdaptation::Alias(alias) = adaptation else {
                continue;
            };

            let source_trait = alias.r#trait.map(|r#trait| SymbolId::class_like(r#trait.value));
            let Some(source) = find_trait_method(parents, source_trait, alias.method.value) else {
                continue;
            };
            let visibility = alias.modifier.map(|modifier| modifier_visibility(modifier.kind));

            if alias.alias.value == alias.method.value {
                if let Some(visibility) = visibility {
                    let target = Path::method(arena, class_name, alias.method.value).id;
                    if let Some(member) = flattened.iter_mut().find(|member| member.name.id == target) {
                        member.visibility = visibility;
                    }
                }
                continue;
            }

            let path = Path::method(arena, class_name, alias.alias.value);
            if seen.insert(path.id) {
                let mut method = source;
                method.name = path;
                if let Some(visibility) = visibility {
                    method.visibility = visibility;
                }
                flattened.push(method);
            }
        }
    }
}

/// Finds a trait method by short name, restricted to `source_trait` when the
/// alias named one.
fn find_trait_method<'arena>(
    parents: &[ClassLikeSymbol<'arena>],
    source_trait: Option<SymbolId>,
    method: &[u8],
) -> Option<MethodMember<'arena>> {
    for parent in parents {
        if source_trait.is_some_and(|source| source != class_id_of(parent)) {
            continue;
        }
        for member in parent.methods().members {
            if last_segment(member.name) == method {
                return Some(*member);
            }
        }
    }

    None
}

fn flatten_properties<'arena, S, A>(
    arena: &'arena A,
    scratch: &S,
    class_name: &'arena [u8],
    own: PropertyMemberList<'arena>,
    parents: &[ClassLikeSymbol<'arena>],
) -> PropertyMemberList<'arena>
where
    S: Arena,
    A: Arena,
{
    let mut flattened = Vec::new_in(scratch);
    flattened.extend(own.members.iter().copied());
    let mut seen = HashSet::with_hasher_in(SymbolIdBuildHasher, scratch);
    seen.extend(own.members.iter().map(|member| member.name.id));

    let mut overrides = Vec::new_in(scratch);
    for (offset, member) in own.members.iter().enumerate() {
        let mut overridden = Vec::new_in(scratch);
        for parent in parents {
            if let Some(properties) = parent.properties() {
                for inherited in properties.members {
                    if SymbolId::property(class_name, last_segment(inherited.name)) == member.name.id {
                        overridden.push(inherited.defining_symbol);
                    }
                }
            }
        }

        if !overridden.is_empty() {
            overrides.push(PropertyOverride { member: offset as u32, overrides: sorted_unique_ids(arena, overridden) });
        }
    }

    for parent in parents {
        let Some(properties) = parent.properties() else {
            continue;
        };
        for inherited in properties.members {
            // Inherited private properties are kept, mirroring `property_exists()`.
            let path = Path::property(arena, class_name, last_segment(inherited.name));
            if seen.insert(path.id) {
                let mut member = *inherited;
                member.name = path;
                flattened.push(member);
            }
        }
    }

    let members = arena.alloc_slice_fill_iter(flattened);

    PropertyMemberList {
        index: sorted_offsets(arena, members, |member| member.name.id),
        members,
        overrides: arena.alloc_slice_fill_iter(overrides),
    }
}

fn flatten_constants<'arena, S, A>(
    arena: &'arena A,
    scratch: &S,
    class_name: &'arena [u8],
    own: ClassLikeConstantMemberList<'arena>,
    parents: &[ClassLikeSymbol<'arena>],
) -> ClassLikeConstantMemberList<'arena>
where
    S: Arena,
    A: Arena,
{
    let mut flattened = Vec::new_in(scratch);
    flattened.extend(own.members.iter().copied());
    let mut seen = HashSet::with_hasher_in(SymbolIdBuildHasher, scratch);
    seen.extend(own.members.iter().map(|member| member.name.id));

    for parent in parents {
        let from_trait = matches!(parent, ClassLikeSymbol::Trait(_));
        for inherited in parent.constants().members {
            if inherited.visibility == Visibility::Private && !from_trait {
                continue;
            }

            let path = Path::class_like_constant(arena, class_name, last_segment(inherited.name));
            if seen.insert(path.id) {
                let mut member = *inherited;
                member.name = path;
                flattened.push(member);
            }
        }
    }

    let members = arena.alloc_slice_fill_iter(flattened);

    ClassLikeConstantMemberList { index: sorted_offsets(arena, members, |member| member.name.id), members }
}

/// The visibility a trait-use modifier sets; defaults to public for the rare
/// modifier that is not a visibility keyword.
fn modifier_visibility(kind: ModifierKind) -> Visibility {
    match kind {
        ModifierKind::Private => Visibility::Private,
        ModifierKind::Protected => Visibility::Protected,
        _ => Visibility::Public,
    }
}

/// The class-like id of a resolved symbol, derived from its first path segment.
fn class_id_of(symbol: &ClassLikeSymbol<'_>) -> SymbolId {
    SymbolId::class_like(symbol.path().segments.first().map_or(&[][..], |segment| segment.as_bytes()))
}

/// The last path segment's bytes - a member's short name without its owner.
fn last_segment(path: Path<'_>) -> &[u8] {
    path.segments.last().map_or(&[][..], |segment| segment.as_bytes())
}

/// Ensures an instantiable class-like exposes a constructor: PHP gives every
/// class an implicit no-argument `__construct` when none is declared or
/// inherited, so `new C()` is always valid. Adds that synthetic constructor when
/// the flattened method set has none; otherwise returns the list unchanged.
fn ensure_default_constructor<'arena, S, A>(
    arena: &'arena A,
    scratch: &S,
    class_name: &'arena [u8],
    origin: crate::symbol::part::origin::Origin,
    span: mago_span::Span,
    methods: MethodMemberList<'arena>,
) -> MethodMemberList<'arena>
where
    S: Arena,
    A: Arena,
{
    if methods.members.iter().any(|member| last_segment(member.name).eq_ignore_ascii_case(b"__construct")) {
        return methods;
    }

    let constructor = MethodMember {
        span,
        visibility: Visibility::Public,
        name: Path::method(arena, class_name, b"__construct"),
        defining_symbol: SymbolId::class_like(class_name),
        flags: mago_flags::U32Flags::empty().with(MethodFlag::Constructor).with(MethodFlag::Magic),
        constraint: crate::symbol::part::constraint::SymbolConstraint::unconstrained(),
        attributes: &[],
        generics: &[],
        params: &[],
        ret: crate::symbol::part::ty::TypeSlot::new(),
        where_constraints: &[],
        throws: &[],
        assertions: &[],
        pure_unless_impure_params: &[],
        self_out: None,
        accessed_globals: &[],
        origin,
    };

    let mut members = Vec::new_in(scratch);
    members.extend(methods.members.iter().copied());
    members.push(constructor);
    let members = arena.alloc_slice_fill_iter(members);

    MethodMemberList {
        index: sorted_offsets(arena, members, |member| member.name.id),
        members,
        overrides: methods.overrides,
    }
}
