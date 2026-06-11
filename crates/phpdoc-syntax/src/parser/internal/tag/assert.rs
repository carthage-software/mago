use mago_allocator::Arena;

use crate::cst::tag::AssertPattern;
use crate::cst::tag::AssertSubject;
use crate::cst::tag::AssertTagValue;
use crate::error::ParseError;
use crate::parser::PHPDocParser;
use crate::token::TokenKind;

impl<'arena, A> PHPDocParser<'arena, A>
where
    A: Arena,
{
    pub(crate) fn parse_assert_tag_value(&mut self) -> Result<AssertTagValue<'arena>, ParseError> {
        let bang = if self.stream.is_at(TokenKind::Bang) { Some(self.stream.consume_span()?) } else { None };
        let equals = if self.stream.is_at(TokenKind::Equals) { Some(self.stream.consume_span()?) } else { None };
        let pattern = self.parse_assert_pattern()?;
        let parameter = self.parse_variable()?;

        let subject = if self.stream.is_at(TokenKind::Arrow) {
            let arrow = self.stream.consume_span()?;
            let member = self.parse_identifier()?;

            if self.stream.is_at(TokenKind::LeftParenthesis) {
                let left_parenthesis = self.stream.consume_span()?;
                let right_parenthesis = self.stream.eat_span(TokenKind::RightParenthesis)?;

                AssertSubject::Method { parameter, arrow, method: member, left_parenthesis, right_parenthesis }
            } else {
                AssertSubject::Property { parameter, arrow, property: member }
            }
        } else {
            AssertSubject::Parameter { variable: parameter }
        };

        let description = self.parse_optional_description(false)?;

        Ok(AssertTagValue { bang, equals, pattern, subject, description })
    }

    fn parse_assert_pattern(&mut self) -> Result<AssertPattern<'arena>, ParseError> {
        if self.stream.is_at(TokenKind::Identifier)
            && matches!(self.stream.peek_kind(1), Some(TokenKind::Variable | TokenKind::ThisVariable))
            && let Some(token) = self.stream.lookahead(0)
        {
            let file_id = self.file_id();

            if token.value.eq_ignore_ascii_case(b"truthy") {
                let token = self.stream.consume()?;

                return Ok(AssertPattern::Truthy(token.span_for(file_id)));
            }

            if token.value.eq_ignore_ascii_case(b"falsy") {
                let token = self.stream.consume()?;

                return Ok(AssertPattern::Falsy(token.span_for(file_id)));
            }

            if token.value.eq_ignore_ascii_case(b"non-empty") {
                let token = self.stream.consume()?;

                return Ok(AssertPattern::NonEmpty(token.span_for(file_id)));
            }
        }

        let r#type = self.parse_type()?;

        Ok(AssertPattern::Type(self.alloc(r#type)))
    }
}
