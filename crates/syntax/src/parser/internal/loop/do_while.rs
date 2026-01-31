use crate::T;
use crate::ast::ast::DoWhile;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_do_while(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<DoWhile<'arena>, ParseError> {
        Ok(DoWhile {
            r#do: self.expect_keyword(stream, T!["do"])?,
            statement: self.arena.alloc(self.parse_statement(stream)?),
            r#while: self.expect_keyword(stream, T!["while"])?,
            left_parenthesis: stream.eat(T!["("])?.span,
            condition: self.arena.alloc(self.parse_expression(stream)?),
            right_parenthesis: stream.eat(T![")"])?.span,
            terminator: self.parse_terminator(stream)?,
        })
    }
}
