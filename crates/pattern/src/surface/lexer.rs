//! Lexer for the GritQL-subset surface grammar.
//!
//! Modelled after [`mago_type_syntax::lexer::TypeLexer`]: a byte-oriented scanner over
//! [`mago_syntax_core::input::Input`] that returns an iterator of [`SurfaceToken`]s.
//! Dispatch on the leading bytes via pattern slices keeps the hot path tight, and
//! multi-byte lexemes are read by small helpers that return `(kind, length)`.

use mago_database::file::FileId;
use mago_span::Position;
use mago_syntax_core::input::Input;
use mago_syntax_core::part_of_identifier;
use mago_syntax_core::start_of_identifier;
use mago_syntax_core::start_of_number;

use crate::surface::keyword;
use crate::surface::token::SurfaceToken;
use crate::surface::token::SurfaceTokenKind;

/// Lexer over a surface-grammar pattern source.
#[derive(Debug)]
pub struct SurfaceLexer<'input> {
    input: Input<'input>,
}

impl<'input> SurfaceLexer<'input> {
    /// Creates a lexer over the given source bytes. Pass [`FileId::zero`] for ephemeral
    /// pattern strings that have no associated file.
    #[inline]
    #[must_use]
    pub fn new(source: &'input str) -> Self {
        Self { input: Input::new(FileId::zero(), source.as_bytes()) }
    }

    /// Returns `true` when the lexer has reached the end of the source.
    #[inline]
    #[must_use]
    pub fn has_reached_eof(&self) -> bool {
        self.input.has_reached_eof()
    }

    /// Advances the lexer and returns the next token, or `None` at end of input.
    #[inline]
    pub fn advance(&mut self) -> Option<SurfaceToken<'input>> {
        if self.input.has_reached_eof() {
            return None;
        }

        let start = self.input.current_position();

        // Whitespace
        let whitespaces = self.input.consume_whitespaces();
        if !whitespaces.is_empty() {
            return Some(self.token(SurfaceTokenKind::Whitespace, whitespaces, start));
        }

        let (kind, length) = match self.input.read(2) {
            [b'/', b'/', ..] => self.read_line_comment(),
            [b'<', b':', ..] => (SurfaceTokenKind::Subtype, 2),
            [b'=', b'>', ..] => (SurfaceTokenKind::Rewrite, 2),
            [b'`', ..] => self.read_backtick(),
            [b'"', ..] => self.read_string(),
            [b'^', ..] => self.read_variable(),
            [b'(', ..] => (SurfaceTokenKind::LeftParen, 1),
            [b')', ..] => (SurfaceTokenKind::RightParen, 1),
            [b'{', ..] => (SurfaceTokenKind::LeftBrace, 1),
            [b'}', ..] => (SurfaceTokenKind::RightBrace, 1),
            [b',', ..] => (SurfaceTokenKind::Comma, 1),
            [b'!', ..] => (SurfaceTokenKind::Bang, 1),
            [start_of_number!(), ..] => self.read_number(),
            [start_of_identifier!(), ..] => self.read_identifier_or_keyword(),
            [_, ..] => (SurfaceTokenKind::Unknown, 1),
            [] => unreachable!(),
        };

        let buffer = self.input.consume(length);
        Some(self.token(kind, buffer, start))
    }

    /// Consumes a `// …` single-line comment up to (but not including) the newline.
    #[inline]
    fn read_line_comment(&self) -> (SurfaceTokenKind, usize) {
        let mut length = 2;
        loop {
            match self.input.peek(length, 1) {
                [b'\n', ..] | [] => break,
                [_, ..] => length += 1,
            }
        }
        (SurfaceTokenKind::LineComment, length)
    }

    /// Consumes a `` `…` `` backtick-delimited snippet. The returned length includes both
    /// backticks; escape sequences (`\`` inside the body) are preserved as-is and the
    /// parser unescapes them when extracting the body.
    #[inline]
    fn read_backtick(&self) -> (SurfaceTokenKind, usize) {
        let mut length = 1;
        loop {
            match self.input.peek(length, 1) {
                [] => return (SurfaceTokenKind::UnterminatedBacktick, length),
                [b'`', ..] => {
                    length += 1;
                    return (SurfaceTokenKind::Backtick, length);
                }
                [b'\\', ..] => match self.input.peek(length + 1, 1) {
                    [] => return (SurfaceTokenKind::UnterminatedBacktick, length),
                    [_, ..] => length += 2,
                },
                [_, ..] => length += 1,
            }
        }
    }

    /// Consumes a `"…"` double-quoted string literal. Handles `\X` escape sequences by
    /// swallowing two bytes at a time; actual escape interpretation is deferred to the
    /// parser.
    #[inline]
    fn read_string(&self) -> (SurfaceTokenKind, usize) {
        let mut length = 1;
        loop {
            match self.input.peek(length, 1) {
                [] => return (SurfaceTokenKind::UnterminatedString, length),
                [b'"', ..] => {
                    length += 1;
                    return (SurfaceTokenKind::String, length);
                }
                [b'\\', ..] => match self.input.peek(length + 1, 1) {
                    [] => return (SurfaceTokenKind::UnterminatedString, length),
                    [_, ..] => length += 2,
                },
                [_, ..] => length += 1,
            }
        }
    }

    /// Consumes a `^name` metavariable reference. The returned slice includes the leading
    /// `^`. Bare `^` with no following identifier byte is still emitted as a
    /// `Variable` token of length 1; the parser rejects it.
    #[inline]
    fn read_variable(&self) -> (SurfaceTokenKind, usize) {
        let mut length = 1;
        if let [b'.', b'.', b'.'] = self.input.peek(length, 3) {
            length += 3;
        }
        while let [part_of_identifier!(), ..] = self.input.peek(length, 1) {
            length += 1;
        }
        (SurfaceTokenKind::Variable, length)
    }

    /// Consumes a decimal integer or float literal. Only base-10, no underscores, no
    /// exponents; the surface grammar has no need for them.
    #[inline]
    fn read_number(&self) -> (SurfaceTokenKind, usize) {
        let mut length = 1;
        while let [b'0'..=b'9', ..] = self.input.peek(length, 1) {
            length += 1;
        }
        if let [b'.', b'0'..=b'9', ..] = self.input.peek(length, 2) {
            length += 1;
            while let [b'0'..=b'9', ..] = self.input.peek(length, 1) {
                length += 1;
            }
            return (SurfaceTokenKind::Float, length);
        }
        (SurfaceTokenKind::Integer, length)
    }

    /// Consumes an identifier and resolves it to a keyword kind if applicable.
    #[inline]
    fn read_identifier_or_keyword(&self) -> (SurfaceTokenKind, usize) {
        let mut length = 1;
        while let [part_of_identifier!(), ..] = self.input.peek(length, 1) {
            length += 1;
        }
        let bytes = self.input.read(length);
        let kind = keyword::lookup(bytes).unwrap_or(SurfaceTokenKind::Identifier);
        (kind, length)
    }

    #[inline]
    fn token(&self, kind: SurfaceTokenKind, bytes: &'input [u8], start: Position) -> SurfaceToken<'input> {
        // SAFETY: every call-site above derives `bytes` from `Input` which was constructed
        // from a `&str` in `SurfaceLexer::new`, so the slice is valid UTF-8.
        let value = unsafe { std::str::from_utf8_unchecked(bytes) };
        SurfaceToken::new(kind, value, start)
    }
}

impl<'input> Iterator for SurfaceLexer<'input> {
    type Item = SurfaceToken<'input>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.advance()
    }
}
