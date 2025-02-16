use mago_ast::ast::*;
use mago_ast::sequence::TokenSeparatedSequence;
use mago_ast::Sequence;
use mago_token::T;

use crate::error::ParseError;
use crate::internal::expression::parse_expression;
use crate::internal::identifier::parse_local_identifier;
use crate::internal::terminator::parse_terminator;
use crate::internal::token_stream::TokenStream;
use crate::internal::utils;

#[inline]
pub fn parse_constant_with_attributes<'i>(
    stream: &mut TokenStream<'_, 'i>,
    attribute_lists: Sequence<'i, AttributeList<'i>>,
) -> Result<Constant<'i>, ParseError> {
    Ok(Constant {
        attribute_lists,
        r#const: utils::expect_keyword(stream, T!["const"])?,
        items: {
            let mut items = stream.vec();
            let mut commas = stream.vec();
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

#[inline]
pub fn parse_constant_item<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<ConstantItem<'i>, ParseError> {
    Ok(ConstantItem {
        name: parse_local_identifier(stream)?,
        equals: utils::expect_span(stream, T!["="])?,
        value: parse_expression(stream)?,
    })
}
