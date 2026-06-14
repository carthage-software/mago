//! `Array` family subtract: empty-array elimination + key/value
//! narrowing via `Negated` intersection conjuncts.

use mago_allocator::Arena;
use mago_allocator::vec::Vec as ScratchVec;
use mago_flags::U8Flags;

use crate::ty::atom::Atom;
use crate::ty::atom::payload::array::ArrayAtom;
use crate::ty::atom::payload::array::ArrayFlag;
use crate::ty::builder::TypeBuilder;
use crate::ty::well_known::TYPE_NEVER;

/// `array<K1, V1> \ array<K2, V2>` (or against `non-empty-array<K2, V2>`).
/// The empty-array singleton survives when the input allows empty and
/// the removed side doesn't; otherwise drops. The non-empty residue is
/// tightened by attaching `Negated(removed)` when key or value
/// parameters differ, so the lattice can later detect
/// overlap-collapsing intersections like
/// `(non-empty-array<K1, V1> & Negated(array<K2, V2>)) ∩ array<K2, V2>
/// ≡ ⊥`. When the parameters are equal and only the input allows empty,
/// the non-empty values of the input are entirely covered by the
/// removed side, so only the empty piece survives.
pub(in crate::ty::subtract) fn array_minus<'scratch, 'arena, S, A>(
    input: Atom<'arena>,
    removed: Atom<'arena>,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
    out: &mut ScratchVec<'scratch, Atom<'arena>, S>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    let (Atom::Array(input_payload), Atom::Array(removed_payload)) = (input, removed) else {
        return false;
    };

    if input_payload.known_items.is_some() || removed_payload.known_items.is_some() {
        return false;
    }

    let input_allows_empty = !input_payload.flags.contains(ArrayFlag::NonEmpty);
    let removed_allows_empty = !removed_payload.flags.contains(ArrayFlag::NonEmpty);

    if input_allows_empty && !removed_allows_empty {
        let empty = empty_array(builder);
        out.push(empty);
    }

    let non_empty_residue = ArrayAtom { flags: input_payload.flags.with(ArrayFlag::NonEmpty), ..*input_payload };
    if input_payload.key_param != removed_payload.key_param || input_payload.value_param != removed_payload.value_param
    {
        let removed_type = builder.union_of(&[removed]);
        let negated = builder.negated(removed_type);
        let head = builder.array(non_empty_residue);
        let intersected = builder.intersected(head, &[negated]);
        out.push(intersected);
    } else if input_allows_empty == removed_allows_empty {
        let residue = builder.array(non_empty_residue);
        out.push(residue);
    }

    true
}

#[inline]
fn empty_array<'arena, S, A>(builder: &mut TypeBuilder<'_, 'arena, S, A>) -> Atom<'arena>
where
    S: Arena,
    A: Arena,
{
    builder.array(ArrayAtom {
        key_param: Some(TYPE_NEVER),
        value_param: Some(TYPE_NEVER),
        known_items: None,
        flags: U8Flags::empty(),
    })
}

/// `array<K, V> \ iterable<K2, V2>`: symmetric to
/// [`crate::ty::subtract::family::list::list_minus_iterable`].
pub(in crate::ty::subtract) fn array_minus_iterable<'scratch, 'arena, S, A>(
    input: Atom<'arena>,
    removed: Atom<'arena>,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
    out: &mut ScratchVec<'scratch, Atom<'arena>, S>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    let Atom::Array(input_payload) = input else {
        return false;
    };

    if input_payload.known_items.is_some() {
        return false;
    }

    let head = builder.array(ArrayAtom { flags: input_payload.flags.with(ArrayFlag::NonEmpty), ..*input_payload });
    let removed_type = builder.union_of(&[removed]);
    let negated = builder.negated(removed_type);
    let intersected = builder.intersected(head, &[negated]);
    out.push(intersected);
    true
}
