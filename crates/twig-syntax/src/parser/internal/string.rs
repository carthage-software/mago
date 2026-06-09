use crate::ast::Expression;
use crate::ast::InterpolatedLiteral;
use crate::ast::InterpolatedString;
use crate::ast::Interpolation;
use crate::ast::StringPart;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::TwigTokenKind;
use mago_allocator::prelude::*;

impl<'arena, A> Parser<'_, 'arena, A>
where
    A: Arena,
{
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
