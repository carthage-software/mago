use crate::cst::r#type::IntMaskOfType;
use crate::cst::r#type::IntMaskType;
use crate::cst::r#type::Type;
use crate::error::ParseError;
use crate::parser::PHPDocParser;

impl<'arena> PHPDocParser<'arena> {
    pub(crate) fn parse_int_mask_type(&mut self) -> Result<Type<'arena>, ParseError> {
        let keyword = self.parse_keyword()?;

        Ok(Type::IntMask(IntMaskType { keyword, parameters: self.parse_generic_parameters()? }))
    }

    pub(crate) fn parse_int_mask_of_type(&mut self) -> Result<Type<'arena>, ParseError> {
        let keyword = self.parse_keyword()?;

        Ok(Type::IntMaskOf(IntMaskOfType { keyword, parameter: self.parse_single_generic_parameter()? }))
    }
}
