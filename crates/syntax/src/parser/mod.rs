use bumpalo::Bump;
use bumpalo::collections::Vec;

use mago_database::file::File;
use mago_database::file::FileId;
use mago_database::file::HasFileId;
use mago_syntax_core::input::Input;

use crate::ast::Program;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::lexer::Lexer;
use crate::parser::stream::TokenStream;

mod internal;

pub mod stream;

#[derive(Debug, Default)]
pub struct State {
    pub within_indirect_variable: bool,
    pub within_string_interpolation: bool,
}

/// The main parser for PHP source code.
///
/// The parser holds an arena reference, the token stream, and parsing state.
#[derive(Debug)]
pub struct Parser<'input, 'arena> {
    pub(crate) arena: &'arena Bump,
    pub(crate) state: State,
    pub(crate) stream: TokenStream<'input, 'arena>,
    pub(crate) errors: Vec<'arena, ParseError>,
}

impl<'input, 'arena> Parser<'input, 'arena> {
    /// Creates a new parser for the given content.
    #[inline]
    fn new(arena: &'arena Bump, file_id: FileId, content: &'input str) -> Self {
        let input = Input::new(file_id, content.as_bytes());
        let lexer = Lexer::new(arena, input);
        let stream = TokenStream::new(arena, lexer);

        Self { arena, state: State::default(), stream, errors: Vec::new_in(arena) }
    }

    /// Parses and returns the program AST.
    fn parse(mut self, source_text: &'arena str, file_id: FileId) -> &'arena Program<'arena> {
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

            match self.parse_statement() {
                Ok(statement) => statements.push(statement),
                Err(err) => self.errors.push(err),
            }
        }

        self.arena.alloc(Program {
            file_id,
            source_text,
            statements: Sequence::new(statements),
            trivia: self.stream.get_trivia(),
            errors: self.errors,
        })
    }
}

pub fn parse_file<'arena>(arena: &'arena Bump, file: &File) -> &'arena Program<'arena> {
    parse_file_content(arena, file.file_id(), file.contents.as_ref())
}

pub fn parse_file_content<'arena>(arena: &'arena Bump, file_id: FileId, content: &str) -> &'arena Program<'arena> {
    let source_text = arena.alloc_str(content);
    Parser::new(arena, file_id, source_text).parse(source_text, file_id)
}
