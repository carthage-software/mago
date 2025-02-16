use mago_ast::ast::*;
use mago_ast::sequence::Sequence;
use mago_token::T;

use crate::error::ParseError;
use crate::internal::expression;
use crate::internal::identifier::parse_local_identifier;
use crate::internal::terminator::parse_terminator;
use crate::internal::token_stream::TokenStream;
use crate::internal::utils;

#[inline]
pub fn parse_enum_case_with_attributes<'i>(
    stream: &mut TokenStream<'_, 'i>,
    attributes: Sequence<'i, AttributeList<'i>>,
) -> Result<EnumCase<'i>, ParseError> {
    Ok(EnumCase {
        attribute_lists: attributes,
        case: utils::expect_keyword(stream, T!["case"])?,
        item: parse_enum_case_item(stream)?,
        terminator: parse_terminator(stream)?,
    })
}

#[inline]
pub fn parse_enum_case_item<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<EnumCaseItem<'i>, ParseError> {
    let name = parse_local_identifier(stream)?;

    Ok(match utils::maybe_peek(stream)?.map(|t| t.kind) {
        Some(T!["="]) => {
            let equals = utils::expect_span(stream, T!["="])?;
            let value = expression::parse_expression(stream)?;

            EnumCaseItem::Backed(EnumCaseBackedItem { name, equals, value: stream.boxed(value) })
        }
        _ => EnumCaseItem::Unit(EnumCaseUnitItem { name }),
    })
}
