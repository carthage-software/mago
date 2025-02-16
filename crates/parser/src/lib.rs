use bumpalo::Bump;
use mago_ast::sequence::Sequence;
use mago_ast::Program;
use mago_interner::ThreadedInterner;
use mago_lexer::input::Input;
use mago_lexer::Lexer;
use mago_source::Source;

use crate::error::ParseError;
use crate::internal::statement::parse_statement;
use crate::internal::token_stream::TokenStream;

pub mod error;

mod internal;

pub fn parse_source<'alloc>(
    interner: &ThreadedInterner,
    bump: &'alloc Bump,
    source: &Source,
) -> (&'alloc Program<'alloc>, Option<ParseError>) {
    let content = interner.lookup(&source.content);
    let lexer = Lexer::new(interner, Input::new(source.identifier, content.as_bytes()));

    construct(interner, bump, lexer)
}

pub fn parse<'alloc>(
    interner: &ThreadedInterner,
    bump: &'alloc Bump,
    input: Input<'_>,
) -> (&'alloc Program<'alloc>, Option<ParseError>) {
    let lexer = Lexer::new(interner, input);

    construct(interner, bump, lexer)
}

fn construct<'i, 'alloc>(
    interner: &'i ThreadedInterner,
    bump: &'alloc Bump,
    lexer: Lexer<'_, 'i>,
) -> (&'alloc Program<'alloc>, Option<ParseError>) {
    let mut stream = TokenStream::new(interner, bump, lexer);

    let mut error = None;
    let statements = {
        let mut statements = bumpalo::vec![in bump];

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

    let program = bump.alloc(Program {
        source: stream.get_position().source,
        statements: Sequence::new(statements),
        trivia: stream.get_trivia(),
    });

    (program, error)
}
