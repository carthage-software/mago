use crate::T;
use crate::ast::ast::*;
use crate::error::ParseError;
use crate::parser::internal::expression;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;

pub fn parse_variable(stream: &mut TokenStream<'_, '_>) -> Result<Variable, ParseError> {
    let token = utils::peek(stream)?;

    Ok(match &token.kind {
        T!["$variable"] => Variable::Direct(parse_direct_variable(stream)?),
        T!["${"] => Variable::Indirect(parse_indirect_variable(stream)?),
        T!["$"] => Variable::Nested(parse_nested_variable(stream)?),
        _ => return Err(utils::unexpected(stream, Some(token), T!["$variable", "${", "$"])),
    })
}

pub fn parse_direct_variable(stream: &mut TokenStream<'_, '_>) -> Result<DirectVariable, ParseError> {
    let token = utils::expect(stream, T!["$variable"])?;

    Ok(DirectVariable { span: token.span, name: token.value })
}

pub fn parse_indirect_variable(stream: &mut TokenStream<'_, '_>) -> Result<IndirectVariable, ParseError> {
    let within_indirect_variable = stream.state.within_indirect_variable;

    let dollar_left_brace = utils::expect_span(stream, T!["${"])?;
    stream.state.within_indirect_variable = true;
    let expression = expression::parse_expression(stream)?;
    stream.state.within_indirect_variable = within_indirect_variable;
    let right_brace = utils::expect_span(stream, T!["}"])?;

    Ok(IndirectVariable { dollar_left_brace, expression: Box::new(expression), right_brace })
}

pub fn parse_nested_variable(stream: &mut TokenStream<'_, '_>) -> Result<NestedVariable, ParseError> {
    Ok(NestedVariable { dollar: utils::expect_span(stream, T!["$"])?, variable: Box::new(parse_variable(stream)?) })
}
