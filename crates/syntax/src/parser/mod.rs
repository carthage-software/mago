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
/// The parser holds an arena reference for allocations and provides methods
/// for parsing PHP files into AST nodes.
#[derive(Debug)]
pub struct Parser<'arena> {
    pub(crate) arena: &'arena Bump,
    pub(crate) state: State,
    errors: Vec<'arena, ParseError>,
}

impl<'arena> Parser<'arena> {
    /// Creates a new parser with the given arena for allocations.
    #[inline]
    pub fn new(arena: &'arena Bump) -> Self {
        Self { arena, state: Default::default(), errors: Vec::new_in(arena) }
    }

    /// Parses a file and returns the program AST along with any parse errors.
    pub fn parse(self, file: &File) -> &'arena Program<'arena> {
        self.parse_content(file.file_id(), file.contents.as_ref())
    }

    /// Parses content with a given file ID and returns the program AST along with any parse errors.
    pub fn parse_content<'input>(mut self, file_id: FileId, content: &'input str) -> &'arena Program<'arena> {
        let source_text = self.arena.alloc_str(content);

        let input = Input::new(file_id, source_text.as_bytes());
        let lexer = Lexer::new(self.arena, input);
        let mut stream = TokenStream::new(self.arena, lexer);
        let mut statements = Vec::new_in(self.arena);

        loop {
            let reached_eof = match stream.has_reached_eof() {
                Ok(eof) => eof,
                Err(err) => {
                    self.errors.push(ParseError::from(err));
                    break;
                }
            };

            if reached_eof {
                break;
            }

            match self.parse_statement(&mut stream) {
                Ok(statement) => statements.push(statement),
                Err(err) => {
                    self.errors.push(err);

                    break;
                }
            }
        }

        self.arena.alloc(Program {
            file_id,
            source_text,
            statements: Sequence::new(statements),
            trivia: stream.get_trivia(),
            errors: self.errors,
        })
    }
}

pub fn parse_file<'arena>(arena: &'arena Bump, file: &File) -> &'arena Program<'arena> {
    Parser::new(arena).parse(file)
}

pub fn parse_file_content<'arena>(arena: &'arena Bump, file_id: FileId, content: &str) -> &'arena Program<'arena> {
    Parser::new(arena).parse_content(file_id, content)
}
