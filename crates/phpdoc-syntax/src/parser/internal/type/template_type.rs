use crate::cst::r#type::TemplateTypeType;
use crate::cst::r#type::Type;
use crate::error::ParseError;
use crate::parser::PHPDocParser;

impl<'arena> PHPDocParser<'arena> {
    pub(crate) fn parse_template_type(&mut self) -> Result<Type<'arena>, ParseError> {
        let keyword = self.parse_keyword()?;

        Ok(Type::TemplateType(TemplateTypeType { keyword, parameters: self.parse_generic_parameters()? }))
    }
}
