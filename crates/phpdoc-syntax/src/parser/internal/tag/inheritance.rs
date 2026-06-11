use crate::cst::tag::ExtendsTagValue;
use crate::cst::tag::ImplementsTagValue;
use crate::cst::tag::InheritorsTagValue;
use crate::cst::tag::RequireExtendsTagValue;
use crate::cst::tag::RequireImplementsTagValue;
use crate::cst::tag::SealedTagValue;
use crate::cst::tag::TagValue;
use crate::cst::tag::UseTagValue;
use crate::error::ParseError;
use crate::parser::PHPDocParser;
use mago_allocator::Arena;

impl<'arena, A> PHPDocParser<'arena, A>
where
    A: Arena,
{
    pub(crate) fn parse_extends_tag_value(&mut self) -> Result<TagValue<'arena>, ParseError> {
        let r#type = self.parse_type()?;
        let r#type = self.alloc(r#type);
        let description = self.parse_optional_description(true)?;

        Ok(TagValue::Extends(ExtendsTagValue { r#type, description }))
    }

    pub(crate) fn parse_implements_tag_value(&mut self) -> Result<TagValue<'arena>, ParseError> {
        let r#type = self.parse_type()?;
        let r#type = self.alloc(r#type);
        let description = self.parse_optional_description(true)?;

        Ok(TagValue::Implements(ImplementsTagValue { r#type, description }))
    }

    pub(crate) fn parse_use_tag_value(&mut self) -> Result<TagValue<'arena>, ParseError> {
        let r#type = self.parse_type()?;
        let r#type = self.alloc(r#type);
        let description = self.parse_optional_description(true)?;

        Ok(TagValue::Use(UseTagValue { r#type, description }))
    }

    pub(crate) fn parse_require_extends_tag_value(&mut self) -> Result<TagValue<'arena>, ParseError> {
        let r#type = self.parse_type()?;
        let r#type = self.alloc(r#type);
        let description = self.parse_optional_description(true)?;

        Ok(TagValue::RequireExtends(RequireExtendsTagValue { r#type, description }))
    }

    pub(crate) fn parse_require_implements_tag_value(&mut self) -> Result<TagValue<'arena>, ParseError> {
        let r#type = self.parse_type()?;
        let r#type = self.alloc(r#type);
        let description = self.parse_optional_description(true)?;

        Ok(TagValue::RequireImplements(RequireImplementsTagValue { r#type, description }))
    }

    pub(crate) fn parse_sealed_tag_value(&mut self) -> Result<TagValue<'arena>, ParseError> {
        let r#type = self.parse_type()?;
        let r#type = self.alloc(r#type);
        let description = self.parse_optional_description(true)?;

        Ok(TagValue::Sealed(SealedTagValue { r#type, description }))
    }

    pub(crate) fn parse_inheritors_tag_value(&mut self) -> Result<TagValue<'arena>, ParseError> {
        let r#type = self.parse_type()?;
        let r#type = self.alloc(r#type);
        let description = self.parse_optional_description(true)?;

        Ok(TagValue::Inheritors(InheritorsTagValue { r#type, description }))
    }
}
