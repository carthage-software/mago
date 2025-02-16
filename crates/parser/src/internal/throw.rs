use mago_ast::ast::*;
use mago_token::T;

use crate::error::ParseError;
use crate::internal::expression::parse_expression;
use crate::internal::token_stream::TokenStream;
use crate::internal::utils;

#[inline]
pub fn parse_throw<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<Throw<'i>, ParseError> {
    let throw = utils::expect_keyword(stream, T!["throw"])?;
    let exception = parse_expression(stream)?;

    Ok(Throw { throw, exception: stream.boxed(exception) })
}
