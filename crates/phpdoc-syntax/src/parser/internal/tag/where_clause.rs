use crate::cst::tag::TagValue;
use crate::cst::tag::WhereTagValue;
use crate::cst::tag::WhereTagValueModifier;
use crate::error::ParseError;
use crate::parser::PHPDocParser;
use crate::token::TokenKind;

impl<'arena> PHPDocParser<'arena> {
    pub(crate) fn parse_where_tag_value(&mut self) -> Result<TagValue<'arena>, ParseError> {
        let name = self.parse_identifier()?;

        let modifier = if self.stream.is_at_value(b"is") {
            WhereTagValueModifier::Is(self.parse_keyword()?)
        } else if self.stream.is_at(TokenKind::Colon) {
            WhereTagValueModifier::Colon(self.stream.consume_span()?)
        } else {
            return Err(ParseError::UnexpectedToken(self.stream.peek()?.span_for(self.file_id())));
        };

        let r#type = self.parse_type()?;
        let r#type = self.alloc(r#type);

        Ok(TagValue::Where(WhereTagValue { name, modifier, r#type }))
    }
}
