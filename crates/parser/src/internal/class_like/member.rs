use either::Either;

use mago_ast::ast::*;
use mago_ast::sequence::Sequence;
use mago_token::T;

use crate::error::ParseError;
use crate::internal::attribute::parse_attribute_list_sequence;
use crate::internal::class_like::constant::parse_class_like_constant_with_attributes_and_modifiers;
use crate::internal::class_like::enum_case::parse_enum_case_with_attributes;
use crate::internal::class_like::method::parse_method_with_attributes_and_modifiers;
use crate::internal::class_like::property::parse_property_with_attributes_and_modifiers;
use crate::internal::class_like::trait_use::parse_trait_use;
use crate::internal::expression;
use crate::internal::identifier;
use crate::internal::modifier::parse_modifier_sequence;
use crate::internal::token_stream::TokenStream;
use crate::internal::utils;
use crate::internal::variable;

#[inline]
pub fn parse_classlike_memeber<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<ClassLikeMember<'i>, ParseError> {
    Ok(match utils::peek(stream)?.kind {
        T!["#["] => {
            let attributes = parse_attribute_list_sequence(stream)?;

            parse_classlike_memeber_with_attributes(stream, attributes)?
        }
        k if k.is_modifier() => {
            let modifiers = parse_modifier_sequence(stream)?;

            parse_classlike_memeber_with_attributes_and_modifiers(stream, Sequence::new(stream.vec()), modifiers)?
        }
        T!["const"] => ClassLikeMember::Constant(parse_class_like_constant_with_attributes_and_modifiers(
            stream,
            Sequence::new(stream.vec()),
            Sequence::new(stream.vec()),
        )?),
        T!["function"] => ClassLikeMember::Method(parse_method_with_attributes_and_modifiers(
            stream,
            Sequence::new(stream.vec()),
            Sequence::new(stream.vec()),
        )?),
        T!["case"] => ClassLikeMember::EnumCase(parse_enum_case_with_attributes(stream, Sequence::new(stream.vec()))?),
        T!["use"] => ClassLikeMember::TraitUse(parse_trait_use(stream)?),
        _ => ClassLikeMember::Property(parse_property_with_attributes_and_modifiers(
            stream,
            Sequence::new(stream.vec()),
            Sequence::new(stream.vec()),
        )?),
    })
}

#[inline]
pub fn parse_classlike_memeber_with_attributes<'i>(
    stream: &mut TokenStream<'_, 'i>,
    attributes: Sequence<'i, AttributeList<'i>>,
) -> Result<ClassLikeMember<'i>, ParseError> {
    Ok(match utils::peek(stream)?.kind {
        k if k.is_modifier() => {
            let modifiers = parse_modifier_sequence(stream)?;

            parse_classlike_memeber_with_attributes_and_modifiers(stream, attributes, modifiers)?
        }
        T!["case"] => ClassLikeMember::EnumCase(parse_enum_case_with_attributes(stream, attributes)?),
        T!["const"] => ClassLikeMember::Constant(parse_class_like_constant_with_attributes_and_modifiers(
            stream,
            attributes,
            Sequence::new(stream.vec()),
        )?),
        T!["function"] => ClassLikeMember::Method(parse_method_with_attributes_and_modifiers(
            stream,
            attributes,
            Sequence::new(stream.vec()),
        )?),
        _ => ClassLikeMember::Property(parse_property_with_attributes_and_modifiers(
            stream,
            attributes,
            Sequence::new(stream.vec()),
        )?),
    })
}

#[inline]
pub fn parse_classlike_memeber_with_attributes_and_modifiers<'i>(
    stream: &mut TokenStream<'_, 'i>,
    attributes: Sequence<'i, AttributeList<'i>>,
    modifiers: Sequence<'i, Modifier>,
) -> Result<ClassLikeMember<'i>, ParseError> {
    Ok(match utils::peek(stream)?.kind {
        T!["const"] => ClassLikeMember::Constant(parse_class_like_constant_with_attributes_and_modifiers(
            stream, attributes, modifiers,
        )?),
        T!["function"] => {
            ClassLikeMember::Method(parse_method_with_attributes_and_modifiers(stream, attributes, modifiers)?)
        }
        _ => ClassLikeMember::Property(parse_property_with_attributes_and_modifiers(stream, attributes, modifiers)?),
    })
}

#[inline]
pub fn parse_classlike_memeber_selector<'i>(
    stream: &mut TokenStream<'_, 'i>,
) -> Result<ClassLikeMemberSelector<'i>, ParseError> {
    let token = utils::peek(stream)?;

    Ok(match token.kind {
        T!["$"] | T!["${"] | T!["$variable"] => ClassLikeMemberSelector::Variable(variable::parse_variable(stream)?),
        T!["{"] => {
            let left_brace = utils::expect_span(stream, T!["{"])?;
            let expression = expression::parse_expression(stream)?;
            let right_brace = utils::expect_span(stream, T!["}"])?;

            ClassLikeMemberSelector::Expression(ClassLikeMemberExpressionSelector {
                left_brace,
                expression: stream.boxed(expression),
                right_brace,
            })
        }
        kind if kind.is_identifier_maybe_reserved() => {
            ClassLikeMemberSelector::Identifier(identifier::parse_local_identifier(stream)?)
        }
        _ => return Err(utils::unexpected(stream, Some(token), T!["$variable", "${", "$", "{", Identifier])),
    })
}

#[inline]
pub fn parse_classlike_constant_selector_or_variable<'i>(
    stream: &mut TokenStream<'_, 'i>,
) -> Result<Either<ClassLikeConstantSelector<'i>, Variable<'i>>, ParseError> {
    let token = utils::peek(stream)?;

    Ok(match token.kind {
        T!["$"] | T!["${"] | T!["$variable"] => Either::Right(variable::parse_variable(stream)?),
        T!["{"] => {
            let left_brace = utils::expect_span(stream, T!["{"])?;
            let expression = expression::parse_expression(stream)?;
            let right_brace = utils::expect_span(stream, T!["}"])?;

            Either::Left(ClassLikeConstantSelector::Expression(ClassLikeMemberExpressionSelector {
                left_brace,
                expression: stream.boxed(expression),
                right_brace,
            }))
        }
        kind if kind.is_identifier_maybe_reserved() => {
            Either::Left(ClassLikeConstantSelector::Identifier(identifier::parse_local_identifier(stream)?))
        }
        _ => return Err(utils::unexpected(stream, Some(token), T!["$variable", "${", "$", "{", Identifier])),
    })
}
