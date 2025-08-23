use crate::T;
use crate::ast::ast::*;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::internal::expression::parse_expression;
use crate::parser::internal::function_like::parameter::parse_function_like_parameter_list;
use crate::parser::internal::function_like::r#return::parse_optional_function_like_return_type_hint;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;

pub fn parse_arrow_function_with_attributes<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
    attributes: Sequence<'arena, AttributeList<'arena>>,
) -> Result<ArrowFunction<'arena>, ParseError> {
    Ok(ArrowFunction {
        attribute_lists: attributes,
        r#static: utils::maybe_expect_keyword(stream, T!["static"])?,
        r#fn: utils::expect_keyword(stream, T!["fn"])?,
        ampersand: utils::maybe_expect(stream, T!["&"])?.map(|t| t.span),
        parameter_list: parse_function_like_parameter_list(stream)?,
        return_type_hint: parse_optional_function_like_return_type_hint(stream)?,
        arrow: utils::expect_span(stream, T!["=>"])?,
        expression: {
            let expression = parse_expression(stream)?;

            stream.alloc(expression)
        },
    })
}
