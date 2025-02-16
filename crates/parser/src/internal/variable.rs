use mago_ast::ast::*;
use mago_token::T;

use crate::error::ParseError;
use crate::internal::expression;
use crate::internal::token_stream::TokenStream;
use crate::internal::utils;

#[inline]
pub fn parse_variable<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<Variable<'i>, ParseError> {
    let token = utils::peek(stream)?;

    Ok(match &token.kind {
        T!["$variable"] => Variable::Direct(parse_direct_variable(stream)?),
        T!["${"] => Variable::Indirect(parse_indirect_variable(stream)?),
        T!["$"] => Variable::Nested(parse_nested_variable(stream)?),
        _ => return Err(utils::unexpected(stream, Some(token), T!["$variable", "${", "$"])),
    })
}

#[inline]
pub fn parse_direct_variable(stream: &mut TokenStream<'_, '_>) -> Result<DirectVariable, ParseError> {
    let token = utils::expect(stream, T!["$variable"])?;

    Ok(DirectVariable { span: token.span, name: token.value })
}

#[inline]
pub fn parse_indirect_variable<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<IndirectVariable<'i>, ParseError> {
    let dollar_left_brace = utils::expect_span(stream, T!["${"])?;
    let expression = expression::parse_expression(stream)?;
    let right_brace = utils::expect_span(stream, T!["}"])?;

    Ok(IndirectVariable { dollar_left_brace, expression: stream.boxed(expression), right_brace })
}

#[inline]
pub fn parse_nested_variable<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<NestedVariable<'i>, ParseError> {
    let dollar = utils::expect_span(stream, T!["$"])?;
    let variable = parse_variable(stream)?;

    Ok(NestedVariable { dollar, variable: stream.boxed(variable) })
}
