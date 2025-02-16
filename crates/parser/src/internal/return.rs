use mago_ast::ast::*;
use mago_token::T;

use crate::error::ParseError;
use crate::internal::expression::parse_expression;
use crate::internal::terminator::parse_terminator;
use crate::internal::token_stream::TokenStream;
use crate::internal::utils;

#[inline]
pub fn parse_return<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<Return<'i>, ParseError> {
    Ok(Return {
        r#return: utils::expect_keyword(stream, T!["return"])?,
        value: if matches!(utils::peek(stream)?.kind, T![";" | "?>"]) { None } else { Some(parse_expression(stream)?) },
        terminator: parse_terminator(stream)?,
    })
}
