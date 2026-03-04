mod keyword;

use mago_database::file::FileId;
use mago_database::file::HasFileId;
use mago_span::Position;
use mago_syntax_core::float_exponent;
use mago_syntax_core::float_separator;
use mago_syntax_core::input::Input;
use mago_syntax_core::number_sign;
use mago_syntax_core::part_of_identifier;
use mago_syntax_core::start_of_binary_number;
use mago_syntax_core::start_of_float_number;
use mago_syntax_core::start_of_hexadecimal_number;
use mago_syntax_core::start_of_identifier;
use mago_syntax_core::start_of_number;
use mago_syntax_core::start_of_octal_number;
use mago_syntax_core::start_of_octal_or_float_number;
use mago_syntax_core::utils::read_digits_of_base;

use crate::error::SyntaxError;
use crate::token::TypeToken;
use crate::token::TypeTokenKind;

#[derive(Debug)]
pub struct TypeLexer<'input> {
    input: Input<'input>,
}

impl<'input> TypeLexer<'input> {
    #[inline]
    #[must_use]
    pub fn new(input: Input<'input>) -> TypeLexer<'input> {
        TypeLexer { input }
    }

    #[inline]
    #[must_use]
    pub fn has_reached_eof(&self) -> bool {
        self.input.has_reached_eof()
    }

    #[inline]
    #[must_use]
    pub fn current_position(&self) -> Position {
        self.input.current_position()
    }

    #[inline]
    #[must_use]
    pub fn slice_in_range(&self, from: u32, to: u32) -> &'input str {
        let bytes_slice = self.input.slice_in_range(from, to);
        bytes_slice.utf8_chunks().next().map_or("", |chunk| chunk.valid())
    }

    #[inline]
    pub fn advance(&mut self) -> Option<Result<TypeToken<'input>, SyntaxError>> {
        if self.input.has_reached_eof() {
            return None;
        }

        let start = self.input.current_position();
        let whitespaces = self.input.consume_whitespaces();
        if !whitespaces.is_empty() {
            let end = self.input.current_position();
            return Some(Ok(self.token(TypeTokenKind::Whitespace, whitespaces, start, end)));
        }

        let (kind, length) = match self.input.read(3) {
            [b'*', ..] => (TypeTokenKind::Asterisk, 1),
            [b'.', b'.', b'.'] => (TypeTokenKind::Ellipsis, 3),
            [b':', b':', ..] => (TypeTokenKind::ColonColon, 2),
            [b'/', b'/', ..] => self.read_single_line_comment(),
            [b'.', start_of_number!(), ..] => self.read_decimal(),
            [start_of_number!(), ..] => self.read_number(),
            [quote @ (b'\'' | b'"'), ..] => self.read_literal_string(*quote),
            [b'\\', start_of_identifier!(), ..] => self.read_fully_qualified_identifier(),
            [start_of_identifier!(), ..] => self.read_identifier_or_keyword(),
            [b'$', start_of_identifier!(), ..] => self.read_variable(),
            [b':', ..] => (TypeTokenKind::Colon, 1),
            [b'=', ..] => (TypeTokenKind::Equals, 1),
            [b'?', ..] => (TypeTokenKind::Question, 1),
            [b'!', ..] => (TypeTokenKind::Exclamation, 1),
            [b'&', ..] => (TypeTokenKind::Ampersand, 1),
            [b'|', ..] => (TypeTokenKind::Pipe, 1),
            [b'>', ..] => (TypeTokenKind::GreaterThan, 1),
            [b'<', ..] => (TypeTokenKind::LessThan, 1),
            [b'(', ..] => (TypeTokenKind::LeftParenthesis, 1),
            [b')', ..] => (TypeTokenKind::RightParenthesis, 1),
            [b'[', ..] => (TypeTokenKind::LeftBracket, 1),
            [b']', ..] => (TypeTokenKind::RightBracket, 1),
            [b'{', ..] => (TypeTokenKind::LeftBrace, 1),
            [b'}', ..] => (TypeTokenKind::RightBrace, 1),
            [b',', ..] => (TypeTokenKind::Comma, 1),
            [b'+', ..] => (TypeTokenKind::Plus, 1),
            [b'-', ..] => (TypeTokenKind::Minus, 1),
            [unknown_byte, ..] => {
                return Some(Err(SyntaxError::UnrecognizedToken(
                    self.file_id(),
                    *unknown_byte,
                    self.input.current_position(),
                )));
            }
            [] => unreachable!(),
        };

        let buffer = self.input.consume(length);
        let end = self.input.current_position();

        Some(Ok(self.token(kind, buffer, start, end)))
    }

    #[inline]
    fn read_variable(&self) -> (TypeTokenKind, usize) {
        let mut length = 2;
        while let [part_of_identifier!(), ..] = self.input.peek(length, 1) {
            length += 1;
        }
        (TypeTokenKind::Variable, length)
    }

    #[inline]
    fn read_single_line_comment(&self) -> (TypeTokenKind, usize) {
        let mut length = 2;
        loop {
            match self.input.peek(length, 1) {
                [b'\n', ..] | [] => break,
                [_, ..] => length += 1,
            }
        }
        (TypeTokenKind::SingleLineComment, length)
    }

    #[inline]
    fn read_decimal(&self) -> (TypeTokenKind, usize) {
        let mut length = read_digits_of_base(&self.input, 2, 10);
        if let float_exponent!() = self.input.peek(length, 1) {
            length += 1;
            if let number_sign!() = self.input.peek(length, 1) {
                length += 1;
            }
            length = read_digits_of_base(&self.input, length, 10);
        }
        (TypeTokenKind::LiteralFloat, length)
    }

    #[inline]
    fn read_number(&self) -> (TypeTokenKind, usize) {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        enum NumberKind {
            Integer,
            Float,
            OctalOrFloat,
            IntegerOrFloat,
        }

        let mut length = 1;
        let (base, kind): (u8, NumberKind) = match self.input.read(3) {
            start_of_binary_number!() => {
                length += 1;
                (2, NumberKind::Integer)
            }
            start_of_octal_number!() => {
                length += 1;
                (8, NumberKind::Integer)
            }
            start_of_hexadecimal_number!() => {
                length += 1;
                (16, NumberKind::Integer)
            }
            start_of_octal_or_float_number!() => (10, NumberKind::OctalOrFloat),
            start_of_float_number!() => (10, NumberKind::Float),
            _ => (10, NumberKind::IntegerOrFloat),
        };

        if kind != NumberKind::Float {
            length = read_digits_of_base(&self.input, length, base);
            if kind == NumberKind::Integer {
                return (TypeTokenKind::LiteralInteger, length);
            }
        }

        let is_float = matches!(self.input.peek(length, 3), float_separator!());
        if !is_float {
            return (TypeTokenKind::LiteralInteger, length);
        }

        if let [b'.'] = self.input.peek(length, 1) {
            length += 1;
            length = read_digits_of_base(&self.input, length, 10);
        }

        if let float_exponent!() = self.input.peek(length, 1) {
            length += 1;
            if let number_sign!() = self.input.peek(length, 1) {
                length += 1;
            }
            length = read_digits_of_base(&self.input, length, 10);
        }

        (TypeTokenKind::LiteralFloat, length)
    }

    #[inline]
    fn read_literal_string(&self, quote: u8) -> (TypeTokenKind, usize) {
        let total = self.input.len();
        let start = self.input.current_offset();
        let mut length = 1;
        let mut last_was_backslash = false;
        let mut partial = false;

        loop {
            let pos = start + length;
            if pos >= total {
                partial = true;
                break;
            }

            let byte = self.input.read_at(pos);
            if *byte == b'\\' {
                last_was_backslash = !last_was_backslash;
                length += 1;
            } else {
                if byte == &quote && !last_was_backslash {
                    length += 1;
                    break;
                }
                length += 1;
                last_was_backslash = false;
            }
        }

        if partial { (TypeTokenKind::PartialLiteralString, length) } else { (TypeTokenKind::LiteralString, length) }
    }

    #[inline]
    fn read_fully_qualified_identifier(&self) -> (TypeTokenKind, usize) {
        let mut length = 2;
        let mut last_was_slash = false;
        loop {
            match self.input.peek(length, 1) {
                [start_of_identifier!(), ..] if last_was_slash => {
                    length += 1;
                    last_was_slash = false;
                }
                [part_of_identifier!(), ..] if !last_was_slash => {
                    length += 1;
                }
                [b'\\', ..] => {
                    if last_was_slash {
                        length -= 1;
                        break;
                    }
                    length += 1;
                    last_was_slash = true;
                }
                _ => break,
            }
        }
        (TypeTokenKind::FullyQualifiedIdentifier, length)
    }

    /// Read an identifier or keyword (including compound keywords with hyphens).
    /// This is the hot path - optimized for common case (simple identifiers).
    #[inline]
    fn read_identifier_or_keyword(&self) -> (TypeTokenKind, usize) {
        let mut length = 1;
        let mut next_is_hyphen = false;
        let mut next_is_backslash = false;

        loop {
            match self.input.peek(length, 2) {
                [part_of_identifier!(), ..] => length += 1,
                [b'-', start_of_identifier!() | part_of_identifier!(), ..] => {
                    next_is_hyphen = true;
                    break;
                }
                [b'\\', start_of_identifier!(), ..] => {
                    next_is_backslash = true;
                    break;
                }
                _ => break,
            }
        }

        if next_is_backslash {
            return self.finish_qualified_identifier(length);
        }

        if !next_is_hyphen {
            let bytes = self.input.read(length);
            if let Some(kind) = keyword::lookup_keyword(bytes) {
                return (kind, length);
            }
            return (TypeTokenKind::Identifier, length);
        }

        let base_len = length;
        loop {
            match self.input.peek(length, 2) {
                [part_of_identifier!(), ..] => length += 1,
                [b'-', start_of_identifier!() | part_of_identifier!(), ..] => length += 1,
                _ => break,
            }
        }

        let bytes = self.input.read(length);
        if let Some(kind) = keyword::lookup_keyword(bytes) {
            return (kind, length);
        }

        let base_bytes = self.input.read(base_len);
        if let Some(kind) = keyword::lookup_keyword(base_bytes) {
            return (kind, base_len);
        }

        (TypeTokenKind::Identifier, base_len)
    }

    /// Continue reading a qualified identifier (with backslashes).
    #[inline]
    fn finish_qualified_identifier(&self, start_len: usize) -> (TypeTokenKind, usize) {
        let mut length = start_len;
        let mut slashes = 0;
        let mut last_was_slash = false;

        loop {
            match self.input.peek(length, 1) {
                [start_of_identifier!(), ..] if last_was_slash => {
                    length += 1;
                    last_was_slash = false;
                }
                [part_of_identifier!(), ..] if !last_was_slash => {
                    length += 1;
                }
                [b'\\', ..] => {
                    if last_was_slash {
                        length -= 1;
                        slashes -= 1;
                        break;
                    }
                    length += 1;
                    slashes += 1;
                    last_was_slash = true;
                }
                _ => break,
            }
        }

        if last_was_slash {
            length -= 1;
            slashes -= 1;
        }

        if slashes > 0 { (TypeTokenKind::QualifiedIdentifier, length) } else { (TypeTokenKind::Identifier, length) }
    }

    #[inline]
    fn token(&self, kind: TypeTokenKind, value: &'input [u8], start: Position, _end: Position) -> TypeToken<'input> {
        let value_str = value.utf8_chunks().next().map_or("", |chunk| chunk.valid());
        debug_assert_eq!(value_str.len(), value.len());
        TypeToken { kind, start, value: value_str }
    }
}

impl HasFileId for TypeLexer<'_> {
    #[inline]
    fn file_id(&self) -> FileId {
        self.input.file_id()
    }
}
