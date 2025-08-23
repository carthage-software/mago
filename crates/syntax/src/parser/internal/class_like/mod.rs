use crate::T;
use crate::ast::ast::*;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::internal::argument::parse_optional_argument_list;
use crate::parser::internal::attribute::parse_attribute_list_sequence;
use crate::parser::internal::class_like::inheritance::parse_optional_extends;
use crate::parser::internal::class_like::inheritance::parse_optional_implements;
use crate::parser::internal::class_like::member::parse_classlike_member;
use crate::parser::internal::identifier::parse_local_identifier;
use crate::parser::internal::modifier::parse_modifier_sequence;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::type_hint::parse_type_hint;
use crate::parser::internal::utils;

pub mod constant;
pub mod enum_case;
pub mod inheritance;
pub mod member;
pub mod method;
pub mod property;
pub mod trait_use;

pub fn parse_interface_with_attributes<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
    attributes: Sequence<'arena, AttributeList<'arena>>,
) -> Result<Interface<'arena>, ParseError> {
    Ok(Interface {
        attribute_lists: attributes,
        interface: utils::expect_keyword(stream, T!["interface"])?,
        name: parse_local_identifier(stream)?,
        extends: parse_optional_extends(stream)?,
        left_brace: utils::expect_span(stream, T!["{"])?,
        members: {
            let mut members = stream.new_vec();
            loop {
                if matches!(utils::maybe_peek(stream)?.map(|t| t.kind), Some(T!["}"])) {
                    break;
                }

                members.push(parse_classlike_member(stream)?);
            }

            Sequence::new(members)
        },
        right_brace: utils::expect_span(stream, T!["}"])?,
    })
}

pub fn parse_class_with_attributes<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
    attributes: Sequence<'arena, AttributeList<'arena>>,
) -> Result<Class<'arena>, ParseError> {
    let modifiers = parse_modifier_sequence(stream)?;

    parse_class_with_attributes_and_modifiers(stream, attributes, modifiers)
}

pub fn parse_class_with_attributes_and_modifiers<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
    attributes: Sequence<'arena, AttributeList<'arena>>,
    modifiers: Sequence<'arena, Modifier<'arena>>,
) -> Result<Class<'arena>, ParseError> {
    Ok(Class {
        attribute_lists: attributes,
        modifiers,
        class: utils::expect_keyword(stream, T!["class"])?,
        name: parse_local_identifier(stream)?,
        extends: parse_optional_extends(stream)?,
        implements: parse_optional_implements(stream)?,
        left_brace: utils::expect_span(stream, T!["{"])?,
        members: {
            let mut members = stream.new_vec();
            loop {
                if matches!(utils::maybe_peek(stream)?.map(|t| t.kind), Some(T!["}"])) {
                    break;
                }

                members.push(parse_classlike_member(stream)?);
            }

            Sequence::new(members)
        },
        right_brace: utils::expect_span(stream, T!["}"])?,
    })
}

pub fn parse_anonymous_class<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<AnonymousClass<'arena>, ParseError> {
    Ok(AnonymousClass {
        new: utils::expect_keyword(stream, T!["new"])?,
        attribute_lists: parse_attribute_list_sequence(stream)?,
        modifiers: parse_modifier_sequence(stream)?,
        class: utils::expect_keyword(stream, T!["class"])?,
        argument_list: parse_optional_argument_list(stream)?,
        extends: parse_optional_extends(stream)?,
        implements: parse_optional_implements(stream)?,
        left_brace: utils::expect_span(stream, T!["{"])?,
        members: {
            let mut members = stream.new_vec();
            loop {
                if matches!(utils::maybe_peek(stream)?.map(|t| t.kind), Some(T!["}"])) {
                    break;
                }

                members.push(parse_classlike_member(stream)?);
            }

            Sequence::new(members)
        },
        right_brace: utils::expect_span(stream, T!["}"])?,
    })
}

pub fn parse_trait_with_attributes<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
    attributes: Sequence<'arena, AttributeList<'arena>>,
) -> Result<Trait<'arena>, ParseError> {
    Ok(Trait {
        attribute_lists: attributes,
        r#trait: utils::expect_keyword(stream, T!["trait"])?,
        name: parse_local_identifier(stream)?,
        left_brace: utils::expect_span(stream, T!["{"])?,
        members: {
            let mut members = stream.new_vec();
            loop {
                if matches!(utils::maybe_peek(stream)?.map(|t| t.kind), Some(T!["}"])) {
                    break;
                }

                members.push(parse_classlike_member(stream)?);
            }
            Sequence::new(members)
        },
        right_brace: utils::expect_span(stream, T!["}"])?,
    })
}

pub fn parse_enum_with_attributes<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
    attributes: Sequence<'arena, AttributeList<'arena>>,
) -> Result<Enum<'arena>, ParseError> {
    Ok(Enum {
        attribute_lists: attributes,
        r#enum: utils::expect_keyword(stream, T!["enum"])?,
        name: parse_local_identifier(stream)?,
        backing_type_hint: parse_optional_enum_backing_type_hint(stream)?,
        implements: parse_optional_implements(stream)?,
        left_brace: utils::expect_span(stream, T!["{"])?,
        members: {
            let mut members = stream.new_vec();
            loop {
                if matches!(utils::maybe_peek(stream)?.map(|t| t.kind), Some(T!["}"])) {
                    break;
                }

                members.push(parse_classlike_member(stream)?);
            }
            Sequence::new(members)
        },
        right_brace: utils::expect_span(stream, T!["}"])?,
    })
}

pub fn parse_optional_enum_backing_type_hint<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<Option<EnumBackingTypeHint<'arena>>, ParseError> {
    Ok(match utils::maybe_peek(stream)?.map(|t| t.kind) {
        Some(T![":"]) => {
            Some(EnumBackingTypeHint { colon: utils::expect_any(stream)?.span, hint: parse_type_hint(stream)? })
        }
        _ => None,
    })
}
