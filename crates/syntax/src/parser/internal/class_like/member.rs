use either::Either;

use crate::T;
use crate::ast::ast::*;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::internal::attribute::parse_attribute_list_sequence;
use crate::parser::internal::class_like::constant::parse_class_like_constant_with_attributes_and_modifiers;
use crate::parser::internal::class_like::enum_case::parse_enum_case_with_attributes;
use crate::parser::internal::class_like::method::parse_method_with_attributes_and_modifiers;
use crate::parser::internal::class_like::property::parse_property_with_attributes_and_modifiers;
use crate::parser::internal::class_like::trait_use::parse_trait_use;
use crate::parser::internal::expression;
use crate::parser::internal::identifier;
use crate::parser::internal::modifier::parse_modifier_sequence;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;
use crate::parser::internal::variable;

pub fn parse_classlike_memeber(stream: &mut TokenStream<'_, '_>) -> Result<ClassLikeMember, ParseError> {
    Ok(match utils::peek(stream)?.kind {
        T!["#["] => {
            let attributes = parse_attribute_list_sequence(stream)?;

            parse_classlike_memeber_with_attributes(stream, attributes)?
        }
        k if k.is_modifier() => {
            let modifiers = parse_modifier_sequence(stream)?;

            parse_classlike_memeber_with_attributes_and_modifiers(stream, Sequence::empty(), modifiers)?
        }
        T!["const"] => ClassLikeMember::Constant(parse_class_like_constant_with_attributes_and_modifiers(
            stream,
            Sequence::empty(),
            Sequence::empty(),
        )?),
        T!["function"] => ClassLikeMember::Method(parse_method_with_attributes_and_modifiers(
            stream,
            Sequence::empty(),
            Sequence::empty(),
        )?),
        T!["case"] => ClassLikeMember::EnumCase(parse_enum_case_with_attributes(stream, Sequence::empty())?),
        T!["use"] => ClassLikeMember::TraitUse(parse_trait_use(stream)?),
        _ => ClassLikeMember::Property(parse_property_with_attributes_and_modifiers(
            stream,
            Sequence::empty(),
            Sequence::empty(),
        )?),
    })
}

pub fn parse_classlike_memeber_with_attributes(
    stream: &mut TokenStream<'_, '_>,
    attributes: Sequence<AttributeList>,
) -> Result<ClassLikeMember, ParseError> {
    Ok(match utils::peek(stream)?.kind {
        k if k.is_modifier() => {
            let modifiers = parse_modifier_sequence(stream)?;

            parse_classlike_memeber_with_attributes_and_modifiers(stream, attributes, modifiers)?
        }
        T!["case"] => ClassLikeMember::EnumCase(parse_enum_case_with_attributes(stream, attributes)?),
        T!["const"] => ClassLikeMember::Constant(parse_class_like_constant_with_attributes_and_modifiers(
            stream,
            attributes,
            Sequence::empty(),
        )?),
        T!["function"] => {
            ClassLikeMember::Method(parse_method_with_attributes_and_modifiers(stream, attributes, Sequence::empty())?)
        }
        _ => ClassLikeMember::Property(parse_property_with_attributes_and_modifiers(
            stream,
            attributes,
            Sequence::empty(),
        )?),
    })
}

pub fn parse_classlike_memeber_with_attributes_and_modifiers(
    stream: &mut TokenStream<'_, '_>,
    attributes: Sequence<AttributeList>,
    modifiers: Sequence<Modifier>,
) -> Result<ClassLikeMember, ParseError> {
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

pub fn parse_classlike_memeber_selector(
    stream: &mut TokenStream<'_, '_>,
) -> Result<ClassLikeMemberSelector, ParseError> {
    let token = utils::peek(stream)?;

    Ok(match token.kind {
        T!["$"] | T!["${"] | T!["$variable"] => ClassLikeMemberSelector::Variable(variable::parse_variable(stream)?),
        T!["{"] => ClassLikeMemberSelector::Expression(ClassLikeMemberExpressionSelector {
            left_brace: utils::expect_span(stream, T!["{"])?,
            expression: Box::new(expression::parse_expression(stream)?),
            right_brace: utils::expect_span(stream, T!["}"])?,
        }),
        kind if kind.is_identifier_maybe_reserved() => {
            ClassLikeMemberSelector::Identifier(identifier::parse_local_identifier(stream)?)
        }
        _ => return Err(utils::unexpected(stream, Some(token), T!["$variable", "${", "$", "{", Identifier])),
    })
}

pub fn parse_classlike_constant_selector_or_variable(
    stream: &mut TokenStream<'_, '_>,
) -> Result<Either<ClassLikeConstantSelector, Variable>, ParseError> {
    let token = utils::peek(stream)?;

    Ok(match token.kind {
        T!["$"] | T!["${"] | T!["$variable"] => Either::Right(variable::parse_variable(stream)?),
        T!["{"] => Either::Left(ClassLikeConstantSelector::Expression(ClassLikeMemberExpressionSelector {
            left_brace: utils::expect_span(stream, T!["{"])?,
            expression: Box::new(expression::parse_expression(stream)?),
            right_brace: utils::expect_span(stream, T!["}"])?,
        })),
        kind if kind.is_identifier_maybe_reserved() => {
            Either::Left(ClassLikeConstantSelector::Identifier(identifier::parse_local_identifier(stream)?))
        }
        _ => return Err(utils::unexpected(stream, Some(token), T!["$variable", "${", "$", "{", Identifier])),
    })
}
