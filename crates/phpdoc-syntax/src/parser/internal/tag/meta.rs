use crate::cst::tag::DeprecatedTagValue;
use crate::cst::tag::GenericTagValue;
use crate::cst::tag::InheritDocTagValue;
use crate::cst::tag::PureUnlessCallableIsImpureTagValue;
use crate::cst::tag::TagValue;
use crate::error::ParseError;
use crate::parser::PHPDocParser;

impl<'arena> PHPDocParser<'arena> {
    pub(crate) fn parse_deprecated_tag_value(&mut self) -> Result<TagValue<'arena>, ParseError> {
        let description = self.parse_text_or_empty()?;

        Ok(TagValue::Deprecated(DeprecatedTagValue { description }))
    }

    pub(crate) fn parse_inherit_doc_tag_value(&mut self) -> Result<TagValue<'arena>, ParseError> {
        let description = self.parse_text_or_empty()?;

        Ok(TagValue::InheritDoc(InheritDocTagValue { description }))
    }

    pub(crate) fn parse_pure_unless_callable_is_impure_tag_value(&mut self) -> Result<TagValue<'arena>, ParseError> {
        let parameter = self.parse_variable()?;
        let description = self.parse_optional_description(false)?;

        Ok(TagValue::PureUnlessCallableIsImpure(PureUnlessCallableIsImpureTagValue { parameter, description }))
    }

    pub(crate) fn parse_generic_tag_value(&mut self) -> Result<TagValue<'arena>, ParseError> {
        let value = self.parse_text_or_empty()?;

        Ok(TagValue::Generic(GenericTagValue { value }))
    }
}
