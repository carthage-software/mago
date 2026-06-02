use crate::cst::r#type::NegatedType;
use crate::cst::r#type::PositedType;
use crate::cst::r#type::Type;
use crate::error::ParseError;
use crate::parser::PHPDocParser;

impl<'arena> PHPDocParser<'arena> {
    pub(crate) fn parse_negated_type(&mut self) -> Result<Type<'arena>, ParseError> {
        let minus = self.stream.consume_span()?;
        let operand = self.parse_primary_type()?;

        Ok(Type::Negated(NegatedType { minus, operand: self.alloc(operand) }))
    }

    pub(crate) fn parse_posited_type(&mut self) -> Result<Type<'arena>, ParseError> {
        let plus = self.stream.consume_span()?;
        let operand = self.parse_primary_type()?;

        Ok(Type::Posited(PositedType { plus, operand: self.alloc(operand) }))
    }
}
