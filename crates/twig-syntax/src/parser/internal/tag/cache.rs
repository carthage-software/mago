use crate::ast::Cache;
use crate::ast::CacheOption;
use crate::ast::Keyword;
use crate::ast::Statement;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::internal::BlockTerminator;
use crate::token::TwigToken;
use crate::token::TwigTokenKind;

impl<'arena> Parser<'_, 'arena> {
    pub(crate) fn parse_cache(
        &mut self,
        open_tag_tok: TwigToken<'arena>,
        keyword_tok: TwigToken<'arena>,
    ) -> Result<Statement<'arena>, ParseError> {
        let open_tag = self.stream.span_of(&open_tag_tok);
        let keyword = self.keyword_from(&keyword_tok);
        let key = self.parse_expression()?;

        let mut ttl = None;
        let mut tags = None;
        loop {
            if let Some(option_keyword) = self.try_consume_name_keyword("ttl")? {
                ttl = Some(self.parse_cache_option(option_keyword)?);
            } else if let Some(option_keyword) = self.try_consume_name_keyword("tags")? {
                tags = Some(self.parse_cache_option(option_keyword)?);
            } else {
                break;
            }
        }

        let close_tag = self.stream.expect_block_end()?;
        let body = self.parse_statements(&BlockTerminator { names: &["endcache"] })?;
        let end_open_tok = self.stream.expect_block_start()?;
        let end_open_tag = self.stream.span_of(&end_open_tok);
        let end_kw_tok = self.stream.expect_name("expected `endcache`")?;
        if end_kw_tok.value != "endcache" {
            return Err(ParseError::MismatchedEndTag {
                expected: "endcache".to_string(),
                got: end_kw_tok.value.to_string(),
                span: self.stream.span_of(&end_kw_tok),
            });
        }
        let end_keyword = self.keyword_from(&end_kw_tok);
        let end_close_tag = self.stream.expect_block_end()?;

        Ok(Statement::Cache(Cache {
            open_tag,
            keyword,
            key,
            ttl,
            tags,
            close_tag,
            body,
            end_open_tag,
            end_keyword,
            end_close_tag,
        }))
    }

    fn parse_cache_option(&mut self, keyword: Keyword<'arena>) -> Result<CacheOption<'arena>, ParseError> {
        let lp_tok = self.stream.expect_kind(TwigTokenKind::LeftParen, "expected `(`")?;
        let left_parenthesis = self.stream.span_of(&lp_tok);
        let value = self.parse_expression()?;
        let rp_tok = self.stream.expect_kind(TwigTokenKind::RightParen, "expected `)`")?;
        let right_parenthesis = self.stream.span_of(&rp_tok);
        Ok(CacheOption { keyword, left_parenthesis, value, right_parenthesis })
    }
}
