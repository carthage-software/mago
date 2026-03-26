use mago_database::file::HasFileId;
use mago_span::HasSpan;
use mago_span::Span;

use crate::T;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::Parser;
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

impl<'input, 'arena> Parser<'input, 'arena> {
    /// Parse a comma-separated sequence.
    ///
    /// This method handles common patterns like:
    /// - `(a, b, c)` - argument lists
    /// - `[1, 2, 3]` - arrays
    /// - `{a, b}` - various constructs
    #[inline(always)]
    pub(crate) fn parse_comma_separated_sequence<T, F>(
        &mut self,
        open_kind: TokenKind,
        close_kind: TokenKind,
        parse_element: F,
    ) -> Result<TokenSeparatedSequenceResult<'arena, T>, ParseError>
    where
        T: HasSpan,
        F: FnMut(&mut Self) -> Result<T, ParseError>,
    {
        self.parse_token_separated_sequence(open_kind, close_kind, T![","], parse_element)
    }

    /// Parse a token-separated sequence.
    ///
    /// This method handles common patterns like:
    /// - `(a, b, c)` - argument lists
    /// - `[1, 2, 3]` - arrays
    /// - `{a, b}` - various constructs
    #[inline]
    pub(crate) fn parse_token_separated_sequence<T, F>(
        &mut self,
        open_kind: TokenKind,
        close_kind: TokenKind,
        separator_kind: TokenKind,
        mut parse_element: F,
    ) -> Result<TokenSeparatedSequenceResult<'arena, T>, ParseError>
    where
        T: HasSpan,
        F: FnMut(&mut Self) -> Result<T, ParseError>,
    {
        let open = self.stream.eat_span(open_kind)?;

        let mut elements = self.new_vec();
        let mut separators = self.new_vec();

        loop {
            match self.stream.peek_kind(0)? {
                Some(kind) if kind == close_kind => break,
                None => return Err(self.stream.unexpected(None, &[close_kind])),
                _ => {}
            }

            match parse_element(self) {
                Ok(element) => elements.push(element),
                Err(err) => {
                    self.errors.push(err);
                }
            }

            match self.stream.peek_kind(0)? {
                Some(kind) if kind == separator_kind => separators.push(self.stream.consume()?),
                _ => break,
            }
        }

        let Some(current) = self.stream.lookahead(0)? else {
            return Err(self.stream.unexpected(None, &[close_kind]));
        };

        if current.kind == close_kind {
            self.stream.consume()?;
        } else {
            self.errors.push(self.stream.unexpected(Some(current), &[close_kind]));
        }

        Ok(TokenSeparatedSequenceResult {
            open,
            sequence: TokenSeparatedSequence::new(elements, separators),
            close: current.span_for(self.stream.file_id()),
        })
    }
}
