use mago_ast::ast::*;
use mago_token::Precedence;
use mago_token::T;

use crate::error::ParseError;
use crate::internal::expression::parse_expression_with_precedence;
use crate::internal::token_stream::TokenStream;
use crate::internal::utils;

#[inline]
pub fn parse_clone<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<Clone<'i>, ParseError> {
    let clone = utils::expect_keyword(stream, T!["clone"])?;
    let object = parse_expression_with_precedence(stream, Precedence::Clone)?;

    Ok(Clone { clone, object: stream.boxed(object) })
}
