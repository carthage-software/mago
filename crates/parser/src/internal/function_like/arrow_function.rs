use mago_ast::ast::*;
use mago_ast::sequence::Sequence;
use mago_token::T;

use crate::error::ParseError;
use crate::internal::expression::parse_expression;
use crate::internal::function_like::parameter::parse_function_like_parameter_list;
use crate::internal::function_like::r#return::parse_optional_function_like_return_type_hint;
use crate::internal::token_stream::TokenStream;
use crate::internal::utils;

pub fn parse_arrow_function_with_attributes(
    stream: &mut TokenStream<'_, '_>,
    attributes: Sequence<AttributeList>,
) -> Result<ArrowFunction, ParseError> {
    Ok(ArrowFunction {
        attribute_lists: attributes,
        r#static: utils::maybe_expect_keyword(stream, T!["static"])?,
        r#fn: utils::expect_keyword(stream, T!["fn"])?,
        ampersand: utils::maybe_expect(stream, T!["&"])?.map(|t| t.span),
        parameter_list: parse_function_like_parameter_list(stream)?,
        return_type_hint: parse_optional_function_like_return_type_hint(stream)?,
        arrow: utils::expect_span(stream, T!["=>"])?,
        expression: Box::new(parse_expression(stream)?),
    })
}
