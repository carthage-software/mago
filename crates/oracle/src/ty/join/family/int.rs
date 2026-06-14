//! Int-family join: range merging and literal-count collapse.

use std::cmp::Ordering;

use mago_allocator::Arena;
use mago_allocator::vec::Vec as ScratchVec;

use crate::ty::atom::Atom;
use crate::ty::atom::payload::scalar::int::IntAtom;
use crate::ty::builder::TypeBuilder;
use crate::ty::well_known::INT;

/// Merge adjacent integer literals and bounded ranges into wider
/// ranges. `Unspecified` and `UnspecifiedLiteral` are dominators /
/// virtual forms and stay as-is.
pub fn apply_merge_int_ranges<'scratch, 'arena, S, A>(
    atoms: &mut ScratchVec<'scratch, Atom<'arena>, S>,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) where
    S: Arena,
    A: Arena,
{
    let mut intervals: ScratchVec<'scratch, (Option<i64>, Option<i64>), S> = builder.scratch_vec();
    let mut other: ScratchVec<'scratch, Atom<'arena>, S> = builder.scratch_vec_with(atoms.len());
    for &atom in atoms.iter() {
        let Atom::Int(payload) = atom else {
            other.push(atom);
            continue;
        };
        match payload {
            IntAtom::Literal(value) => intervals.push((Some(value), Some(value))),
            IntAtom::Range(range) => intervals.push((range.lower(), range.upper())),
            _ => other.push(atom),
        }
    }

    if intervals.is_empty() {
        return;
    }

    intervals.sort_by(|left, right| match (left.0, right.0) {
        (None, None) => Ordering::Equal,
        (None, _) => Ordering::Less,
        (_, None) => Ordering::Greater,
        (Some(left_lower), Some(right_lower)) => left_lower.cmp(&right_lower),
    });

    let mut merged: ScratchVec<'scratch, (Option<i64>, Option<i64>), S> = builder.scratch_vec_with(intervals.len());
    for interval in intervals {
        if let Some(last) = merged.last_mut() {
            let adjacent = match (last.1, interval.0) {
                (None, _) => true,
                (Some(_), None) => true,
                (Some(last_upper), Some(interval_lower)) => {
                    last_upper.checked_add(1).is_some_and(|successor| successor >= interval_lower)
                }
            };
            if adjacent {
                last.1 = match (last.1, interval.1) {
                    (None, _) | (_, None) => None,
                    (Some(left_upper), Some(right_upper)) => Some(left_upper.max(right_upper)),
                };
                continue;
            }
        }
        merged.push(interval);
    }

    let mut new_atoms: ScratchVec<'scratch, Atom<'arena>, S> = other;
    for (lower, upper) in merged {
        let atom = match (lower, upper) {
            (None, None) => INT,
            (Some(lower_value), Some(upper_value)) if lower_value == upper_value => Atom::int_literal(lower_value),
            _ => builder.int_range(lower, upper),
        };
        new_atoms.push(atom);
    }
    *atoms = new_atoms;
}

/// Drop integer literals and add the broad `int` form when the
/// distinct-literal count exceeds `threshold`.
pub fn apply_int_literal_collapse<S>(atoms: &mut ScratchVec<'_, Atom<'_>, S>, threshold: u16)
where
    S: Arena,
{
    if atoms.contains(&INT) {
        return;
    }

    let count = atoms.iter().filter(|atom| matches!(atom, Atom::Int(IntAtom::Literal(_)))).count();

    if count <= usize::from(threshold) {
        return;
    }

    atoms.retain(|atom| !matches!(atom, Atom::Int(IntAtom::Literal(_))));
    let position = atoms.binary_search(&INT).unwrap_or_else(|insertion| insertion);
    atoms.insert(position, INT);
}
