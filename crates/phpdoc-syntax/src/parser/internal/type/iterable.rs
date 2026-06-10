use crate::cst::r#type::IterableType;
use crate::cst::r#type::Type;
use crate::error::ParseError;
use crate::parser::PHPDocParser;
use mago_allocator::Arena;

impl<'arena, A> PHPDocParser<'arena, A>
where
    A: Arena,
{
    pub(crate) fn parse_iterable_type(&mut self) -> Result<Type<'arena>, ParseError> {
        let keyword = self.parse_keyword()?;

        Ok(Type::Iterable(IterableType { keyword, parameters: self.parse_generic_parameters_or_none()? }))
    }
}
