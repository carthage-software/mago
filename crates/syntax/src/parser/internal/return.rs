use crate::T;
use crate::ast::ast::*;
use crate::error::ParseError;
use crate::parser::internal::expression::parse_expression;
use crate::parser::internal::terminator::parse_terminator;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;

pub fn parse_return(stream: &mut TokenStream<'_, '_>) -> Result<Return, ParseError> {
    Ok(Return {
        r#return: utils::expect_keyword(stream, T!["return"])?,
        value: if matches!(utils::peek(stream)?.kind, T![";" | "?>"]) { None } else { Some(parse_expression(stream)?) },
        terminator: parse_terminator(stream)?,
    })
}
