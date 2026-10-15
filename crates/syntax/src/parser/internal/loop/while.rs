use crate::T;
use crate::cst::cst::While;
use crate::cst::cst::WhileBody;
use crate::cst::cst::WhileColonDelimitedBody;
use crate::cst::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::Parser;
use mago_allocator::prelude::*;
use mago_span::Span;

impl<'arena, A> Parser<'_, 'arena, A>
where
    A: Arena,
{
    pub(crate) fn parse_while(&mut self) -> Result<While<'arena>, ParseError> {
        let r#while = self.expect_keyword(T!["while"])?;
        let left_parenthesis = self.stream.eat_span(T!["("])?;
        let condition = self.parse_expression()?;
        let right_parenthesis = self.parse_condition_closing_span()?;

        Ok(While {
            r#while,
            left_parenthesis,
            condition,
            right_parenthesis,
            body: self.parse_while_body(right_parenthesis)?,
        })
    }

    fn parse_while_body(&mut self, right_parenthesis: Span) -> Result<WhileBody<'arena>, ParseError> {
        Ok(match self.stream.peek_kind(0)? {
            Some(T![":"]) => WhileBody::ColonDelimited(self.parse_while_colon_delimited_body()?),
            _ => WhileBody::Statement(self.arena.alloc(self.parse_condition_body(right_parenthesis)?)),
        })
    }

    fn parse_while_colon_delimited_body(&mut self) -> Result<WhileColonDelimitedBody<'arena>, ParseError> {
        Ok(WhileColonDelimitedBody {
            colon: self.stream.eat_span(T![":"])?,
            statements: {
                let mut statements = self.new_vec();
                loop {
                    if matches!(self.stream.peek_kind(0)?, Some(T!["endwhile"])) {
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
            end_while: self.expect_keyword(T!["endwhile"])?,
            terminator: self.parse_terminator()?,
        })
    }
}
