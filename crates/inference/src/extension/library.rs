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
use mago_oracle::ty::atom::payload::array::ListFlag;
use mago_oracle::ty::atom::payload::scalar::float::FloatAtom;
use mago_oracle::ty::atom::payload::scalar::int::IntAtom;
use mago_oracle::ty::atom::payload::scalar::string::StringLiteral;
use mago_oracle::ty::well_known::FALSE;
use mago_oracle::ty::well_known::FLOAT;
use mago_oracle::ty::well_known::NON_NEGATIVE_INT;
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
        match identifier.last_segment() {
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
            _ => None,
        }
    }
}

fn byte_length<'arena, A: Arena>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    argument: Option<Type<'arena>>,
) -> Type<'arena> {
    match argument.and_then(literal_string) {
        Some(bytes) => context.int(bytes.len() as i64),
        None => context.union(&[NON_NEGATIVE_INT]),
    }
}

fn character_length<'arena, A: Arena>(
    context: &mut ExtensionContext<'_, '_, 'arena, A>,
    argument: Option<Type<'arena>>,
) -> Type<'arena> {
    match argument.and_then(literal_string).and_then(|bytes| std::str::from_utf8(bytes).ok()) {
        Some(text) => context.int(text.chars().count() as i64),
        None => context.union(&[NON_NEGATIVE_INT]),
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
