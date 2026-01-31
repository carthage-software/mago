use crate::T;
use crate::ast::ast::AttributeList;
use crate::ast::ast::EnumCase;
use crate::ast::ast::EnumCaseBackedItem;
use crate::ast::ast::EnumCaseItem;
use crate::ast::ast::EnumCaseUnitItem;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_enum_case_with_attributes(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
        attributes: Sequence<'arena, AttributeList<'arena>>,
    ) -> Result<EnumCase<'arena>, ParseError> {
        Ok(EnumCase {
            attribute_lists: attributes,
            case: self.expect_keyword(stream, T!["case"])?,
            item: self.parse_enum_case_item(stream)?,
            terminator: self.parse_terminator(stream)?,
        })
    }

    fn parse_enum_case_item(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<EnumCaseItem<'arena>, ParseError> {
        let name = self.parse_local_identifier(stream)?;

        Ok(match stream.lookahead(0)?.map(|t| t.kind) {
            Some(T!["="]) => {
                let equals = stream.eat(T!["="])?.span;
                let value = self.parse_expression(stream)?;

                EnumCaseItem::Backed(EnumCaseBackedItem { name, equals, value })
            }
            _ => EnumCaseItem::Unit(EnumCaseUnitItem { name }),
        })
    }
}
