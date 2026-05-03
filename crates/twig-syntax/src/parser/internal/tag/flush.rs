use crate::ast::Flush;
use crate::ast::Statement;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::TwigToken;

impl<'arena> Parser<'_, 'arena> {
    pub(crate) fn parse_flush(
        &mut self,
        open_tag_tok: TwigToken<'arena>,
        keyword_tok: TwigToken<'arena>,
    ) -> Result<Statement<'arena>, ParseError> {
        let open_tag = self.stream.span_of(&open_tag_tok);
        let keyword = self.keyword_from(&keyword_tok);
        let close_tag = self.stream.expect_block_end()?;
        Ok(Statement::Flush(Flush { open_tag, keyword, close_tag }))
    }
}
