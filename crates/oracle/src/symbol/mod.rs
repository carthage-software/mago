use mago_allocator::prelude::*;
use mago_span::HasSpan;

use crate::id::SymbolId;
use crate::path::Path;
use crate::symbol::class_like::ClassLikeSymbol;
use crate::symbol::constant::ConstantSymbol;
use crate::symbol::function_like::FunctionLikeSymbol;
use crate::symbol::part::attribute::AppliedAttribute;
use crate::symbol::part::constraint::SymbolConstraint;
use crate::symbol::part::origin::Origin;

pub mod class_like;
pub mod constant;
pub mod function_like;
pub mod part;

/// The fully-resolved index of every symbol the analyzer knows about.
///
/// This is the central store the inference engine and checker query to answer
/// questions like "does `Foo` exist?", "what does `Foo::bar()` return?", or
/// "what classes descend from `Throwable`?". Entries are produced **already
/// linked** by a single linker pass - inheritance is flattened onto each
/// symbol, members carry their declaring symbol, and collisions between
/// declarations of the same name are resolved by [`Origin`] priority before
/// insertion. Consequently every query here is a hash lookup or a slice read,
/// never an inheritance walk.
///
/// Everything is keyed by [`SymbolId`] - the tagged, case-folded hash that
/// distinguishes a class `Foo` from a function `Foo` - so the three symbol
/// kinds live in separate maps without colliding. Class methods, properties,
/// constants, and enum cases are **not** top-level entries: they live inside
/// their [`ClassLikeSymbol`], reachable through its member lists.
///
/// The table holds only resolved definitions. Incremental-analysis state
/// (file signatures, safe-symbol sets) and diagnostics live in the engine
/// layer, not here.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Debug)]
pub struct SymbolTable<'arena, A: Arena> {
    /// Global constants, keyed by their [`SymbolId`].
    pub constants: HashMap<'arena, SymbolId, ConstantSymbol<'arena>, A>,
    /// Free functions, closures, and arrow functions, keyed by their
    /// [`SymbolId`]. Methods are *not* here - they belong to their class-like.
    pub function_likes: HashMap<'arena, SymbolId, FunctionLikeSymbol<'arena>, A>,
    /// Classes, interfaces, traits, enums, and anonymous classes, keyed by
    /// their [`SymbolId`].
    pub class_likes: HashMap<'arena, SymbolId, ClassLikeSymbol<'arena>, A>,
    /// Every namespace that contains at least one symbol, keyed by its
    /// case-folded [`SymbolId`] (PHP namespaces are case-insensitive). Lets the
    /// checker tell a missing class apart from a bare namespace reference.
    pub namespaces: HashSet<'arena, SymbolId, A>,
    /// The *direct* children of each class-like: classes that immediately
    /// extend it, interfaces that immediately extend it, or types that
    /// immediately implement/use it. The reverse of the inheritance edges
    /// stored on each symbol. Each slice is sorted by [`SymbolId`] for
    /// binary-search membership.
    pub direct_descendants: HashMap<'arena, SymbolId, &'arena [SymbolId], A>,
    /// The *transitive* closure of every descendant of each class-like,
    /// precomputed by the linker so subclass enumeration and sealed-type
    /// exhaustiveness are a single read rather than a repeated graph walk.
    /// Each slice is sorted by [`SymbolId`] for binary-search membership.
    pub all_descendants: HashMap<'arena, SymbolId, &'arena [SymbolId], A>,
}

impl<'arena, A: Arena> SymbolTable<'arena, A> {
    /// Creates an empty table whose maps allocate in `arena`.
    #[must_use]
    pub fn new_in(arena: &'arena A) -> Self {
        Self {
            constants: HashMap::new_in(arena),
            function_likes: HashMap::new_in(arena),
            class_likes: HashMap::new_in(arena),
            namespaces: HashSet::new_in(arena),
            direct_descendants: HashMap::new_in(arena),
            all_descendants: HashMap::new_in(arena),
        }
    }

    /// The global constant identified by `id`, or `None` if no such constant
    /// is known.
    #[inline]
    #[must_use]
    pub fn get_constant(&self, id: SymbolId) -> Option<&ConstantSymbol<'arena>> {
        self.constants.get(&id)
    }

    /// The free function, closure, or arrow function identified by `id`, or
    /// `None` if none is known. Methods are not function-likes - look them up
    /// through their [`ClassLikeSymbol`].
    #[inline]
    #[must_use]
    pub fn get_function_like(&self, id: SymbolId) -> Option<FunctionLikeSymbol<'arena>> {
        self.function_likes.get(&id).copied()
    }

    /// The class, interface, trait, enum, or anonymous class identified by
    /// `id`, or `None` if none is known.
    #[inline]
    #[must_use]
    pub fn get_class_like(&self, id: SymbolId) -> Option<ClassLikeSymbol<'arena>> {
        self.class_likes.get(&id).copied()
    }

    /// Whether `name` names a namespace that contains at least one symbol. The
    /// query is case-insensitive and tolerates a leading namespace separator;
    /// the name folds straight into a [`SymbolId`] with no allocation.
    #[inline]
    #[must_use]
    pub fn contains_namespace(&self, name: &[u8]) -> bool {
        self.namespaces.contains(&SymbolId::namespace(name))
    }

    /// Every descendant of `class_like`, transitively. Empty when the symbol
    /// is unknown or has no descendants.
    #[inline]
    #[must_use]
    pub fn descendants_of(&self, class_like: SymbolId) -> &'arena [SymbolId] {
        self.all_descendants.get(&class_like).copied().unwrap_or(&[])
    }

    /// The immediate children of `class_like`. Empty when the symbol is
    /// unknown or has no direct descendants.
    #[inline]
    #[must_use]
    pub fn direct_descendants_of(&self, class_like: SymbolId) -> &'arena [SymbolId] {
        self.direct_descendants.get(&class_like).copied().unwrap_or(&[])
    }

    /// Whether `candidate` transitively descends from `class_like` - a binary
    /// search over the precomputed, sorted descendant closure.
    #[inline]
    #[must_use]
    pub fn is_descendant(&self, class_like: SymbolId, candidate: SymbolId) -> bool {
        self.descendants_of(class_like).binary_search(&candidate).is_ok()
    }

    /// Whether `candidate` is an immediate child of `class_like` - a binary
    /// search over the sorted direct-descendant slice.
    #[inline]
    #[must_use]
    pub fn is_direct_descendant(&self, class_like: SymbolId, candidate: SymbolId) -> bool {
        self.direct_descendants_of(class_like).binary_search(&candidate).is_ok()
    }
}

/// A top-level, resolved symbol: a class-like, a function-like, or a global
/// constant.
///
/// "Top-level" is what separates a `Symbol` from a [`SymbolMember`]: a symbol
/// stands on its own and is keyed directly in the [`SymbolTable`], whereas a
/// member (method, property, ...) only exists inside another symbol. Every symbol
/// carries an identity ([`Path`]/[`SymbolId`]), a provenance ([`Origin`]), the
/// PHP-version range it is available in, and the attributes applied to its
/// declaration.
pub trait Symbol<'arena>: Sized + HasSpan {
    /// The symbol's identity hash. Defaults to the id of its [`Path`].
    fn id(&self) -> SymbolId {
        self.path().id
    }

    /// The symbol's fully-qualified path - its identity plus the displayable
    /// segments of its name.
    fn path(&self) -> Path<'arena>;

    /// Where the symbol comes from (project, dependency, runtime, or an
    /// override stub), which decides who wins a same-name collision.
    fn origin(&self) -> Origin;

    /// Whether the symbol is a polyfill: a declaration standing in for one the
    /// runtime would otherwise provide.
    fn is_polyfill(&self) -> bool;

    /// The attributes written on the symbol's declaration.
    fn applied_attributes(&self) -> &'arena [AppliedAttribute<'arena>];

    /// The constraint gating which PHP versions this symbol is available in,
    /// or `None` when its availability is not constrained.
    fn constraint(&self) -> Option<SymbolConstraint<'arena>>;
}

/// A member that lives inside another symbol: a method, property, property hook,
/// class constant, or enum case.
///
/// A member is not a standalone [`Symbol`] - it has no entry of its own in the
/// [`SymbolTable`] and is reached through its owning class-like. Beyond a
/// symbol's identity and provenance, a member knows the
/// [`defining_symbol`](Self::defining_symbol) it belongs to, which for an
/// inherited member is the ancestor that actually declares it rather than the
/// class the member was reached through.
pub trait SymbolMember<'arena> {
    /// The member's identity hash. Defaults to the id of its [`Path`].
    fn id(&self) -> SymbolId {
        self.path().id
    }

    /// The member's fully-qualified path (e.g. `Foo::$bar`).
    fn path(&self) -> Path<'arena>;

    /// The class-like that declares this member - the ancestor, for an
    /// inherited member, not the class it was reached through.
    fn defining_symbol(&self) -> SymbolId;

    /// Where the member comes from (project, dependency, runtime, or override).
    fn origin(&self) -> Origin;

    /// The attributes written on the member's declaration.
    fn applied_attributes(&self) -> &'arena [AppliedAttribute<'arena>];

    /// The constraint gating which PHP versions this member is available in,
    /// or `None` when its availability is not constrained.
    fn constraint(&self) -> Option<SymbolConstraint<'arena>>;
}
