use either::Either;

use mago_database::file::HasFileId;
use mago_span::Span;

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

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_classlike_member(&mut self) -> Result<ClassLikeMember<'arena>, ParseError> {
        let token = self.stream.lookahead(0)?.ok_or_else(|| self.stream.unexpected(None, &[]))?;
        Ok(match token.kind {
            T!["#["] => {
                let attributes = self.parse_attribute_list_sequence()?;

                self.parse_classlike_member_with_attributes(attributes)?
            }
            k if k.is_modifier() => {
                let modifiers = self.parse_modifier_sequence()?;

                self.parse_classlike_member_with_attributes_and_modifiers(Sequence::empty(self.arena), modifiers)?
            }
            T!["const"] => ClassLikeMember::Constant(self.parse_class_like_constant_with_attributes_and_modifiers(
                Sequence::empty(self.arena),
                Sequence::empty(self.arena),
            )?),
            T!["function"] => ClassLikeMember::Method(self.parse_method_with_attributes_and_modifiers(
                Sequence::empty(self.arena),
                Sequence::empty(self.arena),
            )?),
            T!["case"] => ClassLikeMember::EnumCase(self.parse_enum_case_with_attributes(Sequence::empty(self.arena))?),
            T!["use"] => ClassLikeMember::TraitUse(self.parse_trait_use()?),
            _ => ClassLikeMember::Property(self.parse_property_with_attributes_and_modifiers(
                Sequence::empty(self.arena),
                Sequence::empty(self.arena),
            )?),
        })
    }

    fn parse_classlike_member_with_attributes(
        &mut self,
        attributes: Sequence<'arena, AttributeList<'arena>>,
    ) -> Result<ClassLikeMember<'arena>, ParseError> {
        let token = self.stream.lookahead(0)?.ok_or_else(|| self.stream.unexpected(None, &[]))?;
        Ok(match token.kind {
            k if k.is_modifier() => {
                let modifiers = self.parse_modifier_sequence()?;

                self.parse_classlike_member_with_attributes_and_modifiers(attributes, modifiers)?
            }
            T!["case"] => ClassLikeMember::EnumCase(self.parse_enum_case_with_attributes(attributes)?),
            T!["const"] => ClassLikeMember::Constant(
                self.parse_class_like_constant_with_attributes_and_modifiers(attributes, Sequence::empty(self.arena))?,
            ),
            T!["function"] => ClassLikeMember::Method(
                self.parse_method_with_attributes_and_modifiers(attributes, Sequence::empty(self.arena))?,
            ),
            _ => ClassLikeMember::Property(
                self.parse_property_with_attributes_and_modifiers(attributes, Sequence::empty(self.arena))?,
            ),
        })
    }

    fn parse_classlike_member_with_attributes_and_modifiers(
        &mut self,
        attributes: Sequence<'arena, AttributeList<'arena>>,
        modifiers: Sequence<'arena, Modifier<'arena>>,
    ) -> Result<ClassLikeMember<'arena>, ParseError> {
        let token = self.stream.lookahead(0)?.ok_or_else(|| self.stream.unexpected(None, &[]))?;
        Ok(match token.kind {
            T!["const"] => ClassLikeMember::Constant(
                self.parse_class_like_constant_with_attributes_and_modifiers(attributes, modifiers)?,
            ),
            T!["function"] => {
                ClassLikeMember::Method(self.parse_method_with_attributes_and_modifiers(attributes, modifiers)?)
            }
            _ => ClassLikeMember::Property(self.parse_property_with_attributes_and_modifiers(attributes, modifiers)?),
        })
    }

    pub(crate) fn parse_classlike_member_selector(&mut self) -> Result<ClassLikeMemberSelector<'arena>, ParseError> {
        let token = match self.stream.lookahead(0)? {
            Some(token) => token,
            None => {
                let pos = self.stream.current_position();
                let span = Span::new(self.stream.file_id(), pos, pos);
                return Ok(ClassLikeMemberSelector::Missing(span));
            }
        };

        Ok(match token.kind {
            T!["$"] | T!["${"] | T!["$variable"] => ClassLikeMemberSelector::Variable(self.parse_variable()?),
            T!["{"] => ClassLikeMemberSelector::Expression(ClassLikeMemberExpressionSelector {
                left_brace: self.stream.eat_span(T!["{"])?,
                expression: self.parse_expression()?,
                right_brace: self.stream.eat_span(T!["}"])?,
            }),
            kind if kind.is_identifier_maybe_reserved() => {
                ClassLikeMemberSelector::Identifier(self.parse_local_identifier()?)
            }
            _ => {
                let pos = self.stream.current_position();
                let span = Span::new(self.stream.file_id(), pos, pos);
                let err = self.stream.unexpected(Some(token), T!["$variable", "${", "$", "{", Identifier]);
                self.errors.push(err);
                ClassLikeMemberSelector::Missing(span)
            }
        })
    }

    pub(crate) fn parse_classlike_constant_selector_or_variable(
        &mut self,
    ) -> Result<Either<ClassLikeConstantSelector<'arena>, Variable<'arena>>, ParseError> {
        let token = match self.stream.lookahead(0)? {
            Some(token) => token,
            None => {
                let pos = self.stream.current_position();
                let span = Span::new(self.stream.file_id(), pos, pos);
                return Ok(Either::Left(ClassLikeConstantSelector::Missing(span)));
            }
        };

        Ok(match token.kind {
            T!["$"] | T!["${"] | T!["$variable"] => Either::Right(self.parse_variable()?),
            T!["{"] => Either::Left(ClassLikeConstantSelector::Expression(ClassLikeMemberExpressionSelector {
                left_brace: self.stream.eat_span(T!["{"])?,
                expression: self.parse_expression()?,
                right_brace: self.stream.eat_span(T!["}"])?,
            })),
            kind if kind.is_identifier_maybe_reserved() => {
                Either::Left(ClassLikeConstantSelector::Identifier(self.parse_local_identifier()?))
            }
            _ => {
                let pos = self.stream.current_position();
                let span = Span::new(self.stream.file_id(), pos, pos);
                let err = self.stream.unexpected(Some(token), T!["$variable", "${", "$", "{", Identifier]);
                self.errors.push(err);
                Either::Left(ClassLikeConstantSelector::Missing(span))
            }
        })
    }
}
