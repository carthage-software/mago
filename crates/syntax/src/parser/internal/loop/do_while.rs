use crate::T;
use crate::ast::ast::DoWhile;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_do_while(&mut self) -> Result<DoWhile<'arena>, ParseError> {
        Ok(DoWhile {
            r#do: self.expect_keyword(T!["do"])?,
            statement: self.arena.alloc(self.parse_statement()?),
            r#while: self.expect_keyword(T!["while"])?,
            left_parenthesis: self.stream.eat_span(T!["("])?,
            condition: self.arena.alloc(self.parse_expression()?),
            right_parenthesis: self.stream.eat_span(T![")"])?,
            terminator: self.parse_terminator()?,
        })
    }
}
