use crate::T;
use crate::ast::ast::Instantiation;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;
use crate::token::Precedence;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_instantiation(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<Instantiation<'arena>, ParseError> {
        Ok(Instantiation {
            new: self.expect_keyword(stream, T!["new"])?,
            class: self.arena.alloc(self.parse_expression_with_precedence(stream, Precedence::New)?),
            argument_list: self.parse_optional_argument_list(stream)?,
        })
    }
}
