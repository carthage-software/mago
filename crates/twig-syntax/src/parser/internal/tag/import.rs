use crate::ast::Import;
use crate::ast::Statement;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::TwigToken;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_import(
        &mut self,
        open_tag_tok: TwigToken<'arena>,
        keyword_tok: TwigToken<'arena>,
    ) -> Result<Statement<'arena>, ParseError> {
        let open_tag = self.stream.span_of(&open_tag_tok);
        let keyword = self.keyword_from(&keyword_tok);
        let template = self.parse_expression()?;
        let as_tok = self.stream.expect_name_value("as")?;
        let as_keyword = self.keyword_from(&as_tok);
        let alias = self.expect_flexible_identifier("expected alias name")?;
        let close_tag = self.stream.expect_block_end()?;
        Ok(Statement::Import(Import { open_tag, keyword, template, as_keyword, alias, close_tag }))
    }
}
