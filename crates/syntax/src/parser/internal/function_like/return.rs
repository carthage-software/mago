use crate::T;
use crate::ast::ast::*;
use crate::error::ParseError;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::type_hint;
use crate::parser::internal::utils;

pub fn parse_optional_function_like_return_type_hint(
    stream: &mut TokenStream<'_, '_>,
) -> Result<Option<FunctionLikeReturnTypeHint>, ParseError> {
    Ok(match utils::maybe_peek(stream)?.map(|t| t.kind) {
        Some(T![":"]) => Some(parse_function_like_return_type_hint(stream)?),
        _ => None,
    })
}

pub fn parse_function_like_return_type_hint(
    stream: &mut TokenStream<'_, '_>,
) -> Result<FunctionLikeReturnTypeHint, ParseError> {
    Ok(FunctionLikeReturnTypeHint {
        colon: utils::expect_span(stream, T![":"])?,
        hint: type_hint::parse_type_hint(stream)?,
    })
}
