//! `List` and unsealed `Array` (keyed) family meet rules.

use std::collections::BTreeMap;
use std::num::NonZeroU32;

use mago_allocator::Arena;
use mago_allocator::vec::Vec as ScratchVec;
use mago_flags::U8Flags;

use crate::symbol::SymbolTable;
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
use crate::ty::lattice::LatticeOptions;
use crate::ty::lattice::LatticeReport;
use crate::ty::lattice::is_uninhabited;
use crate::ty::meet;
use crate::ty::well_known;

/// `list<A> ∧ list<B>` is `list<A ∧ B>` (covariant). When either side
/// is non-empty the result is non-empty too. Sealed × sealed lists
/// merge index-wise; sealed × unsealed treats the unsealed side as
/// the rest type for indices beyond the sealed prefix.
pub(in crate::ty::meet) fn list_meet<'arena, S, A>(
    a: Atom<'arena>,
    b: Atom<'arena>,
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Option<Atom<'arena>>
where
    S: Arena,
    A: Arena,
{
    let (Atom::List(a_payload), Atom::List(b_payload)) = (a, b) else {
        return None;
    };

    if a_payload.known_elements.is_some() || b_payload.known_elements.is_some() {
        return sealed_list_meet(*a_payload, *b_payload, symbols, options, report, builder);
    }

    let element_type = meet::compute(a_payload.element_type, b_payload.element_type, symbols, options, report, builder);
    let non_empty = a_payload.flags.contains(ListFlag::NonEmpty) || b_payload.flags.contains(ListFlag::NonEmpty);
    if non_empty && element_type.is_never() {
        return None;
    }

    let mut flags = U8Flags::empty();
    flags.set_value(ListFlag::NonEmpty, non_empty);
    let merged = ListAtom { element_type, known_elements: None, known_count: None, flags };

    let result = builder.list(merged);

    if is_uninhabited(result, symbols, builder) { None } else { Some(result) }
}

#[inline]
fn sealed_list_meet<'scratch, 'arena, S, A>(
    a_payload: ListAtom<'arena>,
    b_payload: ListAtom<'arena>,
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) -> Option<Atom<'arena>>
where
    S: Arena,
    A: Arena,
{
    let a_entries: &[KnownElement<'arena>] = a_payload.known_elements.unwrap_or(&[]);
    let b_entries: &[KnownElement<'arena>] = b_payload.known_elements.unwrap_or(&[]);
    let max_index = a_entries.len().max(b_entries.len());
    let mut merged: ScratchVec<'scratch, KnownElement<'arena>, S> = builder.scratch_vec_with(max_index);
    for index in 0..max_index {
        let a_entry = a_entries.get(index).copied();
        let b_entry = b_entries.get(index).copied();
        let (value, optional) = match (a_entry, b_entry) {
            (Some(a_element), Some(b_element)) => (
                meet::compute(a_element.value, b_element.value, symbols, options, report, builder),
                a_element.optional && b_element.optional,
            ),
            (Some(a_element), None) => (
                meet::compute(a_element.value, b_payload.element_type, symbols, options, report, builder),
                a_element.optional,
            ),
            (None, Some(b_element)) => (
                meet::compute(a_payload.element_type, b_element.value, symbols, options, report, builder),
                b_element.optional,
            ),
            (None, None) => continue,
        };

        if !optional && value.is_never() {
            return None;
        }

        merged.push(KnownElement { index: index as u32, value, optional });
    }

    let known_elements = if merged.is_empty() { None } else { Some(builder.known_elements(&merged)) };
    let non_empty = a_payload.flags.contains(ListFlag::NonEmpty) || b_payload.flags.contains(ListFlag::NonEmpty);
    let known_count = NonZeroU32::new(merged.len() as u32);
    let element_type = meet::compute(a_payload.element_type, b_payload.element_type, symbols, options, report, builder);
    let mut flags = U8Flags::empty();
    flags.set_value(ListFlag::NonEmpty, non_empty);
    let merged_payload = ListAtom { element_type, known_elements, known_count, flags };

    let result = builder.list(merged_payload);

    if is_uninhabited(result, symbols, builder) { None } else { Some(result) }
}

/// `array{...} ∧ array{...}` for two sealed shapes: the result has the
/// union of keys; values at shared keys are met. Optional flags AND-merge
/// (a key is required iff it's required on both sides). Unsealed × unsealed
/// composes the open key/value parameters pointwise.
pub(in crate::ty::meet) fn keyed_array_meet<'scratch, 'arena, S, A>(
    a: Atom<'arena>,
    b: Atom<'arena>,
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) -> Option<Atom<'arena>>
where
    S: Arena,
    A: Arena,
{
    let (Atom::Array(a_payload), Atom::Array(b_payload)) = (a, b) else {
        return None;
    };

    let (Some(a_entries), Some(b_entries)) = (a_payload.known_items, b_payload.known_items) else {
        if a_payload.known_items.is_some() || b_payload.known_items.is_some() {
            return sealed_unsealed_array_meet(*a_payload, *b_payload, symbols, options, report, builder);
        }

        return unsealed_keyed_array_meet(*a_payload, *b_payload, symbols, options, report, builder);
    };

    let mut merged: BTreeMap<ArrayKey<'arena>, KnownItem<'arena>> = BTreeMap::default();
    for entry in a_entries {
        merged.insert(entry.key, *entry);
    }

    for b_entry in b_entries {
        merged
            .entry(b_entry.key)
            .and_modify(|existing| {
                existing.value = meet::compute(existing.value, b_entry.value, symbols, options, report, builder);
                existing.optional = existing.optional && b_entry.optional;
            })
            .or_insert(*b_entry);
    }

    let mut entries: ScratchVec<'scratch, KnownItem<'arena>, S> = builder.scratch_vec();
    entries.extend(merged.into_values());
    let known_items = builder.known_items(&entries);
    let non_empty = a_payload.flags.contains(ArrayFlag::NonEmpty) || b_payload.flags.contains(ArrayFlag::NonEmpty);
    let mut flags = U8Flags::empty();
    flags.set_value(ArrayFlag::NonEmpty, non_empty);
    let merged_payload = ArrayAtom { key_param: None, value_param: None, known_items: Some(known_items), flags };

    let result = builder.array(merged_payload);

    if is_uninhabited(result, symbols, builder) { None } else { Some(result) }
}

/// `array{...} ∧ array<K, V>`: each known item's value gets met
/// against `V`, the key is checked against `K` (a key outside `K`
/// makes that item's value `never`, dropping or contradicting the
/// item depending on `optional`).
#[inline]
fn sealed_unsealed_array_meet<'scratch, 'arena, S, A>(
    a_payload: ArrayAtom<'arena>,
    b_payload: ArrayAtom<'arena>,
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) -> Option<Atom<'arena>>
where
    S: Arena,
    A: Arena,
{
    let (sealed, unsealed) =
        if a_payload.known_items.is_some() { (a_payload, b_payload) } else { (b_payload, a_payload) };
    let key_param = unsealed.key_param;
    let value_param = unsealed.value_param;
    let entries = sealed.known_items?;
    let mut merged: ScratchVec<'scratch, KnownItem<'arena>, S> = builder.scratch_vec_with(entries.len());
    for entry in entries {
        let key_atom = match entry.key {
            ArrayKey::Int(value) => Atom::int_literal(value),
            ArrayKey::String(name) => builder.string_literal(name),
            ArrayKey::Const { .. } => well_known::ARRAY_KEY,
        };

        let key_type = builder.union_of(&[key_atom]);
        let key_compatible = key_param.is_none_or(|key_constraint| {
            crate::ty::lattice::refines(key_type, key_constraint, symbols, options, report, builder)
        });
        let value = if let Some(value_constraint) = value_param {
            meet::compute(entry.value, value_constraint, symbols, options, report, builder)
        } else {
            entry.value
        };

        if !key_compatible || value.is_never() {
            if entry.optional {
                continue;
            }

            return None;
        }

        merged.push(KnownItem { value, ..*entry });
    }

    let known_items = if merged.is_empty() { None } else { Some(builder.known_items(&merged)) };
    let non_empty = sealed.flags.contains(ArrayFlag::NonEmpty) || unsealed.flags.contains(ArrayFlag::NonEmpty);
    let mut flags = U8Flags::empty();
    flags.set_value(ArrayFlag::NonEmpty, non_empty);
    let result = builder.array(ArrayAtom { key_param: None, value_param: None, known_items, flags });

    if is_uninhabited(result, symbols, builder) { None } else { Some(result) }
}

/// `list<E> ∧ array<K, V>`: a list is an int-keyed array, so the meet
/// is a list whose element type is `E ∧ V` and whose key constraint
/// must be compatible with `int`. When `K` excludes integers
/// (e.g. `string`), the intersection is empty. The non-empty flag
/// OR-merges; an empty result on either axis collapses to `None`
/// when the result is forced non-empty.
pub(in crate::ty::meet) fn list_array_meet<'arena, S, A>(
    a: Atom<'arena>,
    b: Atom<'arena>,
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Option<Atom<'arena>>
where
    S: Arena,
    A: Arena,
{
    let (list_atom, array_atom) = if a.kind() == AtomKind::List { (a, b) } else { (b, a) };
    let (Atom::List(list_payload), Atom::Array(array_payload)) = (list_atom, array_atom) else {
        return None;
    };

    if list_payload.known_elements.is_some() || array_payload.known_items.is_some() {
        return None;
    }

    let non_empty =
        list_payload.flags.contains(ListFlag::NonEmpty) || array_payload.flags.contains(ArrayFlag::NonEmpty);

    let array_is_sealed_empty =
        array_payload.key_param.is_none() && array_payload.value_param.is_none() && array_payload.known_items.is_none();
    if array_is_sealed_empty {
        return if list_payload.flags.contains(ListFlag::NonEmpty) { None } else { Some(well_known::EMPTY_ARRAY) };
    }

    let key_compatible = array_payload.key_param.is_none_or(|key_constraint| {
        crate::ty::lattice::refines(well_known::TYPE_INT, key_constraint, symbols, options, report, builder)
    });

    if non_empty && !key_compatible {
        return None;
    }

    let array_value_param = array_payload.value_param.unwrap_or(well_known::TYPE_MIXED);
    let element_type = meet::compute(list_payload.element_type, array_value_param, symbols, options, report, builder);

    if non_empty && element_type.is_never() {
        return None;
    }

    if !key_compatible {
        return Some(well_known::EMPTY_ARRAY);
    }

    let mut flags = U8Flags::empty();
    flags.set_value(ListFlag::NonEmpty, non_empty);

    Some(builder.list(ListAtom { element_type, known_elements: None, known_count: None, flags }))
}

/// `iterable<K, V> ∧ array<K', V', items?>` narrows the array's
/// key/value parameters with the iterable's. The result is still an
/// array shape (the array is the more refined family member); the
/// iterable side is consumed entirely. `known_items` value types
/// also narrow against the iterable's value type.
pub(in crate::ty::meet) fn iterable_array_meet<'scratch, 'arena, S, A>(
    iterable: Atom<'arena>,
    array: Atom<'arena>,
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'scratch, 'arena, S, A>,
) -> Option<Atom<'arena>>
where
    S: Arena,
    A: Arena,
{
    let (Atom::Iterable(iterable_payload), Atom::Array(array_payload)) = (iterable, array) else {
        return None;
    };

    let key_param = match array_payload.key_param {
        Some(array_key) => Some(meet::compute(array_key, iterable_payload.key_type, symbols, options, report, builder)),
        None => Some(iterable_payload.key_type),
    };

    let value_param = match array_payload.value_param {
        Some(array_value) => {
            Some(meet::compute(array_value, iterable_payload.value_type, symbols, options, report, builder))
        }
        None => Some(iterable_payload.value_type),
    };

    let known_items = match array_payload.known_items {
        None => None,
        Some(entries) => {
            let mut narrowed: ScratchVec<'scratch, KnownItem<'arena>, S> = builder.scratch_vec_with(entries.len());
            for entry in entries {
                let value = meet::compute(entry.value, iterable_payload.value_type, symbols, options, report, builder);
                if value.is_never() && !entry.optional {
                    return None;
                }

                narrowed.push(KnownItem { value, ..*entry });
            }

            Some(builder.known_items(&narrowed))
        }
    };

    if array_payload.flags.contains(ArrayFlag::NonEmpty) {
        let key_empty = key_param.is_some_and(|ty| ty.is_never());
        let value_empty = value_param.is_some_and(|ty| ty.is_never());
        if key_empty || value_empty {
            return None;
        }
    }

    Some(builder.array(ArrayAtom { key_param, value_param, known_items, flags: array_payload.flags }))
}

/// `iterable<K, V> ∧ list<E>` narrows the list's element type by the
/// iterable's value type. A list has implicit `int` keys, so a non-empty
/// list only inhabits the iterable when `int <: K`. When `int` doesn't fit
/// `K`, the only shared value is the empty list `{[]}` (an empty iterator
/// inhabits every `iterable<K, V>`): a possibly-empty list meets to the
/// empty list `list<never>`, while a non-empty list has no shared value and
/// the meet is `None`. The matching overlap rule reports the same to keep
/// the lattice consistent.
pub(in crate::ty::meet) fn iterable_list_meet<'arena, S, A>(
    iterable: Atom<'arena>,
    list: Atom<'arena>,
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Option<Atom<'arena>>
where
    S: Arena,
    A: Arena,
{
    let (Atom::Iterable(iterable_payload), Atom::List(list_payload)) = (iterable, list) else {
        return None;
    };

    if !crate::ty::lattice::refines(well_known::TYPE_INT, iterable_payload.key_type, symbols, options, report, builder)
    {
        if list_payload.flags.contains(ListFlag::NonEmpty) {
            return None;
        }

        return Some(builder.list(ListAtom {
            element_type: well_known::TYPE_NEVER,
            known_elements: None,
            known_count: None,
            flags: U8Flags::empty(),
        }));
    }

    let element_type =
        meet::compute(list_payload.element_type, iterable_payload.value_type, symbols, options, report, builder);
    if list_payload.flags.contains(ListFlag::NonEmpty) && element_type.is_never() {
        return None;
    }

    Some(builder.list(ListAtom { element_type, ..*list_payload }))
}

/// Unsealed × unsealed keyed-array meet. A sealed-empty side (`array{}`:
/// no parameters, no known items) admits only the empty array, so
/// meeting it with a non-empty side is `None` and with anything else is
/// exactly `array{}` regardless of the other side's open parameters.
#[inline]
fn unsealed_keyed_array_meet<'arena, S, A>(
    a_payload: ArrayAtom<'arena>,
    b_payload: ArrayAtom<'arena>,
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Option<Atom<'arena>>
where
    S: Arena,
    A: Arena,
{
    let non_empty = a_payload.flags.contains(ArrayFlag::NonEmpty) || b_payload.flags.contains(ArrayFlag::NonEmpty);

    let a_sealed_empty = a_payload.is_sealed() && a_payload.known_items.is_none();
    let b_sealed_empty = b_payload.is_sealed() && b_payload.known_items.is_none();
    if a_sealed_empty || b_sealed_empty {
        return if non_empty { None } else { Some(well_known::EMPTY_ARRAY) };
    }

    let key_param = match (a_payload.key_param, b_payload.key_param) {
        (Some(a_key), Some(b_key)) => Some(meet::compute(a_key, b_key, symbols, options, report, builder)),
        (Some(key), None) | (None, Some(key)) => Some(key),
        (None, None) => None,
    };

    let value_param = match (a_payload.value_param, b_payload.value_param) {
        (Some(a_value), Some(b_value)) => Some(meet::compute(a_value, b_value, symbols, options, report, builder)),
        (Some(value), None) | (None, Some(value)) => Some(value),
        (None, None) => None,
    };

    if non_empty {
        let key_empty = key_param.is_some_and(|ty| ty.is_never());
        let value_empty = value_param.is_some_and(|ty| ty.is_never());
        if key_empty || value_empty {
            return None;
        }
    }

    let mut flags = U8Flags::empty();
    flags.set_value(ArrayFlag::NonEmpty, non_empty);

    Some(builder.array(ArrayAtom { key_param, value_param, known_items: None, flags }))
}
