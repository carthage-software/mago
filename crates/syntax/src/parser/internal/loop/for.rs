use crate::T;
use crate::ast::ast::For;
use crate::ast::ast::ForBody;
use crate::ast::ast::ForColonDelimitedBody;
use crate::ast::sequence::Sequence;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_for(&mut self) -> Result<For<'arena>, ParseError> {
        Ok(For {
            r#for: self.expect_keyword(T!["for"])?,
            left_parenthesis: self.stream.eat_span(T!["("])?,
            initializations: {
                let mut initializations = self.new_vec();
                let mut commas = self.new_vec();
                loop {
                    if matches!(self.stream.peek_kind(0)?, Some(T![";"])) {
                        break;
                    }

                    initializations.push(self.parse_expression()?);

                    match self.stream.peek_kind(0)? {
                        Some(T![","]) => {
                            commas.push(self.stream.consume()?);
                        }
                        _ => {
                            break;
                        }
                    }
                }

                TokenSeparatedSequence::new(initializations, commas)
            },
            initializations_semicolon: self.stream.eat_span(T![";"])?,
            conditions: {
                let mut conditions = self.new_vec();
                let mut commas = self.new_vec();
                loop {
                    if matches!(self.stream.peek_kind(0)?, Some(T![";"])) {
                        break;
                    }

                    conditions.push(self.parse_expression()?);

                    match self.stream.peek_kind(0)? {
                        Some(T![","]) => {
                            commas.push(self.stream.consume()?);
                        }
                        _ => {
                            break;
                        }
                    }
                }

                TokenSeparatedSequence::new(conditions, commas)
            },
            conditions_semicolon: self.stream.eat_span(T![";"])?,
            increments: {
                let mut increments = self.new_vec();
                let mut commas = self.new_vec();
                loop {
                    if matches!(self.stream.peek_kind(0)?, Some(T![")"])) {
                        break;
                    }

                    increments.push(self.parse_expression()?);

                    match self.stream.peek_kind(0)? {
                        Some(T![","]) => {
                            commas.push(self.stream.consume()?);
                        }
                        _ => {
                            break;
                        }
                    }
                }

                TokenSeparatedSequence::new(increments, commas)
            },
            right_parenthesis: self.stream.eat_span(T![")"])?,
            body: self.parse_for_body()?,
        })
    }

    fn parse_for_body(&mut self) -> Result<ForBody<'arena>, ParseError> {
        Ok(match self.stream.peek_kind(0)? {
            Some(T![":"]) => ForBody::ColonDelimited(self.parse_for_colon_delimited_body()?),
            _ => ForBody::Statement(self.arena.alloc(self.parse_statement()?)),
        })
    }

    fn parse_for_colon_delimited_body(&mut self) -> Result<ForColonDelimitedBody<'arena>, ParseError> {
        Ok(ForColonDelimitedBody {
            colon: self.stream.eat_span(T![":"])?,
            statements: {
                let mut statements = self.new_vec();
                loop {
                    if matches!(self.stream.peek_kind(0)?, Some(T!["endfor"])) {
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
            end_for: self.expect_keyword(T!["endfor"])?,
            terminator: self.parse_terminator()?,
        })
    }
}
