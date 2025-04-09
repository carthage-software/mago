use crate::T;
use crate::ast::ast::*;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::internal::expression::parse_expression;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;

pub fn parse_array(stream: &mut TokenStream<'_, '_>) -> Result<Array, ParseError> {
    Ok(Array {
        left_bracket: utils::expect_span(stream, T!["["])?,
        elements: {
            let mut element = Vec::new();
            let mut commas = Vec::new();
            loop {
                let next = utils::peek(stream)?;
                if next.kind == T!["]"] {
                    break;
                }

                element.push(parse_array_element(stream)?);

                let next = utils::peek(stream)?;
                if next.kind == T![","] {
                    commas.push(utils::expect_any(stream)?);
                } else {
                    break;
                }
            }

            TokenSeparatedSequence::new(element, commas)
        },
        right_bracket: utils::expect_span(stream, T!["]"])?,
    })
}

pub fn parse_list(stream: &mut TokenStream<'_, '_>) -> Result<List, ParseError> {
    Ok(List {
        list: utils::expect_keyword(stream, T!["list"])?,
        left_parenthesis: utils::expect_span(stream, T!["("])?,
        elements: {
            let mut element = Vec::new();
            let mut commas = Vec::new();
            loop {
                let next = utils::peek(stream)?;
                if next.kind == T![")"] {
                    break;
                }

                element.push(parse_array_element(stream)?);

                let next = utils::peek(stream)?;
                if next.kind == T![","] {
                    commas.push(utils::expect_any(stream)?);
                } else {
                    break;
                }
            }
            TokenSeparatedSequence::new(element, commas)
        },
        right_parenthesis: utils::expect_span(stream, T![")"])?,
    })
}

pub fn parse_legacy_array(stream: &mut TokenStream<'_, '_>) -> Result<LegacyArray, ParseError> {
    Ok(LegacyArray {
        array: utils::expect_keyword(stream, T!["array"])?,
        left_parenthesis: utils::expect_span(stream, T!["("])?,
        elements: {
            let mut element = Vec::new();
            let mut commas = Vec::new();
            loop {
                let next = utils::peek(stream)?;
                if next.kind == T![")"] {
                    break;
                }

                element.push(parse_array_element(stream)?);

                let next = utils::peek(stream)?;
                if next.kind == T![","] {
                    commas.push(utils::expect_any(stream)?);
                } else {
                    break;
                }
            }
            TokenSeparatedSequence::new(element, commas)
        },
        right_parenthesis: utils::expect_span(stream, T![")"])?,
    })
}

pub fn parse_array_element(stream: &mut TokenStream<'_, '_>) -> Result<ArrayElement, ParseError> {
    Ok(match utils::maybe_peek(stream)?.map(|t| t.kind) {
        Some(T!["..."]) => {
            let ellipsis = utils::expect_any(stream)?.span;
            let value = Box::new(parse_expression(stream)?);

            ArrayElement::Variadic(VariadicArrayElement { ellipsis, value })
        }
        Some(T![","]) => {
            let comma = utils::peek(stream)?.span;

            ArrayElement::Missing(MissingArrayElement { comma })
        }
        _ => {
            let expression = Box::new(parse_expression(stream)?);

            match utils::maybe_peek(stream)?.map(|t| t.kind) {
                Some(T!["=>"]) => {
                    let double_arrow = utils::expect_any(stream)?.span;

                    ArrayElement::KeyValue(KeyValueArrayElement {
                        key: expression,
                        double_arrow,
                        value: Box::new(parse_expression(stream)?),
                    })
                }
                _ => ArrayElement::Value(ValueArrayElement { value: expression }),
            }
        }
    })
}
