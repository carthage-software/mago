use crate::cst::tag::PropertyTagValue;
use crate::cst::tag::TagValue;
use crate::error::ParseError;
use crate::parser::PHPDocParser;
use crate::token::TokenKind;

impl<'arena> PHPDocParser<'arena> {
    pub(crate) fn parse_property_tag_value(&mut self) -> Result<TagValue<'arena>, ParseError> {
        let r#type = if self.stream.is_at(TokenKind::Variable) || self.stream.is_at(TokenKind::ThisVariable) {
            None
        } else {
            let r#type = self.parse_type()?;

            Some(self.alloc(r#type))
        };
        let variable = self.parse_variable()?;
        let description = self.parse_optional_description(false)?;

        Ok(TagValue::Property(PropertyTagValue { r#type, variable, description }))
    }
}
