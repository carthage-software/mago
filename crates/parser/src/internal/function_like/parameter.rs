use mago_ast::ast::*;
use mago_ast::sequence::TokenSeparatedSequence;
use mago_token::Token;
use mago_token::T;

use crate::error::ParseError;
use crate::internal::attribute;
use crate::internal::class_like::property;
use crate::internal::expression;
use crate::internal::modifier;
use crate::internal::token_stream::TokenStream;
use crate::internal::type_hint;
use crate::internal::utils;
use crate::internal::variable;

#[inline]
pub fn parse_optional_function_like_parameter_list<'i>(
    stream: &mut TokenStream<'_, 'i>,
) -> Result<Option<FunctionLikeParameterList<'i>>, ParseError> {
    Ok(match utils::maybe_peek(stream)?.map(|t| t.kind) {
        Some(T!["("]) => Some(parse_function_like_parameter_list(stream)?),
        _ => None,
    })
}

#[inline]
pub fn parse_function_like_parameter_list<'i>(
    stream: &mut TokenStream<'_, 'i>,
) -> Result<FunctionLikeParameterList<'i>, ParseError> {
    Ok(FunctionLikeParameterList {
        left_parenthesis: utils::expect_span(stream, T!["("])?,
        parameters: {
            let mut parameters = stream.vec();
            let mut commas = stream.vec();
            loop {
                let token = utils::peek(stream)?;
                if T![")"] == token.kind {
                    break;
                }

                let parameter = parse_function_like_parameter(stream)?;
                parameters.push(parameter);

                match utils::maybe_expect(stream, T![","])? {
                    Some(comma) => {
                        commas.push(comma);
                    }
                    None => break,
                }
            }

            TokenSeparatedSequence::new(parameters, commas)
        },
        right_parenthesis: utils::expect_span(stream, T![")"])?,
    })
}

#[inline]
pub fn parse_function_like_parameter<'i>(
    stream: &mut TokenStream<'_, 'i>,
) -> Result<FunctionLikeParameter<'i>, ParseError> {
    Ok(FunctionLikeParameter {
        attribute_lists: attribute::parse_attribute_list_sequence(stream)?,
        modifiers: modifier::parse_modifier_sequence(stream)?,
        hint: type_hint::parse_optional_type_hint(stream)?,
        ampersand: utils::maybe_expect(stream, T!["&"])?.map(|token| token.span),
        ellipsis: utils::maybe_expect(stream, T!["..."])?.map(|token| token.span),
        variable: variable::parse_direct_variable(stream)?,
        default_value: parse_optional_function_like_parameter_default_value(stream)?,
        hooks: property::parse_optional_property_hook_list(stream)?,
    })
}

#[inline]
pub fn parse_optional_function_like_parameter_default_value<'i>(
    stream: &mut TokenStream<'_, 'i>,
) -> Result<Option<FunctionLikeParameterDefaultValue<'i>>, ParseError> {
    let token = utils::maybe_peek(stream)?;
    if let Some(Token { kind: T!["="], .. }) = token {
        let equals = utils::expect_any(stream)?.span;
        let value = expression::parse_expression(stream)?;

        Ok(Some(FunctionLikeParameterDefaultValue { equals, value: stream.boxed(value) }))
    } else {
        Ok(None)
    }
}
