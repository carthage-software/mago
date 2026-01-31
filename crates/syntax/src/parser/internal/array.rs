use crate::T;
use crate::ast::ast::Array;
use crate::ast::ast::ArrayElement;
use crate::ast::ast::KeyValueArrayElement;
use crate::ast::ast::LegacyArray;
use crate::ast::ast::List;
use crate::ast::ast::MissingArrayElement;
use crate::ast::ast::ValueArrayElement;
use crate::ast::ast::VariadicArrayElement;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_array(&mut self, stream: &mut TokenStream<'_, 'arena>) -> Result<Array<'arena>, ParseError> {
        let result = self.parse_comma_separated_sequence(stream, T!["["], T!["]"], |p, s| p.parse_array_element(s))?;

        Ok(Array { left_bracket: result.open, elements: result.sequence, right_bracket: result.close })
    }

    pub(crate) fn parse_list(&mut self, stream: &mut TokenStream<'_, 'arena>) -> Result<List<'arena>, ParseError> {
        let list = self.expect_keyword(stream, T!["list"])?;
        let result = self.parse_comma_separated_sequence(stream, T!["("], T![")"], |p, s| p.parse_array_element(s))?;

        Ok(List { list, left_parenthesis: result.open, elements: result.sequence, right_parenthesis: result.close })
    }

    pub(crate) fn parse_legacy_array(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<LegacyArray<'arena>, ParseError> {
        let array = self.expect_keyword(stream, T!["array"])?;
        let result = self.parse_comma_separated_sequence(stream, T!["("], T![")"], |p, s| p.parse_array_element(s))?;

        Ok(LegacyArray {
            array,
            left_parenthesis: result.open,
            elements: result.sequence,
            right_parenthesis: result.close,
        })
    }

    pub(crate) fn parse_array_element(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<ArrayElement<'arena>, ParseError> {
        Ok(match stream.lookahead(0)?.map(|t| t.kind) {
            Some(T!["..."]) => {
                let ellipsis = stream.consume()?.span;
                ArrayElement::Variadic(VariadicArrayElement {
                    ellipsis,
                    value: self.arena.alloc(self.parse_expression(stream)?),
                })
            }
            Some(T![","]) => {
                let next = stream.lookahead(0)?.ok_or_else(|| stream.unexpected(None, &[]))?;
                ArrayElement::Missing(MissingArrayElement { comma: next.span })
            }
            _ => {
                let expr = self.arena.alloc(self.parse_expression(stream)?);

                match stream.lookahead(0)?.map(|t| t.kind) {
                    Some(T!["=>"]) => {
                        let double_arrow = stream.consume()?.span;
                        ArrayElement::KeyValue(KeyValueArrayElement {
                            key: expr,
                            double_arrow,
                            value: self.arena.alloc(self.parse_expression(stream)?),
                        })
                    }
                    _ => ArrayElement::Value(ValueArrayElement { value: expr }),
                }
            }
        })
    }
}
