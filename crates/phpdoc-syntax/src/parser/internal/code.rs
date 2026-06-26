use mago_allocator::Arena;
use mago_span::Position;
use mago_span::Span;

use crate::cst::code::Code;
use crate::cst::keyword::Keyword;
use crate::error::ParseError;
use crate::parser::PHPDocParser;
use crate::token::TokenKind;

const FENCE_MINIMUM: usize = 3;

const FENCE_MAXIMUM: usize = u8::MAX as usize;

impl<'arena, A> PHPDocParser<'arena, A>
where
    A: Arena,
{
    #[inline]
    pub(crate) fn at_code_block(&mut self) -> bool {
        self.stream.is_at(TokenKind::Backtick)
            && self.stream.starts_line(0)
            && self.stream.lookahead(0).is_some_and(|token| token.value.len() >= FENCE_MINIMUM)
    }

    pub(crate) fn parse_code_span(&mut self) -> Result<Code<'arena>, ParseError> {
        let file_id = self.file_id();
        let starts_line = self.stream.starts_line(0);
        let open = self.stream.consume()?;
        let left_bound = open.span_for(file_id);
        let run = open.value.len();
        if run > FENCE_MAXIMUM {
            self.record_error(ParseError::MalformedCodeBlock(left_bound));
        }

        let bound = run.min(FENCE_MAXIMUM) as u8;
        let block = starts_line && run >= FENCE_MINIMUM;

        let language =
            if block && self.stream.peek_kind(0) == Some(TokenKind::Identifier) && !self.stream.starts_line(0) {
                Some(Keyword::from_token(self.stream.consume()?, file_id))
            } else {
                None
            };

        let content_start = self.stream.current_position();

        loop {
            if !block
                && self.stream.starts_line(0)
                && self.stream.current_position() != content_start
                && self.stream.is_at(TokenKind::Tag)
            {
                break;
            }

            match self.stream.peek_kind(0) {
                None => break,
                Some(TokenKind::Backtick) => {
                    let token = self.stream.peek()?;
                    let closes = if block { token.value.len() >= run } else { token.value.len() == run };
                    if closes {
                        let close = self.stream.consume()?;
                        let right_bound = close.span_for(file_id);
                        if close.value.len() > FENCE_MAXIMUM {
                            self.record_error(ParseError::MalformedCodeBlock(right_bound));
                        }

                        let value = self.extract_code_value(content_start, right_bound.start, block);

                        return Ok(Code {
                            span: left_bound.join(right_bound),
                            bound,
                            left_bound,
                            language,
                            value,
                            right_bound,
                        });
                    }

                    self.stream.consume()?;
                }
                Some(_) => {
                    self.stream.consume()?;
                }
            }
        }

        let end = self.stream.current_position();
        let right_bound = Span::new(file_id, end, end);
        let span = left_bound.join(right_bound);
        self.record_error(if block {
            ParseError::UnclosedCodeBlock(span)
        } else {
            ParseError::UnclosedInlineCode(span)
        });
        let value = self.extract_code_value(content_start, end, block);

        Ok(Code { span, bound, left_bound, language, value, right_bound })
    }

    fn extract_code_value(&self, start: Position, end: Position, block: bool) -> &'arena [u8] {
        let raw = self.stream.raw_between(start, end);
        if !block {
            if !raw.contains(&b'\n') {
                return raw;
            }

            let mut buffer = self.new_vec::<u8>();
            let mut first = true;
            for line in raw.split(|&byte| byte == b'\n') {
                let line = line.strip_suffix(b"\r").unwrap_or(line);
                if first {
                    buffer.extend_from_slice(line);
                } else {
                    buffer.push(b' ');
                    buffer.extend_from_slice(strip_docblock_prefix(line).trim_ascii_start());
                }

                first = false;
            }

            return buffer.leak();
        }

        let mut buffer = self.new_vec::<u8>();
        let mut first = true;
        for line in raw.split(|&byte| byte == b'\n') {
            if !first {
                buffer.push(b'\n');
            }

            first = false;
            buffer.extend_from_slice(strip_docblock_prefix(line));
        }

        let mut content: &[u8] = buffer.leak();
        while let [b'\n', rest @ ..] = content {
            content = rest;
        }

        while let [rest @ .., last] = content {
            if matches!(last, b'\n' | b' ' | b'\t' | b'\r') {
                content = rest;
            } else {
                break;
            }
        }

        content
    }
}

#[inline]
fn strip_docblock_prefix(line: &[u8]) -> &[u8] {
    let mut index = 0;
    while index < line.len() && matches!(line[index], b' ' | b'\t') {
        index += 1;
    }

    if line.get(index) != Some(&b'*') {
        return line;
    }

    index += 1;
    if line.get(index) == Some(&b' ') {
        index += 1;
    }

    &line[index..]
}
