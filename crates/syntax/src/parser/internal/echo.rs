use crate::T;
use crate::ast::ast::Echo;
use crate::ast::ast::EchoTag;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_echo_tag(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<EchoTag<'arena>, ParseError> {
        Ok(EchoTag {
            tag: stream.eat(T!["<?="])?.span,
            values: {
                let mut values = self.new_vec();
                let mut commas = self.new_vec();

                loop {
                    if matches!(stream.lookahead(0)?.map(|t| t.kind), Some(T!["?>" | ";"])) {
                        break;
                    }

                    values.push(self.parse_expression(stream)?);

                    if let Some(T![","]) = stream.lookahead(0)?.map(|t| t.kind) {
                        commas.push(stream.consume()?);
                    } else {
                        break;
                    }
                }

                TokenSeparatedSequence::new(values, commas)
            },
            terminator: self.parse_terminator(stream)?,
        })
    }

    pub(crate) fn parse_echo(&mut self, stream: &mut TokenStream<'_, 'arena>) -> Result<Echo<'arena>, ParseError> {
        Ok(Echo {
            echo: self.expect_keyword(stream, T!["echo"])?,
            values: {
                let mut values = self.new_vec();
                let mut commas = self.new_vec();

                loop {
                    if matches!(stream.lookahead(0)?.map(|t| t.kind), Some(T!["?>" | ";"])) {
                        break;
                    }

                    values.push(self.parse_expression(stream)?);

                    if let Some(T![","]) = stream.lookahead(0)?.map(|t| t.kind) {
                        commas.push(stream.consume()?);
                    } else {
                        break;
                    }
                }

                TokenSeparatedSequence::new(values, commas)
            },
            terminator: self.parse_terminator(stream)?,
        })
    }
}
