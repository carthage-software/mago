use mago_allocator::Arena;
use ordered_float::OrderedFloat;

use mago_syntax_core::utils::parse_literal_float;
use mago_syntax_core::utils::parse_literal_integer;

use crate::cst::expression::ConstantExpression;
use crate::cst::expression::FloatConstant;
use crate::cst::expression::IntegerConstant;
use crate::cst::expression::StringConstant;
use crate::error::ParseError;
use crate::parser::PHPDocParser;
use crate::token::TokenKind;

impl<'arena, A> PHPDocParser<'arena, A>
where
    A: Arena,
{
    pub(crate) fn parse_constant_literal(&mut self) -> Result<ConstantExpression<'arena>, ParseError> {
        let file_id = self.file_id();
        let token = self.stream.consume()?;

        let expression = match token.kind {
            TokenKind::LiteralInteger => {
                let value = parse_literal_integer(token.value).unwrap_or(0);

                ConstantExpression::Integer(IntegerConstant { span: token.span_for(file_id), value, raw: token.value })
            }
            TokenKind::LiteralFloat => {
                let value = parse_literal_float(token.value).unwrap_or(0.0);

                ConstantExpression::Float(FloatConstant {
                    span: token.span_for(file_id),
                    value: OrderedFloat(value),
                    raw: token.value,
                })
            }
            _ => {
                let value = token.value.get(1..token.value.len().saturating_sub(1)).unwrap_or(&[]);

                ConstantExpression::String(StringConstant { span: token.span_for(file_id), value, raw: token.value })
            }
        };

        Ok(expression)
    }
}
