use crate::ast::Statement;
use crate::ast::Verbatim;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::TwigToken;
use crate::token::TwigTokenKind;

impl<'arena> Parser<'_, 'arena> {
    pub(crate) fn parse_verbatim(
        &mut self,
        open_tag_tok: TwigToken<'arena>,
        keyword_tok: TwigToken<'arena>,
    ) -> Result<Statement<'arena>, ParseError> {
        let open_tag = self.stream.span_of(&open_tag_tok);
        let keyword = self.keyword_from(&keyword_tok);
        let close_tag = self.stream.expect_block_end()?;

        let body = if let Some(body_tok) = self.stream.try_consume(TwigTokenKind::VerbatimText)? {
            body_tok.value
        } else {
            ""
        };

        let end_open_tok = self.stream.expect_block_start()?;
        let end_open_tag = self.stream.span_of(&end_open_tok);
        let end_kw_tok = self.stream.expect_name("expected `endverbatim` or `endraw`")?;
        let expected = match keyword_tok.value {
            "verbatim" => "endverbatim",
            "raw" => "endraw",
            _ => "endverbatim",
        };
        if end_kw_tok.value != expected {
            return Err(ParseError::MismatchedEndTag {
                expected: expected.to_string(),
                got: end_kw_tok.value.to_string(),
                span: self.stream.span_of(&end_kw_tok),
            });
        }
        let end_keyword = self.keyword_from(&end_kw_tok);
        let end_close_tag = self.stream.expect_block_end()?;

        Ok(Statement::Verbatim(Verbatim {
            open_tag,
            keyword,
            close_tag,
            body,
            end_open_tag,
            end_keyword,
            end_close_tag,
        }))
    }
}
