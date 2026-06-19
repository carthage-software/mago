use mago_allocator::Arena;
use mago_allocator::collections::HashMap;
use mago_allocator::vec::Vec;
use mago_hir::ir::item::annotation::ItemAnnotation;
use mago_hir::ir::r#type::annotation::TypeAnnotation;
use mago_hir::ir::r#type::annotation::TypeAnnotationKind;

use crate::id::SymbolId;
use crate::id::SymbolIdBuildHasher;
use crate::linker::ClassLikeRef;
use crate::linker::lower::Lowerer;
use crate::linker::types::reference_name;
use crate::symbol::SymbolTable;
use crate::symbol::class_like::ClassLikeSymbol;
use crate::symbol::class_like::part::inheritance::GenericArgument;
use crate::symbol::class_like::part::inheritance::InheritedType;
use crate::symbol::class_like::part::inheritance::InheritedTypeList;
use crate::symbol::part::generic::GenericParameter;
use crate::symbol::part::generic::GenericParameterForwarding;
use crate::symbol::part::generic::GenericParameterId;
use crate::ty::well_known;

/// The positional docblock arguments written for each annotated ancestor.
type ArgumentMap<'scratch, 'arena, S> =
    HashMap<'scratch, SymbolId, &'arena [TypeAnnotation<'arena>], S, SymbolIdBuildHasher>;

/// The per-class direct forwarding edges, before transitive closure.
type ForwardingMap<'scratch, 'arena, S> =
    HashMap<'scratch, SymbolId, Vec<'scratch, GenericParameterForwarding<'arena>, S>, S, SymbolIdBuildHasher>;

/// Resolves the docblock generic arguments written on each class-like's
/// inheritance edges (`@extends Foo<T>`, `@implements Bar<int>`, `@uses Baz<T>`)
/// and builds the transitively-closed generic-parameter forwarding relation.
///
/// Native PHP syntax carries no generic arguments on `extends`/`implements`/`use`,
/// so the arguments come exclusively from the docblock. Each positional argument
/// is paired with the ancestor's type-parameter name at that position; when an
/// argument is itself one of the inheriting class's own type parameters, the
/// pairing also records a forwarding edge into the ancestor's slot. All working
/// state lives on the builder's scratch arena.
pub(crate) fn resolve_inheritance<'scratch, 'arena, S, A, St, Ex>(
    lowerer: &mut Lowerer<'_, 'scratch, 'arena, S, A>,
    table: &mut SymbolTable<'arena, A>,
    class_likes: &HashMap<'_, SymbolId, ClassLikeRef<'_, 'arena, St, Ex>, S, SymbolIdBuildHasher>,
) where
    S: Arena,
    A: Arena,
    St: Copy,
    Ex: Copy,
{
    let scratch = lowerer.builder.scratch();
    let mut forwardings: ForwardingMap<'scratch, 'arena, S> = HashMap::with_hasher_in(SymbolIdBuildHasher, scratch);

    for (&id, reference) in class_likes {
        let Some(annotation) = reference.annotation() else {
            continue;
        };

        let arguments = collect_arguments(scratch, annotation);
        if arguments.is_empty() {
            continue;
        }

        let Some(symbol) = table.get_class_like(id) else {
            continue;
        };

        let mut own_names = Vec::new_in(scratch);
        own_names.extend(generics_of(&symbol).iter().map(|parameter| parameter.name));

        let mut direct = Vec::new_in(scratch);
        let rewritten = rewrite_symbol(lowerer, table, symbol, &arguments, &own_names, &mut direct);
        table.class_likes.insert(id, rewritten);

        if !direct.is_empty() {
            forwardings.entry(id).or_insert_with(|| Vec::new_in(scratch)).extend(direct);
        }
    }

    close_forwardings(scratch, &mut forwardings);

    for (id, edges) in forwardings {
        if edges.is_empty() {
            continue;
        }
        let Some(symbol) = table.get_class_like(id) else {
            continue;
        };
        let edges = lowerer.arena.alloc_slice_copy(&edges);
        if let Some(updated) = set_forwardings(lowerer.arena, symbol, edges) {
            table.class_likes.insert(id, updated);
        }
    }
}

/// Maps each annotated ancestor to the positional type arguments written for it.
fn collect_arguments<'scratch, 'arena, S, St, Ex>(
    scratch: &'scratch S,
    annotation: &ItemAnnotation<'arena, SymbolId, St, Ex>,
) -> ArgumentMap<'scratch, 'arena, S>
where
    S: Arena,
{
    let mut arguments = HashMap::with_hasher_in(SymbolIdBuildHasher, scratch);

    for extends in annotation.extends {
        let target = SymbolId::class_like(reference_name(&extends.r#type.kind));
        arguments.insert(target, extends.r#type.type_arguments.map_or(&[][..], |delimited| delimited.as_slice()));
    }
    for implements in annotation.implements {
        let target = SymbolId::class_like(reference_name(&implements.r#type.kind));
        arguments.insert(target, implements.r#type.type_arguments.map_or(&[][..], |delimited| delimited.as_slice()));
    }
    for uses in annotation.uses {
        let target = SymbolId::class_like(reference_name(&uses.r#type.kind));
        arguments.insert(target, uses.r#type.type_arguments.map_or(&[][..], |delimited| delimited.as_slice()));
    }

    arguments
}

/// Rewrites every inheritance edge of `symbol`, attaching resolved generic
/// arguments and collecting the forwarding edges it implies into `direct`.
fn rewrite_symbol<'scratch, 'arena, S, A>(
    lowerer: &mut Lowerer<'_, 'scratch, 'arena, S, A>,
    table: &SymbolTable<'arena, A>,
    symbol: ClassLikeSymbol<'arena>,
    arguments: &ArgumentMap<'scratch, 'arena, S>,
    own_names: &[&'arena [u8]],
    direct: &mut Vec<'scratch, GenericParameterForwarding<'arena>, S>,
) -> ClassLikeSymbol<'arena>
where
    S: Arena,
    A: Arena,
{
    let arena = lowerer.arena;
    match symbol {
        ClassLikeSymbol::Class(class) => {
            let mut updated = *class;
            updated.extends =
                class.extends.map(|edge| rewrite_edge(lowerer, table, edge, arguments, own_names, direct));
            updated.implements = rewrite_list(lowerer, table, class.implements, arguments, own_names, direct);
            updated.uses = rewrite_list(lowerer, table, class.uses, arguments, own_names, direct);
            ClassLikeSymbol::Class(arena.alloc(updated))
        }
        ClassLikeSymbol::Interface(interface) => {
            let mut updated = *interface;
            updated.extends = rewrite_list(lowerer, table, interface.extends, arguments, own_names, direct);
            ClassLikeSymbol::Interface(arena.alloc(updated))
        }
        ClassLikeSymbol::Trait(r#trait) => {
            let mut updated = *r#trait;
            updated.uses = rewrite_list(lowerer, table, r#trait.uses, arguments, own_names, direct);
            ClassLikeSymbol::Trait(arena.alloc(updated))
        }
        ClassLikeSymbol::Enum(r#enum) => {
            let mut updated = *r#enum;
            updated.implements = rewrite_list(lowerer, table, r#enum.implements, arguments, own_names, direct);
            updated.uses = rewrite_list(lowerer, table, r#enum.uses, arguments, own_names, direct);
            ClassLikeSymbol::Enum(arena.alloc(updated))
        }
        ClassLikeSymbol::AnonymousClass(anonymous_class) => {
            let mut updated = *anonymous_class;
            updated.extends =
                anonymous_class.extends.map(|edge| rewrite_edge(lowerer, table, edge, arguments, own_names, direct));
            updated.implements = rewrite_list(lowerer, table, anonymous_class.implements, arguments, own_names, direct);
            updated.uses = rewrite_list(lowerer, table, anonymous_class.uses, arguments, own_names, direct);
            ClassLikeSymbol::AnonymousClass(arena.alloc(updated))
        }
    }
}

/// Rewrites every edge of one list, preserving source order so the existing
/// `target.id`-sorted `index` stays valid.
fn rewrite_list<'scratch, 'arena, S, A>(
    lowerer: &mut Lowerer<'_, 'scratch, 'arena, S, A>,
    table: &SymbolTable<'arena, A>,
    list: InheritedTypeList<'arena>,
    arguments: &ArgumentMap<'scratch, 'arena, S>,
    own_names: &[&'arena [u8]],
    direct: &mut Vec<'scratch, GenericParameterForwarding<'arena>, S>,
) -> InheritedTypeList<'arena>
where
    S: Arena,
    A: Arena,
{
    let scratch = lowerer.builder.scratch();
    let mut edges = Vec::with_capacity_in(list.edges.len(), scratch);
    for edge in list.edges {
        edges.push(rewrite_edge(lowerer, table, *edge, arguments, own_names, direct));
    }

    InheritedTypeList { edges: lowerer.arena.alloc_slice_copy(&edges), index: list.index }
}

/// Attaches resolved generic arguments to a single edge and records the
/// forwardings any type-parameter argument implies.
fn rewrite_edge<'scratch, 'arena, S, A>(
    lowerer: &mut Lowerer<'_, 'scratch, 'arena, S, A>,
    table: &SymbolTable<'arena, A>,
    edge: InheritedType<'arena>,
    arguments: &ArgumentMap<'scratch, 'arena, S>,
    own_names: &[&'arena [u8]],
    direct: &mut Vec<'scratch, GenericParameterForwarding<'arena>, S>,
) -> InheritedType<'arena>
where
    S: Arena,
    A: Arena,
{
    let scratch = lowerer.builder.scratch();
    let Some(annotations) = arguments.get(&edge.target.id) else {
        return edge;
    };
    let Some(ancestor) = table.get_class_like(edge.target.id) else {
        return edge;
    };

    let mut ancestor_names = Vec::new_in(scratch);
    ancestor_names.extend(generics_of(&ancestor).iter().map(|parameter| parameter.name));

    let mut resolved = Vec::with_capacity_in(annotations.len(), scratch);
    for (position, annotation) in annotations.iter().enumerate() {
        let Some(&parameter) = ancestor_names.get(position) else {
            break;
        };

        let ty = lowerer.lower_type_annotation(annotation).unwrap_or(well_known::TYPE_MIXED);
        resolved.push(GenericArgument { parameter, ty });

        if let TypeAnnotationKind::GenericParameter(generic) = &annotation.kind {
            if own_names.contains(&generic.name.value) {
                direct.push(GenericParameterForwarding {
                    parameter: generic.name.value,
                    target: GenericParameterId { defining_entity: edge.target.id, name: parameter },
                });
            }
        }
    }

    InheritedType { arguments: lowerer.arena.alloc_slice_copy(&resolved), ..edge }
}

/// Expands the forwarding relation to its transitive closure: if `C` forwards
/// `TC` into `D::TD` and `D` forwards `TD` into `E::TE`, then `C` forwards `TC`
/// into `E::TE`.
fn close_forwardings<'scratch, 'arena, S>(scratch: &'scratch S, forwardings: &mut ForwardingMap<'scratch, 'arena, S>)
where
    S: Arena,
{
    loop {
        let mut additions = Vec::new_in(scratch);

        for (&owner, edges) in forwardings.iter() {
            for edge in edges {
                let Some(next_edges) = forwardings.get(&edge.target.defining_entity) else {
                    continue;
                };
                for next in next_edges {
                    if next.parameter != edge.target.name {
                        continue;
                    }
                    let candidate = GenericParameterForwarding { parameter: edge.parameter, target: next.target };
                    let known = edges.contains(&candidate)
                        || additions.iter().any(|(existing, edge)| *existing == owner && *edge == candidate);
                    if !known {
                        additions.push((owner, candidate));
                    }
                }
            }
        }

        if additions.is_empty() {
            break;
        }
        for (owner, edge) in additions {
            forwardings.entry(owner).or_insert_with(|| Vec::new_in(scratch)).push(edge);
        }
    }
}

/// Stores the resolved forwarding edges on the kinds that can carry them
/// (class, interface, trait); `None` for enums and anonymous classes.
fn set_forwardings<'arena, A>(
    arena: &'arena A,
    symbol: ClassLikeSymbol<'arena>,
    edges: &'arena [GenericParameterForwarding<'arena>],
) -> Option<ClassLikeSymbol<'arena>>
where
    A: Arena,
{
    match symbol {
        ClassLikeSymbol::Class(class) => {
            let mut updated = *class;
            updated.forwardings = edges;
            Some(ClassLikeSymbol::Class(arena.alloc(updated)))
        }
        ClassLikeSymbol::Interface(interface) => {
            let mut updated = *interface;
            updated.forwardings = edges;
            Some(ClassLikeSymbol::Interface(arena.alloc(updated)))
        }
        ClassLikeSymbol::Trait(r#trait) => {
            let mut updated = *r#trait;
            updated.forwardings = edges;
            Some(ClassLikeSymbol::Trait(arena.alloc(updated)))
        }
        ClassLikeSymbol::Enum(_) | ClassLikeSymbol::AnonymousClass(_) => None,
    }
}

/// The type parameters a class-like declares; empty for enums and anonymous
/// classes, which cannot be generic.
fn generics_of<'arena>(symbol: &ClassLikeSymbol<'arena>) -> &'arena [GenericParameter<'arena>] {
    match symbol {
        ClassLikeSymbol::Class(class) => class.generics,
        ClassLikeSymbol::Interface(interface) => interface.generics,
        ClassLikeSymbol::Trait(r#trait) => r#trait.generics,
        ClassLikeSymbol::Enum(_) | ClassLikeSymbol::AnonymousClass(_) => &[],
    }
}
