//! List-family join: merge unsealed lists of the same `non_empty`
//! flag into a single list whose element type is the type-union of
//! theirs, plus merge fixed-shape sealed lists that differ in a single
//! position.

use std::collections::BTreeSet;

use mago_allocator::Arena;
use mago_allocator::collections::HashMap;
use mago_allocator::vec::Vec as ScratchVec;

use crate::ty::atom::Atom;
use crate::ty::atom::payload::array::KnownElement;
use crate::ty::atom::payload::array::ListAtom;
use crate::ty::atom::payload::array::ListFlag;
use crate::ty::builder::TypeBuilder;

/// Merge multiple unsealed lists with the same `non_empty` flag into a
/// single list whose element type is the type-union of theirs. Sealed
/// lists (those with `known_elements`) and lists with differing
/// `non_empty` flags are left alone.
pub fn apply_merge_list_element_types<'scratch, 'arena, S, A>(
    atoms: &mut ScratchVec<'scratch, Atom<'arena>, S>,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) where
    S: Arena,
    A: Arena,
{
    let mut groups: HashMap<'scratch, bool, ScratchVec<'scratch, usize, S>, S> = HashMap::new_in(builder.scratch());
    for (index, atom) in atoms.iter().enumerate() {
        let Atom::List(payload) = atom else {
            continue;
        };
        if payload.known_elements.is_some() {
            continue;
        }
        groups.entry(payload.flags.contains(ListFlag::NonEmpty)).or_insert_with(|| builder.scratch_vec()).push(index);
    }

    let mut to_remove: BTreeSet<usize> = BTreeSet::new();
    for (non_empty, indices) in &groups {
        if indices.len() < 2 {
            continue;
        }
        let mut merged_atoms: ScratchVec<'scratch, Atom<'arena>, S> = builder.scratch_vec();
        for &index in indices {
            let Atom::List(payload) = atoms[index] else {
                continue;
            };
            merged_atoms.extend_from_slice(payload.element_type.atoms);
        }
        let merged = super::super::compute(&merged_atoms, builder);
        let union_type = builder.union_of(&merged);
        let merged_list = builder.list_of(union_type, *non_empty);
        atoms[indices[0]] = merged_list;
        for &index in &indices[1..] {
            to_remove.insert(index);
        }
    }

    if to_remove.is_empty() {
        return;
    }
    let mut index = 0;
    atoms.retain(|_| {
        let keep = !to_remove.contains(&index);
        index += 1;
        keep
    });
}

/// Merge two fixed-shape sealed lists (`list{…}`) that have the identical
/// shape - same length, same per-position index and optionality, same tail
/// and non-empty flag - and differ in **exactly one** element position, into
/// a single list whose element at that position is the union of theirs.
///
/// This is sound precisely because only one position varies: `list{A, B}`
/// joined with `list{A, C}` is exactly `list{A, B|C}` - factoring the shared
/// position introduces no tuple the union did not already contain. Merging
/// shapes that differ in two or more positions would be unsound (it would
/// admit cross-combinations like `list{A.0, C.1}`), so those are left as
/// distinct members. The pass runs to a fixpoint, so a chain of
/// single-position variants collapses fully.
pub fn apply_merge_sealed_lists<'scratch, 'arena, S, A>(
    atoms: &mut ScratchVec<'scratch, Atom<'arena>, S>,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) where
    S: Arena,
    A: Arena,
{
    loop {
        let mut found: Option<(usize, usize, usize)> = None;
        'search: for left in 0..atoms.len() {
            for right in (left + 1)..atoms.len() {
                if let Some(position) = single_differing_position(atoms[left], atoms[right]) {
                    found = Some((left, right, position));
                    break 'search;
                }
            }
        }

        let Some((left, right, position)) = found else {
            return;
        };

        let merged = merge_lists_at_position(atoms[left], atoms[right], position, builder);
        atoms[left] = merged;
        atoms.remove(right);
    }
}

/// `Some(position)` when `left` and `right` are identically-shaped fixed
/// sealed lists differing in exactly one element position; `None` otherwise.
#[inline]
fn single_differing_position<'arena>(left: Atom<'arena>, right: Atom<'arena>) -> Option<usize> {
    let (Atom::List(left_payload), Atom::List(right_payload)) = (left, right) else {
        return None;
    };
    let (Some(left_elements), Some(right_elements)) = (left_payload.known_elements, right_payload.known_elements)
    else {
        return None;
    };

    if left_payload.flags != right_payload.flags
        || left_payload.known_count != right_payload.known_count
        || left_payload.element_type != right_payload.element_type
        || left_elements.len() != right_elements.len()
    {
        return None;
    }

    let mut differing: Option<usize> = None;
    for (position, (left_element, right_element)) in left_elements.iter().zip(right_elements).enumerate() {
        if left_element.index != right_element.index || left_element.optional != right_element.optional {
            return None;
        }

        if left_element.value != right_element.value {
            if differing.is_some() {
                return None;
            }

            differing = Some(position);
        }
    }

    differing
}

/// Build the merged list: `left`'s shape with the element at `position`
/// replaced by the union of `left`'s and `right`'s element there.
#[inline]
fn merge_lists_at_position<'scratch, 'arena, S, A>(
    left: Atom<'arena>,
    right: Atom<'arena>,
    position: usize,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) -> Atom<'arena>
where
    S: Arena,
    A: Arena,
{
    let (Atom::List(left_payload), Atom::List(right_payload)) = (left, right) else {
        return left;
    };
    let (Some(left_elements), Some(right_elements)) = (left_payload.known_elements, right_payload.known_elements)
    else {
        return left;
    };

    let mut merged_value_atoms: ScratchVec<'scratch, Atom<'arena>, S> =
        builder.scratch_vec_from_slice(left_elements[position].value.atoms);
    merged_value_atoms.extend_from_slice(right_elements[position].value.atoms);
    let merged_value_atoms = super::super::compute(&merged_value_atoms, builder);
    let merged_value = builder.union_of(&merged_value_atoms);

    let mut new_elements: ScratchVec<'scratch, KnownElement<'arena>, S> = builder.scratch_vec_from_slice(left_elements);
    new_elements[position] = KnownElement { value: merged_value, ..left_elements[position] };
    let known_elements = builder.known_elements(&new_elements);

    builder.list(ListAtom { known_elements: Some(known_elements), ..*left_payload })
}
