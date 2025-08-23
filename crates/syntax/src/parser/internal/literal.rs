use ordered_float::OrderedFloat;

use mago_syntax_core::utils::parse_literal_float;
use mago_syntax_core::utils::parse_literal_integer;
use mago_syntax_core::utils::parse_literal_string_in;

use crate::T;
use crate::ast::ast::*;
use crate::error::ParseError;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;

pub fn parse_literal<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<Literal<'arena>, ParseError> {
    let token = utils::expect_any(stream)?;

    Ok(match &token.kind {
        T![LiteralFloat] => Literal::Float(LiteralFloat {
            span: token.span,
            raw: token.value,
            value: OrderedFloat(parse_literal_float(token.value).unwrap_or_else(|| {
                unreachable!("lexer generated invalid float `{}`; this should never happen.", token.value)
            })),
        }),
        T![LiteralInteger] => Literal::Integer(LiteralInteger {
            span: token.span,
            raw: token.value,
            value: parse_literal_integer(token.value),
        }),
        T!["true"] => Literal::True(utils::to_keyword(token)),
        T!["false"] => Literal::False(utils::to_keyword(token)),
        T!["null"] => Literal::Null(utils::to_keyword(token)),
        T![LiteralString] => Literal::String(LiteralString {
            kind: Some(if token.value.starts_with('"') {
                LiteralStringKind::DoubleQuoted
            } else {
                LiteralStringKind::SingleQuoted
            }),
            span: token.span,
            raw: token.value,
            value: parse_literal_string_in(stream.arena(), token.value, None, true),
        }),
        T![PartialLiteralString] => {
            let kind = if token.value.starts_with('"') {
                LiteralStringKind::DoubleQuoted
            } else {
                LiteralStringKind::SingleQuoted
            };

            return Err(ParseError::UnclosedLiteralString(kind, token.span));
        }
        _ => {
            return Err(utils::unexpected(
                stream,
                Some(token),
                T!["true", "false", "null", LiteralFloat, LiteralInteger, LiteralString, PartialLiteralString],
            ));
        }
    })
}
