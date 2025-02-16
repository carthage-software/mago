use mago_ast::ast::*;
use mago_ast::sequence::Sequence;
use mago_token::T;

use crate::error::ParseError;
use crate::internal::expression::parse_expression;
use crate::internal::function_like::parameter::parse_function_like_parameter_list;
use crate::internal::function_like::r#return::parse_optional_function_like_return_type_hint;
use crate::internal::token_stream::TokenStream;
use crate::internal::utils;

#[inline]
pub fn parse_arrow_function_with_attributes<'i>(
    stream: &mut TokenStream<'_, 'i>,
    attribute_lists: Sequence<'i, AttributeList<'i>>,
) -> Result<ArrowFunction<'i>, ParseError> {
    let r#static = utils::maybe_expect_keyword(stream, T!["static"])?;
    let r#fn = utils::expect_keyword(stream, T!["fn"])?;
    let ampersand = utils::maybe_expect(stream, T!["&"])?.map(|t| t.span);
    let parameter_list = parse_function_like_parameter_list(stream)?;
    let return_type_hint = parse_optional_function_like_return_type_hint(stream)?;
    let arrow = utils::expect_span(stream, T!["=>"])?;
    let expression = parse_expression(stream)?;

    Ok(ArrowFunction {
        attribute_lists,
        r#static,
        r#fn,
        ampersand,
        parameter_list,
        return_type_hint,
        arrow,
        expression: stream.boxed(expression),
    })
}
