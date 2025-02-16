use mago_ast::ast::*;
use mago_ast::sequence::TokenSeparatedSequence;
use mago_token::T;

use crate::error::ParseError;
use crate::internal::expression::parse_expression;
use crate::internal::token_stream::TokenStream;
use crate::internal::utils;

#[inline]
pub fn parse_array<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<Array<'i>, ParseError> {
    Ok(Array {
        left_bracket: utils::expect_span(stream, T!["["])?,
        elements: {
            let mut element = stream.vec();
            let mut commas = stream.vec();
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

#[inline]
pub fn parse_list<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<List<'i>, ParseError> {
    Ok(List {
        list: utils::expect_keyword(stream, T!["list"])?,
        left_parenthesis: utils::expect_span(stream, T!["("])?,
        elements: {
            let mut element = stream.vec();
            let mut commas = stream.vec();
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

#[inline]
pub fn parse_legacy_array<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<LegacyArray<'i>, ParseError> {
    Ok(LegacyArray {
        array: utils::expect_keyword(stream, T!["array"])?,
        left_parenthesis: utils::expect_span(stream, T!["("])?,
        elements: {
            let mut element = stream.vec();
            let mut commas = stream.vec();
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

#[inline]
pub fn parse_array_element<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<ArrayElement<'i>, ParseError> {
    Ok(match utils::maybe_peek(stream)?.map(|t| t.kind) {
        Some(T!["..."]) => {
            let ellipsis = utils::expect_any(stream)?.span;
            let value = parse_expression(stream)?;

            ArrayElement::Variadic(VariadicArrayElement { ellipsis, value: stream.boxed(value) })
        }
        Some(T![","]) => {
            let comma = utils::peek(stream)?.span;

            ArrayElement::Missing(MissingArrayElement { comma })
        }
        _ => {
            let expression = parse_expression(stream)?;

            match utils::maybe_peek(stream)?.map(|t| t.kind) {
                Some(T!["=>"]) => {
                    let double_arrow = utils::expect_any(stream)?.span;
                    let value = parse_expression(stream)?;

                    ArrayElement::KeyValue(KeyValueArrayElement {
                        key: stream.boxed(expression),
                        double_arrow,
                        value: stream.boxed(value),
                    })
                }
                _ => ArrayElement::Value(ValueArrayElement { value: stream.boxed(expression) }),
            }
        }
    })
}
