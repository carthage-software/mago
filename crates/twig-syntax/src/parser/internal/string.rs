use crate::cst::Expression;
use crate::cst::InterpolatedLiteral;
use crate::cst::InterpolatedString;
use crate::cst::Interpolation;
use crate::cst::StringConcatenation;
use crate::cst::StringLiteral;
use crate::cst::StringPart;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::TwigTokenKind;
use mago_allocator::prelude::*;

impl<'arena, A> Parser<'_, 'arena, A>
where
    A: Arena,
{
    pub(crate) fn parse_string_expression(&mut self) -> Result<Expression<'arena>, ParseError<'arena>> {
        let mut left = self.parse_string_atom()?;
        let mut next_can_be_string = ends_with_interpolation(&left);
        while let Some(token) = self.stream.lookahead(0)? {
            match token.kind {
                TwigTokenKind::StringSingleQuoted | TwigTokenKind::StringDoubleQuoted if next_can_be_string => {}
                TwigTokenKind::DoubleQuoteStart => {}
                _ => break,
            }

            let right = self.parse_string_atom()?;
            if !next_can_be_string
                && !matches!(&right, Expression::InterpolatedString(string)
                    if matches!(string.parts.first(), Some(StringPart::Interpolation(_))))
            {
                return Err(ParseError::UnexpectedToken(b"unexpected adjacent string", self.stream.span_of(&token)));
            }

            next_can_be_string = ends_with_interpolation(&right);
            left =
                Expression::StringConcatenation(StringConcatenation { lhs: self.alloc(left), rhs: self.alloc(right) });
        }

        Ok(left)
    }

    fn parse_string_atom(&mut self) -> Result<Expression<'arena>, ParseError<'arena>> {
        let token = self.stream.lookahead(0)?.ok_or_else(|| self.stream.unexpected(None, &[]))?;
        if token.kind == TwigTokenKind::DoubleQuoteStart {
            return self.parse_interpolated_string();
        }

        self.stream.consume()?;
        Ok(Expression::String(StringLiteral { raw: token.value, span: self.stream.span_of(&token) }))
    }

    /// Parse an interpolated double-quoted string, producing a sequence of
    /// literal chunks and `#{ ... }` interpolations.
    pub(crate) fn parse_interpolated_string(&mut self) -> Result<Expression<'arena>, ParseError<'arena>> {
        let start_tok = self.stream.consume()?;
        let open_quote = self.stream.span_of(&start_tok);
        let mut parts = self.new_vec();
        loop {
            let token = self
                .stream
                .lookahead(0)?
                .ok_or_else(|| self.stream.unexpected(None, &[TwigTokenKind::DoubleQuoteEnd]))?;
            match token.kind {
                TwigTokenKind::StringPart => {
                    self.stream.consume()?;
                    parts.push(StringPart::Literal(InterpolatedLiteral {
                        value: token.value,
                        span: self.stream.span_of(&token),
                    }));
                }
                TwigTokenKind::InterpolationStart => {
                    let open_tok = self.stream.consume()?;
                    let open_brace = self.stream.span_of(&open_tok);
                    let inner = self.parse_expression()?;
                    let close_tok =
                        self.stream.expect_kind(TwigTokenKind::InterpolationEnd, b"expected `}` closing `#{`")?;
                    let close_brace = self.stream.span_of(&close_tok);
                    parts.push(StringPart::Interpolation(Interpolation {
                        open_brace,
                        expression: self.alloc(inner),
                        close_brace,
                    }));
                }
                TwigTokenKind::DoubleQuoteEnd => {
                    let end_tok = self.stream.consume()?;
                    let close_quote = self.stream.span_of(&end_tok);
                    return Ok(Expression::InterpolatedString(InterpolatedString {
                        open_quote,
                        parts: parts.leak(),
                        close_quote,
                    }));
                }
                _ => {
                    return Err(ParseError::UnexpectedToken(
                        self.arena.alloc_slice_copy(
                            format!("unexpected token {:?} in interpolated string", token.kind).as_bytes(),
                        ),
                        self.stream.span_of(&token),
                    ));
                }
            }
        }
    }
}

fn ends_with_interpolation(expression: &Expression<'_>) -> bool {
    matches!(expression, Expression::InterpolatedString(string)
        if matches!(string.parts.last(), Some(StringPart::Interpolation(_))))
}
