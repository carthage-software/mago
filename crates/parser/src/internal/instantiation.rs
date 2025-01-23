use mago_ast::ast::*;
use mago_token::Precedence;
use mago_token::T;

use crate::error::ParseError;
use crate::internal::argument::parse_optional_argument_list;
use crate::internal::expression::parse_expression_with_precedence;
use crate::internal::token_stream::TokenStream;
use crate::internal::utils;

pub fn parse_instantiation(stream: &mut TokenStream<'_, '_>) -> Result<Instantiation, ParseError> {
    Ok(Instantiation {
        new: utils::expect_keyword(stream, T!["new"])?,
        class: Box::new(parse_expression_with_precedence(stream, Precedence::New)?),
        arguments: parse_optional_argument_list(stream)?,
    })
}
