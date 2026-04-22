use crate::ast::ElseBranch;
use crate::ast::If;
use crate::ast::IfBranch;
use crate::ast::Sequence;
use crate::ast::Statement;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::internal::BlockTerminator;
use crate::token::TwigToken;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_if(
        &mut self,
        open_tag_tok: TwigToken<'arena>,
        keyword_tok: TwigToken<'arena>,
    ) -> Result<Statement<'arena>, ParseError> {
        let first_open_tag = self.stream.span_of(&open_tag_tok);
        let first_keyword = self.keyword_from(&keyword_tok);

        let mut branches = self.new_vec();
        let mut current_open_tag = first_open_tag;
        let mut current_keyword = first_keyword;
        let mut current_condition = self.parse_expression()?;
        let mut current_close_tag = self.stream.expect_block_end()?;
        let mut current_body = self.parse_statements(&BlockTerminator { names: &["elseif", "else", "endif"] })?;

        loop {
            let next_open_tok = self.stream.expect_block_start()?;
            let next_open_tag = self.stream.span_of(&next_open_tok);
            let name_tok = self.stream.expect_name("expected `elseif`, `else`, or `endif`")?;
            match name_tok.value {
                "elseif" => {
                    branches.push(IfBranch {
                        open_tag: current_open_tag,
                        keyword: current_keyword,
                        condition: current_condition,
                        close_tag: current_close_tag,
                        body: current_body,
                    });
                    current_open_tag = next_open_tag;
                    current_keyword = self.keyword_from(&name_tok);
                    current_condition = self.parse_expression()?;
                    current_close_tag = self.stream.expect_block_end()?;
                    current_body = self.parse_statements(&BlockTerminator { names: &["elseif", "else", "endif"] })?;
                }
                "else" => {
                    branches.push(IfBranch {
                        open_tag: current_open_tag,
                        keyword: current_keyword,
                        condition: current_condition,
                        close_tag: current_close_tag,
                        body: current_body,
                    });
                    let else_keyword = self.keyword_from(&name_tok);
                    let else_close_tag = self.stream.expect_block_end()?;
                    let else_body = self.parse_statements(&BlockTerminator { names: &["endif"] })?;
                    let else_branch = Some(ElseBranch {
                        open_tag: next_open_tag,
                        keyword: else_keyword,
                        close_tag: else_close_tag,
                        body: else_body,
                    });

                    let end_open_tok = self.stream.expect_block_start()?;
                    let end_open_tag = self.stream.span_of(&end_open_tok);
                    let end_kw_tok = self.stream.expect_name("expected `endif`")?;
                    if end_kw_tok.value != "endif" {
                        return Err(ParseError::MismatchedEndTag {
                            expected: "endif".to_string(),
                            got: end_kw_tok.value.to_string(),
                            span: self.stream.span_of(&end_kw_tok),
                        });
                    }
                    let end_keyword = self.keyword_from(&end_kw_tok);
                    let end_close_tag = self.stream.expect_block_end()?;
                    return Ok(Statement::If(If {
                        branches: Sequence::new(branches),
                        else_branch,
                        end_open_tag,
                        end_keyword,
                        end_close_tag,
                    }));
                }
                "endif" => {
                    branches.push(IfBranch {
                        open_tag: current_open_tag,
                        keyword: current_keyword,
                        condition: current_condition,
                        close_tag: current_close_tag,
                        body: current_body,
                    });
                    let end_keyword = self.keyword_from(&name_tok);
                    let end_close_tag = self.stream.expect_block_end()?;
                    return Ok(Statement::If(If {
                        branches: Sequence::new(branches),
                        else_branch: None,
                        end_open_tag: next_open_tag,
                        end_keyword,
                        end_close_tag,
                    }));
                }
                other => {
                    return Err(ParseError::UnexpectedToken(
                        format!("unexpected separator `{other}` in `if`"),
                        self.stream.span_of(&name_tok),
                    ));
                }
            }
        }
    }
}
