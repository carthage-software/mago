use std::collections::VecDeque;
use std::fmt::Debug;
use std::hint::unreachable_unchecked;

use memchr::memchr2;
use memchr::memmem;

/// Lookup table for single-character tokens that are ALWAYS single-char
/// (i.e., they can never be part of a multi-character token).
/// Maps byte -> Option<TokenKind>
const SIMPLE_TOKEN_TABLE: [Option<TokenKind>; 256] = {
    let mut table: [Option<TokenKind>; 256] = [None; 256];
    table[b';' as usize] = Some(TokenKind::Semicolon);
    table[b',' as usize] = Some(TokenKind::Comma);
    table[b')' as usize] = Some(TokenKind::RightParenthesis);
    table[b'[' as usize] = Some(TokenKind::LeftBracket);
    table[b']' as usize] = Some(TokenKind::RightBracket);
    table[b'{' as usize] = Some(TokenKind::LeftBrace);
    table[b'}' as usize] = Some(TokenKind::RightBrace);
    table[b'~' as usize] = Some(TokenKind::Tilde);
    table[b'@' as usize] = Some(TokenKind::At);
    table
};

/// Lookup table for identifier start characters (a-z, A-Z, _, 0x80-0xFF)
const IDENT_START_TABLE: [bool; 256] = {
    let mut table = [false; 256];
    let mut i = 0usize;
    while i < 256 {
        table[i] = matches!(i as u8, b'a'..=b'z' | b'A'..=b'Z' | b'_' | 0x80..=0xFF);
        i += 1;
    }

    table
};

use mago_database::file::FileId;
use mago_database::file::HasFileId;
use mago_span::Position;
use mago_syntax_core::float_exponent;
use mago_syntax_core::float_separator;
use mago_syntax_core::input::Input;
use mago_syntax_core::number_sign;
use mago_syntax_core::start_of_binary_number;
use mago_syntax_core::start_of_float_number;
use mago_syntax_core::start_of_hexadecimal_number;
use mago_syntax_core::start_of_identifier;
use mago_syntax_core::start_of_number;
use mago_syntax_core::start_of_octal_number;
use mago_syntax_core::start_of_octal_or_float_number;
use mago_syntax_core::utils::is_part_of_identifier;
use mago_syntax_core::utils::is_start_of_identifier;
use mago_syntax_core::utils::read_digits_of_base;

use crate::error::SyntaxError;
use crate::lexer::internal::mode::HaltStage;
use crate::lexer::internal::mode::Interpolation;
use crate::lexer::internal::mode::LexerMode;
use crate::lexer::internal::utils::NumberKind;
use crate::settings::LexerSettings;
use crate::token::DocumentKind;
use crate::token::Token;
use crate::token::TokenKind;

mod internal;

/// The `Lexer` struct is responsible for tokenizing input source code into discrete tokens
/// based on PHP language syntax. It is designed to work with PHP code from version 7.0 up to 8.4.
///
/// The lexer reads through the provided input and processes it accordingly.
///
/// It identifies PHP-specific tokens, including operators, keywords, comments, strings, and other syntax elements,
/// and produces a sequence of [`Token`] objects that are used in further stages of compilation or interpretation.
///
/// The lexer is designed to be used in a streaming fashion, where it reads the input source code in chunks
/// and produces tokens incrementally. This allows for efficient processing of large source files and
/// minimizes memory usage.
#[derive(Debug)]
pub struct Lexer<'input> {
    input: Input<'input>,
    settings: LexerSettings,
    mode: LexerMode<'input>,
    interpolating: bool,
    /// Buffer for tokens during string interpolation.
    buffer: VecDeque<Token<'input>>,
}

impl<'input> Lexer<'input> {
    /// Initial capacity for the token buffer used during string interpolation.
    /// Pre-allocating avoids reallocation during interpolation processing.
    const BUFFER_INITIAL_CAPACITY: usize = 8;

    /// Creates a new `Lexer` instance.
    ///
    /// # Parameters
    ///
    /// - `input`: The input source code to tokenize.
    /// - `settings`: The lexer settings.
    ///
    /// # Returns
    ///
    /// A new `Lexer` instance that reads from the provided byte slice.
    pub fn new(input: Input<'input>, settings: LexerSettings) -> Lexer<'input> {
        Lexer {
            input,
            settings,
            mode: LexerMode::Inline,
            interpolating: false,
            buffer: VecDeque::with_capacity(Self::BUFFER_INITIAL_CAPACITY),
        }
    }

    /// Creates a new `Lexer` instance for parsing a script block.
    ///
    /// # Parameters
    ///
    /// - `input`: The input source code to tokenize.
    /// - `settings`: The lexer settings.
    ///
    /// # Returns
    ///
    /// A new `Lexer` instance that reads from the provided byte slice.
    pub fn scripting(input: Input<'input>, settings: LexerSettings) -> Lexer<'input> {
        Lexer {
            input,
            settings,
            mode: LexerMode::Script,
            interpolating: false,
            buffer: VecDeque::with_capacity(Self::BUFFER_INITIAL_CAPACITY),
        }
    }

    /// Check if the lexer has reached the end of the input.
    ///
    /// If this method returns `true`, the lexer will not produce any more tokens.
    #[must_use]
    pub fn has_reached_eof(&self) -> bool {
        self.input.has_reached_eof()
    }

    /// Get the current position of the lexer in the input source code.
    #[inline]
    pub const fn current_position(&self) -> Position {
        self.input.current_position()
    }

    /// Tokenizes the next input from the source code.
    ///
    /// This method reads from the input and produces the next [`Token`] based on the current [`LexerMode`].
    /// It handles various lexical elements such as inline text, script code, strings with interpolation,
    /// comments, and different PHP-specific constructs.
    ///
    /// # Returns
    ///
    /// - `Some(Ok(Token))` if a token was successfully parsed.
    /// - `Some(Err(SyntaxError))` if a syntax error occurred while parsing the next token.
    /// - `None` if the end of the input has been reached.
    ///
    /// # Notes
    ///
    /// - It efficiently handles tokenization by consuming input based on patterns specific to PHP syntax.
    /// - The lexer supports complex features like string interpolation and different numeric formats.
    ///
    /// # Errors
    ///
    /// Returns `Some(Err(SyntaxError))` in cases such as:
    ///
    /// - Unrecognized tokens that do not match any known PHP syntax.
    /// - Unexpected tokens in a given context, such as an unexpected end of string.
    ///
    /// # Panics
    ///
    /// This method should not panic under normal operation. If it does, it indicates a bug in the lexer implementation.
    ///
    /// # See Also
    ///
    /// - [`Token`]: Represents a lexical token with its kind, value, and span.
    /// - [`SyntaxError`]: Represents errors that can occur during lexing.
    #[inline]
    pub fn advance(&mut self) -> Option<Result<Token<'input>, SyntaxError>> {
        // Check if there are buffered tokens from string interpolation.
        if !self.interpolating
            && let Some(token) = self.buffer.pop_front()
        {
            return Some(Ok(token));
        }

        if self.input.has_reached_eof() {
            return None;
        }

        match self.mode {
            LexerMode::Inline => {
                let start = self.input.current_position();
                let offset = self.input.current_offset();

                // Shebang is only valid at the absolute start of the file (offset 0).
                if offset == 0
                    && self.input.len() >= 2
                    && unsafe { *self.input.read_at_unchecked(0) } == b'#'
                    && unsafe { *self.input.read_at_unchecked(1) } == b'!'
                {
                    let buffer = self.input.consume_through(b'\n');
                    let end = self.input.current_position();

                    return Some(Ok(self.token(TokenKind::InlineShebang, buffer, start, end)));
                }

                // Get the remaining bytes to scan.
                let bytes = self.input.read_remaining();

                if self.settings.enable_short_tags {
                    if let Some(pos) = memchr::memmem::find(bytes, b"<?") {
                        if pos > 0 {
                            let buffer = self.input.consume(pos);
                            let end = self.input.current_position();

                            return Some(Ok(self.token(TokenKind::InlineText, buffer, start, end)));
                        }

                        if self.input.is_at(b"<?php", true) {
                            let buffer = self.input.consume(5);
                            self.mode = LexerMode::Script;
                            return Some(Ok(self.token(
                                TokenKind::OpenTag,
                                buffer,
                                start,
                                self.input.current_position(),
                            )));
                        }

                        if self.input.is_at(b"<?=", false) {
                            let buffer = self.input.consume(3);
                            self.mode = LexerMode::Script;
                            return Some(Ok(self.token(
                                TokenKind::EchoTag,
                                buffer,
                                start,
                                self.input.current_position(),
                            )));
                        }

                        let buffer = self.input.consume(2);
                        self.mode = LexerMode::Script;
                        return Some(Ok(self.token(
                            TokenKind::ShortOpenTag,
                            buffer,
                            start,
                            self.input.current_position(),
                        )));
                    }
                } else {
                    let iter = memchr::memmem::find_iter(bytes, b"<?");

                    for pos in iter {
                        // SAFETY: `pos` is guaranteed to be within `bytes` by `find_iter`.
                        let candidate = unsafe { bytes.get_unchecked(pos..) };

                        if candidate.len() >= 5
                            && (unsafe { *candidate.get_unchecked(2) } | 0x20) == b'p'
                            && (unsafe { *candidate.get_unchecked(3) } | 0x20) == b'h'
                            && (unsafe { *candidate.get_unchecked(4) } | 0x20) == b'p'
                        {
                            if pos > 0 {
                                let buffer = self.input.consume(pos);
                                let end = self.input.current_position();
                                return Some(Ok(self.token(TokenKind::InlineText, buffer, start, end)));
                            }

                            let buffer = self.input.consume(5);
                            self.mode = LexerMode::Script;
                            return Some(Ok(self.token(
                                TokenKind::OpenTag,
                                buffer,
                                start,
                                self.input.current_position(),
                            )));
                        }

                        if candidate.len() >= 3 && unsafe { *candidate.get_unchecked(2) } == b'=' {
                            if pos > 0 {
                                let buffer = self.input.consume(pos);
                                let end = self.input.current_position();
                                return Some(Ok(self.token(TokenKind::InlineText, buffer, start, end)));
                            }

                            let buffer = self.input.consume(3);
                            self.mode = LexerMode::Script;
                            return Some(Ok(self.token(
                                TokenKind::EchoTag,
                                buffer,
                                start,
                                self.input.current_position(),
                            )));
                        }
                    }
                }

                if self.input.has_reached_eof() {
                    return None;
                }

                let buffer = self.input.consume_remaining();
                let end = self.input.current_position();
                Some(Ok(self.token(TokenKind::InlineText, buffer, start, end)))
            }
            LexerMode::Script => {
                let start = self.input.current_position();
                let whitespaces = self.input.consume_whitespaces();
                if !whitespaces.is_empty() {
                    return Some(Ok(self.token(
                        TokenKind::Whitespace,
                        whitespaces,
                        start,
                        self.input.current_position(),
                    )));
                }

                let first_byte = match self.input.read(1).first() {
                    Some(&b) => b,
                    None => {
                        // SAFETY: we check for EOF before entering scripting section,
                        unsafe { unreachable_unchecked() }
                    }
                };

                if let Some(kind) = SIMPLE_TOKEN_TABLE[first_byte as usize] {
                    let buffer = self.input.consume(1);
                    let end = self.input.current_position();
                    return Some(Ok(self.token(kind, buffer, start, end)));
                }

                if IDENT_START_TABLE[first_byte as usize] {
                    let (token_kind, len) = self.scan_identifier_or_keyword_info();

                    if token_kind == TokenKind::HaltCompiler {
                        self.mode = LexerMode::Halt(HaltStage::LookingForLeftParenthesis);
                    }

                    let buffer = self.input.consume(len);
                    let end = self.input.current_position();
                    return Some(Ok(self.token(token_kind, buffer, start, end)));
                }

                if first_byte == b'$'
                    && let Some(&next) = self.input.read(2).get(1)
                    && IDENT_START_TABLE[next as usize]
                {
                    let (ident_len, _) = self.input.scan_identifier(1);
                    let buffer = self.input.consume(1 + ident_len);
                    let end = self.input.current_position();
                    return Some(Ok(self.token(TokenKind::Variable, buffer, start, end)));
                }

                let mut document_label: &[u8] = &[];

                let (token_kind, len) = match self.input.read(3) {
                    [b'!', b'=', b'='] => (TokenKind::BangEqualEqual, 3),
                    [b'?', b'?', b'='] => (TokenKind::QuestionQuestionEqual, 3),
                    [b'?', b'-', b'>'] => (TokenKind::QuestionMinusGreaterThan, 3),
                    [b'=', b'=', b'='] => (TokenKind::EqualEqualEqual, 3),
                    [b'.', b'.', b'.'] => (TokenKind::DotDotDot, 3),
                    [b'<', b'=', b'>'] => (TokenKind::LessThanEqualGreaterThan, 3),
                    [b'<', b'<', b'='] => (TokenKind::LeftShiftEqual, 3),
                    [b'>', b'>', b'='] => (TokenKind::RightShiftEqual, 3),
                    [b'*', b'*', b'='] => (TokenKind::AsteriskAsteriskEqual, 3),
                    [b'<', b'<', b'<'] if matches_start_of_heredoc_document(&self.input) => {
                        let (length, whitespaces, label_length) = read_start_of_heredoc_document(&self.input, false);

                        document_label = self.input.peek(3 + whitespaces, label_length);

                        (TokenKind::DocumentStart(DocumentKind::Heredoc), length)
                    }
                    [b'<', b'<', b'<'] if matches_start_of_double_quote_heredoc_document(&self.input) => {
                        let (length, whitespaces, label_length) = read_start_of_heredoc_document(&self.input, true);

                        document_label = self.input.peek(4 + whitespaces, label_length);

                        (TokenKind::DocumentStart(DocumentKind::Heredoc), length)
                    }
                    [b'<', b'<', b'<'] if matches_start_of_nowdoc_document(&self.input) => {
                        let (length, whitespaces, label_length) = read_start_of_nowdoc_document(&self.input);

                        document_label = self.input.peek(4 + whitespaces, label_length);

                        (TokenKind::DocumentStart(DocumentKind::Nowdoc), length)
                    }
                    [b'!', b'=', ..] => (TokenKind::BangEqual, 2),
                    [b'&', b'&', ..] => (TokenKind::AmpersandAmpersand, 2),
                    [b'&', b'=', ..] => (TokenKind::AmpersandEqual, 2),
                    [b'.', b'=', ..] => (TokenKind::DotEqual, 2),
                    [b'?', b'?', ..] => (TokenKind::QuestionQuestion, 2),
                    [b'?', b'>', ..] => (TokenKind::CloseTag, 2),
                    [b'=', b'>', ..] => (TokenKind::EqualGreaterThan, 2),
                    [b'=', b'=', ..] => (TokenKind::EqualEqual, 2),
                    [b'+', b'+', ..] => (TokenKind::PlusPlus, 2),
                    [b'+', b'=', ..] => (TokenKind::PlusEqual, 2),
                    [b'%', b'=', ..] => (TokenKind::PercentEqual, 2),
                    [b'-', b'-', ..] => (TokenKind::MinusMinus, 2),
                    [b'-', b'>', ..] => (TokenKind::MinusGreaterThan, 2),
                    [b'-', b'=', ..] => (TokenKind::MinusEqual, 2),
                    [b'<', b'<', ..] => (TokenKind::LeftShift, 2),
                    [b'<', b'=', ..] => (TokenKind::LessThanEqual, 2),
                    [b'<', b'>', ..] => (TokenKind::LessThanGreaterThan, 2),
                    [b'>', b'>', ..] => (TokenKind::RightShift, 2),
                    [b'>', b'=', ..] => (TokenKind::GreaterThanEqual, 2),
                    [b':', b':', ..] => (TokenKind::ColonColon, 2),
                    [b'#', b'[', ..] => (TokenKind::HashLeftBracket, 2),
                    [b'|', b'=', ..] => (TokenKind::PipeEqual, 2),
                    [b'|', b'|', ..] => (TokenKind::PipePipe, 2),
                    [b'/', b'=', ..] => (TokenKind::SlashEqual, 2),
                    [b'^', b'=', ..] => (TokenKind::CaretEqual, 2),
                    [b'*', b'*', ..] => (TokenKind::AsteriskAsterisk, 2),
                    [b'*', b'=', ..] => (TokenKind::AsteriskEqual, 2),
                    [b'|', b'>', ..] => (TokenKind::PipeGreaterThan, 2),
                    [b'/', b'/', ..] => {
                        let remaining = self.input.peek(2, self.input.len() - self.input.current_offset());
                        let comment_len = scan_single_line_comment(remaining);
                        (TokenKind::SingleLineComment, 2 + comment_len)
                    }
                    [b'/', b'*', asterisk] => {
                        let remaining = self.input.peek(2, self.input.len() - self.input.current_offset());
                        match scan_multi_line_comment(remaining) {
                            Some(len) => {
                                let is_docblock = asterisk == &b'*' && len > 2;
                                if is_docblock {
                                    (TokenKind::DocBlockComment, len + 2)
                                } else {
                                    (TokenKind::MultiLineComment, len + 2)
                                }
                            }
                            None => {
                                self.input.consume(remaining.len() + 2);
                                return Some(Err(SyntaxError::UnexpectedEndOfFile(
                                    self.file_id(),
                                    self.input.current_position(),
                                )));
                            }
                        }
                    }
                    [b'\\', start_of_identifier!(), ..] => {
                        let mut length = 1;
                        loop {
                            let (ident_len, ends_with_ns) = self.input.scan_identifier(length);
                            length += ident_len;
                            if ends_with_ns {
                                length += 1; // Include the backslash
                            } else {
                                break;
                            }
                        }

                        (TokenKind::FullyQualifiedIdentifier, length)
                    }
                    [b'$', b'{', ..] => (TokenKind::DollarLeftBrace, 2),
                    [b'$', ..] => (TokenKind::Dollar, 1),
                    [b'!', ..] => (TokenKind::Bang, 1),
                    [b'&', ..] => (TokenKind::Ampersand, 1),
                    [b'?', ..] => (TokenKind::Question, 1),
                    [b'=', ..] => (TokenKind::Equal, 1),
                    [b'`', ..] => (TokenKind::Backtick, 1),
                    [b'+', ..] => (TokenKind::Plus, 1),
                    [b'%', ..] => (TokenKind::Percent, 1),
                    [b'-', ..] => (TokenKind::Minus, 1),
                    [b'<', ..] => (TokenKind::LessThan, 1),
                    [b'>', ..] => (TokenKind::GreaterThan, 1),
                    [b':', ..] => (TokenKind::Colon, 1),
                    [b'|', ..] => (TokenKind::Pipe, 1),
                    [b'^', ..] => (TokenKind::Caret, 1),
                    [b'*', ..] => (TokenKind::Asterisk, 1),
                    [b'/', ..] => (TokenKind::Slash, 1),
                    [quote @ b'\'', ..] => read_literal_string(&self.input, *quote),
                    [quote @ b'"', ..] if matches_literal_double_quote_string(&self.input) => {
                        read_literal_string(&self.input, *quote)
                    }
                    [b'"', ..] => (TokenKind::DoubleQuote, 1),
                    [b'(', ..] => 'parenthesis: {
                        let mut peek_offset = 1;
                        while let Some(&b) = self.input.read(peek_offset + 1).get(peek_offset) {
                            if b.is_ascii_whitespace() {
                                peek_offset += 1;
                            } else {
                                // Check if this byte could start a cast type (case-insensitive)
                                let lower = b | 0x20; // ASCII lowercase
                                if !matches!(lower, b'i' | b'b' | b'f' | b'd' | b'r' | b's' | b'a' | b'o' | b'u') {
                                    break 'parenthesis (TokenKind::LeftParenthesis, 1);
                                }
                                break;
                            }
                        }

                        for (value, kind) in internal::consts::CAST_TYPES {
                            if let Some(length) = self.input.match_sequence_ignore_whitespace(value, true) {
                                break 'parenthesis (kind, length);
                            }
                        }

                        (TokenKind::LeftParenthesis, 1)
                    }
                    [b'#', ..] => {
                        let remaining = self.input.peek(1, self.input.len() - self.input.current_offset());
                        let comment_len = scan_single_line_comment(remaining);
                        (TokenKind::HashComment, 1 + comment_len)
                    }
                    [b'\\', ..] => (TokenKind::NamespaceSeparator, 1),
                    [b'.', start_of_number!(), ..] => {
                        let mut length = read_digits_of_base(&self.input, 2, 10);
                        if let float_exponent!() = self.input.peek(length, 1) {
                            let mut exp_length = length + 1;
                            if let number_sign!() = self.input.peek(exp_length, 1) {
                                exp_length += 1;
                            }

                            let after_exp = read_digits_of_base(&self.input, exp_length, 10);
                            if after_exp > exp_length {
                                length = after_exp;
                            }
                        }

                        (TokenKind::LiteralFloat, length)
                    }
                    [start_of_number!(), ..] => 'number: {
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
                                break 'number (TokenKind::LiteralInteger, length);
                            }
                        }

                        let is_float = matches!(self.input.peek(length, 3), float_separator!());

                        if !is_float {
                            break 'number (TokenKind::LiteralInteger, length);
                        }

                        if let [b'.'] = self.input.peek(length, 1) {
                            length += 1;
                            length = read_digits_of_base(&self.input, length, 10);
                        }

                        if let float_exponent!() = self.input.peek(length, 1) {
                            // Only include exponent if there are digits after it
                            let mut exp_length = length + 1;
                            if let number_sign!() = self.input.peek(exp_length, 1) {
                                exp_length += 1;
                            }
                            let after_exp = read_digits_of_base(&self.input, exp_length, 10);
                            if after_exp > exp_length {
                                // There are digits after the exponent marker
                                length = after_exp;
                            }
                        }

                        (TokenKind::LiteralFloat, length)
                    }
                    [b'.', ..] => (TokenKind::Dot, 1),
                    [unknown_byte, ..] => {
                        let position = self.input.current_position();
                        self.input.consume(1);

                        return Some(Err(SyntaxError::UnrecognizedToken(self.file_id(), *unknown_byte, position)));
                    }
                    [] => {
                        // we check for EOF before entering scripting section,
                        // so this should be unreachable.
                        unreachable!()
                    }
                };

                self.mode = match token_kind {
                    TokenKind::DoubleQuote => LexerMode::DoubleQuoteString(Interpolation::None),
                    TokenKind::Backtick => LexerMode::ShellExecuteString(Interpolation::None),
                    TokenKind::CloseTag => LexerMode::Inline,
                    TokenKind::HaltCompiler => LexerMode::Halt(HaltStage::LookingForLeftParenthesis),
                    TokenKind::DocumentStart(document_kind) => {
                        LexerMode::DocumentString(document_kind, document_label, Interpolation::None)
                    }
                    _ => LexerMode::Script,
                };

                let buffer = self.input.consume(len);
                let end = self.input.current_position();

                Some(Ok(self.token(token_kind, buffer, start, end)))
            }
            LexerMode::DoubleQuoteString(interpolation) => match &interpolation {
                Interpolation::None => {
                    let start = self.input.current_position();

                    let mut length = 0;
                    let mut last_was_slash = false;
                    let mut token_kind = TokenKind::StringPart;
                    loop {
                        match self.input.peek(length, 2) {
                            [b'$', start_of_identifier!(), ..] if !last_was_slash => {
                                let until_offset = read_until_end_of_variable_interpolation(&self.input, length + 2);

                                self.mode =
                                    LexerMode::DoubleQuoteString(Interpolation::Until(start.offset + until_offset));

                                break;
                            }
                            [b'{', b'$', ..] | [b'$', b'{', ..] if !last_was_slash => {
                                let until_offset = read_until_end_of_brace_interpolation(&self.input, length + 2);

                                self.mode =
                                    LexerMode::DoubleQuoteString(Interpolation::Until(start.offset + until_offset));

                                break;
                            }
                            [b'\\', ..] => {
                                length += 1;

                                last_was_slash = !last_was_slash;
                            }
                            [b'"', ..] if !last_was_slash => {
                                if length == 0 {
                                    length += 1;
                                    token_kind = TokenKind::DoubleQuote;

                                    break;
                                }

                                break;
                            }
                            [_, ..] => {
                                length += 1;
                                last_was_slash = false;
                            }
                            [] => {
                                break;
                            }
                        }
                    }

                    let buffer = self.input.consume(length);
                    let end = self.input.current_position();

                    if TokenKind::DoubleQuote == token_kind {
                        self.mode = LexerMode::Script;
                    }

                    Some(Ok(self.token(token_kind, buffer, start, end)))
                }
                Interpolation::Until(offset) => {
                    self.interpolation(*offset, LexerMode::DoubleQuoteString(Interpolation::None))
                }
            },
            LexerMode::ShellExecuteString(interpolation) => match &interpolation {
                Interpolation::None => {
                    let start = self.input.current_position();

                    let mut length = 0;
                    let mut last_was_slash = false;
                    let mut token_kind = TokenKind::StringPart;
                    loop {
                        match self.input.peek(length, 2) {
                            [b'$', start_of_identifier!(), ..] if !last_was_slash => {
                                let until_offset = read_until_end_of_variable_interpolation(&self.input, length + 2);

                                self.mode =
                                    LexerMode::ShellExecuteString(Interpolation::Until(start.offset + until_offset));

                                break;
                            }
                            [b'{', b'$', ..] | [b'$', b'{', ..] if !last_was_slash => {
                                let until_offset = read_until_end_of_brace_interpolation(&self.input, length + 2);

                                self.mode =
                                    LexerMode::ShellExecuteString(Interpolation::Until(start.offset + until_offset));

                                break;
                            }
                            [b'\\', ..] => {
                                length += 1;
                                last_was_slash = true;
                            }
                            [b'`', ..] if !last_was_slash => {
                                if length == 0 {
                                    length += 1;
                                    token_kind = TokenKind::Backtick;

                                    break;
                                }

                                break;
                            }
                            [_, ..] => {
                                length += 1;
                                last_was_slash = false;
                            }
                            [] => {
                                break;
                            }
                        }
                    }

                    let buffer = self.input.consume(length);
                    let end = self.input.current_position();

                    if TokenKind::Backtick == token_kind {
                        self.mode = LexerMode::Script;
                    }

                    Some(Ok(self.token(token_kind, buffer, start, end)))
                }
                Interpolation::Until(offset) => {
                    self.interpolation(*offset, LexerMode::ShellExecuteString(Interpolation::None))
                }
            },
            LexerMode::DocumentString(kind, label, interpolation) => match &kind {
                DocumentKind::Heredoc => match &interpolation {
                    Interpolation::None => {
                        let start = self.input.current_position();

                        let mut length = 0;
                        let mut last_was_slash = false;
                        let mut only_whitespaces = true;
                        let mut token_kind = TokenKind::StringPart;
                        loop {
                            match self.input.peek(length, 2) {
                                [b'\r', b'\n'] => {
                                    length += 2;

                                    break;
                                }
                                [b'\n' | b'\r', ..] => {
                                    length += 1;

                                    break;
                                }
                                [byte, ..] if byte.is_ascii_whitespace() => {
                                    length += 1;
                                }
                                [b'$', start_of_identifier!(), ..] if !last_was_slash => {
                                    let until_offset =
                                        read_until_end_of_variable_interpolation(&self.input, length + 2);

                                    self.mode = LexerMode::DocumentString(
                                        kind,
                                        label,
                                        Interpolation::Until(start.offset + until_offset),
                                    );

                                    break;
                                }
                                [b'{', b'$', ..] | [b'$', b'{', ..] if !last_was_slash => {
                                    let until_offset = read_until_end_of_brace_interpolation(&self.input, length + 2);

                                    self.mode = LexerMode::DocumentString(
                                        kind,
                                        label,
                                        Interpolation::Until(start.offset + until_offset),
                                    );

                                    break;
                                }
                                [b'\\', ..] => {
                                    length += 1;
                                    last_was_slash = true;
                                    only_whitespaces = false;
                                }
                                [_, ..] => {
                                    if only_whitespaces
                                        && self.input.peek(length, label.len()) == label
                                        && self
                                            .input
                                            .peek(length + label.len(), 1)
                                            .first()
                                            .is_none_or(|c| !c.is_ascii_alphanumeric())
                                    {
                                        length += label.len();
                                        token_kind = TokenKind::DocumentEnd;

                                        break;
                                    }

                                    length += 1;
                                    last_was_slash = false;
                                    only_whitespaces = false;
                                }
                                [] => {
                                    break;
                                }
                            }
                        }

                        let buffer = self.input.consume(length);
                        let end = self.input.current_position();

                        if TokenKind::DocumentEnd == token_kind {
                            self.mode = LexerMode::Script;
                        }

                        Some(Ok(self.token(token_kind, buffer, start, end)))
                    }
                    Interpolation::Until(offset) => {
                        self.interpolation(*offset, LexerMode::DocumentString(kind, label, Interpolation::None))
                    }
                },
                DocumentKind::Nowdoc => {
                    let start = self.input.current_position();

                    let mut length = 0;
                    let mut terminated = false;
                    let mut only_whitespaces = true;

                    loop {
                        match self.input.peek(length, 2) {
                            [b'\r', b'\n'] => {
                                length += 2;

                                break;
                            }
                            [b'\n' | b'\r', ..] => {
                                length += 1;

                                break;
                            }
                            [byte, ..] if byte.is_ascii_whitespace() => {
                                length += 1;
                            }
                            [_, ..] => {
                                if only_whitespaces
                                    && self.input.peek(length, label.len()) == label
                                    && self
                                        .input
                                        .peek(length + label.len(), 1)
                                        .first()
                                        .is_none_or(|c| !c.is_ascii_alphanumeric())
                                {
                                    length += label.len();
                                    terminated = true;

                                    break;
                                }

                                only_whitespaces = false;
                                length += 1;
                            }
                            [] => {
                                break;
                            }
                        }
                    }

                    let buffer = self.input.consume(length);
                    let end = self.input.current_position();

                    if terminated {
                        self.mode = LexerMode::Script;

                        return Some(Ok(self.token(TokenKind::DocumentEnd, buffer, start, end)));
                    }

                    Some(Ok(self.token(TokenKind::StringPart, buffer, start, end)))
                }
            },
            LexerMode::Halt(stage) => 'halt: {
                let start = self.input.current_position();
                if let HaltStage::End = stage {
                    let buffer = self.input.consume_remaining();
                    let end = self.input.current_position();

                    break 'halt Some(Ok(self.token(TokenKind::InlineText, buffer, start, end)));
                }

                let whitespaces = self.input.consume_whitespaces();
                if !whitespaces.is_empty() {
                    let end = self.input.current_position();

                    break 'halt Some(Ok(self.token(TokenKind::Whitespace, whitespaces, start, end)));
                }

                match &stage {
                    HaltStage::LookingForLeftParenthesis => {
                        if self.input.is_at(b"(", false) {
                            let buffer = self.input.consume(1);
                            let end = self.input.current_position();

                            self.mode = LexerMode::Halt(HaltStage::LookingForRightParenthesis);

                            Some(Ok(self.token(TokenKind::LeftParenthesis, buffer, start, end)))
                        } else {
                            let byte = self.input.read(1)[0];
                            let position = self.input.current_position();
                            // Consume the unexpected byte to avoid infinite loops
                            self.input.consume(1);
                            Some(Err(SyntaxError::UnexpectedToken(self.file_id(), byte, position)))
                        }
                    }
                    HaltStage::LookingForRightParenthesis => {
                        if self.input.is_at(b")", false) {
                            let buffer = self.input.consume(1);
                            let end = self.input.current_position();

                            self.mode = LexerMode::Halt(HaltStage::LookingForTerminator);

                            Some(Ok(self.token(TokenKind::RightParenthesis, buffer, start, end)))
                        } else {
                            let byte = self.input.read(1)[0];
                            let position = self.input.current_position();
                            self.input.consume(1);
                            Some(Err(SyntaxError::UnexpectedToken(self.file_id(), byte, position)))
                        }
                    }
                    HaltStage::LookingForTerminator => {
                        if self.input.is_at(b";", false) {
                            let buffer = self.input.consume(1);
                            let end = self.input.current_position();

                            self.mode = LexerMode::Halt(HaltStage::End);

                            Some(Ok(self.token(TokenKind::Semicolon, buffer, start, end)))
                        } else if self.input.is_at(b"?>", false) {
                            let buffer = self.input.consume(2);
                            let end = self.input.current_position();

                            self.mode = LexerMode::Halt(HaltStage::End);

                            Some(Ok(self.token(TokenKind::CloseTag, buffer, start, end)))
                        } else {
                            let byte = self.input.read(1)[0];
                            let position = self.input.current_position();
                            self.input.consume(1);
                            Some(Err(SyntaxError::UnexpectedToken(self.file_id(), byte, position)))
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    /// Fast path for scanning identifiers and keywords.
    /// Called when we know the first byte is an identifier start character.
    /// Returns (TokenKind, length) to allow proper mode switching.
    #[inline]
    fn scan_identifier_or_keyword_info(&self) -> (TokenKind, usize) {
        let (mut length, ended_with_slash) = self.input.scan_identifier(0);

        if !ended_with_slash {
            match length {
                6 => {
                    if self.input.is_at(b"public(set)", true) {
                        return (TokenKind::PublicSet, 11);
                    }
                }
                7 => {
                    if self.input.is_at(b"private(set)", true) {
                        return (TokenKind::PrivateSet, 12);
                    }
                }
                9 => {
                    if self.input.is_at(b"protected(set)", true) {
                        return (TokenKind::ProtectedSet, 14);
                    }
                }
                _ => {}
            }
        }

        if !ended_with_slash && let Some(kind) = internal::keyword::lookup_keyword(self.input.read(length)) {
            return (kind, length);
        }

        let mut slashes = 0;
        let mut last_was_slash = false;
        loop {
            match self.input.peek(length, 1) {
                [b'a'..=b'z' | b'A'..=b'Z' | b'_' | 0x80..=0xFF] if last_was_slash => {
                    length += 1;
                    last_was_slash = false;
                }
                [b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | 0x80..=0xFF] if !last_was_slash => {
                    length += 1;
                }
                [b'\\'] if !self.interpolating => {
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

        let kind = if slashes > 0 { TokenKind::QualifiedIdentifier } else { TokenKind::Identifier };

        (kind, length)
    }

    #[inline]
    fn token(&self, kind: TokenKind, v: &'input [u8], start: Position, _end: Position) -> Token<'input> {
        // SAFETY: The input bytes are guaranteed to be valid UTF-8 because:
        // 1. File contents are validated via simdutf8 during database loading
        // 2. Invalid UTF-8 is converted lossily before reaching the lexer
        // 3. All byte slices here are subslices of the validated input
        let value = unsafe { std::str::from_utf8_unchecked(v) };

        Token { kind, start, value }
    }

    #[inline]
    fn interpolation(
        &mut self,
        end_offset: u32,
        post_interpolation_mode: LexerMode<'input>,
    ) -> Option<Result<Token<'input>, SyntaxError>> {
        self.mode = LexerMode::Script;

        let was_interpolating = self.interpolating;
        self.interpolating = true;

        loop {
            let subsequent_token = self.advance()?.ok()?;
            // Check if this token contains the end offset
            let token_start = subsequent_token.start.offset;
            let token_end = token_start + subsequent_token.value.len() as u32;
            let is_final_token = token_start <= end_offset && end_offset <= token_end;

            self.buffer.push_back(subsequent_token);

            if is_final_token {
                break;
            }
        }

        self.mode = post_interpolation_mode;
        self.interpolating = was_interpolating;

        self.advance()
    }
}

impl HasFileId for Lexer<'_> {
    #[inline]
    fn file_id(&self) -> FileId {
        self.input.file_id()
    }
}

#[inline]
fn matches_start_of_heredoc_document(input: &Input) -> bool {
    let total = input.len();
    let base = input.current_offset();

    // Start after the fixed opener (3 bytes).
    let mut length = 3;
    // Consume any following whitespace.
    while base + length < total && input.read_at(base + length).is_ascii_whitespace() {
        length += 1;
    }

    // The next byte must be a valid start-of-identifier.
    if base + length >= total || !is_start_of_identifier(input.read_at(base + length)) {
        return false;
    }
    length += 1; // Include that identifier start.

    // Now continue reading identifier characters until a newline is found.
    loop {
        let pos = base + length;
        if pos >= total {
            return false; // Unexpected EOF
        }

        let byte = *input.read_at(pos);
        if byte == b'\n' {
            return true; // Newline found: valid heredoc opener.
        } else if byte == b'\r' {
            // Handle CRLF: treat '\r' followed by '\n' as a newline as well.
            return pos + 1 < total && *input.read_at(pos + 1) == b'\n';
        } else if is_part_of_identifier(input.read_at(pos)) {
            length += 1;
        } else {
            return false; // Unexpected character.
        }
    }
}

#[inline]
fn matches_start_of_double_quote_heredoc_document(input: &Input) -> bool {
    let total = input.len();
    let base = input.current_offset();

    // Start after the fixed opener (3 bytes), then skip any whitespace.
    let mut length = 3;
    while base + length < total && input.read_at(base + length).is_ascii_whitespace() {
        length += 1;
    }

    // Next, expect an opening double quote.
    if base + length >= total || *input.read_at(base + length) != b'"' {
        return false;
    }
    length += 1;

    // The following byte must be a valid start-of-identifier.
    if base + length >= total || !is_start_of_identifier(input.read_at(base + length)) {
        return false;
    }
    length += 1;

    // Now scan the label. For double‑quoted heredoc, a terminating double quote is required.
    let mut terminated = false;
    loop {
        let pos = base + length;
        if pos >= total {
            return false;
        }
        let byte = input.read_at(pos);
        if *byte == b'\n' {
            // End of line: valid only if a closing double quote was encountered.
            return terminated;
        } else if *byte == b'\r' {
            // Handle CRLF sequences.
            return terminated && pos + 1 < total && *input.read_at(pos + 1) == b'\n';
        } else if !terminated && is_part_of_identifier(byte) {
            length += 1;
        } else if !terminated && *byte == b'"' {
            terminated = true;
            length += 1;
        } else {
            return false;
        }
    }
}

#[inline]
fn matches_start_of_nowdoc_document(input: &Input) -> bool {
    let total = input.len();
    let base = input.current_offset();

    // Start after the fixed opener (3 bytes) and skip whitespace.
    let mut length = 3;
    while base + length < total && input.read_at(base + length).is_ascii_whitespace() {
        length += 1;
    }

    // Now, the next byte must be a single quote.
    if base + length >= total || *input.read_at(base + length) != b'\'' {
        return false;
    }
    length += 1;

    // The following byte must be a valid start-of-identifier.
    if base + length >= total || !is_start_of_identifier(input.read_at(base + length)) {
        return false;
    }
    length += 1;

    // Read the label until a newline. A terminating single quote is required.
    let mut terminated = false;
    loop {
        let pos = base + length;
        if pos >= total {
            return false;
        }
        let byte = *input.read_at(pos);
        if byte == b'\n' {
            return terminated;
        } else if byte == b'\r' {
            return terminated && pos + 1 < total && *input.read_at(pos + 1) == b'\n';
        } else if !terminated && is_part_of_identifier(&byte) {
            length += 1;
        } else if !terminated && byte == b'\'' {
            terminated = true;
            length += 1;
        } else {
            return false;
        }
    }
}

#[inline]
fn matches_literal_double_quote_string(input: &Input) -> bool {
    let total = input.len();
    let base = input.current_offset();

    // Start after the initial double-quote (assumed consumed).
    let mut pos = base + 1;
    loop {
        if pos >= total {
            // Reached EOF: assume literal is complete.
            return true;
        }
        let byte = *input.read_at(pos);
        if byte == b'"' {
            // Encounter a closing double quote.
            return true;
        } else if byte == b'\\' {
            // Skip an escape sequence: assume that the backslash and the escaped character form a pair.
            pos += 2;
            continue;
        }

        // Check for variable interpolation or complex expression start:
        // If two-byte sequences match either "$" followed by a start-of-identifier or "{" and "$", then return false.
        if pos + 1 < total {
            let next = *input.read_at(pos + 1);
            if (byte == b'$' && (is_start_of_identifier(&next) || next == b'{')) || (byte == b'{' && next == b'$') {
                return false;
            }
        }
        pos += 1;
    }
}

#[inline]
fn read_start_of_heredoc_document(input: &Input, double_quoted: bool) -> (usize, usize, usize) {
    let total = input.len();
    let base = input.current_offset();

    // Start reading at offset base+3 (the fixed opener length).
    let mut pos = base + 3;
    let mut whitespaces = 0;
    while pos < total && input.read_at(pos).is_ascii_whitespace() {
        whitespaces += 1;
        pos += 1;
    }

    // The label (or delimiter) starts after:
    //   3 bytes + whitespace bytes + an extra offset:
    //      if double-quoted: 2 bytes (opening and closing quotes around the label)
    //      else: 1 byte.
    let mut length = 3 + whitespaces + if double_quoted { 2 } else { 1 };

    let mut label_length = 1; // Start with at least one byte for the label.
    let mut terminated = false; // For double-quoted heredoc, to track the closing quote.
    loop {
        let pos = base + length;
        // Ensure we haven't run past the input.
        if pos >= total {
            unreachable!("Unexpected end of input while reading heredoc label");
        }

        let byte = *input.read_at(pos);
        if byte == b'\n' {
            // Newline ends the label.
            length += 1;
            return (length, whitespaces, label_length);
        } else if byte == b'\r' {
            // Handle CRLF sequences
            if pos + 1 < total && *input.read_at(pos + 1) == b'\n' {
                length += 2;
            } else {
                length += 1;
            }
            return (length, whitespaces, label_length);
        } else if is_part_of_identifier(&byte) && (!double_quoted || !terminated) {
            // For both unquoted and double-quoted (before the closing quote) heredoc,
            // a valid identifier character is part of the label.
            length += 1;
            label_length += 1;
        } else if double_quoted && !terminated && byte == b'"' {
            // In a double-quoted heredoc, a double quote terminates the label.
            length += 1;
            terminated = true;
        } else {
            unreachable!("Unexpected character encountered in heredoc label");
        }
    }
}

#[inline]
fn read_start_of_nowdoc_document(input: &Input) -> (usize, usize, usize) {
    let total = input.len();
    let base = input.current_offset();

    let mut pos = base + 3;
    let mut whitespaces = 0;
    while pos < total && input.read_at(pos).is_ascii_whitespace() {
        whitespaces += 1;
        pos += 1;
    }

    // For nowdoc, the fixed extra offset is always 2.
    let mut length = 3 + whitespaces + 2;

    let mut label_length = 1;
    let mut terminated = false;
    loop {
        let pos = base + length;
        if pos >= total {
            unreachable!("Unexpected end of input while reading nowdoc label");
        }
        let byte = *input.read_at(pos);

        if byte == b'\n' {
            // A newline indicates the end of the label.
            length += 1;
            return (length, whitespaces, label_length);
        } else if byte == b'\r' {
            // Handle CRLF sequences
            if pos + 1 < total && *input.read_at(pos + 1) == b'\n' {
                length += 2;
            } else {
                length += 1;
            }
            return (length, whitespaces, label_length);
        } else if is_part_of_identifier(&byte) && !terminated {
            // For nowdoc, identifier characters contribute to the label until terminated.
            length += 1;
            label_length += 1;
        } else if !terminated && byte == b'\'' {
            // A single quote terminates the nowdoc label.
            length += 1;
            terminated = true;
        } else {
            unreachable!("Unexpected character encountered in nowdoc label");
        }
    }
}

#[inline]
fn read_literal_string(input: &Input, quote: u8) -> (TokenKind, usize) {
    let total = input.len();
    let start = input.current_offset();
    let mut length = 1; // We assume the opening quote is already consumed.

    let bytes = input.peek(length, total - start - length);
    loop {
        match memchr2(quote, b'\\', &bytes[length - 1..]) {
            Some(pos) => {
                let abs_pos = length - 1 + pos;
                let byte = bytes[abs_pos];

                if byte == b'\\' {
                    length = abs_pos + 2 + 1; // +1 because bytes starts at offset 1
                    if length > total - start {
                        return (TokenKind::PartialLiteralString, total - start);
                    }
                } else {
                    length = abs_pos + 2; // +1 for the quote, +1 because bytes starts at offset 1
                    return (TokenKind::LiteralString, length);
                }
            }
            None => {
                // No quote or backslash found - EOF
                return (TokenKind::PartialLiteralString, total - start);
            }
        }
    }
}

#[inline]
fn read_until_end_of_variable_interpolation(input: &Input, from: usize) -> u32 {
    let total = input.len();
    let base = input.current_offset();
    // `offset` is relative to the current position.
    let mut offset = from;

    loop {
        let abs = base + offset;
        if abs >= total {
            // End of input.
            break;
        }

        // Pattern 1: If the current byte is part of an identifier, simply advance.
        if is_part_of_identifier(input.read_at(abs)) {
            offset += 1;
            continue;
        }

        // Pattern 2: If the current byte is a '[' then we enter a bracketed interpolation.
        if *input.read_at(abs) == b'[' {
            offset += 1;
            let mut nesting = 0;
            loop {
                let abs_inner = base + offset;
                if abs_inner >= total {
                    break;
                }
                let b = input.read_at(abs_inner);
                if *b == b']' {
                    offset += 1;
                    if nesting == 0 {
                        break;
                    }

                    nesting -= 1;
                } else if *b == b'[' {
                    offset += 1;
                    nesting += 1;
                } else if b.is_ascii_whitespace() {
                    // Do not include whitespace.
                    break;
                } else {
                    offset += 1;
                }
            }
            // When bracketed interpolation is processed, exit the loop.
            break;
        }

        // Pattern 3: Check for "->" followed by a valid identifier start.
        if base + offset + 2 < total
            && *input.read_at(abs) == b'-'
            && *input.read_at(base + offset + 1) == b'>'
            && is_start_of_identifier(input.read_at(base + offset + 2))
        {
            offset += 3;
            // Consume any following identifier characters.
            while base + offset < total && is_part_of_identifier(input.read_at(base + offset)) {
                offset += 1;
            }
            break;
        }

        // Pattern 4: Check for "?->" followed by a valid identifier start.
        if base + offset + 3 < total
            && *input.read_at(abs) == b'?'
            && *input.read_at(base + offset + 1) == b'-'
            && *input.read_at(base + offset + 2) == b'>'
            && is_start_of_identifier(input.read_at(base + offset + 3))
        {
            offset += 4;
            while base + offset < total && is_part_of_identifier(input.read_at(base + offset)) {
                offset += 1;
            }
            break;
        }

        // None of the expected patterns matched: exit the loop.
        break;
    }

    offset as u32
}

#[inline]
fn read_until_end_of_brace_interpolation(input: &Input, from: usize) -> u32 {
    let total = input.len();
    let base = input.current_offset();
    let mut offset = from;
    let mut nesting = 0;

    loop {
        let abs = base + offset;
        if abs >= total {
            break;
        }
        match input.read_at(abs) {
            b'}' => {
                offset += 1;
                if nesting == 0 {
                    break;
                }

                nesting -= 1;
            }
            b'{' => {
                offset += 1;
                nesting += 1;
            }
            _ => {
                offset += 1;
            }
        }
    }

    offset as u32
}

/// Scan a multi-line comment using SIMD-accelerated search.
/// Returns Some(length) including the closing */, or None if unterminated.
#[inline]
fn scan_multi_line_comment(bytes: &[u8]) -> Option<usize> {
    // Use SIMD to find */ quickly
    memmem::find(bytes, b"*/").map(|pos| pos + 2)
}

/// Scan a single-line comment using SIMD-accelerated search.
/// Returns the length of the comment body (not including the //).
/// Stops at newline or ?>.
#[inline]
fn scan_single_line_comment(bytes: &[u8]) -> usize {
    let mut pos = 0;
    while pos < bytes.len() {
        match memchr::memchr3(b'\n', b'\r', b'?', &bytes[pos..]) {
            Some(offset) => {
                let found_pos = pos + offset;
                match bytes[found_pos] {
                    b'\n' | b'\r' => return found_pos,
                    b'?' => {
                        // Check if it's ?>
                        if found_pos + 1 < bytes.len() && bytes[found_pos + 1] == b'>' {
                            // Also check for whitespace before ?>
                            if found_pos > 0 && bytes[found_pos - 1].is_ascii_whitespace() {
                                return found_pos - 1;
                            }
                            return found_pos;
                        }
                        // Not ?>, continue searching
                        pos = found_pos + 1;
                    }
                    _ => unreachable!(),
                }
            }
            None => return bytes.len(),
        }
    }

    bytes.len()
}
