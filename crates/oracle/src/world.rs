//! The boundary between the pure type system and the analyzer's view of the
//! world being analyzed.
//!
//! The type system answers questions about types in isolation: "is `int` a
//! subtype of `float`?", "what is the join of `int|null` and `string|null`?".
//! Many real-world questions also depend on facts the surrounding analyzer
//! knows: "does class `Foo` extend `Bar`?", "what type parameters does
//! `Container` declare?", "what type does `Box<T> extends Wrapper<T>` pass to
//! `Wrapper`'s template?".
//!
//! Those facts live in the analyzer (a static analyzer, a language server,
//! mock fixtures for tests) and are exposed via the [`World`] trait. Each
//! lattice operation, narrowing operation, and structural analysis takes a
//! ` World` so the type system can ask whatever it needs without knowing
//! how the analyzer stores it.
//!
//! Class-like, function, and global-constant identities are passed as
//! [`SymbolId`] - the normalized, case-folded hash that the analyzer keys its
//! storage by. Member names (methods, properties, class constants, template
//! parameters, enum cases) are passed and returned as raw `&[u8]`, since they
//! are not standalone symbols. Class-name returns are [`Fqn`] so the caller can
//! both re-query and display them.
//!
//! The `'arena` parameter is the lifetime of the types the world hands back.
//! A long-lived world storing `Type<'world>` answers queries for shorter
//! file lifetimes covariantly: implement `World<'arena>` for every `'arena`
//! that `'world` outlives, and the returned types coerce.

use crate::id::SymbolId;
use crate::path::Path;
use crate::symbol::class_like::part::constant::ClassLikeConstantMember;
use crate::symbol::class_like::part::enum_case::EnumCaseMember;
use crate::symbol::class_like::part::inheritance::InheritedType;
use crate::symbol::class_like::part::visibility::Visibility;
use crate::symbol::part::generic::Variance;
use crate::ty::Type;
use crate::ty::atom::payload::scalar::class_like_string::ClassLikeKind;

/// What the type system needs to know about the world being analyzed.
///
/// All methods are queries that return a scalar, a single value, or a
/// *borrowed* slice into the world's own storage - never an owned collection
/// or iterator. Implementations stay free to store metadata however they like
/// (hash maps, indexed vectors, persistent trees, databases) and the borrowed
/// returns cost no allocation, while the absence of generic returns keeps the
/// trait dyn-compatible.
///
/// All methods are required: the trait gives no defaults, so an
/// implementation cannot accidentally leave a query unanswered. A "this
/// world knows nothing" implementation should return `false` / `0` / `None`
/// explicitly (see [`NullWorld`]).
pub trait World<'arena> {
    /// `true` iff `child` is the same class-like as `ancestor`, or extends /
    /// implements / uses-trait it transitively.
    fn descends_from(&self, child: SymbolId, ancestor: SymbolId) -> bool;

    /// `true` iff `class` directly `use`s `trait_name` (the trait appears in
    /// `class`'s body as `use TraitName;`).
    ///
    /// Asymmetric vs [`descends_from`](Self::descends_from), which closes
    /// over inheritance: `descends_from` returns `true` for any trait in the
    /// chain, but `uses_trait` only for direct usage.
    fn uses_trait(&self, class: SymbolId, trait_name: SymbolId) -> bool;

    /// How many type parameters `class` declares. `0` for unknown or
    /// non-generic classes.
    fn template_parameter_arity(&self, class: SymbolId) -> usize;

    /// The type parameter at `position` in `class`'s declaration, or `None`
    /// if `position >= template_parameter_arity(class)`.
    fn template_parameter_at(&self, class: SymbolId, position: usize) -> Option<TemplateParameter<'arena>>;

    /// The position of `class`'s type parameter named `name`, or `None` if
    /// no such parameter exists.
    fn template_parameter_index(&self, class: SymbolId, name: &[u8]) -> Option<usize>;

    /// The type `child` passes to `ancestor`'s `position`-th type parameter,
    /// expressed in `child`'s template namespace.
    ///
    /// For `class B<T> extends A<string>` with
    /// `inherited_template_argument(B, A, 0)`, returns `Some(string)`. For
    /// `class B<T> extends A<List<T>>`, returns `Some(List<T>)` - the caller
    /// substitutes `child`'s actual arguments to fully resolve.
    ///
    /// Returns `None` when `child` does not descend from `ancestor`, or when
    /// `position >= template_parameter_arity(ancestor)`.
    fn inherited_template_argument(&self, child: SymbolId, ancestor: SymbolId, position: usize)
    -> Option<Type<'arena>>;

    /// `true` iff the template parameter `(from_class, from_parameter)` is the
    /// *same variable* as `(to_class, to_parameter)` through inheritance
    /// forwarding: `from_class` (transitively) extends `to_class` binding its
    /// own `from_parameter` into `to_class`'s `to_parameter` slot
    /// (`class C<TC> extends D<TC>` forwards `C::TC` to `D::T`).
    ///
    /// This is the basis for inherited template subtyping: when it holds, a
    /// `from_parameter` value is also a `to_parameter` value (so `C::TC <:
    /// D::T`), one-directionally - the reverse need not hold, since a bare
    /// `to_class` could be specialised to anything.
    ///
    /// The relation MUST be transitively closed and reflexive: if `C` forwards
    /// `TC` to `D::TD` and `D` forwards `TD` to `E::TE`, then `C` forwards `TC`
    /// to `E::TE`. Without this, the derived subtyping would not be transitive.
    fn template_parameter_forwards_to(
        &self,
        from_class: SymbolId,
        from_parameter: &[u8],
        to_class: SymbolId,
        to_parameter: &[u8],
    ) -> bool;

    /// `true` iff `class` declares or inherits a method named `method`.
    /// Mirrors PHP's `method_exists()` semantics: walks the inheritance
    /// closure (parent classes, implemented interfaces, used traits).
    fn class_has_method(&self, class: SymbolId, method: &[u8]) -> bool;

    /// The declared type of `property` on `class`, walking the inheritance
    /// closure. `None` when the property is absent or its type is unknown.
    fn class_property_type(&self, class: SymbolId, property: &[u8]) -> Option<Type<'arena>>;

    /// `true` iff `class` declares or inherits a property named `property`.
    /// Mirrors PHP's `property_exists()` semantics.
    fn class_has_property(&self, class: SymbolId, property: &[u8]) -> bool;

    /// What kind of enum `enum_name` is.
    ///
    /// Returns `None` when the enum is unknown (or `enum_name` does not name
    /// an enum). The lattice treats `None` conservatively: a structural
    /// narrowing that depends on the backing is rejected.
    fn enum_backing(&self, enum_name: SymbolId) -> Option<EnumBacking<'arena>>;

    /// What kind of class-like `name` declares (class, interface, enum, or
    /// trait), or `None` when the world doesn't know `name`.
    fn class_like_kind(&self, name: SymbolId) -> Option<ClassLikeKind>;

    /// `true` iff `name` cannot be extended (PHP `final class` declaration,
    /// or any enum; enums are implicitly final).
    fn is_final(&self, name: SymbolId) -> bool;

    /// The recorded body of `class::alias` (a `@type` alias declared on the
    /// class), or `None` when the alias is unknown. Used by expansion to
    /// substitute alias bodies in place of `Alias` atoms.
    fn alias_body(&self, class: SymbolId, alias: &[u8]) -> Option<Type<'arena>>;

    /// The declared or inferred type of `class::constant`. `None` when the
    /// constant is unknown. Used by expansion to resolve `MemberReference`
    /// atoms with an `Identifier` selector.
    fn class_constant_type(&self, class: SymbolId, constant: &[u8]) -> Option<Type<'arena>>;

    /// Every constant visible from `class`'s scope (its own and those it
    /// inherits), as a borrowed slice into the world's storage - no
    /// allocation, no assumed ordering on the caller's part. Used by expansion
    /// to resolve `MemberReference` atoms with a wildcard / prefix / suffix
    /// selector (`Foo::*`, `Foo::STATUS_*`): the resolver filters this slice by
    /// the selector and unions the matching constant types. Empty when `class`
    /// is unknown or declares no constants.
    fn class_constants(&self, class: SymbolId) -> &[ClassLikeConstantMember<'arena>];

    /// Every case `enum_name` declares, as a borrowed slice into the world's
    /// storage - no allocation. Used by expansion to resolve a wildcard member
    /// reference against an enum (`Suit::*`, `Suit::HEART_*`) into the union of
    /// the matching cases. Empty for non-enums or unknown names.
    fn enum_cases(&self, enum_name: SymbolId) -> &[EnumCaseMember<'arena>];

    /// The declared or inferred type of a global constant or function
    /// signature. `None` when the name is unknown. Used by expansion to
    /// resolve `GlobalReference` atoms.
    fn global_constant_type(&self, name: SymbolId) -> Option<Type<'arena>>;

    /// How many properties `class` declares or inherits (visible from
    /// `class`'s scope).
    fn class_property_count(&self, class: SymbolId) -> usize;

    /// The `position`-th property of `class`, walking the inheritance
    /// closure in declaration order. Used by expansion to build the shape
    /// returned by `properties-of<C>`.
    fn class_property_at(&self, class: SymbolId, position: usize) -> Option<ClassProperty<'arena>>;

    /// The closed list of *direct* inheritors of `class_like` when the world
    /// treats it as sealed; `None` when the world considers `class_like`
    /// open (anything may extend it).
    ///
    /// "Direct" means immediate children only; the lattice walks transitive
    /// sealing recursively.
    ///
    /// Contract: the inheritors returned must each `descends_from` the
    /// queried class. Inconsistent worlds produce wrong lattice answers the
    /// same way an inconsistent `descends_from` does.
    fn sealed_direct_inheritors(&self, class_like: SymbolId) -> Option<&[InheritedType<'arena>]>;

    /// If `child` is a direct inheritor of a sealed class, returns that
    /// sealed parent. `None` otherwise.
    fn sealed_parent_of(&self, child: SymbolId) -> Option<Path<'arena>>;
}

/// One template parameter of a generic class-like or function.
///
/// Variance is per-parameter and defaults to [`Variance::Invariant`] when
/// the source provides no annotation. `upper_bound` is the `@template T of
/// Foo` constraint, if any; `None` means unbounded (`mixed`-equivalent).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TemplateParameter<'arena> {
    pub name: &'arena [u8],
    pub variance: Variance,
    pub upper_bound: Option<Type<'arena>>,
}

/// One declared property of a class-like, returned by
/// [`World::class_property_at`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClassProperty<'arena> {
    pub name: &'arena [u8],
    pub r#type: Type<'arena>,
    pub visibility: Visibility,
}

/// What an enum case carries beyond its `name`. PHP enums are either pure
/// (only `name`) or backed by `int` / `string` (carrying a `value` property
/// of that scalar type).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnumBacking<'arena> {
    /// Pure enum (`enum X { case A; }`). Cases expose only `name`.
    Pure,
    /// Backed enum (`enum X: string { case A = 'a'; }`). Cases expose `name`
    /// and `value`. The wrapped [`Type`] is the backing type - typically
    /// `int` or `string`.
    Backed(Type<'arena>),
}

/// A no-op [`World`] for queries that don't consult the world.
///
/// Every lookup returns the empty / negative answer. Suitable when the input
/// types contain only scalar / trivial atoms and no object / generic /
/// reference machinery would be exercised.
pub struct NullWorld;

impl<'arena> World<'arena> for NullWorld {
    #[inline]
    fn descends_from(&self, _child: SymbolId, _ancestor: SymbolId) -> bool {
        false
    }

    #[inline]
    fn uses_trait(&self, _class: SymbolId, _trait_name: SymbolId) -> bool {
        false
    }

    #[inline]
    fn template_parameter_arity(&self, _class: SymbolId) -> usize {
        0
    }

    #[inline]
    fn template_parameter_at(&self, _class: SymbolId, _position: usize) -> Option<TemplateParameter<'arena>> {
        None
    }

    #[inline]
    fn template_parameter_index(&self, _class: SymbolId, _name: &[u8]) -> Option<usize> {
        None
    }

    #[inline]
    fn inherited_template_argument(
        &self,
        _child: SymbolId,
        _ancestor: SymbolId,
        _position: usize,
    ) -> Option<Type<'arena>> {
        None
    }

    #[inline]
    fn template_parameter_forwards_to(
        &self,
        from_class: SymbolId,
        from_parameter: &[u8],
        to_class: SymbolId,
        to_parameter: &[u8],
    ) -> bool {
        from_class == to_class && from_parameter == to_parameter
    }

    #[inline]
    fn class_has_method(&self, _class: SymbolId, _method: &[u8]) -> bool {
        false
    }

    #[inline]
    fn class_property_type(&self, _class: SymbolId, _property: &[u8]) -> Option<Type<'arena>> {
        None
    }

    #[inline]
    fn class_has_property(&self, _class: SymbolId, _property: &[u8]) -> bool {
        false
    }

    #[inline]
    fn enum_backing(&self, _enum_name: SymbolId) -> Option<EnumBacking<'arena>> {
        None
    }

    #[inline]
    fn class_like_kind(&self, _name: SymbolId) -> Option<ClassLikeKind> {
        None
    }

    #[inline]
    fn is_final(&self, _name: SymbolId) -> bool {
        false
    }

    #[inline]
    fn alias_body(&self, _class: SymbolId, _alias: &[u8]) -> Option<Type<'arena>> {
        None
    }

    #[inline]
    fn class_constant_type(&self, _class: SymbolId, _constant: &[u8]) -> Option<Type<'arena>> {
        None
    }

    #[inline]
    fn class_constants(&self, _class: SymbolId) -> &[ClassLikeConstantMember<'arena>] {
        &[]
    }

    #[inline]
    fn enum_cases(&self, _enum_name: SymbolId) -> &[EnumCaseMember<'arena>] {
        &[]
    }

    #[inline]
    fn global_constant_type(&self, _name: SymbolId) -> Option<Type<'arena>> {
        None
    }

    #[inline]
    fn class_property_count(&self, _class: SymbolId) -> usize {
        0
    }

    #[inline]
    fn class_property_at(&self, _class: SymbolId, _position: usize) -> Option<ClassProperty<'arena>> {
        None
    }

    #[inline]
    fn sealed_direct_inheritors(&self, _class_like: SymbolId) -> Option<&[InheritedType<'arena>]> {
        None
    }

    #[inline]
    fn sealed_parent_of(&self, _child: SymbolId) -> Option<Path<'arena>> {
        None
    }
}
