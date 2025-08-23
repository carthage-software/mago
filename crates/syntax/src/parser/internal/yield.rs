use crate::T;
use crate::ast::ast::*;
use crate::error::ParseError;
use crate::parser::internal::expression::parse_expression_with_precedence;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;
use crate::token::Precedence;

pub fn parse_yield<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<Yield<'arena>, ParseError> {
    let r#yield = utils::expect_keyword(stream, T!["yield"])?;

    Ok(match utils::peek(stream)?.kind {
        T![";" | "?>"] => Yield::Value(YieldValue { r#yield, value: None }),
        T!["from"] => Yield::From(YieldFrom {
            r#yield,
            from: utils::expect_keyword(stream, T!["from"])?,
            iterator: {
                let expr = parse_expression_with_precedence(stream, Precedence::YieldFrom)?;

                stream.alloc(expr)
            },
        }),
        _ => {
            let key_or_value = parse_expression_with_precedence(stream, Precedence::Yield)?;

            if matches!(utils::maybe_peek(stream)?.map(|t| t.kind), Some(T!["=>"])) {
                Yield::Pair(YieldPair {
                    r#yield,
                    key: stream.alloc(key_or_value),
                    arrow: utils::expect_span(stream, T!["=>"])?,
                    value: {
                        let expr = parse_expression_with_precedence(stream, Precedence::Yield)?;

                        stream.alloc(expr)
                    },
                })
            } else {
                Yield::Value(YieldValue { r#yield, value: Some(stream.alloc(key_or_value)) })
            }
        }
    })
}
