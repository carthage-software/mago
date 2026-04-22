use crate::ast::Set;
use crate::ast::SetBody;
use crate::ast::SetCapture;
use crate::ast::SetInline;
use crate::ast::Statement;
use crate::ast::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::internal::BlockTerminator;
use crate::token::TwigToken;
use crate::token::TwigTokenKind;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_set(
        &mut self,
        open_tag_tok: TwigToken<'arena>,
        keyword_tok: TwigToken<'arena>,
    ) -> Result<Statement<'arena>, ParseError> {
        let open_tag = self.stream.span_of(&open_tag_tok);
        let keyword = self.keyword_from(&keyword_tok);

        let mut name_nodes = self.new_vec();
        let mut name_commas = self.new_vec();
        name_nodes.push(self.expect_flexible_identifier("expected variable name")?);
        while let Some(comma) = self.stream.try_consume(TwigTokenKind::Comma)? {
            name_commas.push(comma);
            name_nodes.push(self.expect_flexible_identifier("expected variable name")?);
        }
        let names = TokenSeparatedSequence::new(name_nodes, name_commas);

        let body = if let Some(eq_tok) = self.stream.try_consume(TwigTokenKind::Equal)? {
            let equal = self.stream.span_of(&eq_tok);
            let mut value_nodes = self.new_vec();
            let mut value_commas = self.new_vec();
            value_nodes.push(self.parse_expression()?);
            while let Some(comma) = self.stream.try_consume(TwigTokenKind::Comma)? {
                value_commas.push(comma);
                value_nodes.push(self.parse_expression()?);
            }
            let values = TokenSeparatedSequence::new(value_nodes, value_commas);
            let close_tag = self.stream.expect_block_end()?;
            SetBody::Inline(SetInline { equal, values, close_tag })
        } else {
            let close_tag = self.stream.expect_block_end()?;
            let body = self.parse_statements(&BlockTerminator { names: &["endset"] })?;
            let end_open_tok = self.stream.expect_block_start()?;
            let end_open_tag = self.stream.span_of(&end_open_tok);
            let end_kw_tok = self.stream.expect_name("expected `endset`")?;
            if end_kw_tok.value != "endset" {
                return Err(ParseError::MismatchedEndTag {
                    expected: "endset".to_string(),
                    got: end_kw_tok.value.to_string(),
                    span: self.stream.span_of(&end_kw_tok),
                });
            }
            let end_keyword = self.keyword_from(&end_kw_tok);
            let end_close_tag = self.stream.expect_block_end()?;
            SetBody::Capture(SetCapture { close_tag, body, end_open_tag, end_keyword, end_close_tag })
        };

        Ok(Statement::Set(Set { open_tag, keyword, names, body }))
    }
}
