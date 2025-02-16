use mago_ast::ast::*;
use mago_token::Precedence;
use mago_token::T;

use crate::error::ParseError;
use crate::internal::expression::parse_expression_with_precedence;
use crate::internal::token_stream::TokenStream;
use crate::internal::utils;

#[inline]
pub fn parse_yield<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<Yield<'i>, ParseError> {
    let r#yield = utils::expect_keyword(stream, T!["yield"])?;

    Ok(match utils::peek(stream)?.kind {
        T![";" | "?>"] => Yield::Value(YieldValue { r#yield, value: None }),
        T!["from"] => {
            let from = utils::expect_keyword(stream, T!["from"])?;
            let iterator = parse_expression_with_precedence(stream, Precedence::YieldFrom)?;

            Yield::From(YieldFrom { r#yield, from, iterator: stream.boxed(iterator) })
        }
        _ => {
            let key_or_value = parse_expression_with_precedence(stream, Precedence::Yield)?;

            if matches!(utils::maybe_peek(stream)?.map(|t| t.kind), Some(T!["=>"])) {
                let arrow = utils::expect_span(stream, T!["=>"])?;
                let value = parse_expression_with_precedence(stream, Precedence::Yield)?;

                Yield::Pair(YieldPair { r#yield, key: stream.boxed(key_or_value), arrow, value: stream.boxed(value) })
            } else {
                Yield::Value(YieldValue { r#yield, value: Some(stream.boxed(key_or_value)) })
            }
        }
    })
}
