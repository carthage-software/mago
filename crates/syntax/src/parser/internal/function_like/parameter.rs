use crate::T;
use crate::ast::ast::*;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::internal::attribute;
use crate::parser::internal::class_like::property;
use crate::parser::internal::expression;
use crate::parser::internal::modifier;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::type_hint;
use crate::parser::internal::utils;
use crate::parser::internal::variable;
use crate::token::Token;

pub fn parse_optional_function_like_parameter_list<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<Option<FunctionLikeParameterList<'arena>>, ParseError> {
    Ok(match utils::maybe_peek(stream)?.map(|t| t.kind) {
        Some(T!["("]) => Some(parse_function_like_parameter_list(stream)?),
        _ => None,
    })
}

pub fn parse_function_like_parameter_list<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<FunctionLikeParameterList<'arena>, ParseError> {
    Ok(FunctionLikeParameterList {
        left_parenthesis: utils::expect_span(stream, T!["("])?,
        parameters: {
            let mut parameters = stream.new_vec();
            let mut commas = stream.new_vec();
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

pub fn parse_function_like_parameter<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<FunctionLikeParameter<'arena>, ParseError> {
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

pub fn parse_optional_function_like_parameter_default_value<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<Option<FunctionLikeParameterDefaultValue<'arena>>, ParseError> {
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
