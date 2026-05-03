use crate::ast::From;
use crate::ast::ImportedMacro;
use crate::ast::Statement;
use crate::ast::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::TwigToken;
use crate::token::TwigTokenKind;

impl<'arena> Parser<'_, 'arena> {
    pub(crate) fn parse_from(
        &mut self,
        open_tag_tok: TwigToken<'arena>,
        keyword_tok: TwigToken<'arena>,
    ) -> Result<Statement<'arena>, ParseError> {
        let open_tag = self.stream.span_of(&open_tag_tok);
        let keyword = self.keyword_from(&keyword_tok);
        let template = self.parse_expression()?;
        let import_tok = self.stream.expect_name_value("import")?;
        let import_keyword = self.keyword_from(&import_tok);

        let mut name_nodes = self.new_vec();
        let mut name_commas = self.new_vec();
        loop {
            let from = self.expect_flexible_identifier("expected imported macro name")?;
            let (as_keyword, to) = if let Some(as_keyword) = self.try_consume_name_keyword("as")? {
                let alias = self.expect_flexible_identifier("expected alias")?;
                (Some(as_keyword), Some(alias))
            } else {
                (None, None)
            };
            name_nodes.push(ImportedMacro { from, as_keyword, to });
            if let Some(comma) = self.stream.try_consume(TwigTokenKind::Comma)? {
                name_commas.push(comma);
            } else {
                break;
            }
        }
        let names = TokenSeparatedSequence::new(name_nodes, name_commas);
        let close_tag = self.stream.expect_block_end()?;

        Ok(Statement::From(From { open_tag, keyword, template, import_keyword, names, close_tag }))
    }
}
