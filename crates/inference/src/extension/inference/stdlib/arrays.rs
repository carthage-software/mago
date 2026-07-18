use mago_allocator::Arena;
use mago_flags::U8Flags;
use mago_hir::ir::expression::Call;
use mago_oracle::assertion::Assertion;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::atom::payload::array::ArrayAtom;
use mago_oracle::ty::atom::payload::array::ArrayFlag;
use mago_oracle::ty::atom::payload::array::ArrayKey;
use mago_oracle::ty::atom::payload::array::KnownElement;
use mago_oracle::ty::atom::payload::array::KnownItem;
use mago_oracle::ty::atom::payload::array::ListAtom;
use mago_oracle::ty::atom::payload::array::ListFlag;
use mago_oracle::ty::atom::payload::callable::CallableAtom;
use mago_oracle::ty::well_known::ARRAY_KEY;
use mago_oracle::ty::well_known::INT;
use mago_oracle::ty::well_known::STRING;

use crate::extension::ExtensionContext;
use crate::extension::inference::stdlib::literal_int;
use crate::extension::inference::stdlib::literal_string;
use crate::extension::inference::stdlib::nth_argument;
use crate::extension::inference::stdlib::positional_arguments;
use crate::flow::Flow;
use crate::reconciler::reconcile;

pub(super) fn element_count<'arena, A>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    argument: Option<Type<'arena>>,
) -> Type<'arena>
where
    A: Arena,
{
    match argument.and_then(count_bounds) {
        Some((min, Some(max))) if min == max => context.ty.int_literal_type(min),
        Some((min, max)) => context.ty.int_range_type(Some(min), max),
        None => context.ty.non_negative_int_type(),
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

pub(super) fn array_keys<'arena, A>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    argument: Option<Type<'arena>>,
) -> Option<Type<'arena>>
where
    A: Arena,
{
    match argument?.atoms {
        [Atom::Array(array)] if array.is_sealed() => {
            let items = array.known_items.unwrap_or(&[]);
            if items.iter().any(|item| item.optional) {
                return None;
            }

            let mut keys = context.ty.scratch_vec_with(items.len());
            for item in items {
                keys.push(array_key_type(context, item.key));
            }

            Some(context.ty.sealed_list_type(&keys, !keys.is_empty()))
        }
        [Atom::Array(array)] if array.known_items.is_none() => {
            Some(context.ty.list_of_type(array.key_param?, array.flags.contains(ArrayFlag::NonEmpty)))
        }
        [Atom::List(list)] if list.element_type.is_never() => {
            let known = list.known_elements.unwrap_or(&[]);
            if known.iter().any(|element| element.optional) {
                return None;
            }

            let mut keys = context.ty.scratch_vec_with(known.len());
            for index in 0..known.len() {
                keys.push(context.ty.int_literal_type(index as i64));
            }

            Some(context.ty.sealed_list_type(&keys, !keys.is_empty()))
        }
        [Atom::List(list)] => {
            let index = context.ty.non_negative_int_type();
            Some(context.ty.list_of_type(index, list.flags.contains(ListFlag::NonEmpty)))
        }
        _ => None,
    }
}

pub(super) fn array_values<'arena, A>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    argument: Option<Type<'arena>>,
) -> Option<Type<'arena>>
where
    A: Arena,
{
    let [Atom::Array(array)] = argument?.atoms else {
        return None;
    };

    if array.is_sealed() {
        let items = array.known_items.unwrap_or(&[]);
        if items.iter().any(|item| item.optional) {
            return None;
        }

        let mut values = context.ty.scratch_vec_with(items.len());
        for item in items {
            values.push(item.value);
        }

        return Some(context.ty.sealed_list_type(&values, !values.is_empty()));
    }

    if array.known_items.is_none() {
        return Some(context.ty.list_of_type(array.value_param?, array.flags.contains(ArrayFlag::NonEmpty)));
    }

    None
}

pub(super) fn array_map<'arena, A>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    call: &Call<'arena, SymbolId, Flow, Type<'arena>>,
) -> Option<Type<'arena>>
where
    A: Arena,
{
    let arguments = positional_arguments(context.ty.arena(), call);
    let [callback, array] = arguments.as_slice() else {
        return None;
    };

    if matches!(callback.atoms, [Atom::Null]) {
        return Some(*array);
    }

    let return_type = callable_return(*callback)?;
    remap_array_values(context, *array, return_type)
}

pub(super) fn array_filter<'arena, A>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    call: &Call<'arena, SymbolId, Flow, Type<'arena>>,
) -> Option<Type<'arena>>
where
    A: Arena,
{
    let array = nth_argument(call, 0)?;
    let has_callback = nth_argument(call, 1).is_some_and(|callback| !matches!(callback.atoms, [Atom::Null]));

    filter_array(context, array, !has_callback)
}

pub(super) fn array_reverse<'arena, A>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    call: &Call<'arena, SymbolId, Flow, Type<'arena>>,
) -> Option<Type<'arena>>
where
    A: Arena,
{
    if nth_argument(call, 1).is_some() {
        return None;
    }

    reverse_list(context, nth_argument(call, 0)?)
}

pub(super) fn array_flip<'arena, A>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    call: &Call<'arena, SymbolId, Flow, Type<'arena>>,
) -> Option<Type<'arena>>
where
    A: Arena,
{
    match nth_argument(call, 0)?.atoms {
        [Atom::Array(array)] if array.known_items.is_none() => {
            let (key, value) = (array.key_param?, array.value_param?);
            if !is_array_key_type(value) {
                return None;
            }

            Some(context.ty.keyed_unsealed_type(value, key, array.flags.contains(ArrayFlag::NonEmpty)))
        }
        [Atom::List(list)] if list.known_elements.is_none() => {
            if !is_array_key_type(list.element_type) {
                return None;
            }

            let value = context.ty.non_negative_int_type();
            Some(context.ty.keyed_unsealed_type(list.element_type, value, list.flags.contains(ListFlag::NonEmpty)))
        }
        _ => None,
    }
}

pub(super) fn array_merge<'arena, A>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    call: &Call<'arena, SymbolId, Flow, Type<'arena>>,
) -> Option<Type<'arena>>
where
    A: Arena,
{
    let arguments = positional_arguments(context.ty.arena(), call);
    if arguments.is_empty() {
        return None;
    }

    let mut all_lists = true;
    let mut all_sealed = true;
    let mut non_empty = false;
    let mut ordered = context.ty.scratch_vec();
    let mut values = context.ty.scratch_vec();
    let mut keys = context.ty.scratch_vec();

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
            return Some(context.ty.sealed_list_type(&ordered, non_empty || !ordered.is_empty()));
        }
        if values.is_empty() {
            return None;
        }

        let value = context.ty.union_of(&values);
        return Some(context.ty.list_of_type(value, non_empty));
    }

    if values.is_empty() {
        return None;
    }

    let key = if keys.is_empty() { context.ty.array_key_type() } else { context.ty.union_of(&keys) };
    let value = context.ty.union_of(&values);
    Some(context.ty.keyed_unsealed_type(key, value, non_empty))
}

fn array_key_atom(key: ArrayKey<'_>) -> Atom<'static> {
    match key {
        ArrayKey::Int(_) => INT,
        ArrayKey::String(_) => STRING,
        ArrayKey::Const { .. } => ARRAY_KEY,
    }
}

pub(super) fn array_column<'arena, A>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    call: &Call<'arena, SymbolId, Flow, Type<'arena>>,
) -> Option<Type<'arena>>
where
    A: Arena,
{
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
        Some(index) => Some(context.ty.keyed_unsealed_type(index, column_type, false)),
        None => Some(context.ty.list_of_type(column_type, false)),
    }
}

fn outer_value_type(ty: Type<'_>) -> Option<Type<'_>> {
    match ty.atoms {
        [Atom::List(list)] if !list.element_type.is_never() => Some(list.element_type),
        [Atom::Array(array)] => array.value_param,
        _ => None,
    }
}

fn literal_array_key(ty: Type<'_>) -> Option<ArrayKey<'_>> {
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

fn callable_return(ty: Type<'_>) -> Option<Type<'_>> {
    match ty.atoms {
        [Atom::Callable(CallableAtom::Closure(signature) | CallableAtom::Signature(signature))] => {
            Some(signature.return_type)
        }
        _ => None,
    }
}

fn array_key_type<'arena, A>(context: &mut ExtensionContext<'_, '_, 'arena, A>, key: ArrayKey<'arena>) -> Type<'arena>
where
    A: Arena,
{
    match key {
        ArrayKey::Int(value) => context.ty.int_literal_type(value),
        ArrayKey::String(value) => context.ty.string_literal_type(value),
        ArrayKey::Const { .. } => context.ty.array_key_type(),
    }
}

/// `$ty` narrowed to its truthy part, used by `array_filter` without a callback.
fn truthy<'arena, A>(context: &mut ExtensionContext<'_, '_, 'arena, A>, ty: Type<'arena>) -> Type<'arena>
where
    A: Arena,
{
    reconcile(&mut *context.ty, context.symbols, Assertion::Truthy, ty)
}

/// The result of `array_filter`: keys become optional and the array possibly
/// empty; when `narrow` (no callback) each value is truthy-narrowed, dropping
/// elements that narrow to `never`.
fn filter_array<'arena, A>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    source: Type<'arena>,
    narrow: bool,
) -> Option<Type<'arena>>
where
    A: Arena,
{
    match source.atoms {
        [Atom::Array(array)] => {
            let mut items = context.ty.scratch_vec();
            if let Some(known) = array.known_items {
                for item in known {
                    let value = if narrow { truthy(context, item.value) } else { item.value };
                    if !value.is_never() {
                        items.push(KnownItem { key: item.key, value, optional: true });
                    }
                }
            }

            let (key_param, value_param) = match (array.key_param, array.value_param) {
                (Some(key), Some(value)) => {
                    let value = if narrow { truthy(context, value) } else { value };
                    if value.is_never() { (None, None) } else { (Some(key), Some(value)) }
                }
                _ => (None, None),
            };

            if items.is_empty() && value_param.is_none() {
                return Some(context.ty.empty_array_type());
            }

            let known_items = (!items.is_empty()).then(|| context.ty.known_items(&items));
            let atom = context.ty.array(ArrayAtom { key_param, value_param, known_items, flags: U8Flags::empty() });

            Some(context.ty.union_of(&[atom]))
        }
        [Atom::List(list)] => {
            let mut values = context.ty.scratch_vec();
            if let Some(known) = list.known_elements {
                for element in known {
                    values.extend_from_slice(element.value.atoms);
                }
            }

            if !list.element_type.is_never() {
                values.extend_from_slice(list.element_type.atoms);
            }

            if values.is_empty() {
                return Some(context.ty.empty_array_type());
            }

            let value = context.ty.union_of(&values);
            let value = if narrow { truthy(context, value) } else { value };
            if value.is_never() {
                return Some(context.ty.empty_array_type());
            }

            let key = context.ty.non_negative_int_type();
            let atom = context.ty.unsealed_keyed_array_atom(key, value, false);

            Some(context.ty.union_of(&[atom]))
        }
        _ => None,
    }
}

/// `array_reverse` of a sealed list: the known elements in reverse order; any
/// other array shape is returned unchanged.
fn reverse_list<'arena, A>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    source: Type<'arena>,
) -> Option<Type<'arena>>
where
    A: Arena,
{
    match source.atoms {
        [Atom::List(list)] if list.element_type.is_never() && list.known_elements.is_some() => {
            let elements = list.known_elements.unwrap_or(&[]);
            let mut reversed = context.ty.scratch_vec_with(elements.len());
            for (index, element) in elements.iter().rev().enumerate() {
                reversed.push(KnownElement { index: index as u32, value: element.value, optional: element.optional });
            }

            let never = context.ty.never_type();
            let known_elements = Some(context.ty.known_elements(&reversed));
            let atom = context.ty.list(ListAtom {
                element_type: never,
                known_elements,
                known_count: list.known_count,
                flags: list.flags,
            });

            Some(context.ty.union_of(&[atom]))
        }
        [Atom::List(_)] | [Atom::Array(_)] => Some(source),
        _ => None,
    }
}

/// `array_map` shape preservation: the same array/list shape with every value
/// replaced by the callback's return type.
fn remap_array_values<'arena, A>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    source: Type<'arena>,
    value: Type<'arena>,
) -> Option<Type<'arena>>
where
    A: Arena,
{
    let atom = match source.atoms {
        [Atom::List(list)] => {
            let known_elements = list.known_elements.map(|elements| {
                let mut remapped = context.ty.scratch_vec_with(elements.len());
                for element in elements {
                    remapped.push(KnownElement { index: element.index, value, optional: element.optional });
                }

                context.ty.known_elements(&remapped)
            });
            let element_type = if list.element_type.is_never() { context.ty.never_type() } else { value };

            context.ty.list(ListAtom { element_type, known_elements, known_count: list.known_count, flags: list.flags })
        }
        [Atom::Array(array)] => {
            let known_items = array.known_items.map(|items| {
                let mut remapped = context.ty.scratch_vec_with(items.len());
                for item in items {
                    remapped.push(KnownItem { key: item.key, value, optional: item.optional });
                }

                context.ty.known_items(&remapped)
            });
            let (key_param, value_param) =
                if array.is_sealed() { (None, None) } else { (array.key_param, Some(value)) };

            context.ty.array(ArrayAtom { key_param, value_param, known_items, flags: array.flags })
        }
        _ => return None,
    };

    Some(context.ty.union_of(&[atom]))
}
