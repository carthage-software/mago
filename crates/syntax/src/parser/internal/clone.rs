use crate::T;
use crate::ast::ast::*;
use crate::error::ParseError;
use crate::parser::internal::expression::parse_expression_with_precedence;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;
use crate::token::Precedence;

pub fn parse_clone(stream: &mut TokenStream<'_, '_>) -> Result<Clone, ParseError> {
    Ok(Clone {
        clone: utils::expect_keyword(stream, T!["clone"])?,
        object: Box::new(parse_expression_with_precedence(stream, Precedence::Clone)?),
    })
}
