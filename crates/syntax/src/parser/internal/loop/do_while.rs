use crate::T;
use crate::cst::cst::DoWhile;
use crate::cst::cst::Terminator;
use crate::error::ParseError;
use crate::parser::Parser;
use mago_allocator::prelude::*;

impl<'arena, A> Parser<'_, 'arena, A>
where
    A: Arena,
{
    pub(crate) fn parse_do_while(&mut self) -> Result<DoWhile<'arena>, ParseError> {
        let r#do = self.expect_keyword(T!["do"])?;
        let statement = self.arena.alloc(self.parse_statement()?);
        let r#while = self.expect_keyword(T!["while"])?;
        let left_parenthesis = self.stream.eat_span(T!["("])?;
        let condition = self.parse_expression()?;
        let right_parenthesis = self.parse_condition_closing_span()?;
        let terminator = if right_parenthesis.length() == 0 {
            self.parse_optional_terminator()?.unwrap_or(Terminator::Missing(right_parenthesis))
        } else {
            self.parse_terminator()?
        };

        Ok(DoWhile { r#do, statement, r#while, left_parenthesis, condition, right_parenthesis, terminator })
    }
}
