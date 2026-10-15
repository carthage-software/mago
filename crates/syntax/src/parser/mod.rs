use mago_allocator::prelude::*;

use mago_database::file::File;
use mago_database::file::FileId;
use mago_database::file::HasFileId;
use mago_span::{HasSpan, Position, Span};
use mago_syntax_core::input::Input;

use crate::cst::Program;
use crate::cst::Statement;
use crate::cst::sequence::Sequence;
use crate::error::ParseError;
use crate::lexer::Lexer;
use crate::parser::stream::TokenStream;
use crate::settings::ParserSettings;
use crate::token::TokenKind;

mod internal;

pub mod stream;

/// Maximum recursion depth for expression parsing.
/// This prevents stack overflow on deeply nested expressions and statements.
const MAX_RECURSION_DEPTH: u16 = 512;

#[derive(Debug, Default)]
pub struct State {
    pub within_string_interpolation: bool,
    pub recursion_depth: u16,
}

/// The main parser for PHP source code.
///
/// The parser holds an arena reference, the token stream, and parsing state.
#[derive(Debug)]
#[allow(clippy::field_scoped_visibility_modifiers)]
pub struct Parser<'input, 'arena, A>
where
    'input: 'arena,
    A: Arena,
{
    pub(crate) arena: &'arena A,
    pub(crate) state: State,
    pub(crate) stream: TokenStream<'input, 'arena, A>,
    pub(crate) errors: Vec<'arena, ParseError, A>,
    pub(crate) recovery_end: Option<Span>,
}

impl<'input, 'arena, A> Parser<'input, 'arena, A>
where
    A: Arena,
{
    /// Creates a new parser for the given content.
    ///
    /// # Parameters
    ///
    /// - `arena`: The memory arena for allocations.
    /// - `file_id`: The ID of the file being parsed.
    /// - `content`: The content to parse.
    /// - `settings`: The parser settings.
    ///
    /// # Returns
    ///
    /// A new `Parser` instance.
    #[inline]
    pub fn new(arena: &'arena A, file_id: FileId, content: &'input [u8], settings: ParserSettings) -> Self {
        let input = Input::new(file_id, content);
        let lexer = Lexer::new(input, settings.lexer);
        let stream = TokenStream::new(arena, lexer, settings.elide_detached_whitespace);

        let end = Position::new(content.len() as u32);
        let recovery_end = settings.recover_incomplete.then_some(Span::new(file_id, end, end));
        Self { arena, state: State::default(), stream, errors: Vec::new_in(arena), recovery_end }
    }

    fn recover_at_eof(&mut self, error: ParseError) -> Result<Span, ParseError> {
        if let Some(end) = self.recovery_end
            && matches!(error, ParseError::UnexpectedEndOfFile(..))
        {
            self.errors.push(error);
            Ok(end)
        } else {
            Err(error)
        }
    }

    fn parse_closing_span(&mut self, kind: TokenKind) -> Result<Span, ParseError> {
        match self.stream.eat_span(kind) {
            Ok(span) => Ok(span),
            Err(error) => self.recover_at_eof(error),
        }
    }

    fn parse_condition_closing_span(&mut self) -> Result<Span, ParseError> {
        let error = match self.stream.eat_span(TokenKind::RightParenthesis) {
            Ok(span) => return Ok(span),
            Err(error) => error,
        };

        let span = match (&error, self.recovery_end) {
            (ParseError::UnexpectedToken(..), Some(_)) => self.empty_span(),
            (ParseError::UnexpectedEndOfFile(..), Some(end)) => end,
            _ => return Err(error),
        };

        self.push_recovered_error(error);

        Ok(span)
    }

    fn parse_condition_body(&mut self, right_parenthesis: Span) -> Result<Statement<'arena>, ParseError> {
        if right_parenthesis.length() == 0 && self.stream.peek_kind(0)? != Some(TokenKind::LeftBrace) {
            return Ok(Statement::Noop(right_parenthesis));
        }

        self.parse_recovered_body()
    }

    fn parse_recovered_body(&mut self) -> Result<Statement<'arena>, ParseError> {
        if let Some(end) = self.recovery_end {
            match self.stream.lookahead(0)? {
                None => {
                    self.push_recovered_error(self.stream.unexpected(None, &[]));
                    return Ok(Statement::Noop(end));
                }
                Some(token) if token.kind == TokenKind::RightBrace => {
                    self.push_recovered_error(self.stream.unexpected(Some(token), &[]));
                    return Ok(Statement::Noop(self.empty_span()));
                }
                Some(_) => {}
            }
        }

        self.parse_statement()
    }

    fn push_recovered_error(&mut self, error: ParseError) {
        let offset = error.span().start.offset;
        if self.errors.last().is_none_or(|last| last.span().start.offset != offset) {
            self.errors.push(error);
        }
    }

    fn empty_span(&self) -> Span {
        let position = self.stream.current_position();
        Span::new(self.stream.file_id(), position, position)
    }

    /// Creates a new parser for the given file.
    ///
    /// # Parameters
    ///
    /// - `arena`: The memory arena for allocations.
    /// - `file`: The file to parse.
    /// - `settings`: The parser settings.
    ///
    /// # Returns
    ///
    /// A new `Parser` instance.
    pub fn for_file(arena: &'arena A, file: &'input File, settings: ParserSettings) -> Self {
        Self::new(arena, file.file_id(), file.contents.as_ref(), settings)
    }

    /// Parses and returns the program CST.
    fn parse(mut self, source_text: &'arena [u8], file_id: FileId) -> &'arena Program<'arena> {
        let mut statements = Vec::new_in(self.arena);

        loop {
            let reached_eof = match self.stream.has_reached_eof() {
                Ok(eof) => eof,
                Err(err) => {
                    self.errors.push(ParseError::from(err));
                    break;
                }
            };

            if reached_eof {
                break;
            }

            // Record position before parsing to detect infinite loops
            let position_before = self.stream.current_position();

            match self.parse_statement() {
                Ok(statement) => statements.push(statement),
                Err(err) => self.errors.push(err),
            }

            // Safety check: if we didn't advance at all, skip a token to prevent infinite loop.
            // This can happen with orphan keywords like `finally`, `catch`, `else`, etc.
            // that are preserved by the expression parser but not handled by the statement parser.
            let position_after = self.stream.current_position();
            if position_after == position_before
                && let Ok(Some(token)) = self.stream.lookahead(0)
            {
                self.errors.push(self.stream.unexpected(Some(token), &[]));
                let _ = self.stream.consume();
            }
        }

        self.arena.alloc(Program {
            file_id,
            source_text,
            statements: Sequence::new(statements),
            trivia: self.stream.get_trivia(),
            errors: self.errors.leak(),
        })
    }
}

/// Parses the given file and returns the program CST.
///
/// # Parameters
///
/// - `arena`: The memory arena for allocations.
/// - `file`: The file to parse.
///
/// # Returns
///
/// The parsed `Program` CST.
#[inline]
pub fn parse_file<'arena, A>(arena: &'arena A, file: &File) -> &'arena Program<'arena>
where
    A: Arena,
{
    parse_file_content(arena, file.file_id(), file.contents.as_ref())
}

/// Parses the given file with custom settings and returns the program CST.
///
/// # Parameters
///
/// - `arena`: The memory arena for allocations.
/// - `file`: The file to parse.
/// - `settings`: The parser settings.
///
/// # Returns
///
/// The parsed `Program` CST.
#[inline]
pub fn parse_file_with_settings<'arena, A>(
    arena: &'arena A,
    file: &File,
    settings: ParserSettings,
) -> &'arena Program<'arena>
where
    A: Arena,
{
    parse_file_content_with_settings(arena, file.file_id(), file.contents.as_ref(), settings)
}

/// Parses a file without copying its source bytes into the arena.
///
/// The file must remain alive while the returned program is in use.
#[inline]
pub fn parse_file_borrowed_with_settings<'arena, A>(
    arena: &'arena A,
    file: &'arena File,
    settings: ParserSettings,
) -> &'arena Program<'arena>
where
    A: Arena,
{
    Parser::for_file(arena, file, settings).parse(file.contents.as_ref(), file.file_id())
}

/// Parses the given file content and returns the program CST.
///
/// # Parameters
///
/// - `arena`: The memory arena for allocations.
/// - `file_id`: The ID of the file being parsed.
/// - `content`: The content to parse.
///
/// # Returns
///
/// The parsed `Program` CST.
pub fn parse_file_content<'arena, A>(arena: &'arena A, file_id: FileId, content: &[u8]) -> &'arena Program<'arena>
where
    A: Arena,
{
    let source_text = arena.alloc_slice_copy(content);
    Parser::new(arena, file_id, source_text, ParserSettings::default()).parse(source_text, file_id)
}

/// Parses the given file content with custom settings and returns the program CST.
///
/// # Parameters
///
/// - `arena`: The memory arena for allocations.
/// - `file_id`: The ID of the file being parsed.
/// - `content`: The content to parse.
/// - `settings`: The parser settings.
///
/// # Returns
///
/// The parsed `Program` CST.
pub fn parse_file_content_with_settings<'arena, A>(
    arena: &'arena A,
    file_id: FileId,
    content: &[u8],
    settings: ParserSettings,
) -> &'arena Program<'arena>
where
    A: Arena,
{
    let source_text = arena.alloc_slice_copy(content);
    Parser::new(arena, file_id, source_text, settings).parse(source_text, file_id)
}
