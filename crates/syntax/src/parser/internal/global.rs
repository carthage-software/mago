use crate::T;
use crate::ast::ast::*;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::internal::terminator::parse_terminator;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;
use crate::parser::internal::variable::parse_variable;

pub fn parse_global(stream: &mut TokenStream<'_, '_>) -> Result<Global, ParseError> {
    Ok(Global {
        global: utils::expect_keyword(stream, T!["global"])?,
        variables: {
            let mut variables = vec![];
            let mut commas = vec![];

            loop {
                if matches!(utils::peek(stream)?.kind, T!["?>" | ";"]) {
                    break;
                }

                variables.push(parse_variable(stream)?);

                match utils::peek(stream)?.kind {
                    T![","] => {
                        commas.push(utils::expect_any(stream)?);
                    }
                    _ => {
                        break;
                    }
                }
            }

            TokenSeparatedSequence::new(variables, commas)
        },
        terminator: parse_terminator(stream)?,
    })
}
