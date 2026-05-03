use crate::ast::Extends;
use crate::ast::Statement;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::TwigToken;

impl<'arena> Parser<'_, 'arena> {
    pub(crate) fn parse_extends(
        &mut self,
        open_tag_tok: TwigToken<'arena>,
        keyword_tok: TwigToken<'arena>,
    ) -> Result<Statement<'arena>, ParseError> {
        let open_tag = self.stream.span_of(&open_tag_tok);
        let keyword = self.keyword_from(&keyword_tok);
        let template = self.parse_expression()?;
        let close_tag = self.stream.expect_block_end()?;
        Ok(Statement::Extends(Extends { open_tag, keyword, template, close_tag }))
    }
}
