use crate::T;
use crate::ast::ast::Static;
use crate::ast::ast::StaticAbstractItem;
use crate::ast::ast::StaticConcreteItem;
use crate::ast::ast::StaticItem;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_static(&mut self, stream: &mut TokenStream<'_, 'arena>) -> Result<Static<'arena>, ParseError> {
        let r#static = self.expect_keyword(stream, T!["static"])?;
        let items = {
            let mut items = self.new_vec();
            let mut commas = self.new_vec();

            loop {
                if matches!(stream.lookahead(0)?.map(|t| t.kind), Some(T!["?>" | ";"])) {
                    break;
                }

                items.push(self.parse_static_item(stream)?);

                if let Some(T![","]) = stream.lookahead(0)?.map(|t| t.kind) {
                    commas.push(stream.consume()?);
                } else {
                    break;
                }
            }

            TokenSeparatedSequence::new(items, commas)
        };
        let terminator = self.parse_terminator(stream)?;

        Ok(Static { r#static, items, terminator })
    }

    pub(crate) fn parse_static_item(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<StaticItem<'arena>, ParseError> {
        let var = self.parse_direct_variable(stream)?;

        Ok(match stream.lookahead(0)?.map(|t| t.kind) {
            Some(T!["="]) => {
                let equals = stream.eat(T!["="])?.span;
                let value = self.parse_expression(stream)?;

                StaticItem::Concrete(StaticConcreteItem { variable: var, equals, value })
            }
            _ => StaticItem::Abstract(StaticAbstractItem { variable: var }),
        })
    }
}
