use crate::T;
use crate::ast::ast::Extends;
use crate::ast::ast::Implements;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_optional_implements(&mut self) -> Result<Option<Implements<'arena>>, ParseError> {
        Ok(match self.stream.peek_kind(0)? {
            Some(T!["implements"]) => Some(Implements {
                implements: self.expect_any_keyword()?,
                types: {
                    let mut types = self.new_vec();
                    let mut commas = self.new_vec();
                    loop {
                        types.push(self.parse_identifier()?);

                        match self.stream.peek_kind(0)? {
                            Some(T![","]) => {
                                commas.push(self.stream.consume()?);
                            }
                            _ => break,
                        }
                    }

                    TokenSeparatedSequence::new(types, commas)
                },
            }),
            _ => None,
        })
    }

    pub(crate) fn parse_optional_extends(&mut self) -> Result<Option<Extends<'arena>>, ParseError> {
        Ok(match self.stream.peek_kind(0)? {
            Some(T!["extends"]) => Some(Extends {
                extends: self.expect_any_keyword()?,
                types: {
                    let mut types = self.new_vec();
                    let mut commas = self.new_vec();
                    loop {
                        types.push(self.parse_identifier()?);

                        match self.stream.peek_kind(0)? {
                            Some(T![","]) => {
                                commas.push(self.stream.consume()?);
                            }
                            _ => break,
                        }
                    }
                    TokenSeparatedSequence::new(types, commas)
                },
            }),
            _ => None,
        })
    }
}
