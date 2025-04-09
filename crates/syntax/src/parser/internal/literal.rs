use ordered_float::OrderedFloat;

use mago_syntax_core::utils::parse_literal_float;
use mago_syntax_core::utils::parse_literal_integer;

use crate::T;
use crate::ast::ast::*;
use crate::error::ParseError;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;

pub fn parse_literal(stream: &mut TokenStream<'_, '_>) -> Result<Literal, ParseError> {
    let token = utils::expect_any(stream)?;

    Ok(match &token.kind {
        T![LiteralFloat] => {
            let source = stream.interner().lookup(&token.value);

            Literal::Float(LiteralFloat {
                span: token.span,
                raw: token.value,
                value: OrderedFloat(parse_literal_float(source).unwrap_or_else(|| {
                    unreachable!("lexer generated invalid float `{}`; this should never happen.", source)
                })),
            })
        }
        T![LiteralInteger] => {
            let source = stream.interner().lookup(&token.value);

            Literal::Integer(LiteralInteger {
                span: token.span,
                raw: token.value,
                value: parse_literal_integer(source).unwrap_or_else(|| {
                    unreachable!("lexer generated invalid integer `{}`; this should never happen.", source,)
                }),
            })
        }
        T!["true"] => Literal::True(utils::to_keyword(token)),
        T!["false"] => Literal::False(utils::to_keyword(token)),
        T!["null"] => Literal::Null(utils::to_keyword(token)),
        T![LiteralString] => {
            let value = stream.interner().lookup(&token.value);

            let kind =
                if value.starts_with('"') { LiteralStringKind::DoubleQuoted } else { LiteralStringKind::SingleQuoted };

            Literal::String(LiteralString { kind, span: token.span, value: token.value })
        }
        T![PartialLiteralString] => {
            let value = stream.interner().lookup(&token.value);

            let kind =
                if value.starts_with('"') { LiteralStringKind::DoubleQuoted } else { LiteralStringKind::SingleQuoted };

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
