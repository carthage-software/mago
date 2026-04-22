use mago_database::file::HasFileId;

use crate::ast::Statement;
use crate::ast::Unknown;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::TwigToken;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_unknown_tag(
        &mut self,
        open_tag_tok: TwigToken<'arena>,
        keyword_tok: TwigToken<'arena>,
    ) -> Result<Statement<'arena>, ParseError> {
        let open_tag = self.stream.span_of(&open_tag_tok);
        let name = self.identifier_from(&keyword_tok);
        loop {
            if self.stream.is_block_end()? {
                break;
            }
            if self.stream.has_reached_eof()? {
                return Err(ParseError::UnexpectedEof(
                    self.stream.file_id(),
                    "expected `%}` closing tag".to_string(),
                    open_tag_tok.start,
                ));
            }
            self.stream.consume()?;
        }
        let close_tag = self.stream.expect_block_end()?;
        Ok(Statement::Unknown(Unknown { open_tag, name, raw: "", close_tag }))
    }
}
