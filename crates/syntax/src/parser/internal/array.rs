use mago_database::file::HasFileId;

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

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_array(&mut self) -> Result<Array<'arena>, ParseError> {
        let result = self.parse_comma_separated_sequence(T!["["], T!["]"], |p| p.parse_array_element())?;

        Ok(Array { left_bracket: result.open, elements: result.sequence, right_bracket: result.close })
    }

    pub(crate) fn parse_list(&mut self) -> Result<List<'arena>, ParseError> {
        let list = self.expect_keyword(T!["list"])?;
        let result = self.parse_comma_separated_sequence(T!["("], T![")"], |p| p.parse_array_element())?;

        Ok(List { list, left_parenthesis: result.open, elements: result.sequence, right_parenthesis: result.close })
    }

    pub(crate) fn parse_legacy_array(&mut self) -> Result<LegacyArray<'arena>, ParseError> {
        let array = self.expect_keyword(T!["array"])?;
        let result = self.parse_comma_separated_sequence(T!["("], T![")"], |p| p.parse_array_element())?;

        Ok(LegacyArray {
            array,
            left_parenthesis: result.open,
            elements: result.sequence,
            right_parenthesis: result.close,
        })
    }

    pub(crate) fn parse_array_element(&mut self) -> Result<ArrayElement<'arena>, ParseError> {
        Ok(match self.stream.peek_kind(0)? {
            Some(T!["..."]) => {
                let ellipsis = self.stream.consume_span()?;
                ArrayElement::Variadic(VariadicArrayElement {
                    ellipsis,
                    value: self.arena.alloc(self.parse_expression()?),
                })
            }
            Some(T![","]) => {
                let next = self.stream.lookahead(0)?.ok_or_else(|| self.stream.unexpected(None, &[]))?;
                ArrayElement::Missing(MissingArrayElement { comma: next.span_for(self.stream.file_id()) })
            }
            _ => {
                let expr = self.arena.alloc(self.parse_expression()?);

                match self.stream.peek_kind(0)? {
                    Some(T!["=>"]) => {
                        let double_arrow = self.stream.consume_span()?;
                        ArrayElement::KeyValue(KeyValueArrayElement {
                            key: expr,
                            double_arrow,
                            value: self.arena.alloc(self.parse_expression()?),
                        })
                    }
                    _ => ArrayElement::Value(ValueArrayElement { value: expr }),
                }
            }
        })
    }
}
