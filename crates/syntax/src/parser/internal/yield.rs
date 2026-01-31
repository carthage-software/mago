use crate::T;
use crate::ast::ast::Yield;
use crate::ast::ast::YieldFrom;
use crate::ast::ast::YieldPair;
use crate::ast::ast::YieldValue;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;
use crate::token::Precedence;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_yield(&mut self, stream: &mut TokenStream<'_, 'arena>) -> Result<Yield<'arena>, ParseError> {
        let r#yield = self.expect_keyword(stream, T!["yield"])?;

        let next = stream.lookahead(0)?.ok_or_else(|| stream.unexpected(None, &[]))?;
        Ok(match next.kind {
            T![";" | "?>"] => Yield::Value(YieldValue { r#yield, value: None }),
            T!["from"] => Yield::From(YieldFrom {
                r#yield,
                from: self.expect_keyword(stream, T!["from"])?,
                iterator: self.arena.alloc(self.parse_expression_with_precedence(stream, Precedence::YieldFrom)?),
            }),
            _ => {
                let key_or_value = self.parse_expression_with_precedence(stream, Precedence::Yield)?;

                if matches!(stream.lookahead(0)?.map(|t| t.kind), Some(T!["=>"])) {
                    Yield::Pair(YieldPair {
                        r#yield,
                        key: self.arena.alloc(key_or_value),
                        arrow: stream.eat(T!["=>"])?.span,
                        value: self.arena.alloc(self.parse_expression_with_precedence(stream, Precedence::Yield)?),
                    })
                } else {
                    Yield::Value(YieldValue { r#yield, value: Some(self.arena.alloc(key_or_value)) })
                }
            }
        })
    }
}
