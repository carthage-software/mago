use mago_ast::ast::*;
use mago_token::T;

use crate::error::ParseError;
use crate::internal::expression::parse_expression;
use crate::internal::statement::parse_statement;
use crate::internal::terminator::parse_terminator;
use crate::internal::token_stream::TokenStream;
use crate::internal::utils;

#[inline]
pub fn parse_do_while<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<DoWhile<'i>, ParseError> {
    let r#do = utils::expect_keyword(stream, T!["do"])?;
    let statement = parse_statement(stream)?;
    let r#while = utils::expect_keyword(stream, T!["while"])?;
    let left_parenthesis = utils::expect_span(stream, T!["("])?;
    let condition = parse_expression(stream)?;
    let right_parenthesis = utils::expect_span(stream, T![")"])?;
    let terminator = parse_terminator(stream)?;

    Ok(DoWhile {
        r#do,
        statement: stream.boxed(statement),
        r#while,
        left_parenthesis,
        condition: stream.boxed(condition),
        right_parenthesis,
        terminator,
    })
}
