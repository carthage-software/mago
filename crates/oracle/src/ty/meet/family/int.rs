//! `Int` family meet: range / literal intersection.

use mago_allocator::Arena;

use crate::ty::atom::Atom;
use crate::ty::atom::payload::scalar::int::IntAtom;
use crate::ty::builder::TypeBuilder;

/// Intersect two `Int` atoms. Subsumption (e.g. `INT ∧ Range(0,10)`)
/// is handled by the caller; this only fires when neither side
/// refines the other, which means both are bounded ranges or distinct
/// literals. The result is `Range(max(lower), min(upper))` collapsed to a
/// `Literal` when the bounds coincide, or `None` when the interval is
/// empty.
pub(in crate::ty::meet) fn int_meet<'arena, S, A>(
    a: Atom<'arena>,
    b: Atom<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Option<Atom<'arena>>
where
    S: Arena,
    A: Arena,
{
    let (Atom::Int(a_payload), Atom::Int(b_payload)) = (a, b) else {
        return None;
    };

    let (a_lower, a_upper) = int_bounds(a_payload);
    let (b_lower, b_upper) = int_bounds(b_payload);

    let lower = match (a_lower, b_lower) {
        (Some(left), Some(right)) => Some(left.max(right)),
        (Some(value), None) | (None, Some(value)) => Some(value),
        (None, None) => None,
    };

    let upper = match (a_upper, b_upper) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (Some(value), None) | (None, Some(value)) => Some(value),
        (None, None) => None,
    };

    match (lower, upper) {
        (Some(lower_bound), Some(upper_bound)) if lower_bound > upper_bound => None,
        (Some(lower_bound), Some(upper_bound)) if lower_bound == upper_bound => Some(Atom::int_literal(lower_bound)),
        _ => Some(builder.int_range_atom(lower, upper)),
    }
}

#[inline]
fn int_bounds(payload: IntAtom<'_>) -> (Option<i64>, Option<i64>) {
    match payload {
        IntAtom::Unspecified | IntAtom::UnspecifiedLiteral => (None, None),
        IntAtom::Literal(value) => (Some(value), Some(value)),
        IntAtom::Range(range) => (range.lower(), range.upper()),
    }
}
