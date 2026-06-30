use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_hir::ir::expression::Call;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::atom::payload::scalar::string::StringAtom;
use mago_oracle::ty::atom::payload::scalar::string::StringLiteral;
use mago_oracle::ty::atom::payload::scalar::string::StringRefinementFlag;
use mago_oracle::ty::well_known::FALSE;
use mago_oracle::ty::well_known::NON_NEGATIVE_INT;
use mago_oracle::ty::well_known::TRUE;

use crate::extension::ExtensionContext;
use crate::extension::inference::stdlib::literal_int;
use crate::extension::inference::stdlib::literal_string;
use crate::extension::inference::stdlib::nth_argument;
use crate::flow::Flow;

const FOLD_LENGTH_LIMIT: usize = 4096;

pub(super) fn byte_length<'arena, A>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    argument: Option<Type<'arena>>,
) -> Type<'arena>
where
    A: Arena,
{
    if let Some(bytes) = argument.and_then(literal_string) {
        return context.ty.int_literal_type(bytes.len() as i64);
    }

    string_length_bound(context, argument)
}

pub(super) fn character_length<'arena, A>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    argument: Option<Type<'arena>>,
) -> Type<'arena>
where
    A: Arena,
{
    if let Some(text) = argument.and_then(literal_string).and_then(|bytes| std::str::from_utf8(bytes).ok()) {
        return context.ty.int_literal_type(text.chars().count() as i64);
    }

    string_length_bound(context, argument)
}

fn string_length_bound<'arena, A>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    argument: Option<Type<'arena>>,
) -> Type<'arena>
where
    A: Arena,
{
    if argument.is_some_and(is_non_empty_string) {
        context.ty.int_range_type(Some(1), None)
    } else {
        context.ty.non_negative_int_type()
    }
}

pub(super) fn byte_ordinal<'arena, A>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    argument: Option<Type<'arena>>,
) -> Type<'arena>
where
    A: Arena,
{
    match argument.and_then(literal_string) {
        Some(bytes) => context.ty.int_literal_type(i64::from(bytes.first().copied().unwrap_or(0))),
        None => context.ty.non_negative_int_type(),
    }
}

pub(super) fn fold_string<'arena, A>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    argument: Option<Type<'arena>>,
    transform: impl Fn(&[u8], &mut Vec<'arena, u8, A>),
) -> Option<Type<'arena>>
where
    A: Arena,
{
    let bytes = argument.and_then(literal_string)?;
    let arena = context.ty.arena();
    let mut out = Vec::with_capacity_in(bytes.len(), arena);
    transform(bytes, &mut out);

    Some(context.ty.string_literal_type(&out))
}

pub(super) fn fold_chr<'arena, A>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    argument: Option<Type<'arena>>,
) -> Option<Type<'arena>>
where
    A: Arena,
{
    let value = argument.and_then(literal_int)?;

    Some(context.ty.string_literal_type(&[value.rem_euclid(256) as u8]))
}

pub(super) fn fold_str_repeat<'arena, A>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    call: &Call<'arena, SymbolId, Flow, Type<'arena>>,
) -> Option<Type<'arena>>
where
    A: Arena,
{
    let string = nth_argument(call, 0).and_then(literal_string)?;
    let times = nth_argument(call, 1).and_then(literal_int)?;
    if times < 0 {
        return None;
    }

    let total = string.len().checked_mul(times as usize)?;
    if total > FOLD_LENGTH_LIMIT {
        return None;
    }

    let arena = context.ty.arena();
    let mut out = Vec::with_capacity_in(total, arena);
    for _ in 0..times {
        out.extend_from_slice(string);
    }

    Some(context.ty.string_literal_type(&out))
}

pub(super) fn fold_str_search<'arena, A>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    call: &Call<'arena, SymbolId, Flow, Type<'arena>>,
    predicate: impl Fn(&[u8], &[u8]) -> bool,
) -> Option<Type<'arena>>
where
    A: Arena,
{
    let haystack = nth_argument(call, 0).and_then(literal_string)?;
    let needle = nth_argument(call, 1).and_then(literal_string)?;

    Some(context.ty.union_of(&[if predicate(haystack, needle) { TRUE } else { FALSE }]))
}

pub(super) fn fold_radix<'arena, A>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    argument: Option<Type<'arena>>,
    radix: u64,
) -> Option<Type<'arena>>
where
    A: Arena,
{
    let value = argument.and_then(literal_int)? as u64;

    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut buffer = [0u8; 64];
    let mut cursor = buffer.len();
    let mut magnitude = value;
    loop {
        cursor -= 1;
        buffer[cursor] = DIGITS[(magnitude % radix) as usize];
        magnitude /= radix;
        if magnitude == 0 {
            break;
        }
    }

    Some(context.ty.string_literal_type(&buffer[cursor..]))
}

pub(super) fn fold_bin2hex<'arena, A>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    argument: Option<Type<'arena>>,
) -> Option<Type<'arena>>
where
    A: Arena,
{
    let bytes = argument.and_then(literal_string)?;
    if bytes.len() > FOLD_LENGTH_LIMIT {
        return None;
    }

    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let arena = context.ty.arena();
    let mut hex = Vec::with_capacity_in(bytes.len() * 2, arena);
    for &byte in bytes {
        hex.push(DIGITS[(byte >> 4) as usize]);
        hex.push(DIGITS[(byte & 0x0f) as usize]);
    }

    Some(context.ty.string_literal_type(&hex))
}

pub(super) fn string_position<'arena, A>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    call: &Call<'arena, SymbolId, Flow, Type<'arena>>,
    last: bool,
    case_insensitive: bool,
) -> Type<'arena>
where
    A: Arena,
{
    if nth_argument(call, 2).is_none()
        && let Some(haystack) = nth_argument(call, 0).and_then(literal_string)
        && let Some(needle) = nth_argument(call, 1).and_then(literal_string)
    {
        return match substring_position(context.ty.arena(), haystack, needle, last, case_insensitive) {
            Some(position) => context.ty.int_literal_type(position as i64),
            None => context.ty.union_of(&[FALSE]),
        };
    }

    context.ty.union_of(&[NON_NEGATIVE_INT, FALSE])
}

fn substring_position<A>(arena: &A, haystack: &[u8], needle: &[u8], last: bool, case_insensitive: bool) -> Option<usize>
where
    A: Arena,
{
    if needle.is_empty() {
        return Some(if last { haystack.len() } else { 0 });
    }
    if needle.len() > haystack.len() {
        return None;
    }

    if case_insensitive {
        let mut lower_haystack = Vec::with_capacity_in(haystack.len(), arena);
        lower_haystack.extend(haystack.iter().map(u8::to_ascii_lowercase));
        let mut lower_needle = Vec::with_capacity_in(needle.len(), arena);
        lower_needle.extend(needle.iter().map(u8::to_ascii_lowercase));

        find_subslice(&lower_haystack, &lower_needle, last)
    } else {
        find_subslice(haystack, needle, last)
    }
}

fn find_subslice(haystack: &[u8], needle: &[u8], last: bool) -> Option<usize> {
    let mut windows = haystack.windows(needle.len());
    if last { windows.rposition(|window| window == needle) } else { windows.position(|window| window == needle) }
}

pub(super) fn fold_str_replace<'arena, A>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    call: &Call<'arena, SymbolId, Flow, Type<'arena>>,
) -> Option<Type<'arena>>
where
    A: Arena,
{
    let search = nth_argument(call, 0).and_then(literal_string)?;
    let replace = nth_argument(call, 1).and_then(literal_string)?;
    let subject = nth_argument(call, 2).and_then(literal_string)?;

    let result = byte_replace(context.ty.arena(), subject, search, replace);
    if result.len() > FOLD_LENGTH_LIMIT {
        return None;
    }

    Some(context.ty.string_literal_type(&result))
}

fn byte_replace<'arena, A>(arena: &'arena A, subject: &[u8], search: &[u8], replace: &[u8]) -> Vec<'arena, u8, A>
where
    A: Arena,
{
    let mut out = Vec::with_capacity_in(subject.len(), arena);
    if search.is_empty() {
        out.extend_from_slice(subject);
        return out;
    }

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

pub(super) fn fold_substr<'arena, A>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    call: &Call<'arena, SymbolId, Flow, Type<'arena>>,
) -> Option<Type<'arena>>
where
    A: Arena,
{
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
    Some(context.ty.string_literal_type(slice))
}

pub(super) fn fold_implode<'arena, A>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    call: &Call<'arena, SymbolId, Flow, Type<'arena>>,
) -> Option<Type<'arena>>
where
    A: Arena,
{
    let separator: &[u8] = match (nth_argument(call, 0), nth_argument(call, 1)) {
        (Some(first), Some(_)) => literal_string(first)?,
        (Some(_), None) => &[],
        _ => return None,
    };
    let array = nth_argument(call, 1).or_else(|| nth_argument(call, 0))?;

    let arena = context.ty.arena();
    let mut joined = Vec::new_in(arena);
    if !append_imploded(&mut joined, array, separator) {
        return None;
    }

    Some(context.ty.string_literal_type(&joined))
}

fn append_imploded<'arena, A>(joined: &mut Vec<'arena, u8, A>, array: Type<'arena>, separator: &[u8]) -> bool
where
    A: Arena,
{
    match array.atoms {
        [Atom::List(list)] if list.element_type.is_never() => {
            let elements = list.known_elements.unwrap_or(&[]);
            if elements.iter().any(|element| element.optional) {
                return false;
            }

            for (index, element) in elements.iter().enumerate() {
                if index > 0 {
                    joined.extend_from_slice(separator);
                }
                if !append_element_string(joined, element.value) {
                    return false;
                }
            }

            true
        }
        [Atom::Array(array)] if array.is_sealed() => {
            let items = array.known_items.unwrap_or(&[]);
            if items.iter().any(|item| item.optional) {
                return false;
            }

            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    joined.extend_from_slice(separator);
                }
                if !append_element_string(joined, item.value) {
                    return false;
                }
            }

            true
        }
        _ => false,
    }
}

fn append_element_string<'arena, A>(joined: &mut Vec<'arena, u8, A>, ty: Type<'arena>) -> bool
where
    A: Arena,
{
    if let Some(bytes) = literal_string(ty) {
        joined.extend_from_slice(bytes);
        return true;
    }
    if let Some(value) = literal_int(ty) {
        append_decimal(joined, value);
        return true;
    }

    false
}

fn append_decimal<A>(joined: &mut Vec<'_, u8, A>, value: i64)
where
    A: Arena,
{
    let mut buffer = [0u8; 20];
    let mut cursor = buffer.len();
    let mut magnitude = value.unsigned_abs();
    loop {
        cursor -= 1;
        buffer[cursor] = b'0' + (magnitude % 10) as u8;
        magnitude /= 10;
        if magnitude == 0 {
            break;
        }
    }
    if value < 0 {
        cursor -= 1;
        buffer[cursor] = b'-';
    }

    joined.extend_from_slice(&buffer[cursor..]);
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
