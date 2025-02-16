use mago_ast::ast::*;
use mago_ast::sequence::TokenSeparatedSequence;
use mago_token::T;

use crate::error::ParseError;
use crate::internal::terminator::parse_terminator;
use crate::internal::token_stream::TokenStream;
use crate::internal::utils;
use crate::internal::variable::parse_variable;

#[inline]
pub fn parse_global<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<Global<'i>, ParseError> {
    Ok(Global {
        global: utils::expect_keyword(stream, T!["global"])?,
        variables: {
            let mut variables = stream.vec();
            let mut commas = stream.vec();

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
