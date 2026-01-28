use std::collections::VecDeque;
use std::fmt::Debug;

use bumpalo::Bump;
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
use mago_syntax_core::utils::is_part_of_identifier;
use mago_syntax_core::utils::is_start_of_identifier;
use mago_syntax_core::utils::read_digits_of_base;

use crate::error::SyntaxError;
use crate::lexer::internal::mode::HaltStage;
use crate::lexer::internal::mode::Interpolation;
use crate::lexer::internal::mode::LexerMode;
use crate::lexer::internal::utils::NumberKind;
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
pub struct Lexer<'input, 'arena> {
    arena: &'arena Bump,
    input: Input<'input>,
    mode: LexerMode<'arena>,
    interpolating: bool,
    buffer: VecDeque<Token<'arena>>,
}

impl<'input, 'arena> Lexer<'input, 'arena> {
    /// Initial capacity for the token buffer used during string interpolation.
    const BUFFER_INITIAL_CAPACITY: usize = 8;

    /// Creates a new `Lexer` instance.
    ///
    /// # Parameters
    ///
    /// - `arena`: The arena to use for allocating tokens.
    /// - `input`: The input source code to tokenize.
    ///
    /// # Returns
    ///
    /// A new `Lexer` instance that reads from the provided byte slice.
    pub fn new(arena: &'arena Bump, input: Input<'input>) -> Lexer<'input, 'arena> {
        Lexer {
            arena,
            input,
            mode: LexerMode::Inline,
            interpolating: false,
            buffer: VecDeque::with_capacity(Self::BUFFER_INITIAL_CAPACITY),
        }
    }

    /// Creates a new `Lexer` instance for parsing a script block.
    ///
    /// # Parameters
    ///
    /// - `arena`: The arena to use for allocating tokens.
    /// - `input`: The input source code to tokenize.
    ///
    /// # Returns
    ///
    /// A new `Lexer` instance that reads from the provided byte slice.
    pub fn scripting(arena: &'arena Bump, input: Input<'input>) -> Lexer<'input, 'arena> {
        Lexer {
            arena,
            input,
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
    #[must_use]
    pub fn get_position(&self) -> Position {
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
    pub fn advance(&mut self) -> Option<Result<Token<'arena>, SyntaxError>> {
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
                if self.input.is_at(b"<?", false) {
                    let (kind, buffer) = if self.input.is_at(b"<?php", true) {
                        (TokenKind::OpenTag, self.input.consume(5))
                    } else if self.input.is_at(b"<?=", false) {
                        (TokenKind::EchoTag, self.input.consume(3))
                    } else {
                        (TokenKind::ShortOpenTag, self.input.consume(2))
                    };

                    let end = self.input.current_position();
                    let tag = self.token(kind, buffer, start, end);

                    self.mode = LexerMode::Script;

                    return Some(Ok(tag));
                }

                if self.input.is_at(b"#!", true) {
                    let buffer = self.input.consume_through(b'\n');
                    let end = self.input.current_position();

                    Some(Ok(self.token(TokenKind::InlineShebang, buffer, start, end)))
                } else {
                    let buffer = self.input.consume_until(b"<?", false);
                    let end = self.input.current_position();

                    Some(Ok(self.token(TokenKind::InlineText, buffer, start, end)))
                }
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
                        let mut length = 2;
                        loop {
                            match self.input.peek(length, 3) {
                                [b'\n' | b'\r', ..] => {
                                    break;
                                }
                                [w, b'?', b'>'] if w.is_ascii_whitespace() => {
                                    break;
                                }
                                [b'?', b'>', ..] | [] => {
                                    break;
                                }
                                [_, ..] => {
                                    length += 1;
                                }
                            }
                        }

                        (TokenKind::SingleLineComment, length)
                    }
                    [b'/', b'*', asterisk] => {
                        let mut length = 2;
                        let mut is_multiline = false;
                        let mut terminated = false;
                        loop {
                            match self.input.peek(length, 2) {
                                [b'*', b'/'] => {
                                    if length == 2 {
                                        is_multiline = true;
                                    }

                                    length += 2;

                                    terminated = true;
                                    break;
                                }
                                [_, ..] => {
                                    length += 1;
                                }
                                [] => {
                                    break;
                                }
                            }
                        }

                        if !terminated {
                            self.input.consume(length);

                            return Some(Err(SyntaxError::UnexpectedEndOfFile(
                                self.file_id(),
                                self.input.current_position(),
                            )));
                        }

                        if !is_multiline && asterisk == &b'*' {
                            (TokenKind::DocBlockComment, length)
                        } else {
                            (TokenKind::MultiLineComment, length)
                        }
                    }
                    [b'\\', start_of_identifier!(), ..] => {
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

                        if last_was_slash {
                            length -= 1;
                        }

                        (TokenKind::FullyQualifiedIdentifier, length)
                    }
                    [b'$', start_of_identifier!(), ..] => {
                        let mut length = 2;
                        while let [part_of_identifier!(), ..] = self.input.peek(length, 1) {
                            length += 1;
                        }

                        (TokenKind::Variable, length)
                    }
                    [b'$', b'{', ..] => (TokenKind::DollarLeftBrace, 2),
                    [b'$', ..] => (TokenKind::Dollar, 1),
                    [b'@', ..] => (TokenKind::At, 1),
                    [b'!', ..] => (TokenKind::Bang, 1),
                    [b'&', ..] => (TokenKind::Ampersand, 1),
                    [b'?', ..] => (TokenKind::Question, 1),
                    [b'=', ..] => (TokenKind::Equal, 1),
                    [b'`', ..] => (TokenKind::Backtick, 1),
                    [b')', ..] => (TokenKind::RightParenthesis, 1),
                    [b';', ..] => (TokenKind::Semicolon, 1),
                    [b'+', ..] => (TokenKind::Plus, 1),
                    [b'%', ..] => (TokenKind::Percent, 1),
                    [b'-', ..] => (TokenKind::Minus, 1),
                    [b'<', ..] => (TokenKind::LessThan, 1),
                    [b'>', ..] => (TokenKind::GreaterThan, 1),
                    [b',', ..] => (TokenKind::Comma, 1),
                    [b'[', ..] => (TokenKind::LeftBracket, 1),
                    [b']', ..] => (TokenKind::RightBracket, 1),
                    [b'{', ..] => (TokenKind::LeftBrace, 1),
                    [b'}', ..] => (TokenKind::RightBrace, 1),
                    [b':', ..] => (TokenKind::Colon, 1),
                    [b'~', ..] => (TokenKind::Tilde, 1),
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
                        for (value, kind) in internal::consts::CAST_TYPES {
                            if let Some(length) = self.input.match_sequence_ignore_whitespace(value, true) {
                                break 'parenthesis (kind, length);
                            }
                        }

                        (TokenKind::LeftParenthesis, 1)
                    }
                    [b'#', ..] => {
                        let mut length = 1;
                        loop {
                            match self.input.peek(length, 3) {
                                [b'\n' | b'\r', ..] => {
                                    break;
                                }
                                [w, b'?', b'>'] if w.is_ascii_whitespace() => {
                                    break;
                                }
                                [b'?', b'>', ..] | [] => {
                                    break;
                                }
                                [_, ..] => {
                                    length += 1;
                                }
                            }
                        }

                        (TokenKind::HashComment, length)
                    }
                    [b'\\', ..] => (TokenKind::NamespaceSeparator, 1),
                    [start_of_identifier!(), ..] => 'identifier: {
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
                                // special case for `private(set)`
                                [b'(', ..] if length == 7 => {
                                    if self.input.is_at(b"private(set)", true) {
                                        break 'identifier (TokenKind::PrivateSet, 7 + 5);
                                    }

                                    break;
                                }
                                // special case for `public(set)`
                                [b'(', ..] if length == 6 => {
                                    if self.input.is_at(b"public(set)", true) {
                                        break 'identifier (TokenKind::PublicSet, 6 + 5);
                                    }

                                    break;
                                }
                                // special case for `protected(set)`
                                [b'(', ..] if length == 9 => {
                                    if self.input.is_at(b"protected(set)", true) {
                                        break 'identifier (TokenKind::ProtectedSet, 9 + 5);
                                    }

                                    break;
                                }
                                _ => {
                                    break;
                                }
                            }
                        }

                        if !ended_with_slash {
                            for (value, kind) in internal::consts::KEYWORD_TYPES {
                                if value.len() != length {
                                    continue;
                                }

                                if self.input.is_at(value, true) {
                                    break 'identifier (kind, value.len());
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
                                [b'\\', ..] if !self.interpolating => {
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

                        if slashes > 0 {
                            (TokenKind::QualifiedIdentifier, length)
                        } else {
                            (TokenKind::Identifier, length)
                        }
                    }
                    [b'.', start_of_number!(), ..] => {
                        let mut length = read_digits_of_base(&self.input, 2, 10);
                        if let float_exponent!() = self.input.peek(length, 1) {
                            length += 1;
                            if let number_sign!() = self.input.peek(length, 1) {
                                length += 1;
                            }

                            length = read_digits_of_base(&self.input, length, 10);
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
                            length += 1;
                            if let number_sign!() = self.input.peek(length, 1) {
                                length += 1;
                            }

                            length = read_digits_of_base(&self.input, length, 10);
                        }

                        (TokenKind::LiteralFloat, length)
                    }
                    [b'.', ..] => (TokenKind::Dot, 1),
                    [unknown_byte, ..] => {
                        return Some(Err(SyntaxError::UnrecognizedToken(
                            self.file_id(),
                            *unknown_byte,
                            self.input.current_position(),
                        )));
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
                    TokenKind::DocumentStart(document_kind) => LexerMode::DocumentString(
                        document_kind,
                        self.arena.alloc_slice_copy(document_label),
                        Interpolation::None,
                    ),
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
                            Some(Err(SyntaxError::UnexpectedToken(
                                self.file_id(),
                                self.input.read(1)[0],
                                self.input.current_position(),
                            )))
                        }
                    }
                    HaltStage::LookingForRightParenthesis => {
                        if self.input.is_at(b")", false) {
                            let buffer = self.input.consume(1);
                            let end = self.input.current_position();

                            self.mode = LexerMode::Halt(HaltStage::LookingForTerminator);

                            Some(Ok(self.token(TokenKind::RightParenthesis, buffer, start, end)))
                        } else {
                            Some(Err(SyntaxError::UnexpectedToken(
                                self.file_id(),
                                self.input.read(1)[0],
                                self.input.current_position(),
                            )))
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
                            Some(Err(SyntaxError::UnexpectedToken(
                                self.file_id(),
                                self.input.read(1)[0],
                                self.input.current_position(),
                            )))
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    #[inline]
    fn token(&mut self, kind: TokenKind, v: &[u8], from: Position, to: Position) -> Token<'arena> {
        // SAFETY: The input bytes are guaranteed to be valid UTF-8 because:
        // 1. File contents are validated via simdutf8 during database loading
        // 2. Invalid UTF-8 is converted lossily before reaching the lexer
        // 3. All byte slices here are subslices of the validated input
        let string = unsafe { std::str::from_utf8_unchecked(v) };

        Token { kind, value: self.arena.alloc_str(string), span: Span::new(self.file_id(), from, to) }
    }

    #[inline]
    fn interpolation(
        &mut self,
        end_offset: u32,
        post_interpolation_mode: LexerMode<'arena>,
    ) -> Option<Result<Token<'arena>, SyntaxError>> {
        self.mode = LexerMode::Script;

        let was_interpolating = self.interpolating;
        self.interpolating = true;

        loop {
            let subsequent_token = self.advance()?.ok()?;
            let is_final_token = subsequent_token.span.has_offset(end_offset);

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

impl HasFileId for Lexer<'_, '_> {
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

    // --- Block 1: Consume Whitespace ---
    // Start reading at offset base+3 (the fixed opener length).
    let mut pos = base + 3;
    let mut whitespaces = 0;
    while pos < total && input.read_at(pos).is_ascii_whitespace() {
        whitespaces += 1;
        pos += 1;
    }

    // --- Block 2: Calculate Initial Label Offset ---
    // The label (or delimiter) starts after:
    //   3 bytes + whitespace bytes + an extra offset:
    //      if double-quoted: 2 bytes (opening and closing quotes around the label)
    //      else: 1 byte.
    let mut length = 3 + whitespaces + if double_quoted { 2 } else { 1 };

    // --- Block 3: Read the Label ---
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

    // --- Block 1: Consume Whitespace ---
    let mut pos = base + 3;
    let mut whitespaces = 0;
    while pos < total && input.read_at(pos).is_ascii_whitespace() {
        whitespaces += 1;
        pos += 1;
    }

    // --- Block 2: Calculate Initial Label Offset ---
    // For nowdoc, the fixed extra offset is always 2.
    let mut length = 3 + whitespaces + 2;

    // --- Block 3: Read the Label ---
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
    let mut last_was_backslash = false;
    let mut partial = false;

    loop {
        let pos = start + length;
        if pos >= total {
            // Reached EOF before closing quote.
            partial = true;
            break;
        }

        let byte = input.read_at(pos);
        if *byte == b'\\' {
            // Toggle the backslash flag.
            last_was_backslash = !last_was_backslash;
            length += 1;
        } else {
            // If we see the closing quote and the previous byte was not an escape.
            if *byte == quote && !last_was_backslash {
                length += 1; // Include the closing quote.
                break;
            }
            length += 1;
            last_was_backslash = false;
        }
    }

    if partial { (TokenKind::PartialLiteralString, length) } else { (TokenKind::LiteralString, length) }
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
