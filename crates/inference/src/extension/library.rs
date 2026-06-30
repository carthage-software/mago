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
use mago_oracle::ty::well_known::NON_NEGATIVE_INT;

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
            b"ord" => Some(byte_ordinal(context, argument)),
            b"chr" => fold_chr(context, argument),
            b"abs" => fold_abs(context, argument),
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
    let argument = argument?;
    if let Some(value) = literal_int(argument) {
        return Some(context.int(value.abs()));
    }

    if let Some(value) = literal_float(argument) {
        return Some(context.float(value.abs()));
    }

    None
}

fn first_argument<'arena>(call: &Call<'arena, SymbolId, Flow, Type<'arena>>) -> Option<Type<'arena>> {
    call.arguments.items.iter().find_map(|argument| match argument {
        Argument::Value(expression) => Some(expression.meta),
        _ => None,
    })
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

fn literal_float(ty: Type<'_>) -> Option<f64> {
    match ty.atoms {
        [Atom::Float(FloatAtom::Literal(value))] => Some(value.0.into_inner()),
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
