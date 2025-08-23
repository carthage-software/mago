use crate::T;
use crate::ast::ast::*;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::internal::expression;
use crate::parser::internal::identifier::parse_local_identifier;
use crate::parser::internal::terminator::parse_terminator;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;

pub fn parse_enum_case_with_attributes<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
    attributes: Sequence<'arena, AttributeList<'arena>>,
) -> Result<EnumCase<'arena>, ParseError> {
    Ok(EnumCase {
        attribute_lists: attributes,
        case: utils::expect_keyword(stream, T!["case"])?,
        item: parse_enum_case_item(stream)?,
        terminator: parse_terminator(stream)?,
    })
}

pub fn parse_enum_case_item<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<EnumCaseItem<'arena>, ParseError> {
    let name = parse_local_identifier(stream)?;

    Ok(match utils::maybe_peek(stream)?.map(|t| t.kind) {
        Some(T!["="]) => {
            let equals = utils::expect_span(stream, T!["="])?;
            let value = expression::parse_expression(stream)?;

            EnumCaseItem::Backed(EnumCaseBackedItem { name, equals, value })
        }
        _ => EnumCaseItem::Unit(EnumCaseUnitItem { name }),
    })
}
