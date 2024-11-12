use fennec_ast::ast::*;
use fennec_token::Precedence;
use fennec_token::T;

use crate::error::ParseError;
use crate::internal::argument::parse_optional_argument_list;
use crate::internal::expression::parse_expression_with_precedence;
use crate::internal::token_stream::TokenStream;
use crate::internal::utils;

pub fn parse_instantiation<'a, 'i>(stream: &mut TokenStream<'a, 'i>) -> Result<Instantiation, ParseError> {
    let new = utils::expect_keyword(stream, T!["new"])?;
    let class = parse_expression_with_precedence(stream, Precedence::New)?;
    let arguments = parse_optional_argument_list(stream)?;

    Ok(Instantiation { new, class, arguments })
}
