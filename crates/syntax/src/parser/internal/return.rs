use crate::T;
use crate::ast::ast::Return;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_return(&mut self, stream: &mut TokenStream<'_, 'arena>) -> Result<Return<'arena>, ParseError> {
        Ok(Return {
            r#return: self.expect_keyword(stream, T!["return"])?,
            value: if matches!(stream.lookahead(0)?.map(|t| t.kind), Some(T![";" | "?>"])) {
                None
            } else {
                Some(self.parse_expression(stream)?)
            },
            terminator: self.parse_terminator(stream)?,
        })
    }
}
