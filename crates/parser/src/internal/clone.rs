use mago_ast::ast::*;
use mago_token::Precedence;
use mago_token::T;

use crate::error::ParseError;
use crate::internal::expression::parse_expression_with_precedence;
use crate::internal::token_stream::TokenStream;
use crate::internal::utils;

pub fn parse_clone(stream: &mut TokenStream<'_, '_>) -> Result<Clone, ParseError> {
    Ok(Clone {
        clone: utils::expect_keyword(stream, T!["clone"])?,
        object: Box::new(parse_expression_with_precedence(stream, Precedence::Clone)?),
    })
}
