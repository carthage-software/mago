use crate::T;
use crate::ast::ast::*;
use crate::ast::sequence::Sequence;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::internal::argument;
use crate::parser::internal::identifier;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;

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
