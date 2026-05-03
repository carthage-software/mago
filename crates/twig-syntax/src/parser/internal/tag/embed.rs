use crate::ast::Embed;
use crate::ast::Statement;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::internal::BlockTerminator;
use crate::token::TwigToken;

impl<'arena> Parser<'_, 'arena> {
    pub(crate) fn parse_embed(
        &mut self,
        open_tag_tok: TwigToken<'arena>,
        keyword_tok: TwigToken<'arena>,
    ) -> Result<Statement<'arena>, ParseError> {
        let open_tag = self.stream.span_of(&open_tag_tok);
        let keyword = self.keyword_from(&keyword_tok);
        let template = self.parse_expression()?;
        let (ignore_missing, with_clause, only_keyword) = self.parse_include_tail()?;
        let close_tag = self.stream.expect_block_end()?;
        let body = self.parse_statements(&BlockTerminator { names: &["endembed"] })?;
        let end_open_tok = self.stream.expect_block_start()?;
        let end_open_tag = self.stream.span_of(&end_open_tok);
        let end_kw_tok = self.stream.expect_name("expected `endembed`")?;
        if end_kw_tok.value != "endembed" {
            return Err(ParseError::MismatchedEndTag {
                expected: "endembed".to_string(),
                got: end_kw_tok.value.to_string(),
                span: self.stream.span_of(&end_kw_tok),
            });
        }
        let end_keyword = self.keyword_from(&end_kw_tok);
        let end_close_tag = self.stream.expect_block_end()?;

        Ok(Statement::Embed(Embed {
            open_tag,
            keyword,
            template,
            ignore_missing,
            with_clause,
            only_keyword,
            close_tag,
            body,
            end_open_tag,
            end_keyword,
            end_close_tag,
        }))
    }
}
