use crate::T;
use crate::ast::ast::Instantiation;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::Precedence;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_instantiation(&mut self) -> Result<Instantiation<'arena>, ParseError> {
        Ok(Instantiation {
            new: self.expect_keyword(T!["new"])?,
            class: self.arena.alloc(self.parse_expression_with_precedence(Precedence::New)?),
            argument_list: self.parse_optional_argument_list()?,
        })
    }
}
