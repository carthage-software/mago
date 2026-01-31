use crate::T;
use crate::ast::ast::AnonymousClass;
use crate::ast::ast::AttributeList;
use crate::ast::ast::Class;
use crate::ast::ast::Enum;
use crate::ast::ast::EnumBackingTypeHint;
use crate::ast::ast::Interface;
use crate::ast::ast::Modifier;
use crate::ast::ast::Trait;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;

pub mod constant;
pub mod enum_case;
pub mod inheritance;
pub mod member;
pub mod method;
pub mod property;
pub mod trait_use;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_interface_with_attributes(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
        attributes: Sequence<'arena, AttributeList<'arena>>,
    ) -> Result<Interface<'arena>, ParseError> {
        Ok(Interface {
            attribute_lists: attributes,
            interface: self.expect_keyword(stream, T!["interface"])?,
            name: self.parse_local_identifier(stream)?,
            extends: self.parse_optional_extends(stream)?,
            left_brace: stream.eat(T!["{"])?.span,
            members: {
                let mut members = self.new_vec();
                loop {
                    if matches!(stream.lookahead(0)?.map(|t| t.kind), Some(T!["}"])) {
                        break;
                    }

                    members.push(self.parse_classlike_member(stream)?);
                }

                Sequence::new(members)
            },
            right_brace: stream.eat(T!["}"])?.span,
        })
    }

    pub(crate) fn parse_class_with_attributes(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
        attributes: Sequence<'arena, AttributeList<'arena>>,
    ) -> Result<Class<'arena>, ParseError> {
        let modifiers = self.parse_modifier_sequence(stream)?;

        self.parse_class_with_attributes_and_modifiers(stream, attributes, modifiers)
    }

    fn parse_class_with_attributes_and_modifiers(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
        attributes: Sequence<'arena, AttributeList<'arena>>,
        modifiers: Sequence<'arena, Modifier<'arena>>,
    ) -> Result<Class<'arena>, ParseError> {
        Ok(Class {
            attribute_lists: attributes,
            modifiers,
            class: self.expect_keyword(stream, T!["class"])?,
            name: self.parse_local_identifier(stream)?,
            extends: self.parse_optional_extends(stream)?,
            implements: self.parse_optional_implements(stream)?,
            left_brace: stream.eat(T!["{"])?.span,
            members: {
                let mut members = self.new_vec();
                loop {
                    if matches!(stream.lookahead(0)?.map(|t| t.kind), Some(T!["}"])) {
                        break;
                    }

                    members.push(self.parse_classlike_member(stream)?);
                }

                Sequence::new(members)
            },
            right_brace: stream.eat(T!["}"])?.span,
        })
    }

    pub(crate) fn parse_anonymous_class(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<AnonymousClass<'arena>, ParseError> {
        Ok(AnonymousClass {
            new: self.expect_keyword(stream, T!["new"])?,
            attribute_lists: self.parse_attribute_list_sequence(stream)?,
            modifiers: self.parse_modifier_sequence(stream)?,
            class: self.expect_keyword(stream, T!["class"])?,
            argument_list: self.parse_optional_argument_list(stream)?,
            extends: self.parse_optional_extends(stream)?,
            implements: self.parse_optional_implements(stream)?,
            left_brace: stream.eat(T!["{"])?.span,
            members: {
                let mut members = self.new_vec();
                loop {
                    if matches!(stream.lookahead(0)?.map(|t| t.kind), Some(T!["}"])) {
                        break;
                    }

                    members.push(self.parse_classlike_member(stream)?);
                }

                Sequence::new(members)
            },
            right_brace: stream.eat(T!["}"])?.span,
        })
    }

    pub(crate) fn parse_trait_with_attributes(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
        attributes: Sequence<'arena, AttributeList<'arena>>,
    ) -> Result<Trait<'arena>, ParseError> {
        Ok(Trait {
            attribute_lists: attributes,
            r#trait: self.expect_keyword(stream, T!["trait"])?,
            name: self.parse_local_identifier(stream)?,
            left_brace: stream.eat(T!["{"])?.span,
            members: {
                let mut members = self.new_vec();
                loop {
                    if matches!(stream.lookahead(0)?.map(|t| t.kind), Some(T!["}"])) {
                        break;
                    }

                    members.push(self.parse_classlike_member(stream)?);
                }
                Sequence::new(members)
            },
            right_brace: stream.eat(T!["}"])?.span,
        })
    }

    pub(crate) fn parse_enum_with_attributes(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
        attributes: Sequence<'arena, AttributeList<'arena>>,
    ) -> Result<Enum<'arena>, ParseError> {
        Ok(Enum {
            attribute_lists: attributes,
            r#enum: self.expect_keyword(stream, T!["enum"])?,
            name: self.parse_local_identifier(stream)?,
            backing_type_hint: self.parse_optional_enum_backing_type_hint(stream)?,
            implements: self.parse_optional_implements(stream)?,
            left_brace: stream.eat(T!["{"])?.span,
            members: {
                let mut members = self.new_vec();
                loop {
                    if matches!(stream.lookahead(0)?.map(|t| t.kind), Some(T!["}"])) {
                        break;
                    }

                    members.push(self.parse_classlike_member(stream)?);
                }
                Sequence::new(members)
            },
            right_brace: stream.eat(T!["}"])?.span,
        })
    }

    fn parse_optional_enum_backing_type_hint(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<Option<EnumBackingTypeHint<'arena>>, ParseError> {
        Ok(match stream.lookahead(0)?.map(|t| t.kind) {
            Some(T![":"]) => {
                Some(EnumBackingTypeHint { colon: stream.consume()?.span, hint: self.parse_type_hint(stream)? })
            }
            _ => None,
        })
    }
}
