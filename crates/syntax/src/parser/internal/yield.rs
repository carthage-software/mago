use crate::T;
use crate::ast::ast::Expression;
use crate::ast::ast::UnaryPrefix;
use crate::ast::ast::UnaryPrefixOperator;
use crate::ast::ast::Yield;
use crate::ast::ast::YieldFrom;
use crate::ast::ast::YieldPair;
use crate::ast::ast::YieldValue;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::Precedence;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_yield(&mut self) -> Result<Yield<'arena>, ParseError> {
        let r#yield = self.expect_keyword(T!["yield"])?;

        let next = self.stream.lookahead(0)?.ok_or_else(|| self.stream.unexpected(None, &[]))?;
        Ok(match next.kind {
            T![";" | "?>"] => Yield::Value(YieldValue { r#yield, value: None }),
            T!["from"] => Yield::From(YieldFrom {
                r#yield,
                from: self.expect_keyword(T!["from"])?,
                iterator: self.arena.alloc(self.parse_expression_with_precedence(Precedence::YieldFrom)?),
            }),
            T!["&"] => {
                let ampersand_span = self.stream.eat_span(T!["&"])?;
                let referenced_expr = self.parse_expression_with_precedence(Precedence::Reference)?;
                let value = self.arena.alloc(Expression::UnaryPrefix(UnaryPrefix {
                    operator: UnaryPrefixOperator::Reference(ampersand_span),
                    operand: referenced_expr,
                }));

                Yield::Value(YieldValue { r#yield, value: Some(value) })
            }
            _ => {
                let key_or_value = self.parse_expression_with_precedence(Precedence::Yield)?;

                if matches!(self.stream.peek_kind(0)?, Some(T!["=>"])) {
                    Yield::Pair(YieldPair {
                        r#yield,
                        key: self.arena.alloc(key_or_value),
                        arrow: self.stream.eat_span(T!["=>"])?,
                        value: self.arena.alloc(self.parse_expression_with_precedence(Precedence::Yield)?),
                    })
                } else {
                    Yield::Value(YieldValue { r#yield, value: Some(self.arena.alloc(key_or_value)) })
                }
            }
        })
    }
}
