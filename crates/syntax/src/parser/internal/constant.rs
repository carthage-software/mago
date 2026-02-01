use crate::T;
use crate::ast::Sequence;
use crate::ast::ast::AttributeList;
use crate::ast::ast::Constant;
use crate::ast::ast::ConstantItem;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_constant_with_attributes(
        &mut self,
        attribute_lists: Sequence<'arena, AttributeList<'arena>>,
    ) -> Result<Constant<'arena>, ParseError> {
        Ok(Constant {
            attribute_lists,
            r#const: self.expect_keyword(T!["const"])?,
            items: {
                let mut items = self.new_vec();
                let mut commas = self.new_vec();
                loop {
                    if matches!(self.stream.peek_kind(0)?, Some(T![";" | "?>"])) {
                        break;
                    }

                    items.push(self.parse_constant_item()?);

                    if let Some(T![","]) = self.stream.peek_kind(0)? {
                        commas.push(self.stream.consume()?);
                    } else {
                        break;
                    }
                }

                TokenSeparatedSequence::new(items, commas)
            },
            terminator: self.parse_terminator()?,
        })
    }

    pub(crate) fn parse_constant_item(&mut self) -> Result<ConstantItem<'arena>, ParseError> {
        Ok(ConstantItem {
            name: self.parse_local_identifier()?,
            equals: self.stream.eat_span(T!["="])?,
            value: self.parse_expression()?,
        })
    }
}
