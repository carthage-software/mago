use mago_ast::ast::*;
use mago_ast::sequence::Sequence;
use mago_ast::sequence::TokenSeparatedSequence;
use mago_token::T;

use crate::error::ParseError;
use crate::internal::argument;
use crate::internal::identifier;
use crate::internal::token_stream::TokenStream;
use crate::internal::utils;

#[inline]
pub fn parse_attribute_list_sequence<'i>(
    stream: &mut TokenStream<'_, 'i>,
) -> Result<Sequence<'i, AttributeList<'i>>, ParseError> {
    let mut inner = stream.vec();
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

#[inline]
pub fn parse_attribute_list<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<AttributeList<'i>, ParseError> {
    Ok(AttributeList {
        hash_left_bracket: utils::expect_span(stream, T!["#["])?,
        attributes: {
            let mut attributes = stream.vec();
            let mut commas = stream.vec();
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

#[inline]
pub fn parse_attribute<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<Attribute<'i>, ParseError> {
    Ok(Attribute {
        name: identifier::parse_identifier(stream)?,
        arguments: argument::parse_optional_argument_list(stream)?,
    })
}
