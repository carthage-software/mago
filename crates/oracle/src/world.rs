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
//! The `'arena` parameter is the lifetime of the types the world hands back.
//! A long-lived world storing `Type<'world>` answers queries for shorter
//! file lifetimes covariantly: implement `World<'arena>` for every `'arena`
//! that `'world` outlives, and the returned types coerce.

use crate::name::Name;
use crate::ty::Type;
use crate::ty::atom::payload::derived::Visibility;
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
    fn descends_from(&self, child: Name<'_>, ancestor: Name<'_>) -> bool;

    /// `true` iff `class` directly `use`s `trait_name` (the trait appears in
    /// `class`'s body as `use TraitName;`).
    ///
    /// Asymmetric vs [`descends_from`](Self::descends_from), which closes
    /// over inheritance: `descends_from` returns `true` for any trait in the
    /// chain, but `uses_trait` only for direct usage.
    fn uses_trait(&self, class: Name<'_>, trait_name: Name<'_>) -> bool;

    /// How many type parameters `class` declares. `0` for unknown or
    /// non-generic classes.
    fn template_parameter_arity(&self, class: Name<'_>) -> usize;

    /// The type parameter at `position` in `class`'s declaration, or `None`
    /// if `position >= template_parameter_arity(class)`.
    fn template_parameter_at(&self, class: Name<'_>, position: usize) -> Option<TemplateParameter<'arena>>;

    /// The position of `class`'s type parameter named `name`, or `None` if
    /// no such parameter exists.
    fn template_parameter_index(&self, class: Name<'_>, name: Name<'_>) -> Option<usize>;

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
    fn inherited_template_argument(&self, child: Name<'_>, ancestor: Name<'_>, position: usize)
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
        from_class: Name<'_>,
        from_parameter: Name<'_>,
        to_class: Name<'_>,
        to_parameter: Name<'_>,
    ) -> bool;

    /// `true` iff `class` declares or inherits a method named `method`.
    /// Mirrors PHP's `method_exists()` semantics: walks the inheritance
    /// closure (parent classes, implemented interfaces, used traits).
    fn class_has_method(&self, class: Name<'_>, method: Name<'_>) -> bool;

    /// The declared type of `property` on `class`, walking the inheritance
    /// closure. `None` when the property is absent or its type is unknown.
    fn class_property_type(&self, class: Name<'_>, property: Name<'_>) -> Option<Type<'arena>>;

    /// `true` iff `class` declares or inherits a property named `property`.
    /// Mirrors PHP's `property_exists()` semantics.
    fn class_has_property(&self, class: Name<'_>, property: Name<'_>) -> bool;

    /// What kind of enum `enum_name` is.
    ///
    /// Returns `None` when the enum is unknown (or `enum_name` does not name
    /// an enum). The lattice treats `None` conservatively: a structural
    /// narrowing that depends on the backing is rejected.
    fn enum_backing(&self, enum_name: Name<'_>) -> Option<EnumBacking<'arena>>;

    /// What kind of class-like `name` declares (class, interface, enum, or
    /// trait), or `None` when the world doesn't know `name`.
    fn class_like_kind(&self, name: Name<'_>) -> Option<ClassLikeKind>;

    /// `true` iff `name` cannot be extended (PHP `final class` declaration,
    /// or any enum; enums are implicitly final).
    fn is_final(&self, name: Name<'_>) -> bool;

    /// The recorded body of `class::alias` (a `@type` alias declared on the
    /// class), or `None` when the alias is unknown. Used by expansion to
    /// substitute alias bodies in place of `Alias` atoms.
    fn alias_body(&self, class: Name<'_>, alias: Name<'_>) -> Option<Type<'arena>>;

    /// The declared or inferred type of `class::constant`. `None` when the
    /// constant is unknown. Used by expansion to resolve `MemberReference`
    /// atoms with an `Identifier` selector.
    fn class_constant_type(&self, class: Name<'_>, constant: Name<'_>) -> Option<Type<'arena>>;

    /// Every constant visible from `class`'s scope (its own and those it
    /// inherits), as a borrowed slice into the world's storage - no
    /// allocation, no assumed ordering on the caller's part. Used by expansion
    /// to resolve `MemberReference` atoms with a wildcard / prefix / suffix
    /// selector (`Foo::*`, `Foo::STATUS_*`): the resolver filters this slice by
    /// the selector and unions the matching constant types. Empty when `class`
    /// is unknown or declares no constants.
    fn class_constants(&self, class: Name<'_>) -> &[ClassConstant<'arena>];

    /// Every case `enum_name` declares, as a borrowed slice into the world's
    /// storage - no allocation. Used by expansion to resolve a wildcard member
    /// reference against an enum (`Suit::*`, `Suit::HEART_*`) into the union of
    /// the matching cases. Empty for non-enums or unknown names.
    fn enum_cases(&self, enum_name: Name<'_>) -> &[Name<'arena>];

    /// The declared or inferred type of a global constant or function
    /// signature. `None` when the name is unknown. Used by expansion to
    /// resolve `GlobalReference` atoms.
    fn global_constant_type(&self, name: Name<'_>) -> Option<Type<'arena>>;

    /// How many properties `class` declares or inherits (visible from
    /// `class`'s scope).
    fn class_property_count(&self, class: Name<'_>) -> usize;

    /// The `position`-th property of `class`, walking the inheritance
    /// closure in declaration order. Used by expansion to build the shape
    /// returned by `properties-of<C>`.
    fn class_property_at(&self, class: Name<'_>, position: usize) -> Option<ClassProperty<'arena>>;

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
    fn sealed_direct_inheritors(&self, class_like: Name<'_>) -> Option<&[Name<'arena>]>;

    /// If `child` is a direct inheritor of a sealed class, returns that
    /// sealed parent. `None` otherwise.
    fn sealed_parent_of(&self, child: Name<'_>) -> Option<Name<'arena>>;
}

/// One template parameter of a generic class-like or function.
///
/// Variance is per-parameter and defaults to [`Variance::Invariant`] when
/// the source provides no annotation. `upper_bound` is the `@template T of
/// Foo` constraint, if any; `None` means unbounded (`mixed`-equivalent).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TemplateParameter<'arena> {
    pub name: Name<'arena>,
    pub variance: Variance,
    pub upper_bound: Option<Type<'arena>>,
}

/// How a template parameter behaves in the value position when comparing
/// generic types.
///
/// **The default is [`Invariant`](Variance::Invariant)** - the only sound
/// default for a class whose template usage is not analysed: a generic
/// mutable container (read AND write of `T`) is invariant, and defaulting to
/// anything looser is unsound. A library author who has audited their class
/// for covariant-only or contravariant-only usage opts in explicitly.
///
/// - [`Covariant`](Variance::Covariant): `Box<int> <: Box<scalar>` when
///   `int <: scalar`. Sound only when `T` appears exclusively in producer
///   (return / read-only) positions.
/// - [`Contravariant`](Variance::Contravariant): `Sink<scalar> <: Sink<int>`
///   when `int <: scalar`. Sound only when `T` appears exclusively in
///   consumer (parameter / write-only) positions.
/// - [`Invariant`](Variance::Invariant): the type argument must match
///   exactly (mutual subtyping).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Variance {
    Covariant,
    Contravariant,
    #[default]
    Invariant,
}

/// One declared property of a class-like, returned by
/// [`World::class_property_at`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClassProperty<'arena> {
    pub name: Name<'arena>,
    pub r#type: Type<'arena>,
    pub visibility: Visibility,
}

/// One constant of a class-like, returned by [`World::class_constants`].
///
/// Carries the constant's name and its declared or inferred type. Unlike
/// [`ClassProperty`], no visibility is tracked: wildcard constant references
/// (`Foo::*`) union every matching constant regardless of visibility, since
/// expansion carries no accessing scope to filter against.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClassConstant<'arena> {
    pub name: Name<'arena>,
    pub r#type: Type<'arena>,
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
    fn descends_from(&self, _child: Name<'_>, _ancestor: Name<'_>) -> bool {
        false
    }

    #[inline]
    fn uses_trait(&self, _class: Name<'_>, _trait_name: Name<'_>) -> bool {
        false
    }

    #[inline]
    fn template_parameter_arity(&self, _class: Name<'_>) -> usize {
        0
    }

    #[inline]
    fn template_parameter_at(&self, _class: Name<'_>, _position: usize) -> Option<TemplateParameter<'arena>> {
        None
    }

    #[inline]
    fn template_parameter_index(&self, _class: Name<'_>, _name: Name<'_>) -> Option<usize> {
        None
    }

    #[inline]
    fn inherited_template_argument(
        &self,
        _child: Name<'_>,
        _ancestor: Name<'_>,
        _position: usize,
    ) -> Option<Type<'arena>> {
        None
    }

    #[inline]
    fn template_parameter_forwards_to(
        &self,
        from_class: Name<'_>,
        from_parameter: Name<'_>,
        to_class: Name<'_>,
        to_parameter: Name<'_>,
    ) -> bool {
        from_class.as_bytes() == to_class.as_bytes() && from_parameter.as_bytes() == to_parameter.as_bytes()
    }

    #[inline]
    fn class_has_method(&self, _class: Name<'_>, _method: Name<'_>) -> bool {
        false
    }

    #[inline]
    fn class_property_type(&self, _class: Name<'_>, _property: Name<'_>) -> Option<Type<'arena>> {
        None
    }

    #[inline]
    fn class_has_property(&self, _class: Name<'_>, _property: Name<'_>) -> bool {
        false
    }

    #[inline]
    fn enum_backing(&self, _enum_name: Name<'_>) -> Option<EnumBacking<'arena>> {
        None
    }

    #[inline]
    fn class_like_kind(&self, _name: Name<'_>) -> Option<ClassLikeKind> {
        None
    }

    #[inline]
    fn is_final(&self, _name: Name<'_>) -> bool {
        false
    }

    #[inline]
    fn alias_body(&self, _class: Name<'_>, _alias: Name<'_>) -> Option<Type<'arena>> {
        None
    }

    #[inline]
    fn class_constant_type(&self, _class: Name<'_>, _constant: Name<'_>) -> Option<Type<'arena>> {
        None
    }

    #[inline]
    fn class_constants(&self, _class: Name<'_>) -> &[ClassConstant<'arena>] {
        &[]
    }

    #[inline]
    fn enum_cases(&self, _enum_name: Name<'_>) -> &[Name<'arena>] {
        &[]
    }

    #[inline]
    fn global_constant_type(&self, _name: Name<'_>) -> Option<Type<'arena>> {
        None
    }

    #[inline]
    fn class_property_count(&self, _class: Name<'_>) -> usize {
        0
    }

    #[inline]
    fn class_property_at(&self, _class: Name<'_>, _position: usize) -> Option<ClassProperty<'arena>> {
        None
    }

    #[inline]
    fn sealed_direct_inheritors(&self, _class_like: Name<'_>) -> Option<&[Name<'arena>]> {
        None
    }

    #[inline]
    fn sealed_parent_of(&self, _child: Name<'_>) -> Option<Name<'arena>> {
        None
    }
}
