use mago_allocator::Arena;
use mago_allocator::collections::HashMap;
use mago_hir::ir::item::expression::anonymous_class::AnonymousClass;
use mago_hir::ir::item::expression::arrow_function::ArrowFunction;
use mago_hir::ir::item::expression::closure::Closure;
use mago_hir::ir::item::statement::class::Class;
use mago_hir::ir::item::statement::constant::Constant;
use mago_hir::ir::item::statement::r#enum::Enum;
use mago_hir::ir::item::statement::function::Function;
use mago_hir::ir::item::statement::interface::Interface;
use mago_hir::ir::item::statement::r#trait::Trait;

use crate::id::SymbolId;
use crate::id::SymbolIdBuildHasher;

pub mod binder;

/// A map from a [`SymbolId`] to the definition it identifies, allocated in the
/// arena and keyed without re-hashing (a `SymbolId` is already a hash).
pub type DefinitionMap<'arena, A, T> = HashMap<'arena, SymbolId, T, A, SymbolIdBuildHasher>;

/// A definition table indexes every definition in a program by its [`SymbolId`].
///
/// Members (methods, properties, class constants, enum cases) are not indexed
/// here; they live inside their class-like definition. Only top-level and
/// expression-level definitions get their own entry.
#[derive(Debug)]
pub struct DefinitionTable<'arena, A, S, E>
where
    A: Arena,
{
    pub constants: DefinitionMap<'arena, A, Constant<'arena, SymbolId, S, E>>,
    pub functions: DefinitionMap<'arena, A, Function<'arena, SymbolId, S, E>>,
    pub classes: DefinitionMap<'arena, A, Class<'arena, SymbolId, S, E>>,
    pub interfaces: DefinitionMap<'arena, A, Interface<'arena, SymbolId, S, E>>,
    pub traits: DefinitionMap<'arena, A, Trait<'arena, SymbolId, S, E>>,
    pub enums: DefinitionMap<'arena, A, Enum<'arena, SymbolId, S, E>>,
    pub anonymous_classes: DefinitionMap<'arena, A, AnonymousClass<'arena, SymbolId, S, E>>,
    pub closures: DefinitionMap<'arena, A, Closure<'arena, SymbolId, S, E>>,
    pub arrow_functions: DefinitionMap<'arena, A, ArrowFunction<'arena, SymbolId, S, E>>,
}

impl<'arena, A, S, E> DefinitionTable<'arena, A, S, E>
where
    A: Arena,
{
    /// Creates an empty definition table with every map allocated in `arena`.
    #[must_use]
    pub fn new_in(arena: &'arena A) -> Self {
        DefinitionTable {
            constants: HashMap::with_hasher_in(SymbolIdBuildHasher, arena),
            functions: HashMap::with_hasher_in(SymbolIdBuildHasher, arena),
            classes: HashMap::with_hasher_in(SymbolIdBuildHasher, arena),
            interfaces: HashMap::with_hasher_in(SymbolIdBuildHasher, arena),
            traits: HashMap::with_hasher_in(SymbolIdBuildHasher, arena),
            enums: HashMap::with_hasher_in(SymbolIdBuildHasher, arena),
            anonymous_classes: HashMap::with_hasher_in(SymbolIdBuildHasher, arena),
            closures: HashMap::with_hasher_in(SymbolIdBuildHasher, arena),
            arrow_functions: HashMap::with_hasher_in(SymbolIdBuildHasher, arena),
        }
    }
}
