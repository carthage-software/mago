use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_hir::ir::expression::Call;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::atom::payload::array::ArrayFlag;
use mago_oracle::ty::atom::payload::array::ArrayKey;
use mago_oracle::ty::atom::payload::array::ListFlag;
use mago_oracle::ty::atom::payload::callable::CallableAtom;
use mago_oracle::ty::well_known::ARRAY_KEY;
use mago_oracle::ty::well_known::INT;
use mago_oracle::ty::well_known::NON_NEGATIVE_INT;
use mago_oracle::ty::well_known::STRING;

use crate::extension::ExtensionContext;
use crate::extension::inference::stdlib::literal_int;
use crate::extension::inference::stdlib::literal_string;
use crate::extension::inference::stdlib::nth_argument;
use crate::extension::inference::stdlib::positional_arguments;
use crate::flow::Flow;

pub(super) fn element_count<'arena, A: Arena>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    argument: Option<Type<'arena>>,
) -> Type<'arena> {
    match argument.and_then(count_bounds) {
        Some((min, Some(max))) if min == max => context.int(min),
        Some((min, max)) => context.int_range(Some(min), max),
        None => context.union(&[NON_NEGATIVE_INT]),
    }
}

fn count_bounds(ty: Type<'_>) -> Option<(i64, Option<i64>)> {
    match ty.atoms {
        [Atom::List(list)] => {
            let known = list.known_elements.unwrap_or(&[]);
            let required = known.iter().filter(|element| !element.optional).count() as i64;
            let sealed = list.element_type.is_never();
            let non_empty = list.flags.contains(ListFlag::NonEmpty);

            Some((required.max(i64::from(non_empty)), sealed.then_some(known.len() as i64)))
        }
        [Atom::Array(array)] => {
            let known = array.known_items.unwrap_or(&[]);
            let required = known.iter().filter(|item| !item.optional).count() as i64;
            let non_empty = array.flags.contains(ArrayFlag::NonEmpty);

            Some((required.max(i64::from(non_empty)), array.is_sealed().then_some(known.len() as i64)))
        }
        _ => None,
    }
}

pub(super) fn array_keys<'arena, A: Arena>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    argument: Option<Type<'arena>>,
) -> Option<Type<'arena>> {
    match argument?.atoms {
        [Atom::Array(array)] if array.is_sealed() => {
            let items = array.known_items.unwrap_or(&[]);
            if items.iter().any(|item| item.optional) {
                return None;
            }

            let arena = context.arena();
            let mut keys = Vec::with_capacity_in(items.len(), arena);
            for item in items {
                keys.push(array_key_type(context, item.key));
            }

            Some(context.list(&keys, !keys.is_empty()))
        }
        [Atom::Array(array)] if array.known_items.is_none() => {
            Some(context.list_of(array.key_param?, array.flags.contains(ArrayFlag::NonEmpty)))
        }
        [Atom::List(list)] if list.element_type.is_never() => {
            let known = list.known_elements.unwrap_or(&[]);
            if known.iter().any(|element| element.optional) {
                return None;
            }

            let arena = context.arena();
            let mut keys = Vec::with_capacity_in(known.len(), arena);
            for index in 0..known.len() {
                keys.push(context.int(index as i64));
            }

            Some(context.list(&keys, !keys.is_empty()))
        }
        [Atom::List(list)] => {
            let index = context.union(&[NON_NEGATIVE_INT]);
            Some(context.list_of(index, list.flags.contains(ListFlag::NonEmpty)))
        }
        _ => None,
    }
}

pub(super) fn array_values<'arena, A: Arena>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    argument: Option<Type<'arena>>,
) -> Option<Type<'arena>> {
    let [Atom::Array(array)] = argument?.atoms else {
        return None;
    };

    if array.is_sealed() {
        let items = array.known_items.unwrap_or(&[]);
        if items.iter().any(|item| item.optional) {
            return None;
        }

        let arena = context.arena();
        let mut values = Vec::with_capacity_in(items.len(), arena);
        for item in items {
            values.push(item.value);
        }

        return Some(context.list(&values, !values.is_empty()));
    }

    if array.known_items.is_none() {
        return Some(context.list_of(array.value_param?, array.flags.contains(ArrayFlag::NonEmpty)));
    }

    None
}

pub(super) fn array_map<'arena, A: Arena>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    call: &Call<'arena, SymbolId, Flow, Type<'arena>>,
) -> Option<Type<'arena>> {
    let arguments = positional_arguments(context.arena(), call);
    let [callback, array] = arguments.as_slice() else {
        return None;
    };

    if matches!(callback.atoms, [Atom::Null]) {
        return Some(*array);
    }

    let return_type = callable_return(*callback)?;
    context.remap_array_values(*array, return_type)
}

pub(super) fn array_filter<'arena, A: Arena>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    call: &Call<'arena, SymbolId, Flow, Type<'arena>>,
) -> Option<Type<'arena>> {
    let array = nth_argument(call, 0)?;
    let has_callback = nth_argument(call, 1).is_some_and(|callback| !matches!(callback.atoms, [Atom::Null]));

    context.filter_array(array, !has_callback)
}

pub(super) fn array_reverse<'arena, A: Arena>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    call: &Call<'arena, SymbolId, Flow, Type<'arena>>,
) -> Option<Type<'arena>> {
    if nth_argument(call, 1).is_some() {
        return None;
    }

    context.reverse_list(nth_argument(call, 0)?)
}

pub(super) fn array_flip<'arena, A: Arena>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    call: &Call<'arena, SymbolId, Flow, Type<'arena>>,
) -> Option<Type<'arena>> {
    match nth_argument(call, 0)?.atoms {
        [Atom::Array(array)] if array.known_items.is_none() => {
            let (key, value) = (array.key_param?, array.value_param?);
            if !is_array_key_type(value) {
                return None;
            }

            Some(context.keyed(value, key, array.flags.contains(ArrayFlag::NonEmpty)))
        }
        [Atom::List(list)] if list.known_elements.is_none() => {
            if !is_array_key_type(list.element_type) {
                return None;
            }

            let value = context.union(&[NON_NEGATIVE_INT]);
            Some(context.keyed(list.element_type, value, list.flags.contains(ListFlag::NonEmpty)))
        }
        _ => None,
    }
}

pub(super) fn array_merge<'arena, A: Arena>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    call: &Call<'arena, SymbolId, Flow, Type<'arena>>,
) -> Option<Type<'arena>> {
    let arena = context.arena();
    let arguments = positional_arguments(arena, call);
    if arguments.is_empty() {
        return None;
    }

    let mut all_lists = true;
    let mut all_sealed = true;
    let mut non_empty = false;
    let mut ordered = Vec::new_in(arena);
    let mut values = Vec::new_in(arena);
    let mut keys = Vec::new_in(arena);

    for argument in &arguments {
        match argument.atoms {
            [Atom::List(list)] => {
                non_empty |= list.flags.contains(ListFlag::NonEmpty);
                if !list.element_type.is_never() {
                    all_sealed = false;
                    values.extend_from_slice(list.element_type.atoms);
                }
                if let Some(known) = list.known_elements {
                    for element in known {
                        ordered.push(element.value);
                        values.extend_from_slice(element.value.atoms);
                        non_empty |= !element.optional;
                    }
                }
                keys.push(INT);
            }
            [Atom::Array(array)] => {
                if array.known_items.is_none() && array.is_sealed() {
                    continue;
                }

                all_lists = false;
                non_empty |= array.flags.contains(ArrayFlag::NonEmpty);
                if let Some(items) = array.known_items {
                    for item in items {
                        values.extend_from_slice(item.value.atoms);
                        keys.push(array_key_atom(item.key));
                        non_empty |= !item.optional;
                    }
                }
                if let (Some(key), Some(value)) = (array.key_param, array.value_param) {
                    keys.extend_from_slice(key.atoms);
                    values.extend_from_slice(value.atoms);
                }
            }
            _ => return None,
        }
    }

    if all_lists {
        if all_sealed {
            return Some(context.list(&ordered, non_empty || !ordered.is_empty()));
        }
        if values.is_empty() {
            return None;
        }

        let value = context.union(&values);
        return Some(context.list_of(value, non_empty));
    }

    if values.is_empty() {
        return None;
    }

    let key = if keys.is_empty() { context.union(&[ARRAY_KEY]) } else { context.union(&keys) };
    let value = context.union(&values);
    Some(context.keyed(key, value, non_empty))
}

fn array_key_atom(key: ArrayKey<'_>) -> Atom<'static> {
    match key {
        ArrayKey::Int(_) => INT,
        ArrayKey::String(_) => STRING,
        ArrayKey::Const { .. } => ARRAY_KEY,
    }
}

pub(super) fn array_column<'arena, A: Arena>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    call: &Call<'arena, SymbolId, Flow, Type<'arena>>,
) -> Option<Type<'arena>> {
    let row = outer_value_type(nth_argument(call, 0)?)?;
    let [Atom::Array(keyed)] = row.atoms else {
        return None;
    };
    let known = keyed.known_items?;

    let column_key = nth_argument(call, 1)?;
    let column_type = if matches!(column_key.atoms, [Atom::Null]) {
        row
    } else {
        let key = literal_array_key(column_key)?;
        known.iter().find(|item| item.key == key)?.value
    };

    let index_type = match nth_argument(call, 2) {
        None => None,
        Some(index) if matches!(index.atoms, [Atom::Null]) => None,
        Some(index) => {
            let key = literal_array_key(index)?;
            let value = known.iter().find(|item| item.key == key)?.value;
            if !is_array_key_type(value) {
                return None;
            }

            Some(value)
        }
    };

    match index_type {
        Some(index) => Some(context.keyed(index, column_type, false)),
        None => Some(context.list_of(column_type, false)),
    }
}

fn outer_value_type<'arena>(ty: Type<'arena>) -> Option<Type<'arena>> {
    match ty.atoms {
        [Atom::List(list)] if !list.element_type.is_never() => Some(list.element_type),
        [Atom::Array(array)] => array.value_param,
        _ => None,
    }
}

fn literal_array_key<'arena>(ty: Type<'arena>) -> Option<ArrayKey<'arena>> {
    if let Some(value) = literal_string(ty) {
        return Some(ArrayKey::String(value));
    }
    if let Some(value) = literal_int(ty) {
        return Some(ArrayKey::Int(value));
    }

    None
}

fn is_array_key_type(ty: Type<'_>) -> bool {
    !ty.atoms.is_empty() && ty.atoms.iter().all(|atom| matches!(atom, Atom::Int(_) | Atom::String(_) | Atom::ArrayKey))
}

fn callable_return<'arena>(ty: Type<'arena>) -> Option<Type<'arena>> {
    match ty.atoms {
        [Atom::Callable(CallableAtom::Closure(signature) | CallableAtom::Signature(signature))] => {
            Some(signature.return_type)
        }
        _ => None,
    }
}

fn array_key_type<'arena, A: Arena>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    key: ArrayKey<'arena>,
) -> Type<'arena> {
    match key {
        ArrayKey::Int(value) => context.int(value),
        ArrayKey::String(value) => context.string(value),
        ArrayKey::Const { .. } => context.union(&[ARRAY_KEY]),
    }
}
