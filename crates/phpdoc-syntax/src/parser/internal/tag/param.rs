use crate::cst::tag::ParamClosureThisTagValue;
use crate::cst::tag::ParamImmediatelyInvokedCallableTagValue;
use crate::cst::tag::ParamLaterInvokedCallableTagValue;
use crate::cst::tag::ParamOutTagValue;
use crate::cst::tag::ParamTagValue;
use crate::cst::tag::TagValue;
use crate::cst::tag::TypelessParamTagValue;
use crate::error::ParseError;
use crate::parser::PHPDocParser;
use crate::token::TokenKind;
use mago_allocator::Arena;

impl<'arena, A> PHPDocParser<'arena, A>
where
    A: Arena,
{
    #[inline]
    fn at_parameter_start(&mut self) -> bool {
        self.stream.is_at(TokenKind::Ampersand)
            || self.stream.is_at(TokenKind::Ellipsis)
            || self.stream.is_at(TokenKind::Variable)
            || self.stream.is_at(TokenKind::ThisVariable)
    }

    pub(crate) fn parse_param_tag_value(&mut self) -> Result<TagValue<'arena>, ParseError> {
        if self.at_parameter_start() {
            let ampersand =
                if self.stream.is_at(TokenKind::Ampersand) { Some(self.stream.consume_span()?) } else { None };
            let ellipsis =
                if self.stream.is_at(TokenKind::Ellipsis) { Some(self.stream.consume_span()?) } else { None };
            let parameter = self.parse_variable()?;
            let description = self.parse_optional_description(false)?;

            return Ok(TagValue::TypelessParam(TypelessParamTagValue { ampersand, ellipsis, parameter, description }));
        }

        let r#type = self.parse_type()?;
        let r#type = self.alloc(r#type);
        let ampersand = if self.stream.is_at(TokenKind::Ampersand) { Some(self.stream.consume_span()?) } else { None };
        let ellipsis = if self.stream.is_at(TokenKind::Ellipsis) { Some(self.stream.consume_span()?) } else { None };
        let parameter = self.parse_variable()?;
        let description = self.parse_optional_description(false)?;

        Ok(TagValue::Param(ParamTagValue { r#type, ampersand, ellipsis, parameter, description }))
    }

    pub(crate) fn parse_param_out_tag_value(&mut self) -> Result<TagValue<'arena>, ParseError> {
        let r#type = self.parse_type()?;
        let r#type = self.alloc(r#type);
        let parameter = self.parse_variable()?;
        let description = self.parse_optional_description(false)?;

        Ok(TagValue::ParamOut(ParamOutTagValue { r#type, parameter, description }))
    }

    pub(crate) fn parse_param_closure_this_tag_value(&mut self) -> Result<TagValue<'arena>, ParseError> {
        let r#type = self.parse_type()?;
        let r#type = self.alloc(r#type);
        let parameter = self.parse_variable()?;
        let description = self.parse_optional_description(false)?;

        Ok(TagValue::ParamClosureThis(ParamClosureThisTagValue { r#type, parameter, description }))
    }

    pub(crate) fn parse_param_immediately_invoked_callable_tag_value(
        &mut self,
    ) -> Result<TagValue<'arena>, ParseError> {
        let parameter = self.parse_variable()?;
        let description = self.parse_optional_description(false)?;

        Ok(TagValue::ParamImmediatelyInvokedCallable(ParamImmediatelyInvokedCallableTagValue {
            parameter,
            description,
        }))
    }

    pub(crate) fn parse_param_later_invoked_callable_tag_value(&mut self) -> Result<TagValue<'arena>, ParseError> {
        let parameter = self.parse_variable()?;
        let description = self.parse_optional_description(false)?;

        Ok(TagValue::ParamLaterInvokedCallable(ParamLaterInvokedCallableTagValue { parameter, description }))
    }
}
