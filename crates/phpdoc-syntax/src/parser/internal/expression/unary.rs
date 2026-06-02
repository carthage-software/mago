use crate::cst::expression::ConstantExpression;
use crate::cst::expression::UnaryPrefixConstantExpression;
use crate::cst::expression::UnaryPrefixConstantOperator;
use crate::error::ParseError;
use crate::parser::PHPDocParser;

impl<'arena> PHPDocParser<'arena> {
    pub(crate) fn parse_negated_constant(&mut self) -> Result<ConstantExpression<'arena>, ParseError> {
        let minus = self.stream.consume_span()?;
        let operand = self.parse_constant_expression()?;

        Ok(ConstantExpression::UnaryPrefix(UnaryPrefixConstantExpression {
            operator: UnaryPrefixConstantOperator::Negation(minus),
            operand: self.alloc(operand),
        }))
    }

    pub(crate) fn parse_posited_constant(&mut self) -> Result<ConstantExpression<'arena>, ParseError> {
        let plus = self.stream.consume_span()?;
        let operand = self.parse_constant_expression()?;

        Ok(ConstantExpression::UnaryPrefix(UnaryPrefixConstantExpression {
            operator: UnaryPrefixConstantOperator::Plus(plus),
            operand: self.alloc(operand),
        }))
    }
}
