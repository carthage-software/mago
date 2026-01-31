use crate::T;
use crate::ast::ast::Global;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_global(&mut self) -> Result<Global<'arena>, ParseError> {
        Ok(Global {
            global: self.expect_keyword(T!["global"])?,
            variables: {
                let mut variables = self.new_vec();
                let mut commas = self.new_vec();

                loop {
                    if matches!(self.stream.lookahead(0)?.map(|t| t.kind), Some(T!["?>" | ";"])) {
                        break;
                    }

                    variables.push(self.parse_variable()?);

                    if let Some(T![","]) = self.stream.lookahead(0)?.map(|t| t.kind) {
                        commas.push(self.stream.consume()?);
                    } else {
                        break;
                    }
                }

                TokenSeparatedSequence::new(variables, commas)
            },
            terminator: self.parse_terminator()?,
        })
    }
}
