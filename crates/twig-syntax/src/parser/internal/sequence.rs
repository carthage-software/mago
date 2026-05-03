use mago_database::file::HasFileId;
use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::TwigTokenKind;

/// The result of parsing a delimited, token-separated sequence.
pub(crate) struct DelimitedSequenceResult<'arena, T> {
    /// Span of the opening delimiter (e.g. `(`, `[`, `{`).
    pub open: Span,
    /// The parsed elements together with their separator tokens.
    pub sequence: TokenSeparatedSequence<'arena, T>,
    /// Span of the closing delimiter (e.g. `)`, `]`, `}`).
    pub close: Span,
}

impl<'arena> Parser<'_, 'arena> {
    /// Parse a comma-separated, bracketed sequence - the shape used by
    /// every `[ ... ]`, `( ... )`, and `{ ... }` list in Twig (array literals,
    /// argument lists, macro parameters, arrow parameters, ...).
    #[inline]
    pub(crate) fn parse_comma_separated_sequence<T, F>(
        &mut self,
        open_kind: TwigTokenKind,
        close_kind: TwigTokenKind,
        parse_element: F,
    ) -> Result<DelimitedSequenceResult<'arena, T>, ParseError>
    where
        T: HasSpan,
        F: FnMut(&mut Self) -> Result<T, ParseError>,
    {
        self.parse_token_separated_sequence(open_kind, close_kind, TwigTokenKind::Comma, parse_element)
    }

    /// Parse a delimited sequence separated by a single token kind. On
    /// a malformed closer the error is pushed to the parser's `errors`
    /// buffer and the close span comes from whatever token is current.
    pub(crate) fn parse_token_separated_sequence<T, F>(
        &mut self,
        open_kind: TwigTokenKind,
        close_kind: TwigTokenKind,
        separator_kind: TwigTokenKind,
        mut parse_element: F,
    ) -> Result<DelimitedSequenceResult<'arena, T>, ParseError>
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
            let error = self.stream.unexpected(Some(current), &[close_kind]);
            self.errors.push(error);
        }

        Ok(DelimitedSequenceResult {
            open,
            sequence: TokenSeparatedSequence::new(elements, separators),
            close: current.span_for(self.stream.file_id()),
        })
    }
}
