use crate::cst::r#type::KeyOfType;
use crate::cst::r#type::Type;
use crate::error::ParseError;
use crate::parser::PHPDocParser;
use mago_allocator::Arena;

impl<'arena, A> PHPDocParser<'arena, A>
where
    A: Arena,
{
    pub(crate) fn parse_key_of_type(&mut self) -> Result<Type<'arena>, ParseError> {
        let keyword = self.parse_keyword()?;

        Ok(Type::KeyOf(KeyOfType { keyword, parameter: self.parse_single_generic_parameter()? }))
    }
}
