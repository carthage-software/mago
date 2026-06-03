use crate::cst::tag::TagValue;
use crate::cst::tag::TemplateTagValue;
use crate::cst::tag::TemplateTagValueBound;
use crate::cst::tag::TemplateTagValueDefault;
use crate::error::ParseError;
use crate::parser::PHPDocParser;
use crate::token::TokenKind;

impl<'arena> PHPDocParser<'arena> {
    pub(crate) fn parse_template_tag_value(
        &mut self,
        parse_description: bool,
    ) -> Result<TemplateTagValue<'arena>, ParseError> {
        let name = self.parse_identifier()?;

        let bound = if self.stream.is_at_value(b"of") || self.stream.is_at_value(b"as") {
            let keyword = self.parse_keyword()?;
            let r#type = self.parse_type()?;
            let r#type = self.alloc(r#type);

            Some(TemplateTagValueBound { keyword, r#type })
        } else {
            None
        };

        let default = if self.stream.is_at(TokenKind::Equals) {
            let equals = self.stream.consume_span()?;
            let r#type = self.parse_type()?;
            let r#type = self.alloc(r#type);

            Some(TemplateTagValueDefault { equals, r#type })
        } else {
            None
        };

        let description = if parse_description { self.parse_optional_description(true)? } else { None };

        Ok(TemplateTagValue { name, bound, default, description })
    }

    pub(crate) fn parse_template_tag(&mut self) -> Result<TagValue<'arena>, ParseError> {
        Ok(TagValue::Template(self.parse_template_tag_value(true)?))
    }
}
