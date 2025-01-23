use mago_ast::ast::*;
use mago_ast::sequence::Sequence;
use mago_ast::sequence::TokenSeparatedSequence;
use mago_token::T;

use crate::error::ParseError;
use crate::internal::expression::parse_expression;
use crate::internal::identifier::parse_local_identifier;
use crate::internal::terminator::parse_terminator;
use crate::internal::token_stream::TokenStream;
use crate::internal::type_hint::parse_type_hint;
use crate::internal::utils;

pub fn parse_class_like_constant_with_attributes_and_modifiers(
    stream: &mut TokenStream<'_, '_>,
    attributes: Sequence<AttributeList>,
    modifiers: Sequence<Modifier>,
) -> Result<ClassLikeConstant, ParseError> {
    Ok(ClassLikeConstant {
        attribute_lists: attributes,
        modifiers,
        r#const: utils::expect_keyword(stream, T!["const"])?,
        hint: match utils::maybe_peek_nth(stream, 1)?.map(|t| t.kind) {
            Some(T!["=" | ";" | "?>"]) => None,
            _ => Some(parse_type_hint(stream)?),
        },
        items: {
            let mut items = vec![];
            let mut commas = vec![];
            loop {
                if matches!(utils::maybe_peek(stream)?.map(|t| t.kind), Some(T![";" | "?>"])) {
                    break;
                }

                items.push(parse_constant_item(stream)?);

                match utils::maybe_expect(stream, T![","])? {
                    Some(comma) => commas.push(comma),
                    None => {
                        break;
                    }
                }
            }

            TokenSeparatedSequence::new(items, commas)
        },
        terminator: parse_terminator(stream)?,
    })
}

pub fn parse_constant_item(stream: &mut TokenStream<'_, '_>) -> Result<ClassLikeConstantItem, ParseError> {
    Ok(ClassLikeConstantItem {
        name: parse_local_identifier(stream)?,
        equals: utils::expect_span(stream, T!["="])?,
        value: parse_expression(stream)?,
    })
}
