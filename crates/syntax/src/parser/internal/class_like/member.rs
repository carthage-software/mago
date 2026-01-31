use either::Either;

use crate::T;
use crate::ast::ast::AttributeList;
use crate::ast::ast::ClassLikeConstantSelector;
use crate::ast::ast::ClassLikeMember;
use crate::ast::ast::ClassLikeMemberExpressionSelector;
use crate::ast::ast::ClassLikeMemberSelector;
use crate::ast::ast::Modifier;
use crate::ast::ast::Variable;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_classlike_member(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<ClassLikeMember<'arena>, ParseError> {
        let token = stream.lookahead(0)?.ok_or_else(|| stream.unexpected(None, &[]))?;
        Ok(match token.kind {
            T!["#["] => {
                let attributes = self.parse_attribute_list_sequence(stream)?;

                self.parse_classlike_member_with_attributes(stream, attributes)?
            }
            k if k.is_modifier() => {
                let modifiers = self.parse_modifier_sequence(stream)?;

                self.parse_classlike_member_with_attributes_and_modifiers(
                    stream,
                    Sequence::empty(self.arena),
                    modifiers,
                )?
            }
            T!["const"] => ClassLikeMember::Constant(self.parse_class_like_constant_with_attributes_and_modifiers(
                stream,
                Sequence::empty(self.arena),
                Sequence::empty(self.arena),
            )?),
            T!["function"] => ClassLikeMember::Method(self.parse_method_with_attributes_and_modifiers(
                stream,
                Sequence::empty(self.arena),
                Sequence::empty(self.arena),
            )?),
            T!["case"] => {
                ClassLikeMember::EnumCase(self.parse_enum_case_with_attributes(stream, Sequence::empty(self.arena))?)
            }
            T!["use"] => ClassLikeMember::TraitUse(self.parse_trait_use(stream)?),
            _ => ClassLikeMember::Property(self.parse_property_with_attributes_and_modifiers(
                stream,
                Sequence::empty(self.arena),
                Sequence::empty(self.arena),
            )?),
        })
    }

    fn parse_classlike_member_with_attributes(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
        attributes: Sequence<'arena, AttributeList<'arena>>,
    ) -> Result<ClassLikeMember<'arena>, ParseError> {
        let token = stream.lookahead(0)?.ok_or_else(|| stream.unexpected(None, &[]))?;
        Ok(match token.kind {
            k if k.is_modifier() => {
                let modifiers = self.parse_modifier_sequence(stream)?;

                self.parse_classlike_member_with_attributes_and_modifiers(stream, attributes, modifiers)?
            }
            T!["case"] => ClassLikeMember::EnumCase(self.parse_enum_case_with_attributes(stream, attributes)?),
            T!["const"] => ClassLikeMember::Constant(self.parse_class_like_constant_with_attributes_and_modifiers(
                stream,
                attributes,
                Sequence::empty(self.arena),
            )?),
            T!["function"] => ClassLikeMember::Method(self.parse_method_with_attributes_and_modifiers(
                stream,
                attributes,
                Sequence::empty(self.arena),
            )?),
            _ => ClassLikeMember::Property(self.parse_property_with_attributes_and_modifiers(
                stream,
                attributes,
                Sequence::empty(self.arena),
            )?),
        })
    }

    fn parse_classlike_member_with_attributes_and_modifiers(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
        attributes: Sequence<'arena, AttributeList<'arena>>,
        modifiers: Sequence<'arena, Modifier<'arena>>,
    ) -> Result<ClassLikeMember<'arena>, ParseError> {
        let token = stream.lookahead(0)?.ok_or_else(|| stream.unexpected(None, &[]))?;
        Ok(match token.kind {
            T!["const"] => ClassLikeMember::Constant(
                self.parse_class_like_constant_with_attributes_and_modifiers(stream, attributes, modifiers)?,
            ),
            T!["function"] => {
                ClassLikeMember::Method(self.parse_method_with_attributes_and_modifiers(stream, attributes, modifiers)?)
            }
            _ => ClassLikeMember::Property(
                self.parse_property_with_attributes_and_modifiers(stream, attributes, modifiers)?,
            ),
        })
    }

    pub(crate) fn parse_classlike_member_selector(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<ClassLikeMemberSelector<'arena>, ParseError> {
        let token = stream.lookahead(0)?.ok_or_else(|| stream.unexpected(None, &[]))?;

        Ok(match token.kind {
            T!["$"] | T!["${"] | T!["$variable"] => ClassLikeMemberSelector::Variable(self.parse_variable(stream)?),
            T!["{"] => ClassLikeMemberSelector::Expression(ClassLikeMemberExpressionSelector {
                left_brace: stream.eat(T!["{"])?.span,
                expression: self.arena.alloc(self.parse_expression(stream)?),
                right_brace: stream.eat(T!["}"])?.span,
            }),
            kind if kind.is_identifier_maybe_reserved() => {
                ClassLikeMemberSelector::Identifier(self.parse_local_identifier(stream)?)
            }
            _ => return Err(stream.unexpected(Some(token), T!["$variable", "${", "$", "{", Identifier])),
        })
    }

    pub(crate) fn parse_classlike_constant_selector_or_variable(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<Either<ClassLikeConstantSelector<'arena>, Variable<'arena>>, ParseError> {
        let token = stream.lookahead(0)?.ok_or_else(|| stream.unexpected(None, &[]))?;

        Ok(match token.kind {
            T!["$"] | T!["${"] | T!["$variable"] => Either::Right(self.parse_variable(stream)?),
            T!["{"] => Either::Left(ClassLikeConstantSelector::Expression(ClassLikeMemberExpressionSelector {
                left_brace: stream.eat(T!["{"])?.span,
                expression: self.arena.alloc(self.parse_expression(stream)?),
                right_brace: stream.eat(T!["}"])?.span,
            })),
            kind if kind.is_identifier_maybe_reserved() => {
                Either::Left(ClassLikeConstantSelector::Identifier(self.parse_local_identifier(stream)?))
            }
            _ => return Err(stream.unexpected(Some(token), T!["$variable", "${", "$", "{", Identifier])),
        })
    }
}
