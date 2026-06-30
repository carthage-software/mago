use mago_allocator::Arena;
use mago_hir::ir::argument::Argument;
use mago_hir::ir::expression::Call;
use mago_hir::ir::expression::CalleeKind;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::atom::payload::array::ArrayFlag;
use mago_oracle::ty::atom::payload::array::ArrayKey;
use mago_oracle::ty::atom::payload::array::ListFlag;
use mago_oracle::ty::atom::payload::callable::CallableAtom;
use mago_oracle::ty::atom::payload::scalar::float::FloatAtom;
use mago_oracle::ty::atom::payload::scalar::int::IntAtom;
use mago_oracle::ty::atom::payload::scalar::string::StringAtom;
use mago_oracle::ty::atom::payload::scalar::string::StringLiteral;
use mago_oracle::ty::atom::payload::scalar::string::StringRefinementFlag;
use mago_oracle::ty::well_known::ARRAY_KEY;
use mago_oracle::ty::well_known::FALSE;
use mago_oracle::ty::well_known::FLOAT;
use mago_oracle::ty::well_known::INT;
use mago_oracle::ty::well_known::NON_NEGATIVE_INT;
use mago_oracle::ty::well_known::STRING;
use mago_oracle::ty::well_known::TRUE;

use crate::extension::ExtensionContext;
use crate::extension::ExtensionInference;
use crate::flow::Flow;

#[derive(Debug, Default, Clone, Copy)]
pub struct StdlibInference;

impl<A: Arena> ExtensionInference<A> for StdlibInference {
    fn infer<'arena>(
        &self,
        context: &mut ExtensionContext<'_, '_, 'arena, A>,
        expression: &Expression<'arena, SymbolId, Flow, Type<'arena>>,
    ) -> Option<Type<'arena>> {
        let ExpressionKind::Call(call) = &expression.kind else {
            return None;
        };

        let CalleeKind::Function(callee) = &call.callee.kind else {
            return None;
        };

        let ExpressionKind::Identifier(identifier) = &callee.kind else {
            return None;
        };

        let argument = first_argument(call);

        match if identifier.imported { identifier.value } else { identifier.last_segment() } {
            b"strlen" => Some(byte_length(context, argument)),
            b"mb_strlen" | b"grapheme_strlen" | b"iconv_strlen" => Some(character_length(context, argument)),
            b"count" | b"sizeof" => Some(element_count(context, argument)),
            b"strtoupper" => fold_string(context, argument, |bytes| bytes.to_ascii_uppercase()),
            b"strtolower" => fold_string(context, argument, |bytes| bytes.to_ascii_lowercase()),
            b"ucfirst" => fold_string(context, argument, ascii_ucfirst),
            b"lcfirst" => fold_string(context, argument, ascii_lcfirst),
            b"strrev" => fold_string(context, argument, |bytes| bytes.iter().rev().copied().collect()),
            b"ord" => Some(byte_ordinal(context, argument)),
            b"chr" => fold_chr(context, argument),
            b"abs" => fold_abs(context, argument),
            b"str_repeat" => fold_str_repeat(context, call),
            b"bin2hex" => fold_bin2hex(context, argument),
            b"dechex" => fold_int_to_string(context, argument, |value| format!("{:x}", value as u64)),
            b"decoct" => fold_int_to_string(context, argument, |value| format!("{:o}", value as u64)),
            b"decbin" => fold_int_to_string(context, argument, |value| format!("{:b}", value as u64)),
            b"intdiv" => fold_intdiv(context, call),
            b"str_contains" => fold_str_search(context, call, |haystack, needle| {
                needle.is_empty() || haystack.windows(needle.len()).any(|window| window == needle)
            }),
            b"str_starts_with" => fold_str_search(context, call, |haystack, needle| haystack.starts_with(needle)),
            b"str_ends_with" => fold_str_search(context, call, |haystack, needle| haystack.ends_with(needle)),
            b"strpos" => Some(string_position(context, call, false, false)),
            b"stripos" => Some(string_position(context, call, false, true)),
            b"strrpos" => Some(string_position(context, call, true, false)),
            b"strripos" => Some(string_position(context, call, true, true)),
            b"array_keys" => array_keys(context, argument),
            b"array_values" => array_values(context, argument),
            b"array_map" => array_map(context, call),
            b"array_filter" => array_filter(context, call),
            b"array_reverse" => array_reverse(context, call),
            b"array_flip" => array_flip(context, call),
            b"array_merge" => array_merge(context, call),
            b"array_column" => array_column(context, call),
            b"str_replace" => fold_str_replace(context, call),
            b"min" => fold_min_max(context, call, false),
            b"max" => fold_min_max(context, call, true),
            b"substr" => fold_substr(context, call),
            b"implode" | b"join" => fold_implode(context, call),
            _ => None,
        }
    }
}

fn byte_length<'arena, A: Arena>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    argument: Option<Type<'arena>>,
) -> Type<'arena> {
    if let Some(bytes) = argument.and_then(literal_string) {
        return context.int(bytes.len() as i64);
    }

    string_length_bound(context, argument)
}

fn character_length<'arena, A: Arena>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    argument: Option<Type<'arena>>,
) -> Type<'arena> {
    if let Some(text) = argument.and_then(literal_string).and_then(|bytes| std::str::from_utf8(bytes).ok()) {
        return context.int(text.chars().count() as i64);
    }

    string_length_bound(context, argument)
}

fn string_length_bound<'arena, A: Arena>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    argument: Option<Type<'arena>>,
) -> Type<'arena> {
    if argument.is_some_and(is_non_empty_string) {
        context.int_range(Some(1), None)
    } else {
        context.union(&[NON_NEGATIVE_INT])
    }
}

fn element_count<'arena, A: Arena>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    argument: Option<Type<'arena>>,
) -> Type<'arena> {
    match argument.and_then(count_bounds) {
        Some((min, Some(max))) if min == max => context.int(min),
        Some((min, max)) => context.int_range(Some(min), max),
        None => context.union(&[NON_NEGATIVE_INT]),
    }
}

fn byte_ordinal<'arena, A: Arena>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    argument: Option<Type<'arena>>,
) -> Type<'arena> {
    match argument.and_then(literal_string) {
        Some(bytes) => context.int(i64::from(bytes.first().copied().unwrap_or(0))),
        None => context.union(&[NON_NEGATIVE_INT]),
    }
}

fn fold_string<'arena, A: Arena>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    argument: Option<Type<'arena>>,
    transform: impl Fn(&[u8]) -> Vec<u8>,
) -> Option<Type<'arena>> {
    let bytes = argument.and_then(literal_string)?;

    Some(context.string(&transform(bytes)))
}

fn fold_chr<'arena, A: Arena>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    argument: Option<Type<'arena>>,
) -> Option<Type<'arena>> {
    let value = argument.and_then(literal_int)?;

    Some(context.string(&[value.rem_euclid(256) as u8]))
}

fn fold_abs<'arena, A: Arena>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    argument: Option<Type<'arena>>,
) -> Option<Type<'arena>> {
    match argument?.atoms {
        [Atom::Int(integer)] => {
            let (lower, upper) = abs_bounds(int_bounds(integer));
            match (lower, upper) {
                (Some(low), Some(high)) if low == high => Some(context.int(low)),
                (low, high) => Some(context.int_range(low, high)),
            }
        }
        [Atom::Float(FloatAtom::Literal(value))] => Some(context.float(value.0.into_inner().abs())),
        [Atom::Float(_)] => Some(context.union(&[FLOAT])),
        _ => None,
    }
}

fn abs_bounds((lower, upper): (Option<i64>, Option<i64>)) -> (Option<i64>, Option<i64>) {
    match (lower, upper) {
        (Some(low), _) if low >= 0 => (lower, upper),
        (_, Some(high)) if high <= 0 => (high.checked_neg(), lower.and_then(i64::checked_neg)),
        (Some(low), Some(high)) => (Some(0), Some(low.checked_neg().unwrap_or(i64::MAX).max(high))),
        _ => (Some(0), None),
    }
}

fn int_bounds(integer: &IntAtom<'_>) -> (Option<i64>, Option<i64>) {
    match integer {
        IntAtom::Literal(value) => (Some(*value), Some(*value)),
        IntAtom::Range(range) => (range.lower(), range.upper()),
        _ => (None, None),
    }
}

fn fold_str_repeat<'arena, A: Arena>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    call: &Call<'arena, SymbolId, Flow, Type<'arena>>,
) -> Option<Type<'arena>> {
    let string = nth_argument(call, 0).and_then(literal_string)?;
    let times = nth_argument(call, 1).and_then(literal_int)?;
    if times < 0 || string.len().checked_mul(times as usize)? > FOLD_LENGTH_LIMIT {
        return None;
    }

    Some(context.string(&string.repeat(times as usize)))
}

fn fold_str_search<'arena, A: Arena>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    call: &Call<'arena, SymbolId, Flow, Type<'arena>>,
    predicate: impl Fn(&[u8], &[u8]) -> bool,
) -> Option<Type<'arena>> {
    let haystack = nth_argument(call, 0).and_then(literal_string)?;
    let needle = nth_argument(call, 1).and_then(literal_string)?;

    Some(context.union(&[if predicate(haystack, needle) { TRUE } else { FALSE }]))
}

fn fold_int_to_string<'arena, A: Arena>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    argument: Option<Type<'arena>>,
    format: impl Fn(i64) -> String,
) -> Option<Type<'arena>> {
    let value = argument.and_then(literal_int)?;

    Some(context.string(format(value).as_bytes()))
}

fn fold_bin2hex<'arena, A: Arena>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    argument: Option<Type<'arena>>,
) -> Option<Type<'arena>> {
    let bytes = argument.and_then(literal_string)?;
    if bytes.len() > FOLD_LENGTH_LIMIT {
        return None;
    }

    let mut hex = Vec::with_capacity(bytes.len() * 2);
    for byte in bytes {
        hex.extend_from_slice(format!("{byte:02x}").as_bytes());
    }

    Some(context.string(&hex))
}

fn fold_intdiv<'arena, A: Arena>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    call: &Call<'arena, SymbolId, Flow, Type<'arena>>,
) -> Option<Type<'arena>> {
    let dividend = nth_argument(call, 0).and_then(literal_int)?;
    let divisor = nth_argument(call, 1).and_then(literal_int)?;

    dividend.checked_div(divisor).map(|quotient| context.int(quotient))
}

fn string_position<'arena, A: Arena>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    call: &Call<'arena, SymbolId, Flow, Type<'arena>>,
    last: bool,
    case_insensitive: bool,
) -> Type<'arena> {
    if nth_argument(call, 2).is_none()
        && let Some(haystack) = nth_argument(call, 0).and_then(literal_string)
        && let Some(needle) = nth_argument(call, 1).and_then(literal_string)
    {
        return match substring_position(haystack, needle, last, case_insensitive) {
            Some(position) => context.int(position as i64),
            None => context.union(&[FALSE]),
        };
    }

    context.union(&[NON_NEGATIVE_INT, FALSE])
}

fn substring_position(haystack: &[u8], needle: &[u8], last: bool, case_insensitive: bool) -> Option<usize> {
    let (haystack, needle) = if case_insensitive {
        (haystack.to_ascii_lowercase(), needle.to_ascii_lowercase())
    } else {
        (haystack.to_vec(), needle.to_vec())
    };

    if needle.is_empty() {
        return Some(if last { haystack.len() } else { 0 });
    }
    if needle.len() > haystack.len() {
        return None;
    }

    let mut windows = haystack.windows(needle.len());
    if last { windows.rposition(|window| window == needle) } else { windows.position(|window| window == needle) }
}

fn array_keys<'arena, A: Arena>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    argument: Option<Type<'arena>>,
) -> Option<Type<'arena>> {
    match argument?.atoms {
        [Atom::Array(array)] if array.is_sealed() => {
            let items = array.known_items.unwrap_or(&[]);
            if items.iter().any(|item| item.optional) {
                return None;
            }

            let keys: Vec<Type<'arena>> = items.iter().map(|item| array_key_type(context, item.key)).collect();
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

            let keys: Vec<Type<'arena>> = (0..known.len()).map(|index| context.int(index as i64)).collect();
            Some(context.list(&keys, !keys.is_empty()))
        }
        [Atom::List(list)] => {
            let index = context.union(&[NON_NEGATIVE_INT]);
            Some(context.list_of(index, list.flags.contains(ListFlag::NonEmpty)))
        }
        _ => None,
    }
}

fn array_values<'arena, A: Arena>(
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

        let values: Vec<Type<'arena>> = items.iter().map(|item| item.value).collect();
        return Some(context.list(&values, !values.is_empty()));
    }

    if array.known_items.is_none() {
        return Some(context.list_of(array.value_param?, array.flags.contains(ArrayFlag::NonEmpty)));
    }

    None
}

fn array_map<'arena, A: Arena>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    call: &Call<'arena, SymbolId, Flow, Type<'arena>>,
) -> Option<Type<'arena>> {
    let arguments = positional_arguments(call);
    let [callback, array] = arguments.as_slice() else {
        return None;
    };

    if matches!(callback.atoms, [Atom::Null]) {
        return Some(*array);
    }

    let return_type = callable_return(*callback)?;
    context.remap_array_values(*array, return_type)
}

fn array_filter<'arena, A: Arena>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    call: &Call<'arena, SymbolId, Flow, Type<'arena>>,
) -> Option<Type<'arena>> {
    let array = nth_argument(call, 0)?;
    let has_callback = nth_argument(call, 1).is_some_and(|callback| !matches!(callback.atoms, [Atom::Null]));

    context.filter_array(array, !has_callback)
}

fn array_reverse<'arena, A: Arena>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    call: &Call<'arena, SymbolId, Flow, Type<'arena>>,
) -> Option<Type<'arena>> {
    if nth_argument(call, 1).is_some() {
        return None;
    }

    context.reverse_list(nth_argument(call, 0)?)
}

fn array_flip<'arena, A: Arena>(
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

fn fold_str_replace<'arena, A: Arena>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    call: &Call<'arena, SymbolId, Flow, Type<'arena>>,
) -> Option<Type<'arena>> {
    let search = nth_argument(call, 0).and_then(literal_string)?;
    let replace = nth_argument(call, 1).and_then(literal_string)?;
    let subject = nth_argument(call, 2).and_then(literal_string)?;

    let result = byte_replace(subject, search, replace);
    if result.len() > FOLD_LENGTH_LIMIT {
        return None;
    }

    Some(context.string(&result))
}

fn byte_replace(subject: &[u8], search: &[u8], replace: &[u8]) -> Vec<u8> {
    if search.is_empty() {
        return subject.to_vec();
    }

    let mut out = Vec::with_capacity(subject.len());
    let mut index = 0;
    while index < subject.len() {
        if subject[index..].starts_with(search) {
            out.extend_from_slice(replace);
            index += search.len();
        } else {
            out.push(subject[index]);
            index += 1;
        }
    }

    out
}

fn array_merge<'arena, A: Arena>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    call: &Call<'arena, SymbolId, Flow, Type<'arena>>,
) -> Option<Type<'arena>> {
    let arguments = positional_arguments(call);
    if arguments.is_empty() {
        return None;
    }

    let mut all_lists = true;
    let mut all_sealed = true;
    let mut non_empty = false;
    let mut ordered: Vec<Type<'arena>> = Vec::new();
    let mut values: Vec<Atom<'arena>> = Vec::new();
    let mut keys: Vec<Atom<'arena>> = Vec::new();

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

fn array_column<'arena, A: Arena>(
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

fn fold_min_max<'arena, A: Arena>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    call: &Call<'arena, SymbolId, Flow, Type<'arena>>,
    max: bool,
) -> Option<Type<'arena>> {
    let atoms = candidate_atoms(call)?;
    if atoms.is_empty() {
        return None;
    }

    match reduce_int_bounds(&atoms, max) {
        Some((lower, upper)) => Some(int_type(context, lower, upper)),
        None => Some(context.union(&atoms)),
    }
}

fn candidate_atoms<'arena>(call: &Call<'arena, SymbolId, Flow, Type<'arena>>) -> Option<Vec<Atom<'arena>>> {
    let arguments: Vec<Type<'arena>> = positional_arguments(call);
    match arguments.as_slice() {
        [] => None,
        [single] => array_value_atoms(*single),
        many => Some(many.iter().flat_map(|ty| ty.atoms.iter().copied()).collect()),
    }
}

fn positional_arguments<'arena>(call: &Call<'arena, SymbolId, Flow, Type<'arena>>) -> Vec<Type<'arena>> {
    call.arguments
        .items
        .iter()
        .filter_map(|argument| match argument {
            Argument::Value(expression) => Some(expression.meta),
            _ => None,
        })
        .collect()
}

fn array_value_atoms<'arena>(ty: Type<'arena>) -> Option<Vec<Atom<'arena>>> {
    let mut atoms = Vec::new();
    match ty.atoms {
        [Atom::Array(array)] => {
            if let Some(items) = array.known_items {
                for item in items {
                    atoms.extend_from_slice(item.value.atoms);
                }
            }
            if let Some(value) = array.value_param {
                atoms.extend_from_slice(value.atoms);
            }
        }
        [Atom::List(list)] => {
            if let Some(elements) = list.known_elements {
                for element in elements {
                    atoms.extend_from_slice(element.value.atoms);
                }
            }
            if !list.element_type.is_never() {
                atoms.extend_from_slice(list.element_type.atoms);
            }
        }
        _ => return None,
    }

    Some(atoms)
}

fn reduce_int_bounds(atoms: &[Atom<'_>], max: bool) -> Option<(Option<i64>, Option<i64>)> {
    let mut iterator = atoms.iter();
    let (mut lower, mut upper) = atom_int_bounds(iterator.next()?)?;

    for atom in iterator {
        let (other_lower, other_upper) = atom_int_bounds(atom)?;
        if max {
            lower = pick_bound(lower, other_lower, i64::max, true);
            upper = pick_bound(upper, other_upper, i64::max, false);
        } else {
            lower = pick_bound(lower, other_lower, i64::min, false);
            upper = pick_bound(upper, other_upper, i64::min, true);
        }
    }

    Some((lower, upper))
}

fn pick_bound(current: Option<i64>, other: Option<i64>, combine: fn(i64, i64) -> i64, keep_known: bool) -> Option<i64> {
    match (current, other) {
        (Some(left), Some(right)) => Some(combine(left, right)),
        (left, right) if keep_known => left.or(right),
        _ => None,
    }
}

fn atom_int_bounds(atom: &Atom<'_>) -> Option<(Option<i64>, Option<i64>)> {
    match atom {
        Atom::Int(integer) => Some(int_bounds(integer)),
        _ => None,
    }
}

fn int_type<'arena, A: Arena>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    lower: Option<i64>,
    upper: Option<i64>,
) -> Type<'arena> {
    match (lower, upper) {
        (Some(low), Some(high)) if low == high => context.int(low),
        (low, high) => context.int_range(low, high),
    }
}

fn fold_substr<'arena, A: Arena>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    call: &Call<'arena, SymbolId, Flow, Type<'arena>>,
) -> Option<Type<'arena>> {
    let string = nth_argument(call, 0).and_then(literal_string)?;
    let offset = nth_argument(call, 1).and_then(literal_int)?;
    let length = match nth_argument(call, 2) {
        None => None,
        Some(length) => Some(literal_int(length)?),
    };

    let len = string.len() as i64;
    let start = if offset < 0 { (len + offset).max(0) } else { offset.min(len) };
    let end = match length {
        None => len,
        Some(length) if length < 0 => (len + length).max(start),
        Some(length) => (start + length).min(len),
    };

    let slice = if end > start { &string[start as usize..end as usize] } else { &[][..] };
    Some(context.string(slice))
}

fn fold_implode<'arena, A: Arena>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    call: &Call<'arena, SymbolId, Flow, Type<'arena>>,
) -> Option<Type<'arena>> {
    let separator: &[u8] = match (nth_argument(call, 0), nth_argument(call, 1)) {
        (Some(first), Some(_)) => literal_string(first)?,
        (Some(_), None) => &[],
        _ => return None,
    };
    let array = nth_argument(call, 1).or_else(|| nth_argument(call, 0))?;
    let parts = array_string_parts(array)?;

    let mut joined = Vec::new();
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            joined.extend_from_slice(separator);
        }
        joined.extend_from_slice(part);
    }

    Some(context.string(&joined))
}

fn array_string_parts(ty: Type<'_>) -> Option<Vec<Vec<u8>>> {
    let parts = match ty.atoms {
        [Atom::List(list)] if list.element_type.is_never() => {
            let elements = list.known_elements.unwrap_or(&[]);
            if elements.iter().any(|element| element.optional) {
                return None;
            }

            elements.iter().map(|element| element_string(element.value)).collect::<Option<Vec<_>>>()?
        }
        [Atom::Array(array)] if array.is_sealed() => {
            let items = array.known_items.unwrap_or(&[]);
            if items.iter().any(|item| item.optional) {
                return None;
            }

            items.iter().map(|item| element_string(item.value)).collect::<Option<Vec<_>>>()?
        }
        _ => return None,
    };

    Some(parts)
}

fn element_string(ty: Type<'_>) -> Option<Vec<u8>> {
    if let Some(bytes) = literal_string(ty) {
        return Some(bytes.to_vec());
    }
    if let Some(value) = literal_int(ty) {
        return Some(value.to_string().into_bytes());
    }

    None
}

fn is_non_empty_string(ty: Type<'_>) -> bool {
    !ty.atoms.is_empty()
        && ty.atoms.iter().all(|atom| matches!(atom, Atom::String(string) if string_atom_non_empty(string)))
}

fn string_atom_non_empty(string: &StringAtom<'_>) -> bool {
    match string.literal {
        StringLiteral::Value(value) => !value.is_empty(),
        _ => string.flags.contains(StringRefinementFlag::NonEmpty),
    }
}

const FOLD_LENGTH_LIMIT: usize = 4096;

fn first_argument<'arena>(call: &Call<'arena, SymbolId, Flow, Type<'arena>>) -> Option<Type<'arena>> {
    nth_argument(call, 0)
}

fn nth_argument<'arena>(call: &Call<'arena, SymbolId, Flow, Type<'arena>>, index: usize) -> Option<Type<'arena>> {
    call.arguments
        .items
        .iter()
        .filter_map(|argument| match argument {
            Argument::Value(expression) => Some(expression.meta),
            _ => None,
        })
        .nth(index)
}

fn literal_string<'arena>(ty: Type<'arena>) -> Option<&'arena [u8]> {
    match ty.atoms {
        [Atom::String(string)] => match string.literal {
            StringLiteral::Value(value) => Some(value),
            _ => None,
        },
        _ => None,
    }
}

fn literal_int(ty: Type<'_>) -> Option<i64> {
    match ty.atoms {
        [Atom::Int(IntAtom::Literal(value))] => Some(*value),
        _ => None,
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

fn ascii_ucfirst(bytes: &[u8]) -> Vec<u8> {
    let mut out = bytes.to_vec();
    if let Some(first) = out.first_mut() {
        first.make_ascii_uppercase();
    }

    out
}

fn ascii_lcfirst(bytes: &[u8]) -> Vec<u8> {
    let mut out = bytes.to_vec();
    if let Some(first) = out.first_mut() {
        first.make_ascii_lowercase();
    }

    out
}
