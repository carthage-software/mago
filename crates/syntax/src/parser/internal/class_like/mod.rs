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

pub mod constant;
pub mod enum_case;
pub mod inheritance;
pub mod member;
pub mod method;
pub mod property;
pub mod trait_use;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_interface_with_attributes(
        &mut self,
        attributes: Sequence<'arena, AttributeList<'arena>>,
    ) -> Result<Interface<'arena>, ParseError> {
        Ok(Interface {
            attribute_lists: attributes,
            interface: self.expect_keyword(T!["interface"])?,
            name: self.parse_local_identifier()?,
            extends: self.parse_optional_extends()?,
            left_brace: self.stream.eat(T!["{"])?.span,
            members: {
                let mut members = self.new_vec();
                loop {
                    if matches!(self.stream.lookahead(0)?.map(|t| t.kind), Some(T!["}"])) {
                        break;
                    }

                    let position_before = self.stream.current_position();
                    match self.parse_classlike_member() {
                        Ok(member) => members.push(member),
                        Err(err) => self.errors.push(err),
                    }
                    if self.stream.current_position() == position_before {
                        if let Ok(Some(token)) = self.stream.lookahead(0) {
                            if token.kind == T!["}"] {
                                break;
                            }
                            self.errors.push(self.stream.unexpected(Some(token), &[]));
                            let _ = self.stream.consume();
                        } else {
                            break;
                        }
                    }
                }

                Sequence::new(members)
            },
            right_brace: self.stream.eat(T!["}"])?.span,
        })
    }

    pub(crate) fn parse_class_with_attributes(
        &mut self,
        attributes: Sequence<'arena, AttributeList<'arena>>,
    ) -> Result<Class<'arena>, ParseError> {
        let modifiers = self.parse_modifier_sequence()?;

        self.parse_class_with_attributes_and_modifiers(attributes, modifiers)
    }

    fn parse_class_with_attributes_and_modifiers(
        &mut self,
        attributes: Sequence<'arena, AttributeList<'arena>>,
        modifiers: Sequence<'arena, Modifier<'arena>>,
    ) -> Result<Class<'arena>, ParseError> {
        Ok(Class {
            attribute_lists: attributes,
            modifiers,
            class: self.expect_keyword(T!["class"])?,
            name: self.parse_local_identifier()?,
            extends: self.parse_optional_extends()?,
            implements: self.parse_optional_implements()?,
            left_brace: self.stream.eat(T!["{"])?.span,
            members: {
                let mut members = self.new_vec();
                loop {
                    if matches!(self.stream.lookahead(0)?.map(|t| t.kind), Some(T!["}"])) {
                        break;
                    }

                    let position_before = self.stream.current_position();
                    match self.parse_classlike_member() {
                        Ok(member) => members.push(member),
                        Err(err) => self.errors.push(err),
                    }
                    if self.stream.current_position() == position_before {
                        if let Ok(Some(token)) = self.stream.lookahead(0) {
                            if token.kind == T!["}"] {
                                break;
                            }
                            self.errors.push(self.stream.unexpected(Some(token), &[]));
                            let _ = self.stream.consume();
                        } else {
                            break;
                        }
                    }
                }

                Sequence::new(members)
            },
            right_brace: self.stream.eat(T!["}"])?.span,
        })
    }

    pub(crate) fn parse_anonymous_class(&mut self) -> Result<AnonymousClass<'arena>, ParseError> {
        Ok(AnonymousClass {
            new: self.expect_keyword(T!["new"])?,
            attribute_lists: self.parse_attribute_list_sequence()?,
            modifiers: self.parse_modifier_sequence()?,
            class: self.expect_keyword(T!["class"])?,
            argument_list: self.parse_optional_argument_list()?,
            extends: self.parse_optional_extends()?,
            implements: self.parse_optional_implements()?,
            left_brace: self.stream.eat(T!["{"])?.span,
            members: {
                let mut members = self.new_vec();
                loop {
                    if matches!(self.stream.lookahead(0)?.map(|t| t.kind), Some(T!["}"])) {
                        break;
                    }

                    let position_before = self.stream.current_position();
                    match self.parse_classlike_member() {
                        Ok(member) => members.push(member),
                        Err(err) => self.errors.push(err),
                    }
                    if self.stream.current_position() == position_before {
                        if let Ok(Some(token)) = self.stream.lookahead(0) {
                            if token.kind == T!["}"] {
                                break;
                            }
                            self.errors.push(self.stream.unexpected(Some(token), &[]));
                            let _ = self.stream.consume();
                        } else {
                            break;
                        }
                    }
                }

                Sequence::new(members)
            },
            right_brace: self.stream.eat(T!["}"])?.span,
        })
    }

    pub(crate) fn parse_trait_with_attributes(
        &mut self,
        attributes: Sequence<'arena, AttributeList<'arena>>,
    ) -> Result<Trait<'arena>, ParseError> {
        Ok(Trait {
            attribute_lists: attributes,
            r#trait: self.expect_keyword(T!["trait"])?,
            name: self.parse_local_identifier()?,
            left_brace: self.stream.eat(T!["{"])?.span,
            members: {
                let mut members = self.new_vec();
                loop {
                    if matches!(self.stream.lookahead(0)?.map(|t| t.kind), Some(T!["}"])) {
                        break;
                    }

                    let position_before = self.stream.current_position();
                    match self.parse_classlike_member() {
                        Ok(member) => members.push(member),
                        Err(err) => self.errors.push(err),
                    }
                    if self.stream.current_position() == position_before {
                        if let Ok(Some(token)) = self.stream.lookahead(0) {
                            if token.kind == T!["}"] {
                                break;
                            }
                            self.errors.push(self.stream.unexpected(Some(token), &[]));
                            let _ = self.stream.consume();
                        } else {
                            break;
                        }
                    }
                }
                Sequence::new(members)
            },
            right_brace: self.stream.eat(T!["}"])?.span,
        })
    }

    pub(crate) fn parse_enum_with_attributes(
        &mut self,
        attributes: Sequence<'arena, AttributeList<'arena>>,
    ) -> Result<Enum<'arena>, ParseError> {
        Ok(Enum {
            attribute_lists: attributes,
            r#enum: self.expect_keyword(T!["enum"])?,
            name: self.parse_local_identifier()?,
            backing_type_hint: self.parse_optional_enum_backing_type_hint()?,
            implements: self.parse_optional_implements()?,
            left_brace: self.stream.eat(T!["{"])?.span,
            members: {
                let mut members = self.new_vec();
                loop {
                    if matches!(self.stream.lookahead(0)?.map(|t| t.kind), Some(T!["}"])) {
                        break;
                    }

                    let position_before = self.stream.current_position();
                    match self.parse_classlike_member() {
                        Ok(member) => members.push(member),
                        Err(err) => self.errors.push(err),
                    }
                    if self.stream.current_position() == position_before {
                        if let Ok(Some(token)) = self.stream.lookahead(0) {
                            if token.kind == T!["}"] {
                                break;
                            }
                            self.errors.push(self.stream.unexpected(Some(token), &[]));
                            let _ = self.stream.consume();
                        } else {
                            break;
                        }
                    }
                }
                Sequence::new(members)
            },
            right_brace: self.stream.eat(T!["}"])?.span,
        })
    }

    fn parse_optional_enum_backing_type_hint(&mut self) -> Result<Option<EnumBackingTypeHint<'arena>>, ParseError> {
        Ok(match self.stream.lookahead(0)?.map(|t| t.kind) {
            Some(T![":"]) => {
                Some(EnumBackingTypeHint { colon: self.stream.consume()?.span, hint: self.parse_type_hint()? })
            }
            _ => None,
        })
    }
}
