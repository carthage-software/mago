use core::hint::unreachable_unchecked;

use crate::ast::Argument;
use crate::ast::ArgumentList;
use crate::ast::NamedArgument;
use crate::ast::NamedArgumentSeparator;
use crate::ast::PositionalArgument;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::TwigTokenKind;

impl<'arena> Parser<'_, 'arena> {
    /// Parse an optional `(arg, ...)` list - returns `None` when the next
    /// token is not `(`.
    pub(crate) fn parse_optional_argument_list(&mut self) -> Result<Option<ArgumentList<'arena>>, ParseError> {
        if self.stream.is_at(TwigTokenKind::LeftParen)? { Ok(Some(self.parse_argument_list()?)) } else { Ok(None) }
    }

    /// Parse `(arg, arg, ...)` including the surrounding parens.
    pub(crate) fn parse_argument_list(&mut self) -> Result<ArgumentList<'arena>, ParseError> {
        let result = self.parse_comma_separated_sequence(
            TwigTokenKind::LeftParen,
            TwigTokenKind::RightParen,
            Self::parse_argument,
        )?;

        Ok(ArgumentList { left_parenthesis: result.open, arguments: result.sequence, right_parenthesis: result.close })
    }

    /// Parse a single argument - `name = value`, `name: value`, `...value`
    /// or a plain positional expression.
    pub(crate) fn parse_argument(&mut self) -> Result<Argument<'arena>, ParseError> {
        let is_named = self.stream.peek_kind(0)? == Some(TwigTokenKind::Name)
            && matches!(self.stream.peek_kind(1)?, Some(TwigTokenKind::Equal) | Some(TwigTokenKind::Colon));
        if is_named {
            let name_tok = self.stream.consume()?;
            let separator_tok = self.stream.consume()?;
            let separator = match separator_tok.kind {
                TwigTokenKind::Equal => NamedArgumentSeparator::Equal(self.stream.span_of(&separator_tok)),
                TwigTokenKind::Colon => NamedArgumentSeparator::Colon(self.stream.span_of(&separator_tok)),
                // SAFETY: `is_named` above guarantees the just-consumed separator is either
                // `=` or `:`; the two arms above are exhaustive for that contract, so this
                // branch is provably unreachable and we promise the optimizer can prune it.
                _ => unsafe { unreachable_unchecked() },
            };
            let value = self.parse_expression()?;
            return Ok(Argument::Named(NamedArgument {
                name: self.identifier_from(&name_tok),
                separator,
                value: self.alloc(value),
            }));
        }

        let ellipsis = match self.stream.try_consume(TwigTokenKind::DotDotDot)? {
            Some(token) => Some(self.stream.span_of(&token)),
            None => None,
        };
        let value = self.parse_expression()?;
        Ok(Argument::Positional(PositionalArgument { ellipsis, value: self.alloc(value) }))
    }
}
