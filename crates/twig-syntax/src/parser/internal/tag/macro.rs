use crate::ast::Macro;
use crate::ast::MacroArgument;
use crate::ast::Statement;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::internal::BlockTerminator;
use crate::token::TwigToken;
use crate::token::TwigTokenKind;
use mago_allocator::prelude::*;

impl<'arena, A> Parser<'_, 'arena, A>
where
    A: Arena,
{
    pub(crate) fn parse_macro(
        &mut self,
        open_tag_tok: TwigToken<'arena>,
        keyword_tok: TwigToken<'arena>,
    ) -> Result<Statement<'arena>, ParseError<'arena>> {
        let open_tag = self.stream.span_of(&open_tag_tok);
        let keyword = self.keyword_from(&keyword_tok);
        let name = self.expect_flexible_identifier(b"expected macro name")?;

        let parameters = self.parse_comma_separated_sequence(
            TwigTokenKind::LeftParen,
            TwigTokenKind::RightParen,
            Self::parse_macro_argument,
        )?;
        let close_tag = self.stream.expect_block_end()?;

        let body = self.parse_statements(&BlockTerminator { names: &[b"endmacro"] })?;
        let end_open_tok = self.stream.expect_block_start()?;
        let end_open_tag = self.stream.span_of(&end_open_tok);
        let end_kw_tok = self.stream.expect_name(b"expected `endmacro`")?;
        if end_kw_tok.value != b"endmacro" {
            return Err(ParseError::MismatchedEndTag {
                expected: b"endmacro",
                got: end_kw_tok.value,
                span: self.stream.span_of(&end_kw_tok),
            });
        }
        let end_keyword = self.keyword_from(&end_kw_tok);
        let end_name = if let Some(closing_tok) = self.stream.try_consume(TwigTokenKind::Name)? {
            if closing_tok.value != name.value {
                return Err(ParseError::MismatchedEndTag {
                    expected: name.value,
                    got: closing_tok.value,
                    span: self.stream.span_of(&closing_tok),
                });
            }
            Some(self.identifier_from(&closing_tok))
        } else {
            None
        };
        let end_close_tag = self.stream.expect_block_end()?;

        Ok(Statement::Macro(Macro {
            open_tag,
            keyword,
            name,
            left_parenthesis: parameters.open,
            arguments: parameters.sequence,
            right_parenthesis: parameters.close,
            close_tag,
            body,
            end_open_tag,
            end_keyword,
            end_name,
            end_close_tag,
        }))
    }

    fn parse_macro_argument(&mut self) -> Result<MacroArgument<'arena>, ParseError<'arena>> {
        let name = self.expect_flexible_identifier(b"expected macro argument name")?;
        let (equal, default) = if let Some(eq_tok) = self.stream.try_consume(TwigTokenKind::Equal)? {
            let value = self.parse_expression()?;
            (Some(self.stream.span_of(&eq_tok)), Some(value))
        } else {
            (None, None)
        };
        Ok(MacroArgument { name, equal, default })
    }
}
