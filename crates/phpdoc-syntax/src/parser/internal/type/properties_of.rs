use crate::cst::r#type::PropertiesOfFilter;
use crate::cst::r#type::PropertiesOfType;
use crate::cst::r#type::Type;
use crate::error::ParseError;
use crate::parser::PHPDocParser;

impl<'arena> PHPDocParser<'arena> {
    pub(crate) fn parse_properties_of_type(&mut self, filter: PropertiesOfFilter) -> Result<Type<'arena>, ParseError> {
        let keyword = self.parse_keyword()?;

        Ok(Type::PropertiesOf(PropertiesOfType { filter, keyword, parameter: self.parse_single_generic_parameter()? }))
    }
}
