//! Array-family join: keyed-array param merge, shape merge, shape
//! collapse, empty-array overwrite, and int-keyed → list rewrite.

use std::collections::BTreeSet;
use std::collections::HashMap;
use std::num::NonZeroU32;

use mago_allocator::Arena;
use mago_flags::U8Flags;

use crate::ty::atom::Atom;
use crate::ty::atom::kind::AtomKind;
use crate::ty::atom::payload::array::ArrayAtom;
use crate::ty::atom::payload::array::ArrayFlag;
use crate::ty::atom::payload::array::ArrayKey;
use crate::ty::atom::payload::array::KnownElement;
use crate::ty::atom::payload::array::KnownItem;
use crate::ty::atom::payload::array::ListAtom;
use crate::ty::atom::payload::array::ListFlag;
use crate::ty::builder::TypeBuilder;
use crate::ty::well_known::ARRAY_KEY_MIXED;
use crate::ty::well_known::EMPTY_ARRAY;
use crate::ty::well_known::TYPE_NEVER;

/// Merge multiple unsealed keyed arrays with the same `non_empty` flag
/// into a single keyed array with unioned key+value parameters. Sealed
/// keyed arrays (with `known_items`) are left to
/// [`apply_merge_array_shapes`].
pub fn apply_merge_keyed_array_params<'arena, S, A>(
    atoms: &mut Vec<Atom<'arena>>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) where
    S: Arena,
    A: Arena,
{
    let mut groups: HashMap<bool, Vec<usize>> = HashMap::new();
    for (index, atom) in atoms.iter().enumerate() {
        let Atom::Array(payload) = atom else {
            continue;
        };
        if payload.known_items.is_some() || payload.key_param.is_none() || payload.value_param.is_none() {
            continue;
        }
        groups.entry(payload.flags.contains(ArrayFlag::NonEmpty)).or_default().push(index);
    }

    let mut to_remove: BTreeSet<usize> = BTreeSet::new();
    for (non_empty, indices) in &groups {
        if indices.len() < 2 {
            continue;
        }
        let mut key_atoms: Vec<Atom<'arena>> = Vec::new();
        let mut value_atoms: Vec<Atom<'arena>> = Vec::new();
        for &index in indices {
            let Atom::Array(payload) = atoms[index] else {
                continue;
            };
            if let (Some(key_param), Some(value_param)) = (payload.key_param, payload.value_param) {
                key_atoms.extend_from_slice(key_param.atoms);
                value_atoms.extend_from_slice(value_param.atoms);
            }
        }
        let key_canonical = super::super::compute(&key_atoms, builder);
        let value_canonical = super::super::compute(&value_atoms, builder);
        let key_type = builder.union_of(&key_canonical);
        let value_type = builder.union_of(&value_canonical);
        let merged_array = builder.keyed_unsealed(key_type, value_type, *non_empty);
        atoms[indices[0]] = merged_array;
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

/// When the union has more than `threshold` array shapes, replace
/// them all with the general `array<array-key, mixed>` form.
pub fn apply_array_shape_collapse(atoms: &mut Vec<Atom<'_>>, threshold: u16) {
    let shape_count = atoms
        .iter()
        .filter(|atom| matches!(atom.kind(), AtomKind::Array | AtomKind::List) && **atom != EMPTY_ARRAY)
        .count();

    if shape_count <= usize::from(threshold) {
        return;
    }

    atoms.retain(|atom| !(matches!(atom.kind(), AtomKind::Array | AtomKind::List) && *atom != EMPTY_ARRAY));

    let position = atoms.binary_search(&ARRAY_KEY_MIXED).unwrap_or_else(|insertion| insertion);
    atoms.insert(position, ARRAY_KEY_MIXED);
}

/// Drop `EMPTY_ARRAY` from the union when another `Array` or `List`
/// atom is present.
pub fn apply_overwrite_empty_array(atoms: &mut Vec<Atom<'_>>) {
    if !atoms.iter().any(|atom| matches!(atom.kind(), AtomKind::Array | AtomKind::List)) {
        return;
    }

    let has_other_array =
        atoms.iter().any(|atom| *atom != EMPTY_ARRAY && matches!(atom.kind(), AtomKind::Array | AtomKind::List));
    if has_other_array {
        atoms.retain(|atom| *atom != EMPTY_ARRAY);
    }
}

/// Detect keyed-array atoms whose `known_items` use contiguous integer
/// keys `0..n-1` (and whose key/value rest types are absent or
/// list-compatible) and rewrite them as `List` atoms.
pub fn apply_rewrite_int_keyed_to_list<'arena, S, A>(
    atoms: &mut [Atom<'arena>],
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) where
    S: Arena,
    A: Arena,
{
    for atom in atoms.iter_mut() {
        let Atom::Array(payload) = *atom else {
            continue;
        };
        if payload.key_param.is_some() {
            continue;
        }
        let Some(entries) = payload.known_items else {
            continue;
        };
        let mut indexed: Vec<(i64, KnownItem<'arena>)> = Vec::with_capacity(entries.len());
        let mut all_int = true;
        for entry in entries {
            match entry.key {
                ArrayKey::Int(key) => indexed.push((key, *entry)),
                _ => {
                    all_int = false;
                    break;
                }
            }
        }
        if !all_int {
            continue;
        }
        indexed.sort_by_key(|(key, _)| *key);
        if !(0..indexed.len()).all(|position| indexed[position].0 == position as i64) {
            continue;
        }

        let known_elements: Vec<KnownElement<'arena>> = indexed
            .iter()
            .map(|(key, entry)| KnownElement { index: *key as u32, value: entry.value, optional: entry.optional })
            .collect();
        let known_count = NonZeroU32::new(known_elements.len() as u32);
        let known_elements = builder.known_elements(&known_elements);
        let mut flags = U8Flags::empty();
        flags.set_value(ListFlag::NonEmpty, payload.flags.contains(ArrayFlag::NonEmpty));

        *atom = builder.list(ListAtom {
            element_type: payload.value_param.unwrap_or(TYPE_NEVER),
            known_elements: Some(known_elements),
            known_count,
            flags,
        });
    }
}

/// When the union contains multiple keyed-array atoms that share at
/// least one literal key, fold them into a single shape whose value
/// at every shared key is the union of the source values.
pub fn apply_merge_array_shapes<'arena, S, A>(
    atoms: &mut Vec<Atom<'arena>>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) where
    S: Arena,
    A: Arena,
{
    let mut shapes: Vec<usize> = atoms
        .iter()
        .enumerate()
        .filter_map(|(index, atom)| {
            matches!(atom, Atom::Array(payload) if payload.known_items.is_some()).then_some(index)
        })
        .collect();

    if shapes.len() < 2 {
        return;
    }

    let head_index = shapes.remove(0);
    let Atom::Array(head_payload) = atoms[head_index] else {
        return;
    };
    let Some(head_known) = head_payload.known_items else {
        return;
    };
    let mut new_known: Vec<KnownItem<'arena>> = head_known.to_vec();
    let mut absorbed: Vec<usize> = Vec::new();
    let mut accumulated_non_empty = head_payload.flags.contains(ArrayFlag::NonEmpty);

    for &shape_index in &shapes {
        let Atom::Array(other) = atoms[shape_index] else {
            continue;
        };
        if other.key_param != head_payload.key_param || other.value_param != head_payload.value_param {
            continue;
        }

        let Some(other_entries) = other.known_items else {
            continue;
        };
        let shares_key =
            other_entries.iter().any(|other_entry| new_known.iter().any(|entry| entry.key == other_entry.key));
        if !shares_key {
            continue;
        }

        for other_entry in other_entries {
            if let Some(existing) = new_known.iter_mut().find(|entry| entry.key == other_entry.key) {
                let mut merged_atoms: Vec<Atom<'arena>> = existing.value.atoms.to_vec();
                merged_atoms.extend_from_slice(other_entry.value.atoms);
                existing.value = builder.union_of(&merged_atoms);
                existing.optional = existing.optional || other_entry.optional;
            } else {
                new_known.push(*other_entry);
            }
        }

        accumulated_non_empty = accumulated_non_empty || other.flags.contains(ArrayFlag::NonEmpty);
        absorbed.push(shape_index);
    }

    if absorbed.is_empty() {
        return;
    }

    new_known.sort_by_key(|entry| entry.key);
    let known_items = builder.known_items(&new_known);
    let mut flags = U8Flags::empty();
    flags.set_value(ArrayFlag::NonEmpty, accumulated_non_empty);

    atoms[head_index] = builder.array(ArrayAtom { known_items: Some(known_items), flags, ..*head_payload });

    let mut absorbed_set: BTreeSet<usize> = absorbed.into_iter().collect();
    let mut index = 0;
    atoms.retain(|_| {
        let keep = !absorbed_set.remove(&index);
        index += 1;
        keep
    });
}
