//! `scalar` container: `bool | true | false | int | float | string |
//! class-like-string | array-key | numeric | scalar`.
//!
//! `null`, `void`, `never`, `mixed`, objects, resources, arrays, callables,
//! and iterables are NOT scalars.

use crate::ty::atom::Atom;
use crate::ty::atom::kind::AtomKind;

#[inline]
#[must_use]
pub const fn refines(input: Atom<'_>, _container: Atom<'_>) -> bool {
    matches!(
        input.kind(),
        AtomKind::Bool
            | AtomKind::True
            | AtomKind::False
            | AtomKind::Int
            | AtomKind::Float
            | AtomKind::String
            | AtomKind::ClassLikeString
            | AtomKind::ArrayKey
            | AtomKind::Numeric
            | AtomKind::Scalar
    )
}
