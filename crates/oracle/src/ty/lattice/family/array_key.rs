//! `array-key` container: `int | string | class-like-string` fit. Floats
//! and bools are explicitly NOT array keys.

use crate::ty::atom::Atom;
use crate::ty::atom::kind::AtomKind;

#[inline]
#[must_use]
pub const fn refines(input: Atom<'_>, _container: Atom<'_>) -> bool {
    matches!(input.kind(), AtomKind::Int | AtomKind::String | AtomKind::ClassLikeString)
}
