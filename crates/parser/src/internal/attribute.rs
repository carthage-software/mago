use mago_ast::ast::*;
use mago_ast::sequence::Sequence;
use mago_ast::sequence::TokenSeparatedSequence;
use mago_token::T;

use crate::error::ParseError;
use crate::internal::argument;
use crate::internal::identifier;
use crate::internal::token_stream::TokenStream;
use crate::internal::utils;

pub fn parse_attribute_list_sequence(stream: &mut TokenStream<'_, '_>) -> Result<Sequence<AttributeList>, ParseError> {
    let mut inner = Vec::new();
    loop {
        let next = utils::peek(stream)?;
        if next.kind == T!["#["] {
            inner.push(parse_attribute_list(stream)?);
        } else {
            break;
        }
    }

    Ok(Sequence::new(inner))
}

pub fn parse_attribute_list(stream: &mut TokenStream<'_, '_>) -> Result<AttributeList, ParseError> {
    Ok(AttributeList {
        hash_left_bracket: utils::expect_span(stream, T!["#["])?,
        attributes: {
            let mut attributes = Vec::new();
            let mut commas = Vec::new();
            loop {
                let next = utils::peek(stream)?;
                if next.kind == T!["]"] {
                    break;
                }

                attributes.push(parse_attribute(stream)?);

                let next = utils::peek(stream)?;
                if next.kind == T![","] {
                    commas.push(utils::expect_any(stream)?);
                } else {
                    break;
                }
            }

            TokenSeparatedSequence::new(attributes, commas)
        },
        right_bracket: utils::expect_span(stream, T!["]"])?,
    })
}

pub fn parse_attribute(stream: &mut TokenStream<'_, '_>) -> Result<Attribute, ParseError> {
    Ok(Attribute {
        name: identifier::parse_identifier(stream)?,
        arguments: argument::parse_optional_argument_list(stream)?,
    })
}
