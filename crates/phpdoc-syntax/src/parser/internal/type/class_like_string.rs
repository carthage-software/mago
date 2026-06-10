use crate::cst::r#type::ClassLikeStringType;
use crate::cst::r#type::ClassStringType;
use crate::cst::r#type::EnumStringType;
use crate::cst::r#type::InterfaceStringType;
use crate::cst::r#type::TraitStringType;
use crate::cst::r#type::Type;
use crate::error::ParseError;
use crate::parser::PHPDocParser;
use mago_allocator::Arena;

impl<'arena, A> PHPDocParser<'arena, A>
where
    A: Arena,
{
    pub(crate) fn parse_class_string_type(&mut self) -> Result<Type<'arena>, ParseError> {
        let keyword = self.parse_keyword()?;

        Ok(Type::ClassString(ClassStringType { keyword, parameter: self.parse_single_generic_parameter_or_none()? }))
    }

    pub(crate) fn parse_class_like_string_type(&mut self) -> Result<Type<'arena>, ParseError> {
        let keyword = self.parse_keyword()?;

        Ok(Type::ClassLikeString(ClassLikeStringType {
            keyword,
            parameter: self.parse_single_generic_parameter_or_none()?,
        }))
    }

    pub(crate) fn parse_interface_string_type(&mut self) -> Result<Type<'arena>, ParseError> {
        let keyword = self.parse_keyword()?;

        Ok(Type::InterfaceString(InterfaceStringType {
            keyword,
            parameter: self.parse_single_generic_parameter_or_none()?,
        }))
    }

    pub(crate) fn parse_enum_string_type(&mut self) -> Result<Type<'arena>, ParseError> {
        let keyword = self.parse_keyword()?;

        Ok(Type::EnumString(EnumStringType { keyword, parameter: self.parse_single_generic_parameter_or_none()? }))
    }

    pub(crate) fn parse_trait_string_type(&mut self) -> Result<Type<'arena>, ParseError> {
        let keyword = self.parse_keyword()?;

        Ok(Type::TraitString(TraitStringType { keyword, parameter: self.parse_single_generic_parameter_or_none()? }))
    }
}
