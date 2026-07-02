//! Int family: `int`, `literal-int`, integer literals, bounded ranges.
//!
//! Container variants accept inputs as follows:
//!
//! - `Unspecified` (general `int`) accepts any Int-kind input.
//! - `UnspecifiedLiteral` (`literal-int`) accepts `Literal(_)` and itself.
//! - `Literal(N)` accepts only the same literal (handled by reflexivity).
//! - `Range(R)` accepts `Literal(N)` if `N ∈ R`, and `Range(R')` if
//!   `R' ⊆ R`.
//!
//! "Non-zero int" and similar open complements are expressed as
//! `int & !int(0)` via the universal `Negated` machinery - no dedicated
//! variant lives in this family.

use crate::ty::atom::Atom;
use crate::ty::atom::payload::scalar::int::IntAtom;
use crate::ty::atom::payload::scalar::int::IntRange;

#[inline]
#[must_use]
pub fn refines(input: Atom<'_>, container: Atom<'_>) -> bool {
    let (Atom::Int(input_payload), Atom::Int(container_payload)) = (input, container) else {
        return false;
    };

    match (input_payload, container_payload) {
        (_, IntAtom::Unspecified) => true,
        (IntAtom::Literal(_) | IntAtom::UnspecifiedLiteral, IntAtom::UnspecifiedLiteral) => true,
        (IntAtom::Literal(value), IntAtom::Range(range)) => range_contains_value(*range, value),
        (IntAtom::Range(inner), IntAtom::Range(outer)) => range_contains_range(*outer, *inner),
        _ => false,
    }
}

#[inline]
const fn range_contains_value(range: IntRange, value: i64) -> bool {
    let lower_ok = match range.lower() {
        Some(lower) => lower <= value,
        None => true,
    };
    let upper_ok = match range.upper() {
        Some(upper) => value <= upper,
        None => true,
    };

    lower_ok && upper_ok
}

#[inline]
const fn range_contains_range(outer: IntRange, inner: IntRange) -> bool {
    let lower_ok = match (outer.lower(), inner.lower()) {
        (None, _) => true,
        (Some(_), None) => false,
        (Some(outer_lower), Some(inner_lower)) => outer_lower <= inner_lower,
    };
    let upper_ok = match (outer.upper(), inner.upper()) {
        (None, _) => true,
        (Some(_), None) => false,
        (Some(outer_upper), Some(inner_upper)) => inner_upper <= outer_upper,
    };

    lower_ok && upper_ok
}
