use fennec_ast::ast::*;
use fennec_token::Precedence;
use fennec_token::T;

use crate::error::ParseError;
use crate::internal::expression::parse_expression_with_precedence;
use crate::internal::token_stream::TokenStream;
use crate::internal::utils;

pub fn parse_clone<'a, 'i>(stream: &mut TokenStream<'a, 'i>) -> Result<Clone, ParseError> {
    Ok(Clone {
        clone: utils::expect_keyword(stream, T!["clone"])?,
        object: parse_expression_with_precedence(stream, Precedence::Clone)?,
    })
}
