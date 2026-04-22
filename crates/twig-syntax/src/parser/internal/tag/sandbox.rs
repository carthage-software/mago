use crate::ast::Sandbox;
use crate::ast::Statement;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::internal::BlockTerminator;
use crate::token::TwigToken;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_sandbox(
        &mut self,
        open_tag_tok: TwigToken<'arena>,
        keyword_tok: TwigToken<'arena>,
    ) -> Result<Statement<'arena>, ParseError> {
        let open_tag = self.stream.span_of(&open_tag_tok);
        let keyword = self.keyword_from(&keyword_tok);
        let close_tag = self.stream.expect_block_end()?;
        let body = self.parse_statements(&BlockTerminator { names: &["endsandbox"] })?;
        let end_open_tok = self.stream.expect_block_start()?;
        let end_open_tag = self.stream.span_of(&end_open_tok);
        let end_kw_tok = self.stream.expect_name("expected `endsandbox`")?;
        if end_kw_tok.value != "endsandbox" {
            return Err(ParseError::MismatchedEndTag {
                expected: "endsandbox".to_string(),
                got: end_kw_tok.value.to_string(),
                span: self.stream.span_of(&end_kw_tok),
            });
        }
        let end_keyword = self.keyword_from(&end_kw_tok);
        let end_close_tag = self.stream.expect_block_end()?;

        Ok(Statement::Sandbox(Sandbox { open_tag, keyword, close_tag, body, end_open_tag, end_keyword, end_close_tag }))
    }
}
