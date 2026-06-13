//! Bool family: `bool`, `true`, `false`.
//!
//! `true` and `false` are both subtypes of `bool`. They are not subtypes of
//! each other. `bool` is not a subtype of `true` or `false` (a `bool` could
//! be either at runtime).

use crate::ty::atom::Atom;

/// `true | false <: bool`. The dispatcher passes `container` for symmetry
/// with other families; here it must be `bool`. Reflexivity
/// (`bool <: bool`) is the dispatcher's responsibility.
#[inline]
#[must_use]
pub const fn refines(input: Atom<'_>, _container: Atom<'_>) -> bool {
    matches!(input, Atom::True | Atom::False)
}
