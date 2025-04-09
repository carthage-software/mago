use mago_interner::ThreadedInterner;
use mago_source::Source;
use mago_syntax_core::input::Input;

use crate::ast::Program;
use crate::ast::sequence::Sequence;
use crate::lexer::Lexer;

use crate::error::ParseError;
use crate::parser::internal::statement::parse_statement;
use crate::parser::internal::token_stream::TokenStream;

mod internal;

pub fn parse_source(interner: &ThreadedInterner, source: &Source) -> (Program, Option<ParseError>) {
    let content = interner.lookup(&source.content);
    let lexer = Lexer::new(interner, Input::new(source.identifier, content.as_bytes()));

    construct(interner, lexer)
}

pub fn parse(interner: &ThreadedInterner, input: Input<'_>) -> (Program, Option<ParseError>) {
    let lexer = Lexer::new(interner, input);

    construct(interner, lexer)
}

fn construct<'i>(interner: &'i ThreadedInterner, lexer: Lexer<'_, 'i>) -> (Program, Option<ParseError>) {
    let mut stream = TokenStream::new(interner, lexer);

    let mut error = None;
    let statements = {
        let mut statements = Vec::new();

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

    (
        Program {
            source: stream.get_position().source,
            statements: Sequence::new(statements),
            trivia: stream.get_trivia(),
        },
        error,
    )
}
