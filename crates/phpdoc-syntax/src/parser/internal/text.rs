use mago_allocator::Arena;
use mago_span::Position;
use mago_span::Span;

use crate::cst::text::InlineTag;
use crate::cst::text::PlainText;
use crate::cst::text::Text;
use crate::cst::text::TextSegment;
use crate::error::ParseError;
use crate::parser::PHPDocParser;
use crate::token::TokenKind;

impl<'arena, A> PHPDocParser<'arena, A>
where
    A: Arena,
{
    #[inline]
    fn at_description_boundary(&mut self) -> bool {
        match self.stream.peek_kind(0) {
            None => true,
            Some(TokenKind::Tag) => self.stream.starts_line(0),
            Some(TokenKind::RightBrace) => self.stream.in_inline_tag(),
            _ => false,
        }
    }

    pub(crate) fn parse_text(&mut self) -> Result<Option<Text<'arena>>, ParseError> {
        if self.at_description_boundary() {
            return Ok(None);
        }

        let file_id = self.file_id();
        let start = self.stream.peek()?.start;
        let mut segments = self.new_vec::<TextSegment<'arena>>();
        let mut plain_start = start;
        let mut plain_end = start;
        let mut end = start;

        loop {
            match self.stream.peek_kind(0) {
                None => break,
                Some(TokenKind::Tag) if self.stream.starts_line(0) => break,
                Some(TokenKind::RightBrace) if self.stream.in_inline_tag() => break,
                Some(TokenKind::Backtick) if self.at_code_block() => break,
                _ => {}
            }

            if self.stream.peek_kind(0) == Some(TokenKind::LeftBrace)
                && self.stream.lookahead(1).is_some_and(|token| token.kind == TokenKind::Tag)
            {
                if plain_end.offset > plain_start.offset {
                    segments.push(TextSegment::PlainText(PlainText {
                        span: Span::new(file_id, plain_start, plain_end),
                        value: self.stream.raw_between(plain_start, plain_end),
                    }));
                }

                let inline_tag = self.parse_inline_tag()?;
                end = inline_tag.right_brace.end;
                segments.push(TextSegment::InlineTag(inline_tag));

                plain_start = self.stream.current_position();
                plain_end = plain_start;
                continue;
            }

            if self.stream.peek_kind(0) == Some(TokenKind::Backtick) {
                if plain_end.offset > plain_start.offset {
                    segments.push(TextSegment::PlainText(PlainText {
                        span: Span::new(file_id, plain_start, plain_end),
                        value: self.stream.raw_between(plain_start, plain_end),
                    }));
                }

                let code = self.parse_code_span()?;
                end = code.right_bound.end;
                segments.push(TextSegment::InlineCode(code));

                plain_start = self.stream.current_position();
                plain_end = plain_start;
                continue;
            }

            let token = self.stream.consume()?;
            plain_end = Position::new(token.start.offset + token.value.len() as u32);
            end = plain_end;
        }

        if plain_end.offset > plain_start.offset {
            segments.push(TextSegment::PlainText(PlainText {
                span: Span::new(file_id, plain_start, plain_end),
                value: self.stream.raw_between(plain_start, plain_end),
            }));
        }

        Ok(Some(Text { span: Span::new(file_id, start, end), segments: segments.leak() }))
    }

    fn parse_inline_tag(&mut self) -> Result<InlineTag<'arena>, ParseError> {
        let left_brace = self.stream.eat_span(TokenKind::LeftBrace)?;

        self.stream.enter_inline_tag();
        let tag = self.parse_tag();
        self.stream.leave_inline_tag();
        let tag = tag?;

        let right_brace = match self.stream.eat_span(TokenKind::RightBrace) {
            Ok(span) => span,
            Err(_) => {
                let position = self.stream.current_position();
                let synthesized = Span::new(self.file_id(), position, position);
                self.record_error(ParseError::UnclosedInlineTag(left_brace.join(synthesized)));

                synthesized
            }
        };

        Ok(InlineTag { left_brace, tag, right_brace })
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

                Ok(Text { span: Span::new(self.file_id(), position, position), segments: &[] })
            }
        }
    }
}
