use crate::T;
use crate::ast::ast::AttributeList;
use crate::ast::ast::ClassLikeConstant;
use crate::ast::ast::ClassLikeConstantItem;
use crate::ast::ast::Modifier;
use crate::ast::sequence::Sequence;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_class_like_constant_with_attributes_and_modifiers(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
        attributes: Sequence<'arena, AttributeList<'arena>>,
        modifiers: Sequence<'arena, Modifier<'arena>>,
    ) -> Result<ClassLikeConstant<'arena>, ParseError> {
        Ok(ClassLikeConstant {
            attribute_lists: attributes,
            modifiers,
            r#const: self.expect_keyword(stream, T!["const"])?,
            hint: match stream.lookahead(1)?.map(|t| t.kind) {
                Some(
                    crate::token::TokenKind::Equal
                    | crate::token::TokenKind::Semicolon
                    | crate::token::TokenKind::CloseTag,
                ) => None,
                _ => Some(self.parse_type_hint(stream)?),
            },
            items: {
                let mut items = self.new_vec();
                let mut commas = self.new_vec();
                loop {
                    if matches!(stream.lookahead(0)?.map(|t| t.kind), Some(T![";" | "?>"])) {
                        break;
                    }

                    items.push(self.parse_class_like_constant_item(stream)?);

                    match stream.lookahead(0)?.map(|t| t.kind) {
                        Some(T![","]) => commas.push(stream.consume()?),
                        _ => {
                            break;
                        }
                    }
                }

                TokenSeparatedSequence::new(items, commas)
            },
            terminator: self.parse_terminator(stream)?,
        })
    }

    fn parse_class_like_constant_item(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<ClassLikeConstantItem<'arena>, ParseError> {
        Ok(ClassLikeConstantItem {
            name: self.parse_local_identifier(stream)?,
            equals: stream.eat(T!["="])?.span,
            value: self.parse_expression(stream)?,
        })
    }
}
