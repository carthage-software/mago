use mago_ast::ast::*;
use mago_token::T;

use crate::error::ParseError;
use crate::internal::expression::parse_expression;
use crate::internal::token_stream::TokenStream;
use crate::internal::utils;

pub fn parse_throw(stream: &mut TokenStream<'_, '_>) -> Result<Throw, ParseError> {
    Ok(Throw { throw: utils::expect_keyword(stream, T!["throw"])?, exception: Box::new(parse_expression(stream)?) })
}
