use mago_ast::ast::*;
use mago_ast::sequence::Sequence;
use mago_token::T;

use crate::error::ParseError;
use crate::internal::argument::parse_optional_argument_list;
use crate::internal::attribute::parse_attribute_list_sequence;
use crate::internal::class_like::inheritance::parse_optional_extends;
use crate::internal::class_like::inheritance::parse_optional_implements;
use crate::internal::class_like::member::parse_classlike_memeber;
use crate::internal::identifier::parse_local_identifier;
use crate::internal::modifier::parse_modifier_sequence;
use crate::internal::token_stream::TokenStream;
use crate::internal::type_hint::parse_type_hint;
use crate::internal::utils;

pub mod constant;
pub mod enum_case;
pub mod inheritance;
pub mod member;
pub mod method;
pub mod property;
pub mod trait_use;

#[inline]
pub fn parse_interface_with_attributes<'i>(
    stream: &mut TokenStream<'_, 'i>,
    attributes: Sequence<'i, AttributeList<'i>>,
) -> Result<Interface<'i>, ParseError> {
    Ok(Interface {
        attribute_lists: attributes,
        interface: utils::expect_keyword(stream, T!["interface"])?,
        name: parse_local_identifier(stream)?,
        extends: parse_optional_extends(stream)?,
        left_brace: utils::expect_span(stream, T!["{"])?,
        members: {
            let mut members = stream.vec();
            loop {
                if matches!(utils::maybe_peek(stream)?.map(|t| t.kind), Some(T!["}"])) {
                    break;
                }

                members.push(parse_classlike_memeber(stream)?);
            }

            Sequence::new(members)
        },
        right_brace: utils::expect_span(stream, T!["}"])?,
    })
}

#[inline]
pub fn parse_class_with_attributes<'i>(
    stream: &mut TokenStream<'_, 'i>,
    attributes: Sequence<'i, AttributeList<'i>>,
) -> Result<Class<'i>, ParseError> {
    let modifiers = parse_modifier_sequence(stream)?;

    parse_class_with_attributes_and_modifiers(stream, attributes, modifiers)
}

#[inline]
pub fn parse_class_with_attributes_and_modifiers<'i>(
    stream: &mut TokenStream<'_, 'i>,
    attributes: Sequence<'i, AttributeList<'i>>,
    modifiers: Sequence<'i, Modifier>,
) -> Result<Class<'i>, ParseError> {
    Ok(Class {
        attribute_lists: attributes,
        modifiers,
        class: utils::expect_keyword(stream, T!["class"])?,
        name: parse_local_identifier(stream)?,
        extends: parse_optional_extends(stream)?,
        implements: parse_optional_implements(stream)?,
        left_brace: utils::expect_span(stream, T!["{"])?,
        members: {
            let mut members = stream.vec();
            loop {
                if matches!(utils::maybe_peek(stream)?.map(|t| t.kind), Some(T!["}"])) {
                    break;
                }

                members.push(parse_classlike_memeber(stream)?);
            }

            Sequence::new(members)
        },
        right_brace: utils::expect_span(stream, T!["}"])?,
    })
}

#[inline]
pub fn parse_anonymous_class<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<AnonymousClass<'i>, ParseError> {
    Ok(AnonymousClass {
        new: utils::expect_keyword(stream, T!["new"])?,
        attribute_lists: parse_attribute_list_sequence(stream)?,
        modifiers: parse_modifier_sequence(stream)?,
        class: utils::expect_keyword(stream, T!["class"])?,
        arguments: parse_optional_argument_list(stream)?,
        extends: parse_optional_extends(stream)?,
        implements: parse_optional_implements(stream)?,
        left_brace: utils::expect_span(stream, T!["{"])?,
        members: {
            let mut members = stream.vec();
            loop {
                if matches!(utils::maybe_peek(stream)?.map(|t| t.kind), Some(T!["}"])) {
                    break;
                }

                members.push(parse_classlike_memeber(stream)?);
            }

            Sequence::new(members)
        },
        right_brace: utils::expect_span(stream, T!["}"])?,
    })
}

#[inline]
pub fn parse_trait_with_attributes<'i>(
    stream: &mut TokenStream<'_, 'i>,
    attributes: Sequence<'i, AttributeList<'i>>,
) -> Result<Trait<'i>, ParseError> {
    Ok(Trait {
        attribute_lists: attributes,
        r#trait: utils::expect_keyword(stream, T!["trait"])?,
        name: parse_local_identifier(stream)?,
        left_brace: utils::expect_span(stream, T!["{"])?,
        members: {
            let mut members = stream.vec();
            loop {
                if matches!(utils::maybe_peek(stream)?.map(|t| t.kind), Some(T!["}"])) {
                    break;
                }

                members.push(parse_classlike_memeber(stream)?);
            }
            Sequence::new(members)
        },
        right_brace: utils::expect_span(stream, T!["}"])?,
    })
}

#[inline]
pub fn parse_enum_with_attributes<'i>(
    stream: &mut TokenStream<'_, 'i>,
    attributes: Sequence<'i, AttributeList<'i>>,
) -> Result<Enum<'i>, ParseError> {
    Ok(Enum {
        attribute_lists: attributes,
        r#enum: utils::expect_keyword(stream, T!["enum"])?,
        name: parse_local_identifier(stream)?,
        backing_type_hint: parse_optional_enum_backing_type_hint(stream)?,
        implements: parse_optional_implements(stream)?,
        left_brace: utils::expect_span(stream, T!["{"])?,
        members: {
            let mut members = stream.vec();
            loop {
                if matches!(utils::maybe_peek(stream)?.map(|t| t.kind), Some(T!["}"])) {
                    break;
                }

                members.push(parse_classlike_memeber(stream)?);
            }
            Sequence::new(members)
        },
        right_brace: utils::expect_span(stream, T!["}"])?,
    })
}

#[inline]
pub fn parse_optional_enum_backing_type_hint<'i>(
    stream: &mut TokenStream<'_, 'i>,
) -> Result<Option<EnumBackingTypeHint<'i>>, ParseError> {
    Ok(match utils::maybe_peek(stream)?.map(|t| t.kind) {
        Some(T![":"]) => {
            Some(EnumBackingTypeHint { colon: utils::expect_any(stream)?.span, hint: parse_type_hint(stream)? })
        }
        _ => None,
    })
}
