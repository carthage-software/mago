use crate::T;
use crate::ast::ast::*;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::internal::expression::parse_expression;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;

pub fn parse_array<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<Array<'arena>, ParseError> {
    Ok(Array {
        left_bracket: utils::expect_span(stream, T!["["])?,
        elements: {
            let mut element = stream.new_vec();
            let mut commas = stream.new_vec();
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

pub fn parse_list<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<List<'arena>, ParseError> {
    Ok(List {
        list: utils::expect_keyword(stream, T!["list"])?,
        left_parenthesis: utils::expect_span(stream, T!["("])?,
        elements: {
            let mut element = stream.new_vec();
            let mut commas = stream.new_vec();
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

pub fn parse_legacy_array<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<LegacyArray<'arena>, ParseError> {
    Ok(LegacyArray {
        array: utils::expect_keyword(stream, T!["array"])?,
        left_parenthesis: utils::expect_span(stream, T!["("])?,
        elements: {
            let mut element = stream.new_vec();
            let mut commas = stream.new_vec();
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

pub fn parse_array_element<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<ArrayElement<'arena>, ParseError> {
    Ok(match utils::maybe_peek(stream)?.map(|t| t.kind) {
        Some(T!["..."]) => {
            let ellipsis = utils::expect_any(stream)?.span;
            let value = {
                let expression = parse_expression(stream)?;

                stream.alloc(expression)
            };

            ArrayElement::Variadic(VariadicArrayElement { ellipsis, value })
        }
        Some(T![","]) => {
            let comma = utils::peek(stream)?.span;

            ArrayElement::Missing(MissingArrayElement { comma })
        }
        _ => {
            let expression = {
                let expression = parse_expression(stream)?;

                stream.alloc(expression)
            };

            match utils::maybe_peek(stream)?.map(|t| t.kind) {
                Some(T!["=>"]) => {
                    let double_arrow = utils::expect_any(stream)?.span;

                    ArrayElement::KeyValue(KeyValueArrayElement {
                        key: expression,
                        double_arrow,
                        value: {
                            let expression = parse_expression(stream)?;

                            stream.alloc(expression)
                        },
                    })
                }
                _ => ArrayElement::Value(ValueArrayElement { value: expression }),
            }
        }
    })
}
