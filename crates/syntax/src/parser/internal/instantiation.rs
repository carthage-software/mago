use crate::T;
use crate::ast::ast::*;
use crate::error::ParseError;
use crate::parser::internal::argument::parse_optional_argument_list;
use crate::parser::internal::expression::parse_expression_with_precedence;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;
use crate::token::Precedence;

pub fn parse_instantiation(stream: &mut TokenStream<'_, '_>) -> Result<Instantiation, ParseError> {
    Ok(Instantiation {
        new: utils::expect_keyword(stream, T!["new"])?,
        class: Box::new(parse_expression_with_precedence(stream, Precedence::New)?),
        arguments: parse_optional_argument_list(stream)?,
    })
}
