use crate::T;
use crate::ast::Sequence;
use crate::ast::ast::AttributeList;
use crate::ast::ast::Constant;
use crate::ast::ast::ConstantItem;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_constant_with_attributes(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
        attribute_lists: Sequence<'arena, AttributeList<'arena>>,
    ) -> Result<Constant<'arena>, ParseError> {
        Ok(Constant {
            attribute_lists,
            r#const: self.expect_keyword(stream, T!["const"])?,
            items: {
                let mut items = self.new_vec();
                let mut commas = self.new_vec();
                loop {
                    if matches!(stream.lookahead(0)?.map(|t| t.kind), Some(T![";" | "?>"])) {
                        break;
                    }

                    items.push(self.parse_constant_item(stream)?);

                    if let Some(T![","]) = stream.lookahead(0)?.map(|t| t.kind) {
                        commas.push(stream.consume()?);
                    } else {
                        break;
                    }
                }

                TokenSeparatedSequence::new(items, commas)
            },
            terminator: self.parse_terminator(stream)?,
        })
    }

    pub(crate) fn parse_constant_item(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<ConstantItem<'arena>, ParseError> {
        Ok(ConstantItem {
            name: self.parse_local_identifier(stream)?,
            equals: stream.eat(T!["="])?.span,
            value: self.parse_expression(stream)?,
        })
    }
}
