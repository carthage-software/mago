use crate::cst::r#type::Type;
use crate::cst::r#type::ValueOfType;
use crate::error::ParseError;
use crate::parser::PHPDocParser;
use mago_allocator::Arena;

impl<'arena, A> PHPDocParser<'arena, A>
where
    A: Arena,
{
    pub(crate) fn parse_value_of_type(&mut self) -> Result<Type<'arena>, ParseError> {
        let keyword = self.parse_keyword()?;

        Ok(Type::ValueOf(ValueOfType { keyword, parameter: self.parse_single_generic_parameter()? }))
    }
}
