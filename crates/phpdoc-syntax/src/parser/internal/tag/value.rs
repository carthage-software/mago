use crate::cst::tag::MixinTagValue;
use crate::cst::tag::ReturnTagValue;
use crate::cst::tag::SelfOutTagValue;
use crate::cst::tag::TagValue;
use crate::cst::tag::ThrowsTagValue;
use crate::cst::tag::VarTagValue;
use crate::error::ParseError;
use crate::parser::PHPDocParser;
use crate::token::TokenKind;
use mago_allocator::Arena;

impl<'arena, A> PHPDocParser<'arena, A>
where
    A: Arena,
{
    pub(crate) fn parse_return_tag_value(&mut self) -> Result<ReturnTagValue<'arena>, ParseError> {
        let r#type = self.parse_type()?;
        let r#type = self.alloc(r#type);
        let description = self.parse_optional_description(true)?;

        Ok(ReturnTagValue { r#type, description })
    }

    pub(crate) fn parse_throws_tag_value(&mut self) -> Result<TagValue<'arena>, ParseError> {
        let r#type = self.parse_type()?;
        let r#type = self.alloc(r#type);
        let description = self.parse_optional_description(true)?;

        Ok(TagValue::Throws(ThrowsTagValue { r#type, description }))
    }

    pub(crate) fn parse_mixin_tag_value(&mut self) -> Result<TagValue<'arena>, ParseError> {
        let r#type = self.parse_type()?;
        let r#type = self.alloc(r#type);
        let description = self.parse_optional_description(true)?;

        Ok(TagValue::Mixin(MixinTagValue { r#type, description }))
    }

    pub(crate) fn parse_self_out_tag_value(&mut self) -> Result<TagValue<'arena>, ParseError> {
        let r#type = self.parse_type()?;
        let r#type = self.alloc(r#type);
        let description = self.parse_optional_description(true)?;

        Ok(TagValue::SelfOut(SelfOutTagValue { r#type, description }))
    }

    pub(crate) fn parse_var_tag_value(&mut self) -> Result<TagValue<'arena>, ParseError> {
        let r#type = self.parse_type_without_conditional()?;
        let r#type = self.alloc(r#type);
        let variable = if self.stream.is_at(TokenKind::Variable) || self.stream.is_at(TokenKind::ThisVariable) {
            Some(self.parse_variable()?)
        } else {
            None
        };
        let description = self.parse_optional_description(variable.is_none())?;

        Ok(TagValue::Var(VarTagValue { r#type, variable, description }))
    }
}
