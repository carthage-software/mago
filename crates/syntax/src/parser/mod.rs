use bumpalo::Bump;

use mago_database::file::File;
use mago_database::file::HasFileId;
use mago_syntax_core::input::Input;

use crate::ast::Program;
use crate::ast::sequence::Sequence;
use crate::lexer::Lexer;

use crate::error::ParseError;
use crate::parser::internal::statement::parse_statement;
use crate::parser::internal::token_stream::TokenStream;

mod internal;

pub fn parse_file<'arena>(arena: &'arena Bump, file: &File) -> (&'arena Program<'arena>, Option<ParseError>) {
    let source_text = arena.alloc_str(file.contents.as_ref());
    let input = Input::new(file.id, source_text.as_bytes());
    let lexer = Lexer::new(arena, input);

    let mut stream = TokenStream::new(arena, lexer);

    let mut error = None;
    let statements = {
        let mut statements = stream.new_vec();

        loop {
            match stream.has_reached_eof() {
                Ok(false) => match parse_statement(&mut stream) {
                    Ok(statement) => {
                        statements.push(statement);
                    }
                    Err(parse_error) => {
                        error = Some(parse_error);

                        break;
                    }
                },
                Ok(true) => {
                    break;
                }
                Err(syntax_error) => {
                    error = Some(ParseError::from(syntax_error));

                    break;
                }
            }
        }

        statements
    };

    let program = arena.alloc(Program {
        file_id: stream.file_id(),
        source_text,
        statements: Sequence::new(statements),
        trivia: stream.get_trivia(),
    });

    (program, error)
}
