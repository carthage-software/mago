use crate::cst::tag::AssertTagMethodValue;
use crate::cst::tag::AssertTagPropertyValue;
use crate::cst::tag::AssertTagValue;
use crate::cst::tag::TagValue;
use crate::error::ParseError;
use crate::parser::PHPDocParser;
use crate::token::TokenKind;
use mago_allocator::Arena;

impl<'arena, A> PHPDocParser<'arena, A>
where
    A: Arena,
{
    pub(crate) fn parse_assert_tag_value(&mut self) -> Result<TagValue<'arena>, ParseError> {
        let bang = if self.stream.is_at(TokenKind::Bang) { Some(self.stream.consume_span()?) } else { None };
        let equals = if self.stream.is_at(TokenKind::Equals) { Some(self.stream.consume_span()?) } else { None };
        let r#type = self.parse_type()?;
        let r#type = self.alloc(r#type);
        let parameter = self.parse_variable()?;

        if self.stream.is_at(TokenKind::Arrow) {
            let arrow = self.stream.consume_span()?;
            let member = self.parse_identifier()?;

            if self.stream.is_at(TokenKind::LeftParenthesis) {
                let left_parenthesis = self.stream.consume_span()?;
                let right_parenthesis = self.stream.eat_span(TokenKind::RightParenthesis)?;
                let description = self.parse_optional_description(false)?;

                return Ok(TagValue::AssertMethod(AssertTagMethodValue {
                    bang,
                    equals,
                    r#type,
                    parameter,
                    arrow,
                    method: member,
                    left_parenthesis,
                    right_parenthesis,
                    description,
                }));
            }

            let description = self.parse_optional_description(false)?;

            return Ok(TagValue::AssertProperty(AssertTagPropertyValue {
                bang,
                equals,
                r#type,
                parameter,
                arrow,
                property: member,
                description,
            }));
        }

        let description = self.parse_optional_description(false)?;

        Ok(TagValue::Assert(AssertTagValue { bang, equals, r#type, parameter, description }))
    }
}
