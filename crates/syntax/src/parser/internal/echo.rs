use crate::T;
use crate::ast::ast::Echo;
use crate::ast::ast::EchoTag;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_echo_tag(&mut self) -> Result<EchoTag<'arena>, ParseError> {
        Ok(EchoTag {
            tag: self.stream.eat(T!["<?="])?.span,
            values: {
                let mut values = self.new_vec();
                let mut commas = self.new_vec();

                loop {
                    if matches!(self.stream.lookahead(0)?.map(|t| t.kind), Some(T!["?>" | ";"])) {
                        break;
                    }

                    values.push(self.parse_expression()?);

                    if let Some(T![","]) = self.stream.lookahead(0)?.map(|t| t.kind) {
                        commas.push(self.stream.consume()?);
                    } else {
                        break;
                    }
                }

                TokenSeparatedSequence::new(values, commas)
            },
            terminator: self.parse_terminator()?,
        })
    }

    pub(crate) fn parse_echo(&mut self) -> Result<Echo<'arena>, ParseError> {
        Ok(Echo {
            echo: self.expect_keyword(T!["echo"])?,
            values: {
                let mut values = self.new_vec();
                let mut commas = self.new_vec();

                loop {
                    if matches!(self.stream.lookahead(0)?.map(|t| t.kind), Some(T!["?>" | ";"])) {
                        break;
                    }

                    values.push(self.parse_expression()?);

                    if let Some(T![","]) = self.stream.lookahead(0)?.map(|t| t.kind) {
                        commas.push(self.stream.consume()?);
                    } else {
                        break;
                    }
                }

                TokenSeparatedSequence::new(values, commas)
            },
            terminator: self.parse_terminator()?,
        })
    }
}
