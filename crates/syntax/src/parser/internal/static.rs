use crate::T;
use crate::ast::ast::Static;
use crate::ast::ast::StaticAbstractItem;
use crate::ast::ast::StaticConcreteItem;
use crate::ast::ast::StaticItem;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_static(&mut self) -> Result<Static<'arena>, ParseError> {
        let r#static = self.expect_keyword(T!["static"])?;
        let items = {
            let mut items = self.new_vec();
            let mut commas = self.new_vec();

            loop {
                if matches!(self.stream.peek_kind(0)?, Some(T!["?>" | ";"])) {
                    break;
                }

                items.push(self.parse_static_item()?);

                if let Some(T![","]) = self.stream.peek_kind(0)? {
                    commas.push(self.stream.consume()?);
                } else {
                    break;
                }
            }

            TokenSeparatedSequence::new(items, commas)
        };
        let terminator = self.parse_terminator()?;

        Ok(Static { r#static, items, terminator })
    }

    pub(crate) fn parse_static_item(&mut self) -> Result<StaticItem<'arena>, ParseError> {
        let var = self.parse_direct_variable()?;

        Ok(match self.stream.peek_kind(0)? {
            Some(T!["="]) => {
                let equals = self.stream.eat_span(T!["="])?;
                let value = self.parse_expression()?;

                StaticItem::Concrete(StaticConcreteItem { variable: var, equals, value })
            }
            _ => StaticItem::Abstract(StaticAbstractItem { variable: var }),
        })
    }
}
