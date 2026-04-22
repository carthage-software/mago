use crate::ast::BlockAlias;
use crate::ast::Statement;
use crate::ast::TokenSeparatedSequence;
use crate::ast::Use;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::TwigToken;
use crate::token::TwigTokenKind;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_use(
        &mut self,
        open_tag_tok: TwigToken<'arena>,
        keyword_tok: TwigToken<'arena>,
    ) -> Result<Statement<'arena>, ParseError> {
        let open_tag = self.stream.span_of(&open_tag_tok);
        let keyword = self.keyword_from(&keyword_tok);
        let template = self.parse_expression()?;

        let (with_keyword, aliases) = if let Some(with_keyword) = self.try_consume_name_keyword("with")? {
            let mut alias_nodes = self.new_vec();
            let mut alias_commas = self.new_vec();
            loop {
                let from = self.expect_flexible_identifier("expected block name to import")?;
                let (as_keyword, to) = if let Some(as_keyword) = self.try_consume_name_keyword("as")? {
                    let alias = self.expect_flexible_identifier("expected alias name")?;
                    (Some(as_keyword), Some(alias))
                } else {
                    (None, None)
                };
                alias_nodes.push(BlockAlias { from, as_keyword, to });
                if let Some(comma) = self.stream.try_consume(TwigTokenKind::Comma)? {
                    alias_commas.push(comma);
                } else {
                    break;
                }
            }
            (Some(with_keyword), TokenSeparatedSequence::new(alias_nodes, alias_commas))
        } else {
            (None, TokenSeparatedSequence::empty(self.arena))
        };

        let close_tag = self.stream.expect_block_end()?;

        Ok(Statement::Use(Use { open_tag, keyword, template, with_keyword, aliases, close_tag }))
    }
}
