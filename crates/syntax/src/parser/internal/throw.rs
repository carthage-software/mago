use crate::T;
use crate::ast::ast::Throw;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_throw(&mut self) -> Result<Throw<'arena>, ParseError> {
        Ok(Throw { throw: self.expect_keyword(T!["throw"])?, exception: self.arena.alloc(self.parse_expression()?) })
    }
}
