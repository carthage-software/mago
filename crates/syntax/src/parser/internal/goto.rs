use crate::T;
use crate::ast::ast::Goto;
use crate::ast::ast::Label;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_goto(&mut self) -> Result<Goto<'arena>, ParseError> {
        Ok(Goto {
            goto: self.expect_keyword(T!["goto"])?,
            label: self.parse_local_identifier()?,
            terminator: self.parse_terminator()?,
        })
    }

    pub(crate) fn parse_label(&mut self) -> Result<Label<'arena>, ParseError> {
        Ok(Label { name: self.parse_local_identifier()?, colon: self.stream.eat_span(T![":"])? })
    }
}
