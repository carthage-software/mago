use crate::T;
use crate::ast::ast::*;
use crate::error::ParseError;
use crate::parser::internal::expression::parse_expression;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;

pub fn parse_throw<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<Throw<'arena>, ParseError> {
    Ok(Throw {
        throw: utils::expect_keyword(stream, T!["throw"])?,
        exception: {
            let expression = parse_expression(stream)?;

            stream.alloc(expression)
        },
    })
}
