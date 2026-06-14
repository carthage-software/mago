//! Single-call predicates over a [`Type`].
//!
//! Each function answers one structural question. The naming is consistent
//! across the module:
//!
//! - **`is_X`** - *guaranteed*: every atom of the type is in family `X`.
//!   Conservative: returns `false` when any atom is outside `X` (including
//!   `never` for the all-bottom type).
//! - **`contains_X`** - *possible at the top level*: at least one top-level
//!   atom of the type is in family `X`.
//! - **`is_truthy` / `is_falsy`** - every atom guaranteed truthy / falsy at
//!   runtime.
//! - **`could_be_truthy` / `could_be_falsy`** - at least one atom could be
//!   truthy / falsy.
//! - **`*_anywhere`** - the question recurses through every nested-type
//!   carrier.
//!
//! All predicates are pure functions of the [`Type`] (no
//! [`World`](crate::world::World), no options). Kind-family checks are
//! single mask tests against the precomputed [`AtomKindSet`].

pub(crate) mod atom;

use crate::ty::atom::kind::AtomKind;
use crate::ty::atom::set::AtomKindSet;
use crate::ty::inspect;
use crate::ty::Type;
use crate::ty::well_known;

/// `true` iff `ty` is the bottom type (no values).
#[inline]
#[must_use]
pub fn is_never(ty: Type<'_>) -> bool {
    ty.is_never()
}

/// `true` iff `ty` is the unconstrained top (`mixed` with no axes).
#[inline]
#[must_use]
pub fn is_mixed(ty: Type<'_>) -> bool {
    ty == well_known::TYPE_MIXED
}

/// `true` iff `ty` is a single-atom union.
#[inline]
#[must_use]
pub fn is_singleton(ty: Type<'_>) -> bool {
    ty.is_single()
}

/// `true` iff `ty` is a multi-atom union.
#[inline]
#[must_use]
pub fn is_union(ty: Type<'_>) -> bool {
    ty.is_union()
}

macro_rules! is_kind {
    ($(#[$doc:meta])* $name:ident, $mask:expr) => {
        $(#[$doc])*
        #[inline]
        #[must_use]
        pub fn $name(ty: Type<'_>) -> bool {
            const MASK: AtomKindSet = $mask;
            !ty.kinds.is_empty() && ty.kinds.is_subset_of(MASK)
        }
    };
}

macro_rules! contains_kind {
    ($(#[$doc:meta])* $name:ident, $mask:expr) => {
        $(#[$doc])*
        #[inline]
        #[must_use]
        pub fn $name(ty: Type<'_>) -> bool {
            const MASK: AtomKindSet = $mask;
            ty.kinds.intersects(MASK)
        }
    };
}

const BOOL_MASK: AtomKindSet = AtomKindSet::of(AtomKind::Bool).with(AtomKind::True).with(AtomKind::False);
const ARRAY_MASK: AtomKindSet = AtomKindSet::of(AtomKind::Array).with(AtomKind::List);
const OBJECT_MASK: AtomKindSet = AtomKindSet::of(AtomKind::Object)
    .with(AtomKind::Enum)
    .with(AtomKind::ObjectShape)
    .with(AtomKind::HasMethod)
    .with(AtomKind::HasProperty)
    .with(AtomKind::ObjectAny);
const SCALAR_MASK: AtomKindSet = AtomKindSet::of(AtomKind::Scalar)
    .with(AtomKind::Int)
    .with(AtomKind::Float)
    .with(AtomKind::String)
    .with(AtomKind::Bool)
    .with(AtomKind::True)
    .with(AtomKind::False)
    .with(AtomKind::ClassLikeString)
    .with(AtomKind::Numeric)
    .with(AtomKind::ArrayKey);
const NUMERIC_MASK: AtomKindSet = AtomKindSet::of(AtomKind::Numeric).with(AtomKind::Int).with(AtomKind::Float);

is_kind!(is_int, AtomKindSet::of(AtomKind::Int));
is_kind!(is_float, AtomKindSet::of(AtomKind::Float));
is_kind!(is_string, AtomKindSet::of(AtomKind::String));
is_kind!(is_bool, BOOL_MASK);
is_kind!(is_null, AtomKindSet::of(AtomKind::Null));
is_kind!(is_void, AtomKindSet::of(AtomKind::Void));
is_kind!(is_list, AtomKindSet::of(AtomKind::List));
is_kind!(is_keyed_array, AtomKindSet::of(AtomKind::Array));
is_kind!(is_array, ARRAY_MASK);
is_kind!(is_iterable, AtomKindSet::of(AtomKind::Iterable));
is_kind!(is_object, OBJECT_MASK);
is_kind!(is_resource, AtomKindSet::of(AtomKind::Resource));
is_kind!(is_callable, AtomKindSet::of(AtomKind::Callable));
is_kind!(is_array_key, AtomKindSet::of(AtomKind::ArrayKey));
is_kind!(is_scalar, SCALAR_MASK);
is_kind!(is_numeric, NUMERIC_MASK);

contains_kind!(contains_int, AtomKindSet::of(AtomKind::Int));
contains_kind!(contains_float, AtomKindSet::of(AtomKind::Float));
contains_kind!(contains_string, AtomKindSet::of(AtomKind::String));
contains_kind!(contains_bool, BOOL_MASK);
contains_kind!(contains_null, AtomKindSet::of(AtomKind::Null));
contains_kind!(contains_void, AtomKindSet::of(AtomKind::Void));
contains_kind!(contains_array, ARRAY_MASK);
contains_kind!(contains_iterable, AtomKindSet::of(AtomKind::Iterable));
contains_kind!(contains_object, OBJECT_MASK);
contains_kind!(contains_resource, AtomKindSet::of(AtomKind::Resource));
contains_kind!(contains_callable, AtomKindSet::of(AtomKind::Callable));
contains_kind!(contains_mixed, AtomKindSet::of(AtomKind::Mixed));

/// `true` iff every atom of `ty` is guaranteed truthy at runtime.
/// Vacuously `false` for the empty type (`never`).
#[inline]
#[must_use]
pub fn is_truthy(ty: Type<'_>) -> bool {
    !ty.atoms.is_empty() && ty.atoms.iter().all(|entry| atom::is_truthy(*entry))
}

/// `true` iff every atom of `ty` is guaranteed falsy at runtime.
/// Vacuously `false` for the empty type (`never`).
#[inline]
#[must_use]
pub fn is_falsy(ty: Type<'_>) -> bool {
    !ty.atoms.is_empty() && ty.atoms.iter().all(|entry| atom::is_falsy(*entry))
}

/// `true` iff at least one atom of `ty` could be truthy at runtime. `never`
/// and `void` cannot be truthy; everything else that isn't *guaranteed*
/// falsy could be.
#[inline]
#[must_use]
pub fn could_be_truthy(ty: Type<'_>) -> bool {
    ty.atoms.iter().any(|entry| atom::could_be_truthy(*entry))
}

/// `true` iff at least one atom of `ty` could be falsy at runtime. `never`
/// cannot be anything; `void` is treated as falsy per PHP semantics.
#[inline]
#[must_use]
pub fn could_be_falsy(ty: Type<'_>) -> bool {
    ty.atoms.iter().any(|entry| atom::could_be_falsy(*entry))
}

/// `true` iff every atom of `ty` is a literal-shaped value (specific int /
/// float / string literal, `true`, `false`, `null`, `void`).
#[inline]
#[must_use]
pub fn is_literal(ty: Type<'_>) -> bool {
    !ty.atoms.is_empty() && ty.atoms.iter().all(|entry| atom::is_literal(*entry))
}

/// `true` iff `ty` is a single literal atom. Equivalent to
/// `is_literal(ty) && is_singleton(ty)`. The most useful "can I
/// constant-fold this?" check.
#[inline]
#[must_use]
pub fn is_constant_foldable(ty: Type<'_>) -> bool {
    is_singleton(ty) && is_literal(ty)
}

/// `true` iff any atom anywhere in `ty`'s tree is a `mixed` (the
/// family-level top, including narrowed mixed variants).
#[inline]
#[must_use]
pub fn contains_mixed_anywhere(ty: Type<'_>) -> bool {
    inspect::any(ty, |entry| entry.kind() == AtomKind::Mixed)
}

/// `true` iff any atom anywhere in `ty`'s tree is a free template
/// parameter.
#[inline]
#[must_use]
pub fn contains_template_anywhere(ty: Type<'_>) -> bool {
    inspect::any(ty, |entry| entry.kind() == AtomKind::GenericParameter)
}

/// `true` iff any atom anywhere in `ty`'s tree is a placeholder (the
/// inference-time hole `_`).
#[inline]
#[must_use]
pub fn contains_placeholder_anywhere(ty: Type<'_>) -> bool {
    inspect::any(ty, |entry| entry.kind() == AtomKind::Placeholder)
}

/// `true` iff any atom anywhere in `ty`'s tree is unresolved.
///
/// Unresolved means `Alias`, `Reference`, `MemberReference`,
/// `GlobalReference`, `Conditional`, or `Derived`. The analyzer typically
/// needs to expand such a type before doing further reasoning.
#[inline]
#[must_use]
pub fn contains_unresolved_anywhere(ty: Type<'_>) -> bool {
    const UNRESOLVED_MASK: AtomKindSet = AtomKindSet::of(AtomKind::Alias)
        .with(AtomKind::Reference)
        .with(AtomKind::MemberReference)
        .with(AtomKind::GlobalReference)
        .with(AtomKind::Conditional)
        .with(AtomKind::Derived);

    inspect::any(ty, |entry| UNRESOLVED_MASK.contains(entry.kind()))
}

/// `true` iff `ty`'s tree contains no unresolved atom (no `Alias`,
/// `Reference`, `MemberReference`, `GlobalReference`, `Conditional`, or
/// `Derived` at any depth).
#[inline]
#[must_use]
pub fn is_fully_resolved(ty: Type<'_>) -> bool {
    !contains_unresolved_anywhere(ty)
}
