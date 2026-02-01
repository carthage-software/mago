use crate::T;
use crate::ast::ast::Break;
use crate::ast::ast::Continue;
use crate::error::ParseError;
use crate::parser::Parser;

pub mod do_while;
pub mod r#for;
pub mod foreach;
pub mod r#while;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_continue(&mut self) -> Result<Continue<'arena>, ParseError> {
        Ok(Continue {
            r#continue: self.expect_keyword(T!["continue"])?,
            level: match self.stream.peek_kind(0)? {
                Some(T![";" | "?>"]) => None,
                _ => Some(self.parse_expression()?),
            },
            terminator: self.parse_terminator()?,
        })
    }

    pub(crate) fn parse_break(&mut self) -> Result<Break<'arena>, ParseError> {
        Ok(Break {
            r#break: self.expect_keyword(T!["break"])?,
            level: match self.stream.peek_kind(0)? {
                Some(T![";" | "?>"]) => None,
                _ => Some(self.parse_expression()?),
            },
            terminator: self.parse_terminator()?,
        })
    }
}
