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

pub fn parse_optional_function_like_parameter_list(
    stream: &mut TokenStream<'_, '_>,
) -> Result<Option<FunctionLikeParameterList>, ParseError> {
    Ok(match utils::maybe_peek(stream)?.map(|t| t.kind) {
        Some(T!["("]) => Some(parse_function_like_parameter_list(stream)?),
        _ => None,
    })
}

pub fn parse_function_like_parameter_list(
    stream: &mut TokenStream<'_, '_>,
) -> Result<FunctionLikeParameterList, ParseError> {
    Ok(FunctionLikeParameterList {
        left_parenthesis: utils::expect_span(stream, T!["("])?,
        parameters: {
            let mut parameters = Vec::new();
            let mut commas = Vec::new();
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

pub fn parse_function_like_parameter(stream: &mut TokenStream<'_, '_>) -> Result<FunctionLikeParameter, ParseError> {
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

pub fn parse_optional_function_like_parameter_default_value(
    stream: &mut TokenStream<'_, '_>,
) -> Result<Option<FunctionLikeParameterDefaultValue>, ParseError> {
    let token = utils::maybe_peek(stream)?;
    if let Some(Token { kind: T!["="], .. }) = token {
        Ok(Some(FunctionLikeParameterDefaultValue {
            equals: utils::expect_any(stream)?.span,
            value: expression::parse_expression(stream)?,
        }))
    } else {
        Ok(None)
    }
}
