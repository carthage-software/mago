use crate::ast::Statement;
use crate::ast::With;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::internal::BlockTerminator;
use crate::token::TwigToken;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_with(
        &mut self,
        open_tag_tok: TwigToken<'arena>,
        keyword_tok: TwigToken<'arena>,
    ) -> Result<Statement<'arena>, ParseError> {
        let open_tag = self.stream.span_of(&open_tag_tok);
        let keyword = self.keyword_from(&keyword_tok);

        let mut variables = None;
        let mut only_keyword = None;
        if !self.stream.is_block_end()? {
            variables = Some(self.parse_expression()?);
            only_keyword = self.try_consume_name_keyword("only")?;
        }
        let close_tag = self.stream.expect_block_end()?;
        let body = self.parse_statements(&BlockTerminator { names: &["endwith"] })?;
        let end_open_tok = self.stream.expect_block_start()?;
        let end_open_tag = self.stream.span_of(&end_open_tok);
        let end_kw_tok = self.stream.expect_name("expected `endwith`")?;
        if end_kw_tok.value != "endwith" {
            return Err(ParseError::MismatchedEndTag {
                expected: "endwith".to_string(),
                got: end_kw_tok.value.to_string(),
                span: self.stream.span_of(&end_kw_tok),
            });
        }
        let end_keyword = self.keyword_from(&end_kw_tok);
        let end_close_tag = self.stream.expect_block_end()?;

        Ok(Statement::With(With {
            open_tag,
            keyword,
            variables,
            only_keyword,
            close_tag,
            body,
            end_open_tag,
            end_keyword,
            end_close_tag,
        }))
    }
}
