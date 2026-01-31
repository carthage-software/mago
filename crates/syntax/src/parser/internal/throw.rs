use crate::T;
use crate::ast::ast::Throw;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_throw(&mut self, stream: &mut TokenStream<'_, 'arena>) -> Result<Throw<'arena>, ParseError> {
        Ok(Throw {
            throw: self.expect_keyword(stream, T!["throw"])?,
            exception: self.arena.alloc(self.parse_expression(stream)?),
        })
    }
}
