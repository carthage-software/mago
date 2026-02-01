use crate::T;
use crate::ast::ast::AttributeList;
use crate::ast::ast::EnumCase;
use crate::ast::ast::EnumCaseBackedItem;
use crate::ast::ast::EnumCaseItem;
use crate::ast::ast::EnumCaseUnitItem;
use crate::ast::sequence::Sequence;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_enum_case_with_attributes(
        &mut self,
        attributes: Sequence<'arena, AttributeList<'arena>>,
    ) -> Result<EnumCase<'arena>, ParseError> {
        Ok(EnumCase {
            attribute_lists: attributes,
            case: self.expect_keyword(T!["case"])?,
            item: self.parse_enum_case_item()?,
            terminator: self.parse_terminator()?,
        })
    }

    fn parse_enum_case_item(&mut self) -> Result<EnumCaseItem<'arena>, ParseError> {
        let name = self.parse_local_identifier()?;

        Ok(match self.stream.peek_kind(0)? {
            Some(T!["="]) => {
                let equals = self.stream.eat_span(T!["="])?;
                let value = self.parse_expression()?;

                EnumCaseItem::Backed(EnumCaseBackedItem { name, equals, value })
            }
            _ => EnumCaseItem::Unit(EnumCaseUnitItem { name }),
        })
    }
}
