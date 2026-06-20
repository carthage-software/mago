mod class;
mod descendants;
mod index;
mod infer;
mod inheritance;
mod lower;
mod members;
mod resolve;
mod sealing;
mod tags;
mod types;

use mago_allocator::Arena;
use mago_allocator::collections::HashMap;

use mago_hir::ir::item::expression::anonymous_class::AnonymousClass;
use mago_hir::ir::item::statement::class::Class;
use mago_hir::ir::item::statement::r#enum::Enum;
use mago_hir::ir::item::statement::interface::Interface;
use mago_hir::ir::item::statement::r#trait::Trait;

use crate::definition::DefinitionTable;
use crate::id::SymbolId;
use crate::id::SymbolIdBuildHasher;
use crate::linker::lower::Lowerer;
use crate::symbol::Symbol;
use crate::symbol::SymbolTable;
use crate::symbol::function_like::FunctionLikeSymbol;
use crate::symbol::part::origin::Origin;
use crate::ty::builder::TypeBuilder;

/// A class-like declaration that won origin-priority resolution, paired with the
/// origin of the file it came from. The five class-like kinds share one id space
/// so inheritance lookups during resolution are a single map query.
pub(crate) enum ClassLikeRef<'def, 'arena, St, Ex> {
    Class(&'def Class<'arena, SymbolId, St, Ex>, Origin),
    Interface(&'def Interface<'arena, SymbolId, St, Ex>, Origin),
    Trait(&'def Trait<'arena, SymbolId, St, Ex>, Origin),
    Enum(&'def Enum<'arena, SymbolId, St, Ex>, Origin),
    AnonymousClass(&'def AnonymousClass<'arena, SymbolId, St, Ex>, Origin),
}

impl<'arena, St, Ex> ClassLikeRef<'_, 'arena, St, Ex> {
    pub(crate) const fn origin(&self) -> Origin {
        match self {
            ClassLikeRef::Class(_, origin)
            | ClassLikeRef::Interface(_, origin)
            | ClassLikeRef::Trait(_, origin)
            | ClassLikeRef::Enum(_, origin)
            | ClassLikeRef::AnonymousClass(_, origin) => *origin,
        }
    }

    /// The docblock annotation attached to the declaration, if any.
    pub(crate) const fn annotation(
        &self,
    ) -> Option<&'arena mago_hir::ir::item::annotation::ItemAnnotation<'arena, SymbolId, St, Ex>> {
        match self {
            ClassLikeRef::Class(class, _) => class.annotation,
            ClassLikeRef::Interface(interface, _) => interface.annotation,
            ClassLikeRef::Trait(r#trait, _) => r#trait.annotation,
            ClassLikeRef::Enum(r#enum, _) => r#enum.annotation,
            ClassLikeRef::AnonymousClass(anonymous_class, _) => anonymous_class.annotation,
        }
    }

    /// The declaration's member items in source order.
    pub(crate) fn members(&self) -> &'arena [mago_hir::ir::item::member::MemberItem<'arena, SymbolId, St, Ex>] {
        match self {
            ClassLikeRef::Class(class, _) => class.members.as_slice(),
            ClassLikeRef::Interface(interface, _) => interface.members.as_slice(),
            ClassLikeRef::Trait(r#trait, _) => r#trait.members.as_slice(),
            ClassLikeRef::Enum(r#enum, _) => r#enum.members.as_slice(),
            ClassLikeRef::AnonymousClass(anonymous_class, _) => anonymous_class.members.as_slice(),
        }
    }
}

/// Resolves a set of per-file [`DefinitionTable`]s into one fully-linked
/// [`SymbolTable`].
///
/// This subsumes the scan and populate phases of the classic codebase pipeline:
/// it picks a winner for every duplicated symbol by [`Origin`] priority, lowers
/// each winning declaration into its resolved symbol, flattens inheritance onto
/// each class-like (members, inherited-type lists, generic forwarding), and
/// precomputes the descendant and namespace indices the [`SymbolTable`](crate::symbol::SymbolTable)
/// read API relies on.
pub fn link<'arena, A, S, St, Ex>(
    arena: &'arena A,
    scratch: &S,
    definitions: &[DefinitionTable<'arena, A, St, Ex>],
) -> SymbolTable<'arena, A>
where
    A: Arena,
    S: Arena,
    St: Copy,
    Ex: Copy,
{
    let mut table = SymbolTable::new_in(arena);
    let mut builder = TypeBuilder::new(arena, scratch);
    let mut lowerer = Lowerer { arena, builder: &mut builder };

    let class_likes = collect_class_likes(scratch, definitions);
    lower_function_likes(&mut lowerer, definitions, &mut table);
    lower_constants(&mut lowerer, definitions, &mut table);

    for (&id, reference) in &class_likes {
        let symbol = lowerer.class_like_shell(reference);
        register_namespaces(class::class_like_name(reference), &mut table);
        table.class_likes.insert(id, symbol);
    }

    for definition in definitions {
        for (_, function) in &definition.functions {
            register_namespaces(function.name.value, &mut table);
        }
        for (_, constant) in &definition.constants {
            register_namespaces(constant.name.value, &mut table);
        }
    }

    inheritance::resolve_inheritance(&mut lowerer, &mut table, &class_likes);
    sealing::resolve_sealing(arena, scratch, &mut table, &class_likes);

    resolve::resolve(arena, scratch, &mut table, &class_likes);
    descendants::build_descendants(arena, scratch, &mut table);

    table
}

/// Records every namespace prefix of a fully-qualified `name` as containing a
/// symbol, so the checker can tell a missing class apart from a namespace.
fn register_namespaces<A>(name: &[u8], table: &mut SymbolTable<'_, A>)
where
    A: Arena,
{
    let name = name.strip_prefix(b"\\").unwrap_or(name);
    let mut end = 0;
    while let Some(offset) = name[end..].iter().position(|&byte| byte == b'\\') {
        end += offset;
        table.namespaces.insert(SymbolId::namespace(&name[..end]));
        end += 1;
    }
}

/// Lowers every winning free function, closure, and arrow function into the
/// table's `function_likes` map. Closures and arrow functions carry unique
/// synthetic names, so only free functions ever contend an origin tie.
fn lower_function_likes<'arena, S, A, St, Ex>(
    lowerer: &mut Lowerer<'_, '_, 'arena, S, A>,
    definitions: &[DefinitionTable<'arena, A, St, Ex>],
    table: &mut SymbolTable<'arena, A>,
) where
    S: Arena,
    A: Arena,
{
    let arena = lowerer.arena;
    for definition in definitions {
        let origin = definition.origin;

        for (&id, function) in &definition.functions {
            if wins(table.function_likes.get(&id).map(|existing| existing.origin()), origin) {
                let symbol = lowerer.function(function, origin);
                table.function_likes.insert(id, FunctionLikeSymbol::Function(arena.alloc(symbol)));
            }
        }
        for (&id, closure) in &definition.closures {
            let symbol = lowerer.closure(closure, origin);
            table.function_likes.insert(id, FunctionLikeSymbol::Closure(arena.alloc(symbol)));
        }
        for (&id, arrow_function) in &definition.arrow_functions {
            let symbol = lowerer.arrow_function(arrow_function, origin);
            table.function_likes.insert(id, FunctionLikeSymbol::ArrowFunction(arena.alloc(symbol)));
        }
    }
}

/// Lowers every winning global constant into the table's `constants` map.
fn lower_constants<'arena, S, A, St, Ex>(
    lowerer: &mut Lowerer<'_, '_, 'arena, S, A>,
    definitions: &[DefinitionTable<'arena, A, St, Ex>],
    table: &mut SymbolTable<'arena, A>,
) where
    S: Arena,
    A: Arena,
{
    for definition in definitions {
        let origin = definition.origin;

        for (&id, constant) in &definition.constants {
            if wins(table.constants.get(&id).map(|existing| existing.origin), origin) {
                let symbol = lowerer.constant(constant, origin);
                table.constants.insert(id, symbol);
            }
        }
    }
}

/// Whether a `candidate` origin should replace whatever currently occupies a
/// slot: it wins when the slot is empty or its origin is strictly lower
/// priority (the first declaration keeps an equal-origin tie).
fn wins(existing: Option<Origin>, candidate: Origin) -> bool {
    existing.is_none_or(|existing| candidate > existing)
}

/// Selects, for every class-like id, the declaration whose file has the highest
/// [`Origin`] priority (Project > Override > Runtime > Dependency); the first
/// declaration wins an equal-origin tie.
fn collect_class_likes<'def, 'arena, S, St, Ex>(
    scratch: &'def S,
    definitions: &'def [DefinitionTable<'arena, impl Arena, St, Ex>],
) -> HashMap<'def, SymbolId, ClassLikeRef<'def, 'arena, St, Ex>, S, SymbolIdBuildHasher>
where
    S: Arena,
{
    let mut winners = HashMap::with_hasher_in(SymbolIdBuildHasher, scratch);

    for table in definitions {
        let origin = table.origin;

        for (&id, class) in &table.classes {
            consider(&mut winners, id, ClassLikeRef::Class(class, origin));
        }
        for (&id, interface) in &table.interfaces {
            consider(&mut winners, id, ClassLikeRef::Interface(interface, origin));
        }
        for (&id, r#trait) in &table.traits {
            consider(&mut winners, id, ClassLikeRef::Trait(r#trait, origin));
        }
        for (&id, r#enum) in &table.enums {
            consider(&mut winners, id, ClassLikeRef::Enum(r#enum, origin));
        }
        for (&id, anonymous_class) in &table.anonymous_classes {
            consider(&mut winners, id, ClassLikeRef::AnonymousClass(anonymous_class, origin));
        }
    }

    winners
}

/// Inserts `candidate` under `id` unless an equal-or-higher-origin declaration
/// already occupies the slot.
fn consider<'def, 'arena, S, St, Ex>(
    winners: &mut HashMap<'def, SymbolId, ClassLikeRef<'def, 'arena, St, Ex>, S, SymbolIdBuildHasher>,
    id: SymbolId,
    candidate: ClassLikeRef<'def, 'arena, St, Ex>,
) where
    S: Arena,
{
    match winners.get(&id) {
        Some(existing) if existing.origin() >= candidate.origin() => {}
        _ => {
            winners.insert(id, candidate);
        }
    }
}
