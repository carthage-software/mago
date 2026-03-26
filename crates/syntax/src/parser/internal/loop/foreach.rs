use crate::T;
use crate::ast::ast::Foreach;
use crate::ast::ast::ForeachBody;
use crate::ast::ast::ForeachColonDelimitedBody;
use crate::ast::ast::ForeachKeyValueTarget;
use crate::ast::ast::ForeachTarget;
use crate::ast::ast::ForeachValueTarget;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_foreach(&mut self) -> Result<Foreach<'arena>, ParseError> {
        Ok(Foreach {
            foreach: self.expect_keyword(T!["foreach"])?,
            left_parenthesis: self.stream.eat_span(T!["("])?,
            expression: self.arena.alloc(self.parse_expression()?),
            r#as: self.expect_keyword(T!["as"])?,
            target: self.parse_foreach_target()?,
            right_parenthesis: self.stream.eat_span(T![")"])?,
            body: self.parse_foreach_body()?,
        })
    }

    fn parse_foreach_target(&mut self) -> Result<ForeachTarget<'arena>, ParseError> {
        let key_or_value = self.arena.alloc(self.parse_expression()?);

        Ok(match self.stream.peek_kind(0)? {
            Some(T!["=>"]) => ForeachTarget::KeyValue(ForeachKeyValueTarget {
                key: key_or_value,
                double_arrow: self.stream.consume_span()?,
                value: self.arena.alloc(self.parse_expression()?),
            }),
            _ => ForeachTarget::Value(ForeachValueTarget { value: key_or_value }),
        })
    }

    fn parse_foreach_body(&mut self) -> Result<ForeachBody<'arena>, ParseError> {
        Ok(match self.stream.peek_kind(0)? {
            Some(T![":"]) => ForeachBody::ColonDelimited(self.parse_foreach_colon_delimited_body()?),
            _ => ForeachBody::Statement(self.arena.alloc(self.parse_statement()?)),
        })
    }

    fn parse_foreach_colon_delimited_body(&mut self) -> Result<ForeachColonDelimitedBody<'arena>, ParseError> {
        Ok(ForeachColonDelimitedBody {
            colon: self.stream.eat_span(T![":"])?,
            statements: {
                let mut statements = self.new_vec();
                loop {
                    if matches!(self.stream.peek_kind(0)?, Some(T!["endforeach"])) {
                        break;
                    }

                    let position_before = self.stream.current_position();
                    statements.push(self.parse_statement()?);
                    if self.stream.current_position() == position_before {
                        if let Ok(Some(token)) = self.stream.lookahead(0) {
                            self.errors.push(self.stream.unexpected(Some(token), &[]));
                            let _ = self.stream.consume();
                        } else {
                            break;
                        }
                    }
                }

                Sequence::new(statements)
            },
            end_foreach: self.expect_keyword(T!["endforeach"])?,
            terminator: self.parse_terminator()?,
        })
    }
}
