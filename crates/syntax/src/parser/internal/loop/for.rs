use crate::T;
use crate::ast::ast::For;
use crate::ast::ast::ForBody;
use crate::ast::ast::ForColonDelimitedBody;
use crate::ast::sequence::Sequence;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_for(&mut self, stream: &mut TokenStream<'_, 'arena>) -> Result<For<'arena>, ParseError> {
        Ok(For {
            r#for: self.expect_keyword(stream, T!["for"])?,
            left_parenthesis: stream.eat(T!["("])?.span,
            initializations: {
                let mut initializations = self.new_vec();
                let mut commas = self.new_vec();
                loop {
                    if matches!(stream.lookahead(0)?.map(|t| t.kind), Some(T![";"])) {
                        break;
                    }

                    initializations.push(self.parse_expression(stream)?);

                    match stream.lookahead(0)?.map(|t| t.kind) {
                        Some(T![","]) => {
                            commas.push(stream.consume()?);
                        }
                        _ => {
                            break;
                        }
                    }
                }

                TokenSeparatedSequence::new(initializations, commas)
            },
            initializations_semicolon: stream.eat(T![";"])?.span,
            conditions: {
                let mut conditions = self.new_vec();
                let mut commas = self.new_vec();
                loop {
                    if matches!(stream.lookahead(0)?.map(|t| t.kind), Some(T![";"])) {
                        break;
                    }

                    conditions.push(self.parse_expression(stream)?);

                    match stream.lookahead(0)?.map(|t| t.kind) {
                        Some(T![","]) => {
                            commas.push(stream.consume()?);
                        }
                        _ => {
                            break;
                        }
                    }
                }

                TokenSeparatedSequence::new(conditions, commas)
            },
            conditions_semicolon: stream.eat(T![";"])?.span,
            increments: {
                let mut increments = self.new_vec();
                let mut commas = self.new_vec();
                loop {
                    if matches!(stream.lookahead(0)?.map(|t| t.kind), Some(T![")"])) {
                        break;
                    }

                    increments.push(self.parse_expression(stream)?);

                    match stream.lookahead(0)?.map(|t| t.kind) {
                        Some(T![","]) => {
                            commas.push(stream.consume()?);
                        }
                        _ => {
                            break;
                        }
                    }
                }

                TokenSeparatedSequence::new(increments, commas)
            },
            right_parenthesis: stream.eat(T![")"])?.span,
            body: self.parse_for_body(stream)?,
        })
    }

    fn parse_for_body(&mut self, stream: &mut TokenStream<'_, 'arena>) -> Result<ForBody<'arena>, ParseError> {
        Ok(match stream.lookahead(0)?.map(|t| t.kind) {
            Some(T![":"]) => ForBody::ColonDelimited(self.parse_for_colon_delimited_body(stream)?),
            _ => ForBody::Statement(self.arena.alloc(self.parse_statement(stream)?)),
        })
    }

    fn parse_for_colon_delimited_body(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<ForColonDelimitedBody<'arena>, ParseError> {
        Ok(ForColonDelimitedBody {
            colon: stream.eat(T![":"])?.span,
            statements: {
                let mut statements = self.new_vec();
                loop {
                    if matches!(stream.lookahead(0)?.map(|t| t.kind), Some(T!["endfor"])) {
                        break;
                    }

                    statements.push(self.parse_statement(stream)?);
                }

                Sequence::new(statements)
            },
            end_for: self.expect_keyword(stream, T!["endfor"])?,
            terminator: self.parse_terminator(stream)?,
        })
    }
}
