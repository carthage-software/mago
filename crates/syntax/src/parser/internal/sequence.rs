use mago_span::HasSpan;
use mago_span::Span;

use crate::T;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;
use crate::token::TokenKind;

/// Result of parsing a token separated sequence.
pub struct TokenSeparatedSequenceResult<'arena, T> {
    /// The opening delimiter span.
    pub open: Span,
    /// The parsed elements and their separators.
    pub sequence: TokenSeparatedSequence<'arena, T>,
    /// The closing delimiter span.
    pub close: Span,
}

impl<'arena> Parser<'arena> {
    /// Parse a comma-separated sequence.
    ///
    /// This method handles common patterns like:
    /// - `(a, b, c)` - argument lists
    /// - `[1, 2, 3]` - arrays
    /// - `{a, b}` - various constructs
    ///
    /// The `parse_element` closure receives `&mut self` (Parser) and `&mut TokenStream`.
    #[inline(always)]
    pub(crate) fn parse_comma_separated_sequence<T, F>(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
        open_kind: TokenKind,
        close_kind: TokenKind,
        parse_element: F,
    ) -> Result<TokenSeparatedSequenceResult<'arena, T>, ParseError>
    where
        T: HasSpan,
        F: FnMut(&mut Self, &mut TokenStream<'_, 'arena>) -> Result<T, ParseError>,
    {
        self.parse_token_separated_sequence(stream, open_kind, close_kind, T![","], parse_element)
    }

    /// Parse a token-separated sequence.
    ///
    /// This method handles common patterns like:
    /// - `(a, b, c)` - argument lists
    /// - `[1, 2, 3]` - arrays
    /// - `{a, b}` - various constructs
    ///
    /// The `parse_element` closure receives `&mut self` (Parser) and `&mut TokenStream`.
    #[inline]
    pub(crate) fn parse_token_separated_sequence<T, F>(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
        open_kind: TokenKind,
        close_kind: TokenKind,
        separator_kind: TokenKind,
        mut parse_element: F,
    ) -> Result<TokenSeparatedSequenceResult<'arena, T>, ParseError>
    where
        T: HasSpan,
        F: FnMut(&mut Self, &mut TokenStream<'_, 'arena>) -> Result<T, ParseError>,
    {
        let open = stream.eat(open_kind)?.span;

        let mut elements = self.new_vec();
        let mut separators = self.new_vec();

        loop {
            match stream.lookahead(0)?.map(|t| t.kind) {
                Some(kind) if kind == close_kind => break,
                None => return Err(stream.unexpected(None, &[close_kind])),
                _ => {}
            }

            match parse_element(self, stream) {
                Ok(element) => elements.push(element),
                Err(err) => {
                    self.errors.push(err);
                }
            }

            match stream.lookahead(0)?.map(|t| t.kind) {
                Some(kind) if kind == separator_kind => separators.push(stream.consume()?),
                _ => break,
            }
        }

        let Some(current) = stream.lookahead(0)? else {
            return Err(stream.unexpected(None, &[close_kind]));
        };

        if current.kind == close_kind {
            stream.consume()?;
        } else {
            self.errors.push(stream.unexpected(Some(current), &[close_kind]));
        }

        Ok(TokenSeparatedSequenceResult {
            open,
            sequence: TokenSeparatedSequence::new(elements, separators),
            close: current.span,
        })
    }
}
