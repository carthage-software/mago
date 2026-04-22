use crate::ast::Apply;
use crate::ast::FilterApplication;
use crate::ast::Statement;
use crate::ast::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::internal::BlockTerminator;
use crate::token::TwigToken;
use crate::token::TwigTokenKind;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_apply(
        &mut self,
        open_tag_tok: TwigToken<'arena>,
        keyword_tok: TwigToken<'arena>,
    ) -> Result<Statement<'arena>, ParseError> {
        let open_tag = self.stream.span_of(&open_tag_tok);
        let keyword = self.keyword_from(&keyword_tok);

        let mut filter_nodes = self.new_vec();
        let mut filter_pipes = self.new_vec();

        let first_name = self.expect_identifier("expected filter name after `apply`")?;
        let first_argument_list = self.parse_optional_argument_list()?;
        filter_nodes.push(FilterApplication { name: first_name, argument_list: first_argument_list });

        while let Some(pipe_tok) = self.stream.try_consume(TwigTokenKind::Pipe)? {
            filter_pipes.push(pipe_tok);
            let name = self.expect_identifier("expected filter name after `|`")?;
            let argument_list = self.parse_optional_argument_list()?;
            filter_nodes.push(FilterApplication { name, argument_list });
        }
        let filters = TokenSeparatedSequence::new(filter_nodes, filter_pipes);
        let close_tag = self.stream.expect_block_end()?;

        let body = self.parse_statements(&BlockTerminator { names: &["endapply"] })?;
        let end_open_tok = self.stream.expect_block_start()?;
        let end_open_tag = self.stream.span_of(&end_open_tok);
        let end_kw_tok = self.stream.expect_name("expected `endapply`")?;
        if end_kw_tok.value != "endapply" {
            return Err(ParseError::MismatchedEndTag {
                expected: "endapply".to_string(),
                got: end_kw_tok.value.to_string(),
                span: self.stream.span_of(&end_kw_tok),
            });
        }
        let end_keyword = self.keyword_from(&end_kw_tok);
        let end_close_tag = self.stream.expect_block_end()?;

        Ok(Statement::Apply(Apply {
            open_tag,
            keyword,
            filters,
            close_tag,
            body,
            end_open_tag,
            end_keyword,
            end_close_tag,
        }))
    }
}
