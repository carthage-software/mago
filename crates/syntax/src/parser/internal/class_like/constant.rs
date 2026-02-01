use crate::T;
use crate::ast::ast::AttributeList;
use crate::ast::ast::ClassLikeConstant;
use crate::ast::ast::ClassLikeConstantItem;
use crate::ast::ast::Modifier;
use crate::ast::sequence::Sequence;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_class_like_constant_with_attributes_and_modifiers(
        &mut self,
        attributes: Sequence<'arena, AttributeList<'arena>>,
        modifiers: Sequence<'arena, Modifier<'arena>>,
    ) -> Result<ClassLikeConstant<'arena>, ParseError> {
        Ok(ClassLikeConstant {
            attribute_lists: attributes,
            modifiers,
            r#const: self.expect_keyword(T!["const"])?,
            hint: match self.stream.lookahead(1)?.map(|t| t.kind) {
                Some(
                    crate::token::TokenKind::Equal
                    | crate::token::TokenKind::Semicolon
                    | crate::token::TokenKind::CloseTag,
                ) => None,
                _ => Some(self.parse_type_hint()?),
            },
            items: {
                let mut items = self.new_vec();
                let mut commas = self.new_vec();
                loop {
                    if matches!(self.stream.lookahead(0)?.map(|t| t.kind), Some(T![";" | "?>"])) {
                        break;
                    }

                    items.push(self.parse_class_like_constant_item()?);

                    match self.stream.lookahead(0)?.map(|t| t.kind) {
                        Some(T![","]) => commas.push(self.stream.consume()?),
                        _ => {
                            break;
                        }
                    }
                }

                TokenSeparatedSequence::new(items, commas)
            },
            terminator: self.parse_terminator()?,
        })
    }

    fn parse_class_like_constant_item(&mut self) -> Result<ClassLikeConstantItem<'arena>, ParseError> {
        Ok(ClassLikeConstantItem {
            name: self.parse_local_identifier()?,
            equals: self.stream.eat(T!["="])?.span,
            value: self.parse_expression()?,
        })
    }
}
