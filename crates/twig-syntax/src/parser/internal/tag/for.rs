use crate::ast::ElseBranch;
use crate::ast::For;
use crate::ast::ForIfClause;
use crate::ast::Statement;
use crate::ast::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::internal::BlockTerminator;
use crate::token::TwigToken;
use crate::token::TwigTokenKind;

impl<'arena> Parser<'_, 'arena> {
    pub(crate) fn parse_for(
        &mut self,
        open_tag_tok: TwigToken<'arena>,
        keyword_tok: TwigToken<'arena>,
    ) -> Result<Statement<'arena>, ParseError> {
        let open_tag = self.stream.span_of(&open_tag_tok);
        let keyword = self.keyword_from(&keyword_tok);

        let mut target_nodes = self.new_vec();
        let mut target_commas = self.new_vec();
        target_nodes.push(self.expect_flexible_identifier("expected loop variable name")?);
        while let Some(comma) = self.stream.try_consume(TwigTokenKind::Comma)? {
            target_commas.push(comma);
            target_nodes.push(self.expect_flexible_identifier("expected second loop variable name")?);
        }
        let targets = TokenSeparatedSequence::new(target_nodes, target_commas);

        let Some(in_tok) = self.stream.try_consume(TwigTokenKind::In)? else {
            let next = self.stream.lookahead(0)?;
            return Err(self.stream.unexpected(next, &[TwigTokenKind::In]));
        };
        let in_keyword = self.keyword_from(&in_tok);

        let sequence = self.parse_expression()?;
        let if_clause = if let Some(keyword) = self.try_consume_name_keyword("if")? {
            let condition = self.parse_expression()?;
            Some(ForIfClause { keyword, condition })
        } else {
            None
        };
        let close_tag = self.stream.expect_block_end()?;
        let body = self.parse_statements(&BlockTerminator { names: &["else", "endfor"] })?;

        let next_open_tok = self.stream.expect_block_start()?;
        let next_open_tag = self.stream.span_of(&next_open_tok);
        let next_name_tok = self.stream.expect_name("expected `else` or `endfor`")?;

        let (else_branch, end_open_tag, end_keyword, end_close_tag) = match next_name_tok.value {
            "else" => {
                let else_keyword = self.keyword_from(&next_name_tok);
                let else_close_tag = self.stream.expect_block_end()?;
                let else_body = self.parse_statements(&BlockTerminator { names: &["endfor"] })?;
                let else_branch = ElseBranch {
                    open_tag: next_open_tag,
                    keyword: else_keyword,
                    close_tag: else_close_tag,
                    body: else_body,
                };
                let end_open_tok = self.stream.expect_block_start()?;
                let end_open_tag = self.stream.span_of(&end_open_tok);
                let end_kw_tok = self.stream.expect_name("expected `endfor`")?;
                if end_kw_tok.value != "endfor" {
                    return Err(ParseError::MismatchedEndTag {
                        expected: "endfor".to_string(),
                        got: end_kw_tok.value.to_string(),
                        span: self.stream.span_of(&end_kw_tok),
                    });
                }
                let end_keyword = self.keyword_from(&end_kw_tok);
                let end_close_tag = self.stream.expect_block_end()?;
                (Some(else_branch), end_open_tag, end_keyword, end_close_tag)
            }
            "endfor" => {
                let end_keyword = self.keyword_from(&next_name_tok);
                let end_close_tag = self.stream.expect_block_end()?;
                (None, next_open_tag, end_keyword, end_close_tag)
            }
            other => {
                return Err(ParseError::MismatchedEndTag {
                    expected: "endfor".to_string(),
                    got: other.to_string(),
                    span: self.stream.span_of(&next_name_tok),
                });
            }
        };

        Ok(Statement::For(For {
            open_tag,
            keyword,
            targets,
            in_keyword,
            sequence,
            if_clause,
            close_tag,
            body,
            else_branch,
            end_open_tag,
            end_keyword,
            end_close_tag,
        }))
    }
}
