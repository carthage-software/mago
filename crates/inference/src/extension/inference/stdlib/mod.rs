use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_hir::ir::argument::Argument;
use mago_hir::ir::expression::Call;
use mago_hir::ir::expression::CalleeKind;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::atom::payload::scalar::int::IntAtom;
use mago_oracle::ty::atom::payload::scalar::string::StringLiteral;

use crate::extension::ExtensionContext;
use crate::extension::ExtensionInference;
use crate::flow::Flow;

mod arrays;
mod math;
mod strings;

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
            b"strlen" => Some(strings::byte_length(context, argument)),
            b"mb_strlen" | b"grapheme_strlen" | b"iconv_strlen" => Some(strings::character_length(context, argument)),
            b"count" | b"sizeof" => Some(arrays::element_count(context, argument)),
            b"strtoupper" => strings::fold_string(context, argument, |bytes, out| {
                out.extend(bytes.iter().map(u8::to_ascii_uppercase))
            }),
            b"strtolower" => strings::fold_string(context, argument, |bytes, out| {
                out.extend(bytes.iter().map(u8::to_ascii_lowercase))
            }),
            b"ucfirst" => strings::fold_string(context, argument, |bytes, out| {
                out.extend_from_slice(bytes);
                if let Some(first) = out.first_mut() {
                    first.make_ascii_uppercase();
                }
            }),
            b"lcfirst" => strings::fold_string(context, argument, |bytes, out| {
                out.extend_from_slice(bytes);
                if let Some(first) = out.first_mut() {
                    first.make_ascii_lowercase();
                }
            }),
            b"strrev" => strings::fold_string(context, argument, |bytes, out| out.extend(bytes.iter().rev().copied())),
            b"ord" => Some(strings::byte_ordinal(context, argument)),
            b"chr" => strings::fold_chr(context, argument),
            b"abs" => math::fold_abs(context, argument),
            b"str_repeat" => strings::fold_str_repeat(context, call),
            b"bin2hex" => strings::fold_bin2hex(context, argument),
            b"dechex" => strings::fold_radix(context, argument, 16),
            b"decoct" => strings::fold_radix(context, argument, 8),
            b"decbin" => strings::fold_radix(context, argument, 2),
            b"intdiv" => math::fold_intdiv(context, call),
            b"str_contains" => strings::fold_str_search(context, call, |haystack, needle| {
                needle.is_empty() || haystack.windows(needle.len()).any(|window| window == needle)
            }),
            b"str_starts_with" => {
                strings::fold_str_search(context, call, |haystack, needle| haystack.starts_with(needle))
            }
            b"str_ends_with" => strings::fold_str_search(context, call, |haystack, needle| haystack.ends_with(needle)),
            b"strpos" => Some(strings::string_position(context, call, false, false)),
            b"stripos" => Some(strings::string_position(context, call, false, true)),
            b"strrpos" => Some(strings::string_position(context, call, true, false)),
            b"strripos" => Some(strings::string_position(context, call, true, true)),
            b"array_keys" => arrays::array_keys(context, argument),
            b"array_values" => arrays::array_values(context, argument),
            b"array_map" => arrays::array_map(context, call),
            b"array_filter" => arrays::array_filter(context, call),
            b"array_reverse" => arrays::array_reverse(context, call),
            b"array_flip" => arrays::array_flip(context, call),
            b"array_merge" => arrays::array_merge(context, call),
            b"array_column" => arrays::array_column(context, call),
            b"str_replace" => strings::fold_str_replace(context, call),
            b"min" => math::fold_min_max(context, call, false),
            b"max" => math::fold_min_max(context, call, true),
            b"substr" => strings::fold_substr(context, call),
            b"implode" | b"join" => strings::fold_implode(context, call),
            _ => None,
        }
    }
}

fn first_argument<'arena>(call: &Call<'arena, SymbolId, Flow, Type<'arena>>) -> Option<Type<'arena>> {
    nth_argument(call, 0)
}

pub(super) fn nth_argument<'arena>(
    call: &Call<'arena, SymbolId, Flow, Type<'arena>>,
    index: usize,
) -> Option<Type<'arena>> {
    call.arguments
        .items
        .iter()
        .filter_map(|argument| match argument {
            Argument::Value(expression) => Some(expression.meta),
            _ => None,
        })
        .nth(index)
}

pub(super) fn positional_arguments<'arena, A>(
    arena: &'arena A,
    call: &Call<'arena, SymbolId, Flow, Type<'arena>>,
) -> Vec<'arena, Type<'arena>, A>
where
    A: Arena,
{
    let mut arguments = Vec::new_in(arena);
    for argument in call.arguments.items.iter() {
        if let Argument::Value(expression) = argument {
            arguments.push(expression.meta);
        }
    }

    arguments
}

pub(super) fn literal_string(ty: Type<'_>) -> Option<&[u8]> {
    match ty.atoms {
        [Atom::String(string)] => match string.literal {
            StringLiteral::Value(value) => Some(value),
            _ => None,
        },
        _ => None,
    }
}

pub(super) fn literal_int(ty: Type<'_>) -> Option<i64> {
    match ty.atoms {
        [Atom::Int(IntAtom::Literal(value))] => Some(*value),
        _ => None,
    }
}
