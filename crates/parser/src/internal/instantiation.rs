use mago_ast::ast::*;
use mago_token::Precedence;
use mago_token::T;

use crate::error::ParseError;
use crate::internal::argument::parse_optional_argument_list;
use crate::internal::expression::parse_expression_with_precedence;
use crate::internal::token_stream::TokenStream;
use crate::internal::utils;

#[inline]
pub fn parse_instantiation<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<Instantiation<'i>, ParseError> {
    let new = utils::expect_keyword(stream, T!["new"])?;
    let class = parse_expression_with_precedence(stream, Precedence::New)?;
    let arguments = parse_optional_argument_list(stream)?;

    Ok(Instantiation { new, class: stream.boxed(class), arguments })
}
