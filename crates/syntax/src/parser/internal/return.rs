use crate::T;
use crate::ast::ast::Return;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_return(&mut self) -> Result<Return<'arena>, ParseError> {
        Ok(Return {
            r#return: self.expect_keyword(T!["return"])?,
            value: if matches!(self.stream.peek_kind(0)?, Some(T![";" | "?>"])) {
                None
            } else {
                Some(self.parse_expression()?)
            },
            terminator: self.parse_terminator()?,
        })
    }
}
