use crate::ast::ElseBranch;
use crate::ast::Guard;
use crate::ast::GuardKind;
use crate::ast::Statement;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::internal::BlockTerminator;
use crate::token::TwigToken;
use crate::token::TwigTokenKind;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_guard(
        &mut self,
        open_tag_tok: TwigToken<'arena>,
        keyword_tok: TwigToken<'arena>,
    ) -> Result<Statement<'arena>, ParseError> {
        let open_tag = self.stream.span_of(&open_tag_tok);
        let keyword = self.keyword_from(&keyword_tok);
        let type_tok = self.stream.expect_name("expected `function`, `filter`, or `test`")?;
        let kind = match type_tok.value {
            "function" => GuardKind::Function,
            "filter" => GuardKind::Filter,
            "test" => GuardKind::Test,
            other => {
                return Err(ParseError::UnexpectedToken(
                    format!("unsupported guard type `{}`", other),
                    self.stream.span_of(&type_tok),
                ));
            }
        };
        let kind_keyword = self.keyword_from(&type_tok);
        let name = self.expect_flexible_identifier("expected callable name")?;
        let second_word = if kind == GuardKind::Test && self.stream.peek_kind(0)? == Some(TwigTokenKind::Name) {
            let word_tok = self.stream.consume()?;
            Some(self.identifier_from(&word_tok))
        } else {
            None
        };
        let close_tag = self.stream.expect_block_end()?;
        let body = self.parse_statements(&BlockTerminator { names: &["else", "endguard"] })?;

        let next_open_tok = self.stream.expect_block_start()?;
        let next_open_tag = self.stream.span_of(&next_open_tok);
        let next_name_tok = self.stream.expect_name("expected `else` or `endguard`")?;
        let (else_branch, end_open_tag, end_keyword, end_close_tag) = match next_name_tok.value {
            "else" => {
                let else_keyword = self.keyword_from(&next_name_tok);
                let else_close_tag = self.stream.expect_block_end()?;
                let else_body = self.parse_statements(&BlockTerminator { names: &["endguard"] })?;
                let else_branch = ElseBranch {
                    open_tag: next_open_tag,
                    keyword: else_keyword,
                    close_tag: else_close_tag,
                    body: else_body,
                };
                let end_open_tok = self.stream.expect_block_start()?;
                let end_open_tag = self.stream.span_of(&end_open_tok);
                let end_kw_tok = self.stream.expect_name("expected `endguard`")?;
                if end_kw_tok.value != "endguard" {
                    return Err(ParseError::MismatchedEndTag {
                        expected: "endguard".to_string(),
                        got: end_kw_tok.value.to_string(),
                        span: self.stream.span_of(&end_kw_tok),
                    });
                }
                let end_keyword = self.keyword_from(&end_kw_tok);
                let end_close_tag = self.stream.expect_block_end()?;
                (Some(else_branch), end_open_tag, end_keyword, end_close_tag)
            }
            "endguard" => {
                let end_keyword = self.keyword_from(&next_name_tok);
                let end_close_tag = self.stream.expect_block_end()?;
                (None, next_open_tag, end_keyword, end_close_tag)
            }
            other => {
                return Err(ParseError::MismatchedEndTag {
                    expected: "endguard".to_string(),
                    got: other.to_string(),
                    span: self.stream.span_of(&next_name_tok),
                });
            }
        };

        Ok(Statement::Guard(Guard {
            open_tag,
            keyword,
            kind_keyword,
            kind,
            name,
            second_word,
            close_tag,
            body,
            else_branch,
            end_open_tag,
            end_keyword,
            end_close_tag,
        }))
    }
}
