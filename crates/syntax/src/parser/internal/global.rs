use crate::T;
use crate::ast::ast::Global;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_global(&mut self, stream: &mut TokenStream<'_, 'arena>) -> Result<Global<'arena>, ParseError> {
        Ok(Global {
            global: self.expect_keyword(stream, T!["global"])?,
            variables: {
                let mut variables = self.new_vec();
                let mut commas = self.new_vec();

                loop {
                    if matches!(stream.lookahead(0)?.map(|t| t.kind), Some(T!["?>" | ";"])) {
                        break;
                    }

                    variables.push(self.parse_variable(stream)?);

                    if let Some(T![","]) = stream.lookahead(0)?.map(|t| t.kind) {
                        commas.push(stream.consume()?);
                    } else {
                        break;
                    }
                }

                TokenSeparatedSequence::new(variables, commas)
            },
            terminator: self.parse_terminator(stream)?,
        })
    }
}
