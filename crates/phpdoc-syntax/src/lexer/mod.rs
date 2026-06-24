use mago_database::file::FileId;
use mago_database::file::HasFileId;
use mago_span::Position;
use mago_syntax_core::input::Input;
use mago_syntax_core::utils::is_part_of_identifier;
use mago_syntax_core::utils::is_start_of_identifier;
use mago_syntax_core::utils::read_digits_of_base;

use crate::token::Token;
use crate::token::TokenKind;

const TAG_TABLE: [bool; 256] = {
    let mut table = [false; 256];
    let mut i = 0;
    while i < 256 {
        table[i] = matches!(i as u8, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'-' | b'\\');
        i += 1;
    }

    table
};

#[inline]
const fn is_identifier_start(byte: u8) -> bool {
    is_start_of_identifier(&byte) || byte >= 0x80
}

#[derive(Debug)]
pub struct DocblockLexer<'arena> {
    input: Input<'arena>,
    inline_code: Option<usize>,
}

impl<'arena> DocblockLexer<'arena> {
    #[inline]
    #[must_use]
    pub fn new(input: Input<'arena>) -> DocblockLexer<'arena> {
        DocblockLexer { input, inline_code: None }
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
    pub fn advance(&mut self) -> Option<Token<'arena>> {
        if self.input.has_reached_eof() {
            return None;
        }

        let start = self.input.current_position();
        let whitespaces = self.input.consume_whitespaces();
        if !whitespaces.is_empty() {
            if memchr::memchr2(b'\n', b'\r', whitespaces).is_some() {
                self.inline_code = None;
            }

            return Some(Token::new(TokenKind::Whitespace, whitespaces, start));
        }

        let remaining = self.input.read_remaining();
        let first = remaining[0];
        let second = remaining.get(1).copied();

        let (kind, length) = match first {
            b'|' => (TokenKind::Pipe, 1),
            b'&' => (TokenKind::Ampersand, 1),
            b'?' => (TokenKind::Question, 1),
            b'!' => (TokenKind::Bang, 1),
            b'(' => (TokenKind::LeftParenthesis, 1),
            b')' => (TokenKind::RightParenthesis, 1),
            b'<' => (TokenKind::LeftAngleBracket, 1),
            b'>' => (TokenKind::RightAngleBracket, 1),
            b'[' => (TokenKind::LeftBracket, 1),
            b']' => (TokenKind::RightBracket, 1),
            b'{' => (TokenKind::LeftBrace, 1),
            b'}' => (TokenKind::RightBrace, 1),
            b',' => (TokenKind::Comma, 1),
            b':' if second == Some(b':') => (TokenKind::ColonColon, 2),
            b':' => (TokenKind::Colon, 1),
            b'=' if second == Some(b'>') => (TokenKind::DoubleArrow, 2),
            b'=' => (TokenKind::Equals, 1),
            b'*' if second == Some(b'/') => (TokenKind::ClosingMarker, 2),
            b'*' => (TokenKind::Asterisk, 1),
            b'.' if remaining.get(..3) == Some(b"...") => (TokenKind::Ellipsis, 3),
            b'.' if second.is_some_and(|b| b.is_ascii_digit()) => self.read_number(),
            b'-' if second == Some(b'>') => (TokenKind::Arrow, 2),
            b'-' => (TokenKind::Minus, 1),
            b'+' => (TokenKind::Plus, 1),
            b'`' => (TokenKind::Backtick, remaining.iter().take_while(|&&byte| byte == b'`').count()),
            b'/' if self.inline_code.is_none() => self.read_slash(remaining),
            b'\'' | b'"' if self.inline_code.is_none() => self.read_string(first),
            b'$' => self.read_variable().unwrap_or_else(|| self.read_other()),
            b'@' => self.read_tag().unwrap_or_else(|| self.read_other()),
            b'0'..=b'9' => self.read_number(),
            b'\\' if second.is_some_and(is_identifier_start) => (TokenKind::Identifier, self.read_identifier()),
            b if is_identifier_start(b) => (TokenKind::Identifier, self.read_identifier()),
            _ => self.read_other(),
        };

        if kind == TokenKind::Backtick {
            match self.inline_code {
                None => self.inline_code = Some(length),
                Some(open) if open == length => self.inline_code = None,
                Some(_) => {}
            }
        }

        let value = self.input.consume(length);

        Some(Token::new(kind, value, start))
    }

    #[inline]
    fn read_slash(&self, remaining: &[u8]) -> (TokenKind, usize) {
        if remaining.get(1) == Some(&b'/') && self.at_line_comment_boundary() {
            return self.read_line_comment();
        }

        if remaining.get(..3) == Some(b"/**")
            && let Some(after) = remaining.get(3).copied()
            && after.is_ascii_whitespace()
        {
            return (TokenKind::OpeningMarker, if after == b' ' { 4 } else { 3 });
        }

        self.read_other()
    }

    /// A `//` only opens a line comment when it stands at the start of the
    /// (meaningful) line — i.e. the byte before it is whitespace, or it is the
    /// very start of the input. When `//` is stuck to other text (e.g. the
    /// `://` in a `https://` URL), it is not a comment and is lexed as text.
    #[inline]
    fn at_line_comment_boundary(&self) -> bool {
        let offset = self.input.current_offset();

        offset == 0 || self.input.read_at(offset - 1).is_ascii_whitespace()
    }

    #[inline]
    fn read_line_comment(&self) -> (TokenKind, usize) {
        let tail = &self.input.read_remaining()[2..];

        let newline = memchr::memchr2(b'\n', b'\r', tail);
        let marker = memchr::memmem::find(tail, b"*/");

        let stop = match (newline, marker) {
            (Some(newline), Some(marker)) => newline.min(marker),
            (Some(stop), None) | (None, Some(stop)) => stop,
            (None, None) => tail.len(),
        };

        (TokenKind::LineComment, 2 + stop)
    }

    #[inline]
    fn read_other(&self) -> (TokenKind, usize) {
        let remaining = self.input.read_remaining();
        let mut length = 0;

        while length < remaining.len() {
            let byte = remaining[length];

            if byte.is_ascii_whitespace() || byte == b'{' || byte == b'}' || byte == b'`' {
                break;
            }

            if byte == b'*' && remaining.get(length + 1) == Some(&b'/') {
                break;
            }

            length += 1;
        }

        (TokenKind::Other, length.max(1))
    }

    #[inline]
    fn read_identifier(&self) -> usize {
        let remaining = self.input.read_remaining();
        let total = remaining.len();
        let mut length = 0;

        loop {
            let (segment, ends_with_namespace_separator) = self.input.scan_identifier(length);
            length += segment;

            if ends_with_namespace_separator {
                length += 1;
                continue;
            }

            let mut consumed_hyphen = false;
            while length < total && remaining[length] == b'-' {
                length += 1;
                consumed_hyphen = true;
            }

            if !consumed_hyphen {
                break;
            }
        }

        length
    }

    #[inline]
    fn read_variable(&self) -> Option<(TokenKind, usize)> {
        let remaining = self.input.read_remaining();

        if remaining.get(..5) == Some(b"$this") && remaining.get(5).is_none_or(|b| !is_part_of_identifier(b)) {
            return Some((TokenKind::ThisVariable, 5));
        }

        if remaining.get(1).copied().is_some_and(is_identifier_start) {
            let (name, _) = self.input.scan_identifier(1);

            return Some((TokenKind::Variable, 1 + name));
        }

        None
    }

    #[inline]
    fn read_tag(&self) -> Option<(TokenKind, usize)> {
        let remaining = self.input.read_remaining();

        if remaining.get(1).is_none_or(|byte| !byte.is_ascii_alphabetic()) {
            return None;
        }

        let mut length = 1 + self.input.scan_while(1, &TAG_TABLE);

        if remaining.get(length) == Some(&b':')
            && remaining.get(length + 1).is_some_and(|byte| byte.is_ascii_alphabetic())
        {
            length += 1 + self.input.scan_while(length + 1, &TAG_TABLE);
        }

        Some((TokenKind::Tag, length))
    }

    #[inline]
    fn read_string(&self, quote: u8) -> (TokenKind, usize) {
        let remaining = self.input.read_remaining();
        let mut index = 1;

        while let Some(tail) = remaining.get(index..) {
            let Some(stop) = memchr::memchr2(quote, b'\\', tail) else {
                break;
            };

            if memchr::memchr2(b'\r', b'\n', tail).is_some_and(|newline| newline < stop) {
                break;
            }

            if tail[stop] == quote {
                let kind = if quote == b'\'' { TokenKind::SingleQuotedString } else { TokenKind::DoubleQuotedString };

                return (kind, index + stop + 1);
            }

            if tail.get(stop + 1).is_none_or(|byte| matches!(byte, b'\r' | b'\n')) {
                break;
            }

            index += stop + 2;
        }

        let rest = &remaining[1..];
        let mut end = rest.len();
        if let Some(newline) = memchr::memchr2(b'\n', b'\r', rest) {
            end = end.min(newline);
        }

        if let Some(marker) = memchr::memmem::find(rest, b"*/") {
            end = end.min(marker);
        }

        (TokenKind::PartialString, 1 + end)
    }

    #[inline]
    fn read_number(&self) -> (TokenKind, usize) {
        let remaining = self.input.read_remaining();
        let mut length = 0;
        let mut is_float = false;

        if remaining.get(length) == Some(&b'0')
            && let Some(base) = remaining.get(length + 1).copied().and_then(|byte| match byte {
                b'b' | b'B' => Some(2u8),
                b'o' | b'O' => Some(8u8),
                b'x' | b'X' => Some(16u8),
                _ => None,
            })
        {
            let after_prefix = length + 2;
            let end = read_digits_of_base(&self.input, after_prefix, base);
            if end > after_prefix {
                return (TokenKind::LiteralInteger, end);
            }
        }

        length = read_digits_of_base(&self.input, length, 10);

        if remaining.get(length) == Some(&b'.') && remaining.get(length + 1).is_some_and(u8::is_ascii_digit) {
            is_float = true;
            length = read_digits_of_base(&self.input, length + 1, 10);
        }

        if matches!(remaining.get(length).copied(), Some(b'e' | b'E')) {
            let mut exponent = length + 1;
            if matches!(remaining.get(exponent).copied(), Some(b'+' | b'-')) {
                exponent += 1;
            }

            if remaining.get(exponent).is_some_and(u8::is_ascii_digit) {
                is_float = true;
                length = read_digits_of_base(&self.input, exponent, 10);
            }
        }

        if is_float { (TokenKind::LiteralFloat, length) } else { (TokenKind::LiteralInteger, length) }
    }
}

impl HasFileId for DocblockLexer<'_> {
    #[inline]
    fn file_id(&self) -> FileId {
        self.input.file_id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_sample<T>(source: T, expected: Vec<TokenKind>)
    where
        T: AsRef<[u8]>,
    {
        let bytes = source.as_ref();

        let input = Input::new(FileId::zero(), bytes);
        let mut lexer = DocblockLexer::new(input);
        let mut kinds = Vec::new();
        let mut reconstructed = Vec::new();
        while let Some(token) = lexer.advance() {
            kinds.push(token.kind);
            reconstructed.extend_from_slice(token.value);
        }

        assert_eq!(kinds, expected, "Failed to assert token kinds for input: {:?}", String::from_utf8_lossy(bytes));
        assert_eq!(
            reconstructed,
            bytes,
            "Failed to assert reconstructability for input: {:?}",
            String::from_utf8_lossy(bytes)
        );
    }

    macro_rules! test {
        ($name:ident, $source:expr, $expected:expr) => {
            #[test]
            fn $name() {
                test_sample($source, $expected);
            }
        };
    }

    test!(
        lexes_single_line_param,
        b"/** @param int<0, 100>|null $count */",
        vec![
            TokenKind::OpeningMarker,
            TokenKind::Tag,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::LeftAngleBracket,
            TokenKind::LiteralInteger,
            TokenKind::Comma,
            TokenKind::Whitespace,
            TokenKind::LiteralInteger,
            TokenKind::RightAngleBracket,
            TokenKind::Pipe,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Variable,
            TokenKind::Whitespace,
            TokenKind::ClosingMarker,
        ]
    );

    test!(
        lexes_multiline_var_shape_absorbing_star,
        "/**\n* @var array{a: int}\n*/",
        vec![
            TokenKind::OpeningMarker,
            TokenKind::Whitespace,
            TokenKind::Asterisk,
            TokenKind::Whitespace,
            TokenKind::Tag,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::LeftBrace,
            TokenKind::Identifier,
            TokenKind::Colon,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::RightBrace,
            TokenKind::Whitespace,
            TokenKind::ClosingMarker,
        ]
    );

    test!(
        lexes_namespaced_and_hyphenated_identifiers_and_this,
        b"\\Foo\\Bar non-empty-string $this",
        vec![
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::ThisVariable,
        ]
    );

    test!(
        lexes_signed_numbers_and_arrow_and_other,
        b"@var -1|0|1.5e-3 -> #x",
        vec![
            TokenKind::Tag,
            TokenKind::Whitespace,
            TokenKind::Minus,
            TokenKind::LiteralInteger,
            TokenKind::Pipe,
            TokenKind::LiteralInteger,
            TokenKind::Pipe,
            TokenKind::LiteralFloat,
            TokenKind::Whitespace,
            TokenKind::Arrow,
            TokenKind::Whitespace,
            TokenKind::Other,
        ]
    );

    test!(
        lexes_bare_radix_prefix_as_zero_then_identifier,
        b"0x",
        vec![TokenKind::LiteralInteger, TokenKind::Identifier]
    );

    test!(reconstructs_empty, b"", vec![]);

    test!(
        reconstructs_simple,
        b"/** @param int $x */",
        vec![
            TokenKind::OpeningMarker,
            TokenKind::Tag,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Variable,
            TokenKind::Whitespace,
            TokenKind::ClosingMarker
        ]
    );

    test!(
        reconstructs_multiline,
        b"/**\n * @return void\n */",
        vec![
            TokenKind::OpeningMarker,
            TokenKind::Whitespace,
            TokenKind::Asterisk,
            TokenKind::Whitespace,
            TokenKind::Tag,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::ClosingMarker
        ]
    );

    test!(
        reconstructs_with_asterisks,
        indoc::indoc! {
            r#"/**
            * This is a docblock with various tokens.
            *
            * @param int<0, 100>|null $count This is a parameter description.
            * @return array{a: int} This is a return type description.
            * @throws \Exception This is a throws description.
            */"#
        },
        vec![
            TokenKind::OpeningMarker,
            TokenKind::Whitespace,
            TokenKind::Asterisk,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Other,
            TokenKind::Whitespace,
            TokenKind::Asterisk,
            TokenKind::Whitespace,
            TokenKind::Asterisk,
            TokenKind::Whitespace,
            TokenKind::Tag,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::LeftAngleBracket,
            TokenKind::LiteralInteger,
            TokenKind::Comma,
            TokenKind::Whitespace,
            TokenKind::LiteralInteger,
            TokenKind::RightAngleBracket,
            TokenKind::Pipe,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Variable,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Other,
            TokenKind::Whitespace,
            TokenKind::Asterisk,
            TokenKind::Whitespace,
            TokenKind::Tag,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::LeftBrace,
            TokenKind::Identifier,
            TokenKind::Colon,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::RightBrace,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Other,
            TokenKind::Whitespace,
            TokenKind::Asterisk,
            TokenKind::Whitespace,
            TokenKind::Tag,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Other,
            TokenKind::Whitespace,
            TokenKind::ClosingMarker
        ]
    );

    test!(
        multi_line_comment_crlf_with_multibyte_char,
        indoc::indoc! {
            r#"/**\r\n * blah blah ‰©\r\n */"#
        },
        vec![
            TokenKind::Other,
            TokenKind::Whitespace,
            TokenKind::Asterisk,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::ClosingMarker
        ]
    );

    test!(
        lex_multi_line_comment_missing_whitespace_after_asterisk,
        indoc::indoc! {
            "/**
            * This is a multi-line comment.
            *It has multiple lines.
            * Each line starts with an asterisk.
            */"
        },
        vec![
            TokenKind::OpeningMarker,
            TokenKind::Whitespace,
            TokenKind::Asterisk,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Other,
            TokenKind::Whitespace,
            TokenKind::Asterisk,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Other,
            TokenKind::Whitespace,
            TokenKind::Asterisk,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Other,
            TokenKind::Whitespace,
            TokenKind::ClosingMarker
        ]
    );

    test!(smoke_only_horizontal_and_vertical_whitespace, "   \t\t  \x0b\x0c   ", vec![TokenKind::Whitespace]);

    test!(smoke_only_newlines, "\n\n\r\n\n\r\r\n", vec![TokenKind::Whitespace]);

    test!(
        smoke_chinese,
        "你好世界 @param 整数 $值 描述文本",
        vec![
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Tag,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Variable,
            TokenKind::Whitespace,
            TokenKind::Identifier
        ]
    );

    test!(
        smoke_emoji,
        "🎉 @return 🚀|🌟 $x 🍕🍔🍟",
        vec![
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Tag,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Pipe,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Variable,
            TokenKind::Whitespace,
            TokenKind::Identifier
        ]
    );

    test!(
        smoke_invalid_utf8,
        b"\xff\xfe\xfd\xfc foo \x80\x81\x82 bar",
        vec![
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier
        ]
    );

    test!(smoke_null_and_control_bytes, b"\x00\x01\x02int\x00string\x1f\x7f", vec![TokenKind::Other]);

    test!(smoke_operator_soup, b"@@@|||&&&???:::==>><<<-->>->=>", vec![TokenKind::Other]);

    test!(
        smoke_bracket_soup,
        b"{{{[[[((( )))]]]}}}<><><>",
        vec![
            TokenKind::LeftBrace,
            TokenKind::LeftBrace,
            TokenKind::LeftBrace,
            TokenKind::LeftBracket,
            TokenKind::LeftBracket,
            TokenKind::LeftBracket,
            TokenKind::LeftParenthesis,
            TokenKind::LeftParenthesis,
            TokenKind::LeftParenthesis,
            TokenKind::Whitespace,
            TokenKind::RightParenthesis,
            TokenKind::RightParenthesis,
            TokenKind::RightParenthesis,
            TokenKind::RightBracket,
            TokenKind::RightBracket,
            TokenKind::RightBracket,
            TokenKind::RightBrace,
            TokenKind::RightBrace,
            TokenKind::RightBrace,
            TokenKind::LeftAngleBracket,
            TokenKind::RightAngleBracket,
            TokenKind::LeftAngleBracket,
            TokenKind::RightAngleBracket,
            TokenKind::LeftAngleBracket,
            TokenKind::RightAngleBracket
        ]
    );

    test!(
        smoke_leading_trailing_stars,
        b"***array<int>***",
        vec![
            TokenKind::Asterisk,
            TokenKind::Asterisk,
            TokenKind::Asterisk,
            TokenKind::Identifier,
            TokenKind::LeftAngleBracket,
            TokenKind::Identifier,
            TokenKind::RightAngleBracket,
            TokenKind::Asterisk,
            TokenKind::Asterisk,
            TokenKind::Asterisk
        ]
    );

    test!(
        smoke_closing_marker_edges,
        b"*/ foo */ bar */",
        vec![
            TokenKind::ClosingMarker,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::ClosingMarker,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::ClosingMarker
        ]
    );

    test!(
        smoke_open_without_close,
        b"/** @param int $x no closing marker here",
        vec![
            TokenKind::OpeningMarker,
            TokenKind::Tag,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Variable,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier
        ]
    );

    test!(smoke_empty_docblock, b"/**/", vec![TokenKind::Other, TokenKind::ClosingMarker]);

    test!(
        smoke_nested_markers,
        b"/** /** */ */",
        vec![
            TokenKind::OpeningMarker,
            TokenKind::OpeningMarker,
            TokenKind::ClosingMarker,
            TokenKind::Whitespace,
            TokenKind::ClosingMarker
        ]
    );

    test!(
        smoke_unterminated_single_quote,
        b"'unterminated string spanning the rest of input",
        vec![TokenKind::PartialString]
    );

    test!(smoke_unterminated_double_quote, b"\"esc \\\" still going to the end", vec![TokenKind::PartialString]);

    test!(
        smoke_backslash_soup,
        b"\\\\\\Foo\\\\Bar\\\\ \\ \\1 \\$",
        vec![
            TokenKind::Other,
            TokenKind::Whitespace,
            TokenKind::Other,
            TokenKind::Whitespace,
            TokenKind::Other,
            TokenKind::Whitespace,
            TokenKind::Other
        ]
    );

    test!(
        smoke_dollar_soup,
        b"$$$ $123 $ $this$that $0 $_",
        vec![
            TokenKind::Other,
            TokenKind::Whitespace,
            TokenKind::Other,
            TokenKind::Whitespace,
            TokenKind::Other,
            TokenKind::Whitespace,
            TokenKind::ThisVariable,
            TokenKind::Variable,
            TokenKind::Whitespace,
            TokenKind::Other,
            TokenKind::Whitespace,
            TokenKind::Variable
        ]
    );

    test!(
        smoke_number_garbage,
        b"---+++...0x0o0b1_2_.3.4e+-e 1.2.3.4",
        vec![
            TokenKind::Minus,
            TokenKind::Minus,
            TokenKind::Minus,
            TokenKind::Plus,
            TokenKind::Plus,
            TokenKind::Plus,
            TokenKind::Ellipsis,
            TokenKind::LiteralInteger,
            TokenKind::Identifier,
            TokenKind::LiteralFloat,
            TokenKind::LiteralFloat,
            TokenKind::Identifier,
            TokenKind::Plus,
            TokenKind::Minus,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::LiteralFloat,
            TokenKind::LiteralFloat,
            TokenKind::LiteralFloat
        ]
    );

    test!(
        smoke_at_garbage,
        b"@@ @1 @- @ @param@return @a:b:c",
        vec![
            TokenKind::Other,
            TokenKind::Whitespace,
            TokenKind::Other,
            TokenKind::Whitespace,
            TokenKind::Other,
            TokenKind::Whitespace,
            TokenKind::Other,
            TokenKind::Whitespace,
            TokenKind::Tag,
            TokenKind::Tag,
            TokenKind::Whitespace,
            TokenKind::Tag,
            TokenKind::Colon,
            TokenKind::Identifier
        ]
    );

    test!(
        smoke_mixed_everything,
        "int|string\t你好 @var $x */ 🎉 ::* <T of object> -1.5e3 array{a:int,'b'?:\"c\"}",
        vec![
            TokenKind::Identifier,
            TokenKind::Pipe,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Tag,
            TokenKind::Whitespace,
            TokenKind::Variable,
            TokenKind::Whitespace,
            TokenKind::ClosingMarker,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::ColonColon,
            TokenKind::Asterisk,
            TokenKind::Whitespace,
            TokenKind::LeftAngleBracket,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::RightAngleBracket,
            TokenKind::Whitespace,
            TokenKind::Minus,
            TokenKind::LiteralFloat,
            TokenKind::Whitespace,
            TokenKind::Identifier,
            TokenKind::LeftBrace,
            TokenKind::Identifier,
            TokenKind::Colon,
            TokenKind::Identifier,
            TokenKind::Comma,
            TokenKind::SingleQuotedString,
            TokenKind::Question,
            TokenKind::Colon,
            TokenKind::DoubleQuotedString,
            TokenKind::RightBrace
        ]
    );

    test!(
        smoke_high_byte_boundaries,
        b"\xe4\xbd\xa0<\xf0\x9f\x8e\x89>\xc3\x28",
        vec![
            TokenKind::Identifier,
            TokenKind::LeftAngleBracket,
            TokenKind::Identifier,
            TokenKind::RightAngleBracket,
            TokenKind::Identifier,
            TokenKind::LeftParenthesis
        ]
    );
}
