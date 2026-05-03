use crate::ast::Deprecated;
use crate::ast::DeprecatedOption;
use crate::ast::Statement;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::TwigToken;
use crate::token::TwigTokenKind;

impl<'arena> Parser<'_, 'arena> {
    pub(crate) fn parse_deprecated(
        &mut self,
        open_tag_tok: TwigToken<'arena>,
        keyword_tok: TwigToken<'arena>,
    ) -> Result<Statement<'arena>, ParseError> {
        let open_tag = self.stream.span_of(&open_tag_tok);
        let keyword = self.keyword_from(&keyword_tok);
        let message = self.parse_expression()?;

        let mut options = self.new_vec();
        while self.stream.peek_kind(0)? == Some(TwigTokenKind::Name) {
            let option_tok = self.stream.consume()?;
            if !(option_tok.value == "package" || option_tok.value == "version") {
                return Err(ParseError::UnexpectedToken(
                    format!("unknown option `{}` on `deprecated`", option_tok.value),
                    self.stream.span_of(&option_tok),
                ));
            }
            let name = self.identifier_from(&option_tok);
            let Some(eq_tok) = self.stream.try_consume(TwigTokenKind::Equal)? else {
                let next = self.stream.lookahead(0)?;
                return Err(self.stream.unexpected(next, &[TwigTokenKind::Equal]));
            };
            let equal = self.stream.span_of(&eq_tok);
            let value = self.parse_expression()?;
            options.push(DeprecatedOption { name, equal, value });
        }
        let close_tag = self.stream.expect_block_end()?;

        Ok(Statement::Deprecated(Deprecated { open_tag, keyword, message, options, close_tag }))
    }
}
