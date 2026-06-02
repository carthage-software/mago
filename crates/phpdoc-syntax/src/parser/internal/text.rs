use mago_span::Position;
use mago_span::Span;

use crate::cst::text::Text;
use crate::error::ParseError;
use crate::parser::PHPDocParser;
use crate::parser::internal::tag::is_inherit_doc_name;
use crate::token::TokenKind;

impl<'arena> PHPDocParser<'arena> {
    #[inline]
    fn at_description_boundary(&mut self) -> bool {
        match self.stream.peek_kind(0) {
            None => true,
            Some(TokenKind::Tag) => self.stream.starts_line(0),
            _ => false,
        }
    }

    pub(crate) fn parse_text(&mut self) -> Result<Option<Text<'arena>>, ParseError> {
        if self.at_description_boundary() {
            return Ok(None);
        }

        let file_id = self.file_id();
        let start = self.stream.peek()?.start;
        let mut end = start;
        let mut inline_inherit_doc_start: Option<Position> = None;

        loop {
            match self.stream.peek_kind(0) {
                None => break,
                Some(TokenKind::Tag) if self.stream.starts_line(0) => break,
                _ => {}
            }

            if self.stream.peek_kind(0) == Some(TokenKind::LeftBrace)
                && self.stream.lookahead(1).is_some_and(|token| {
                    token.kind == TokenKind::Tag
                        && is_inherit_doc_name(token.value.strip_prefix(b"@").unwrap_or(token.value))
                })
            {
                inline_inherit_doc_start = Some(self.stream.peek()?.start);
            }

            let token = self.stream.consume()?;
            end = Position::new(token.start.offset + token.value.len() as u32);

            if token.kind == TokenKind::RightBrace
                && let Some(brace_start) = inline_inherit_doc_start.take()
            {
                self.record_inherit_doc(Span::new(file_id, brace_start, end));
            }
        }

        let value = self.stream.raw_between(start, end);

        Ok(Some(Text { span: Span::new(file_id, start, end), value }))
    }

    pub(crate) fn parse_optional_description(&mut self, limit_start: bool) -> Result<Option<Text<'arena>>, ParseError> {
        if limit_start && (self.stream.is_at(TokenKind::Pipe) || self.stream.is_at(TokenKind::Ampersand)) {
            return Err(ParseError::UnexpectedToken(self.stream.peek()?.span_for(self.file_id())));
        }

        self.parse_text()
    }

    pub(crate) fn parse_text_or_empty(&mut self) -> Result<Text<'arena>, ParseError> {
        match self.parse_text()? {
            Some(text) => Ok(text),
            None => {
                let position = self.stream.current_position();

                Ok(Text { span: Span::new(self.file_id(), position, position), value: &[] })
            }
        }
    }
}
