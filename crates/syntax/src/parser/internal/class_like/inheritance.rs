use crate::T;
use crate::ast::ast::Extends;
use crate::ast::ast::Implements;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_optional_implements(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<Option<Implements<'arena>>, ParseError> {
        Ok(match stream.lookahead(0)?.map(|t| t.kind) {
            Some(T!["implements"]) => Some(Implements {
                implements: self.expect_any_keyword(stream)?,
                types: {
                    let mut types = self.new_vec();
                    let mut commas = self.new_vec();
                    loop {
                        types.push(self.parse_identifier(stream)?);

                        match stream.lookahead(0)?.map(|t| t.kind) {
                            Some(T![","]) => {
                                commas.push(stream.consume()?);
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

    pub(crate) fn parse_optional_extends(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<Option<Extends<'arena>>, ParseError> {
        Ok(match stream.lookahead(0)?.map(|t| t.kind) {
            Some(T!["extends"]) => Some(Extends {
                extends: self.expect_any_keyword(stream)?,
                types: {
                    let mut types = self.new_vec();
                    let mut commas = self.new_vec();
                    loop {
                        types.push(self.parse_identifier(stream)?);

                        match stream.lookahead(0)?.map(|t| t.kind) {
                            Some(T![","]) => {
                                commas.push(stream.consume()?);
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
