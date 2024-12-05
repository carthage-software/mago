use fennec_ast::ast::*;
use fennec_ast::sequence::TokenSeparatedSequence;
use fennec_token::T;

use crate::error::ParseError;
use crate::internal::expression::parse_expression;
use crate::internal::token_stream::TokenStream;
use crate::internal::utils;

pub fn parse_array<'a, 'i>(stream: &mut TokenStream<'a, 'i>) -> Result<Array, ParseError> {
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

pub fn parse_list<'a, 'i>(stream: &mut TokenStream<'a, 'i>) -> Result<List, ParseError> {
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

pub fn parse_legacy_array<'a, 'i>(stream: &mut TokenStream<'a, 'i>) -> Result<LegacyArray, ParseError> {
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

pub fn parse_array_element<'a, 'i>(stream: &mut TokenStream<'a, 'i>) -> Result<ArrayElement, ParseError> {
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
