use mago_database::file::FileId;
use mago_database::file::HasFileId;
use mago_span::Position;
use mago_span::Span;
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
    #[must_use]
    pub fn new(input: Input<'input>) -> TypeLexer<'input> {
        TypeLexer { input }
    }

    #[must_use]
    pub fn has_reached_eof(&self) -> bool {
        self.input.has_reached_eof()
    }

    #[must_use]
    pub fn current_position(&self) -> Position {
        self.input.current_position()
    }

    /// Returns a string slice within a specified absolute range.
    ///
    /// This method exposes the underlying `Input::slice_in_range` functionality but
    /// returns a `&str` instead of a `&[u8]`. It assumes the source is valid UTF-8.
    ///
    /// # Arguments
    ///
    /// * `from` - The absolute starting byte offset.
    /// * `to` - The absolute ending byte offset (exclusive).
    #[inline]
    #[must_use]
    pub fn slice_in_range(&self, from: u32, to: u32) -> &'input str {
        let bytes_slice = self.input.slice_in_range(from, to);

        // Reuse the same safe UTF-8 conversion logic as the `token` method.
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
            [b'n' | b'N', b'o' | b'O', b'n' | b'N'] => {
                if self.input.is_at(b"non-positive-int", true) {
                    (TypeTokenKind::NonPositiveInt, 16)
                } else if self.input.is_at(b"non-negative-int", true) {
                    (TypeTokenKind::NonNegativeInt, 16)
                } else if self.input.is_at(b"non-empty-literal-string", true) {
                    (TypeTokenKind::NonEmptyUnspecifiedLiteralString, 26)
                } else if self.input.is_at(b"non-empty-string", true) {
                    (TypeTokenKind::NonEmptyString, 16)
                } else if self.input.is_at(b"non-empty-array", true) {
                    (TypeTokenKind::NonEmptyArray, 15)
                } else if self.input.is_at(b"non-empty-list", true) {
                    (TypeTokenKind::NonEmptyList, 14)
                } else if self.input.is_at(b"non-falsy-string", true) {
                    (TypeTokenKind::NonFalsyString, 16)
                } else if self.input.is_at(b"non-empty-lowercase-string", true) {
                    (TypeTokenKind::NonEmptyLowercaseString, 26)
                } else if self.input.is_at(b"non-empty-mixed", true) {
                    (TypeTokenKind::NonEmptyMixed, 15)
                } else {
                    self.read_identifier()
                }
            }
            [b'p' | b'P', b'u' | b'U', b'r' | b'R'] => {
                if self.input.is_at(b"pure-closure", true) {
                    (TypeTokenKind::PureClosure, 12)
                } else if self.input.is_at(b"pure-callable", true) {
                    (TypeTokenKind::PureCallable, 13)
                } else {
                    self.read_identifier()
                }
            }
            [b'n' | b'N', b'e' | b'E', b'v' | b'V'] => {
                if self.input.is_at(b"never-return", true) {
                    (TypeTokenKind::NeverReturn, 12)
                } else if self.input.is_at(b"never-returns", true) {
                    (TypeTokenKind::NeverReturns, 13)
                } else {
                    self.read_identifier()
                }
            }
            [b't' | b'T', b'r' | b'R', b'u' | b'U'] => {
                if self.input.is_at(b"truthy-string", true) {
                    (TypeTokenKind::TruthyString, 13)
                } else {
                    self.read_identifier()
                }
            }
            [b't' | b'T', b'r' | b'R', b'a' | b'A'] => {
                if self.input.is_at(b"trait-string", true) {
                    (TypeTokenKind::TraitString, 12)
                } else {
                    self.read_identifier()
                }
            }
            [b'a' | b'A', b's' | b'S', b's' | b'S'] => {
                if self.input.is_at(b"associative-array", true) {
                    (TypeTokenKind::AssociativeArray, 17)
                } else {
                    self.read_identifier()
                }
            }
            [b'c' | b'C', b'l' | b'L', b'a' | b'A'] => {
                if self.input.is_at(b"class-string", true) {
                    (TypeTokenKind::ClassString, 12)
                } else {
                    self.read_identifier()
                }
            }
            [b'e' | b'E', b'n' | b'N', b'u' | b'U'] => {
                if self.input.is_at(b"enum-string", true) {
                    (TypeTokenKind::EnumString, 11)
                } else {
                    self.read_identifier()
                }
            }
            [b'i' | b'I', b'n' | b'N', b't' | b'T'] => {
                if self.input.is_at(b"interface-string", true) {
                    (TypeTokenKind::InterfaceString, 16)
                } else if self.input.is_at(b"int-mask-of", true) {
                    (TypeTokenKind::IntMaskOf, 11)
                } else if self.input.is_at(b"int-mask", true) {
                    (TypeTokenKind::IntMask, 8)
                } else {
                    self.read_identifier()
                }
            }
            [b'c' | b'C', b'l' | b'L', b'o' | b'O'] => {
                if self.input.is_at(b"closed-resource", true) {
                    (TypeTokenKind::ClosedResource, 15)
                } else {
                    self.read_identifier()
                }
            }
            [b's' | b'S', b't' | b'T', b'r' | b'R'] => {
                if self.input.is_at(b"stringable-object", true) {
                    (TypeTokenKind::StringableObject, 17)
                } else {
                    self.read_identifier()
                }
            }
            [b'n' | b'N', b'u' | b'U', b'm' | b'M'] => {
                if self.input.is_at(b"numeric-string", true) {
                    (TypeTokenKind::NumericString, 14)
                } else {
                    self.read_identifier()
                }
            }
            [b'l' | b'L', b'i' | b'I', b't' | b'T'] => {
                if self.input.is_at(b"literal-string", true) {
                    (TypeTokenKind::UnspecifiedLiteralString, 14)
                } else if self.input.is_at(b"literal-int", true) {
                    (TypeTokenKind::UnspecifiedLiteralInt, 11)
                } else if self.input.is_at(b"literal-float", true) {
                    (TypeTokenKind::UnspecifiedLiteralFloat, 13)
                } else {
                    self.read_identifier()
                }
            }
            [b'l' | b'L', b'o' | b'O', b'w' | b'W'] => {
                if self.input.is_at(b"lowercase-string", true) {
                    (TypeTokenKind::LowercaseString, 16)
                } else {
                    self.read_identifier()
                }
            }
            [b'o' | b'O', b'p' | b'P', b'e' | b'E'] => {
                if self.input.is_at(b"open-resource", true) {
                    (TypeTokenKind::OpenResource, 13)
                } else {
                    self.read_identifier()
                }
            }
            [b'a' | b'A', b'r' | b'R', b'r' | b'R'] => {
                if self.input.is_at(b"array-key", true) {
                    (TypeTokenKind::ArrayKey, 9)
                } else {
                    self.read_identifier()
                }
            }
            [b'n' | b'N', b'o' | b'O', b'-'] => {
                if self.input.is_at(b"no-return", true) {
                    (TypeTokenKind::NoReturn, 9)
                } else {
                    self.read_identifier()
                }
            }
            [b'v' | b'V', b'a' | b'A', b'l' | b'L'] => {
                if self.input.is_at(b"value-of", true) {
                    (TypeTokenKind::ValueOf, 8)
                } else {
                    self.read_identifier()
                }
            }
            [b'k' | b'K', b'e' | b'E', b'y' | b'Y'] => {
                if self.input.is_at(b"key-of", true) {
                    (TypeTokenKind::KeyOf, 6)
                } else {
                    self.read_identifier()
                }
            }
            [b'p' | b'P', b'r' | b'R', b'o' | b'O'] => {
                if self.input.is_at(b"protected-properties-of", true) {
                    (TypeTokenKind::ProtectedPropertiesOf, 23)
                } else if self.input.is_at(b"properties-of", true) {
                    (TypeTokenKind::PropertiesOf, 13)
                } else {
                    self.read_identifier()
                }
            }
            [b'p' | b'P', b'u' | b'U', b'b' | b'B'] => {
                if self.input.is_at(b"public-properties-of", true) {
                    (TypeTokenKind::PublicPropertiesOf, 20)
                } else {
                    self.read_identifier()
                }
            }
            [b'p' | b'P', b'r' | b'R', b'i' | b'I'] => {
                if self.input.is_at(b"private-properties-of", true) {
                    (TypeTokenKind::PrivatePropertiesOf, 21)
                } else {
                    self.read_identifier()
                }
            }
            [b'p' | b'P', b'o' | b'O', b's' | b'S'] => {
                if self.input.is_at(b"positive-int", true) {
                    (TypeTokenKind::PositiveInt, 12)
                } else {
                    self.read_identifier()
                }
            }
            [b'n' | b'N', b'e' | b'E', b'g' | b'G'] => {
                if self.input.is_at(b"negative-int", true) {
                    (TypeTokenKind::NegativeInt, 12)
                } else {
                    self.read_identifier()
                }
            }
            [b'.', b'.', b'.'] => (TypeTokenKind::Ellipsis, 3),
            [b':', b':', ..] => (TypeTokenKind::ColonColon, 2),
            [b'/', b'/', ..] => self.read_single_line_comment(),
            [b'.', start_of_number!(), ..] => self.read_decimal(),
            [start_of_number!(), ..] => self.read_number(),
            [quote @ (b'\'' | b'"'), ..] => self.read_literal_string(*quote),
            [b'\\', start_of_identifier!(), ..] => self.read_fully_qualified_identifier(),
            [start_of_identifier!(), ..] => self.read_identifier(),
            [b'$', start_of_identifier!(), ..] => {
                let mut length = 2;
                while let [part_of_identifier!(), ..] = self.input.peek(length, 1) {
                    length += 1;
                }

                (TypeTokenKind::Variable, length)
            }
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
            [] => {
                unreachable!()
            }
        };

        let buffer = self.input.consume(length);
        let end = self.input.current_position();

        Some(Ok(self.token(kind, buffer, start, end)))
    }

    fn read_single_line_comment(&self) -> (TypeTokenKind, usize) {
        let mut length = 2;
        loop {
            match self.input.peek(length, 1) {
                [b'\n', ..] | [] => {
                    break;
                }
                [_, ..] => {
                    length += 1;
                }
            }
        }

        (TypeTokenKind::SingleLineComment, length)
    }

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

    fn read_number(&self) -> (TypeTokenKind, usize) {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub enum NumberKind {
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

    fn read_literal_string(&self, quote: u8) -> (TypeTokenKind, usize) {
        let total = self.input.len();
        let start = self.input.current_offset();
        let mut length = 1; // We assume the opening quote is already consumed.
        let mut last_was_backslash = false;
        let mut partial = false;

        loop {
            let pos = start + length;
            if pos >= total {
                // Reached EOF before closing quote.
                partial = true;
                break;
            }

            let byte = self.input.read_at(pos);
            if matches!(byte, b'\\') {
                // Toggle the backslash flag.
                last_was_backslash = !last_was_backslash;
                length += 1;
            } else {
                // If we see the closing quote and the previous byte was not an escape.
                if byte == &quote && !last_was_backslash {
                    length += 1; // Include the closing quote.
                    break;
                }

                length += 1;
                last_was_backslash = false;
            }
        }

        if partial { (TypeTokenKind::PartialLiteralString, length) } else { (TypeTokenKind::LiteralString, length) }
    }

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
                _ => {
                    break;
                }
            }
        }

        (TypeTokenKind::FullyQualifiedIdentifier, length)
    }

    fn read_identifier(&self) -> (TypeTokenKind, usize) {
        const KEYWORD_TYPES: [(&[u8], TypeTokenKind); 28] = [
            (b"list", TypeTokenKind::List),
            (b"int", TypeTokenKind::Int),
            (b"integer", TypeTokenKind::Integer),
            (b"string", TypeTokenKind::String),
            (b"float", TypeTokenKind::Float),
            (b"double", TypeTokenKind::Double),
            (b"real", TypeTokenKind::Real),
            (b"bool", TypeTokenKind::Bool),
            (b"boolean", TypeTokenKind::Boolean),
            (b"false", TypeTokenKind::False),
            (b"true", TypeTokenKind::True),
            (b"object", TypeTokenKind::Object),
            (b"callable", TypeTokenKind::Callable),
            (b"array", TypeTokenKind::Array),
            (b"iterable", TypeTokenKind::Iterable),
            (b"null", TypeTokenKind::Null),
            (b"mixed", TypeTokenKind::Mixed),
            (b"resource", TypeTokenKind::Resource),
            (b"void", TypeTokenKind::Void),
            (b"scalar", TypeTokenKind::Scalar),
            (b"numeric", TypeTokenKind::Numeric),
            (b"never", TypeTokenKind::Never),
            (b"nothing", TypeTokenKind::Nothing),
            (b"as", TypeTokenKind::As),
            (b"is", TypeTokenKind::Is),
            (b"not", TypeTokenKind::Not),
            (b"min", TypeTokenKind::Min),
            (b"max", TypeTokenKind::Max),
        ];

        let mut length = 1;
        let mut ended_with_slash = false;
        loop {
            match self.input.peek(length, 2) {
                [part_of_identifier!(), ..] => {
                    length += 1;
                }
                [b'\\', start_of_identifier!(), ..] => {
                    ended_with_slash = true;
                    break;
                }
                _ => {
                    break;
                }
            }
        }

        if !ended_with_slash {
            for (value, kind) in KEYWORD_TYPES {
                let keyword_length = value.len();
                if keyword_length != length {
                    continue;
                }

                if self.input.is_at(value, true) {
                    return (kind, keyword_length);
                }
            }
        }

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
                        last_was_slash = false;

                        break;
                    }
                    length += 1;
                    slashes += 1;
                    last_was_slash = true;
                }
                _ => {
                    break;
                }
            }
        }

        if last_was_slash {
            length -= 1;
            slashes -= 1;
        }

        if slashes > 0 { (TypeTokenKind::QualifiedIdentifier, length) } else { (TypeTokenKind::Identifier, length) }
    }

    #[inline]
    fn token(&self, kind: TypeTokenKind, value: &'input [u8], from: Position, to: Position) -> TypeToken<'input> {
        let mut value_chunks = value.utf8_chunks();
        let value_str = if let Some(chunk) = value_chunks.next() {
            let valid = chunk.valid();

            debug_assert_eq!(valid.len(), value.len());

            valid
        } else {
            ""
        };

        TypeToken { kind, value: value_str, span: Span::new(self.file_id(), from, to) }
    }
}

impl HasFileId for TypeLexer<'_> {
    fn file_id(&self) -> FileId {
        self.input.file_id()
    }
}
