//! Integer-range / literal subtract.

use mago_allocator::Arena;
use mago_allocator::vec::Vec as ScratchVec;

use crate::ty::atom::Atom;
use crate::ty::atom::payload::scalar::int::IntAtom;
use crate::ty::builder::TypeBuilder;

/// Difference of two integer atoms when neither side fully refines the
/// other. Produces 0, 1, or 2 surviving pieces, each of which is a
/// `Range` collapsed to a `Literal` when its bounds coincide.
pub(in crate::ty::subtract) fn int_minus<'scratch, 'arena, S, A>(
    input: Atom<'arena>,
    removed: Atom<'arena>,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
    out: &mut ScratchVec<'scratch, Atom<'arena>, S>,
) where
    S: Arena,
    A: Arena,
{
    let (Atom::Int(input_payload), Atom::Int(removed_payload)) = (input, removed) else {
        out.push(input);
        return;
    };

    let (input_lower, input_upper) = int_bounds(input_payload);
    let (removed_lower, removed_upper) = int_bounds(removed_payload);

    if let Some(removed_lower_value) = removed_lower
        && let Some(upper_bound) = removed_lower_value.checked_sub(1)
    {
        let input_starts_below = match input_lower {
            Some(value) => value < removed_lower_value,
            None => true,
        };

        if input_starts_below {
            let piece_upper = match input_upper {
                Some(value) => Some(value.min(upper_bound)),
                None => Some(upper_bound),
            };

            if non_empty_interval(input_lower, piece_upper) {
                let piece = make_int_piece(input_lower, piece_upper, builder);
                out.push(piece);
            }
        }
    }

    if let Some(removed_upper_value) = removed_upper
        && let Some(lower_bound) = removed_upper_value.checked_add(1)
    {
        let input_ends_above = match input_upper {
            Some(value) => value > removed_upper_value,
            None => true,
        };

        if input_ends_above {
            let piece_lower = match input_lower {
                Some(value) => Some(value.max(lower_bound)),
                None => Some(lower_bound),
            };

            if non_empty_interval(piece_lower, input_upper) {
                let piece = make_int_piece(piece_lower, input_upper, builder);
                out.push(piece);
            }
        }
    }
}

#[inline]
const fn non_empty_interval(lower: Option<i64>, upper: Option<i64>) -> bool {
    match (lower, upper) {
        (Some(lower_value), Some(upper_value)) => lower_value <= upper_value,
        _ => true,
    }
}

#[inline]
const fn int_bounds(payload: IntAtom<'_>) -> (Option<i64>, Option<i64>) {
    match payload {
        IntAtom::Unspecified | IntAtom::UnspecifiedLiteral => (None, None),
        IntAtom::Literal(value) => (Some(value), Some(value)),
        IntAtom::Range(range) => (range.lower(), range.upper()),
    }
}

#[inline]
fn make_int_piece<'arena, S, A>(
    lower: Option<i64>,
    upper: Option<i64>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Atom<'arena>
where
    S: Arena,
    A: Arena,
{
    match (lower, upper) {
        (Some(lower_value), Some(upper_value)) if lower_value == upper_value => Atom::int_literal(lower_value),
        _ => builder.int_range(lower, upper),
    }
}
