use mago_database::file::HasFileId;

use crate::ast::Array;
use crate::ast::ArrayElement;
use crate::ast::Expression;
use crate::ast::MissingArrayElement;
use crate::ast::ValueArrayElement;
use crate::ast::VariadicArrayElement;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::TwigTokenKind;

impl<'input, 'arena> Parser<'input, 'arena> {
    /// Parse an array literal: `[ elements ]`.
    pub(crate) fn parse_array(&mut self) -> Result<Expression<'arena>, ParseError> {
        let result = self.parse_comma_separated_sequence(
            TwigTokenKind::LeftBracket,
            TwigTokenKind::RightBracket,
            Self::parse_array_element,
        )?;

        Ok(Expression::Array(Array {
            left_bracket: result.open,
            elements: result.sequence,
            right_bracket: result.close,
        }))
    }

    /// Parse a single array element: a value, a variadic (`...`) element,
    /// or a missing element (for destructuring holes like `[, second]`).
    pub(crate) fn parse_array_element(&mut self) -> Result<ArrayElement<'arena>, ParseError> {
        match self.stream.peek_kind(0)? {
            Some(TwigTokenKind::DotDotDot) => {
                let ellipsis = self.stream.consume_span()?;
                let value = self.parse_expression()?;
                Ok(ArrayElement::Variadic(VariadicArrayElement { ellipsis, value: self.alloc(value) }))
            }
            Some(TwigTokenKind::Comma) => {
                let current =
                    self.stream.lookahead(0)?.ok_or_else(|| self.stream.unexpected(None, &[TwigTokenKind::Comma]))?;
                Ok(ArrayElement::Missing(MissingArrayElement { comma: current.span_for(self.stream.file_id()) }))
            }
            _ => {
                let value = self.parse_expression()?;
                Ok(ArrayElement::Value(ValueArrayElement { value: self.alloc(value) }))
            }
        }
    }
}
