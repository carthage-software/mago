use mago_allocator::Arena;
use mago_span::Span;
use mago_syntax_core::cst::TokenSeparatedSequence;

use crate::cst::expression::ArrayConstant;
use crate::cst::expression::ArrayConstantItem;
use crate::cst::expression::ConstantExpression;
use crate::cst::keyword::Keyword;
use crate::error::ParseError;
use crate::parser::PHPDocParser;
use crate::token::TokenKind;

impl<'arena, A> PHPDocParser<'arena, A>
where
    A: Arena,
{
    pub(crate) fn parse_bracket_array_constant(&mut self) -> Result<ConstantExpression<'arena>, ParseError> {
        let left_delimiter = self.stream.consume_span()?;

        self.parse_array_constant(None, left_delimiter, TokenKind::RightBracket)
    }

    pub(crate) fn parse_array_constant(
        &mut self,
        keyword: Option<Keyword<'arena>>,
        left_delimiter: Span,
        closing: TokenKind,
    ) -> Result<ConstantExpression<'arena>, ParseError> {
        let mut items = self.new_vec::<ArrayConstantItem<'arena>>();
        let mut commas = self.new_vec::<Span>();

        while !self.stream.is_at(closing) {
            items.push(self.parse_array_constant_item()?);

            if self.stream.is_at(TokenKind::Comma) {
                commas.push(self.stream.consume_span()?);
            } else {
                break;
            }
        }

        let right_delimiter = self.stream.eat_span(closing)?;

        Ok(ConstantExpression::Array(ArrayConstant {
            keyword,
            left_delimiter,
            items: TokenSeparatedSequence::new(items, commas),
            right_delimiter,
        }))
    }

    fn parse_array_constant_item(&mut self) -> Result<ArrayConstantItem<'arena>, ParseError> {
        let first = self.parse_constant_expression()?;

        if self.stream.is_at(TokenKind::DoubleArrow) {
            let key = self.alloc(first);
            let double_arrow = self.stream.consume_span()?;
            let value = self.parse_constant_expression()?;
            let value = self.alloc(value);

            Ok(ArrayConstantItem { key: Some(key), double_arrow: Some(double_arrow), value })
        } else {
            let value = self.alloc(first);

            Ok(ArrayConstantItem { key: None, double_arrow: None, value })
        }
    }
}
