use mago_span::Span;

use crate::cst::element::Element;
use crate::cst::text::Text;
use crate::parser::PHPDocParser;
use crate::token::TokenKind;

impl<'arena> PHPDocParser<'arena> {
    pub(crate) fn parse_element(&mut self) -> Element<'arena> {
        if self.stream.is_at(TokenKind::Tag) {
            match self.parse_tag() {
                Ok(tag) => return Element::Tag(self.alloc(tag)),
                Err(error) => self.record_error(error),
            }
        } else {
            match self.parse_text() {
                Ok(Some(text)) => return Element::Text(text),
                Ok(None) => {}
                Err(error) => self.record_error(error),
            }
        }

        let file_id = self.file_id();
        let position = self.stream.current_position();
        if let Ok(token) = self.stream.consume() {
            return Element::Text(Text { span: token.span_for(file_id), value: token.value });
        }

        Element::Text(Text { span: Span::new(file_id, position, position), value: &[] })
    }
}
