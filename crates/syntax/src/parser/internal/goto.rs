use crate::T;
use crate::ast::ast::Goto;
use crate::ast::ast::Label;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_goto(&mut self, stream: &mut TokenStream<'_, 'arena>) -> Result<Goto<'arena>, ParseError> {
        Ok(Goto {
            goto: self.expect_keyword(stream, T!["goto"])?,
            label: self.parse_local_identifier(stream)?,
            terminator: self.parse_terminator(stream)?,
        })
    }

    pub(crate) fn parse_label(&self, stream: &mut TokenStream<'_, 'arena>) -> Result<Label<'arena>, ParseError> {
        Ok(Label { name: self.parse_local_identifier(stream)?, colon: stream.eat(T![":"])?.span })
    }
}
