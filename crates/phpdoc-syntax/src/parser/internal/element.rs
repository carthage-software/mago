use mago_allocator::Arena;
use mago_span::Span;

use crate::cst::element::Element;
use crate::cst::text::PlainText;
use crate::cst::text::Text;
use crate::cst::text::TextSegment;
use crate::parser::PHPDocParser;
use crate::token::TokenKind;

impl<'arena, A> PHPDocParser<'arena, A>
where
    A: Arena,
{
    pub(crate) fn parse_element(&mut self) -> Element<'arena> {
        if self.stream.is_at(TokenKind::Tag) {
            match self.parse_tag() {
                Ok(tag) => return Element::Tag(self.alloc(tag)),
                Err(error) => self.record_error(error),
            }
        } else if self.at_code_block() {
            match self.parse_code_span() {
                Ok(code) => return Element::Code(self.alloc(code)),
                Err(error) => self.record_error(error),
            }
        } else {
            match self.parse_text() {
                Ok(Some(text)) => return Element::Text(self.alloc(text)),
                Ok(None) => {}
                Err(error) => self.record_error(error),
            }
        }

        let file_id = self.file_id();
        let position = self.stream.current_position();
        if let Ok(token) = self.stream.consume() {
            let span = token.span_for(file_id);
            let mut segments = self.new_vec::<TextSegment<'arena>>();
            segments.push(TextSegment::PlainText(PlainText { span, value: token.value }));

            return Element::Text(self.alloc(Text { span, segments: segments.leak() }));
        }

        Element::Text(self.alloc(Text { span: Span::new(file_id, position, position), segments: &[] }))
    }
}
