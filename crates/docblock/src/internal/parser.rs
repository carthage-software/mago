use mago_allocator::prelude::*;

use mago_span::Span;

use crate::document::Code;
use crate::document::Document;
use crate::document::Element;
use crate::document::Tag;
use crate::document::Text;
use crate::document::TextSegment;
use crate::error::ParseError;

use super::token::Token;

pub fn parse_document<'arena, A>(
    span: Span,
    tokens: &[Token<'arena>],
    arena: &'arena A,
) -> Result<Document<'arena>, ParseError>
where
    A: Arena,
{
    let mut elements = Vec::new_in(arena);
    let mut i = 0;

    while i < tokens.len() {
        match &tokens[i] {
            Token::Line { content, .. } => {
                if content.starts_with(b"@") {
                    let (tag, new_i) = parse_tag(tokens, i, arena)?;
                    elements.push(Element::Tag(tag));
                    i = new_i;
                } else if content.starts_with(b"```") {
                    let (code, new_i) = parse_code_block(tokens, i, arena)?;
                    elements.push(Element::Code(code));
                    i = new_i;
                } else if is_indented_line(content) {
                    let (code, new_i) = parse_indented_code(tokens, i, arena)?;
                    elements.push(Element::Code(code));
                    i = new_i;
                } else {
                    let (text, new_i) = parse_text(tokens, i, arena)?;
                    elements.push(Element::Text(text));
                    i = new_i;
                }
            }
            Token::EmptyLine { span } => {
                elements.push(Element::Line(*span));
                i += 1;
            }
        }
    }

    Ok(Document { span, elements: elements.leak() })
}

fn is_indented_line(content: &[u8]) -> bool {
    content.starts_with(b" ") || content.starts_with(b"\t")
}

fn parse_tag<'arena, A>(
    tokens: &[Token<'arena>],
    start_index: usize,
    arena: &'arena A,
) -> Result<(Tag<'arena>, usize), ParseError>
where
    A: Arena,
{
    let mut i = start_index;
    let Token::Line { content, span } = &tokens[i] else {
        return Err(ParseError::ExpectedLine(tokens[i].span()));
    };

    let mut name_end = None;
    for (offset, &b) in content[1..].iter().enumerate() {
        if b.is_ascii_whitespace() || b == b'(' {
            name_end = Some(1 + offset);
            break;
        }
    }

    let tag_name = if let Some(end) = name_end { &content[1..end] } else { &content[1..] };

    if tag_name.is_empty()
        || !tag_name
            .iter()
            .all(|&b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-' || b == b':' || b == b'\\' || b >= 0x80)
    {
        return Err(ParseError::InvalidTagName(span.subspan(0, tag_name.len() as u32 + 1)));
    }

    let after_name = &content[1 + tag_name.len()..];

    let mut metadata: Option<&'arena [u8]> = None;
    let mut after_metadata = after_name;
    if after_name.starts_with(b"(") {
        let mut depth: usize = 0;
        let mut close_pos = None;
        for (pos, &b) in after_name.iter().enumerate() {
            match b {
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        close_pos = Some(pos);
                        break;
                    }
                }
                _ => {}
            }
        }

        if let Some(close) = close_pos {
            metadata = Some(arena.alloc_slice_copy(&after_name[..=close]));
            after_metadata = &after_name[close + 1..];
        }
    }

    let description_part;
    let description_start;
    let trimmed = after_metadata.trim_ascii_start();
    if trimmed.is_empty() {
        description_part = b"" as &[u8];
        description_start = span.start.forward(content.len() as u32);
    } else {
        let offset = content.len() - trimmed.len();
        description_part = trimmed;
        description_start = span.start.forward(offset as u32);
    }

    let mut description: std::vec::Vec<u8> = description_part.to_vec();
    let mut end_span = *span;

    i += 1;
    while i < tokens.len() {
        match &tokens[i] {
            Token::Line { content, span } => {
                if content.is_empty()
                    || content.trim_ascii().is_empty()
                    || content.starts_with(b"@")
                    || content.starts_with(b"```")
                {
                    break;
                }
                description.push(b'\n');
                description.extend_from_slice(content);
                end_span = *span;
                i += 1;
            }
            Token::EmptyLine { .. } => {
                break;
            }
        }
    }

    let kind = tag_name.into();

    let tag_span = Span::new(span.file_id, span.start, end_span.end);

    let tag = Tag {
        span: tag_span,
        name: tag_name,
        kind,
        metadata,
        description: arena.alloc_slice_copy(&description),
        description_span: Span::new(span.file_id, description_start, end_span.end),
    };

    Ok((tag, i))
}

fn parse_code_block<'arena, A>(
    tokens: &[Token<'arena>],
    start_index: usize,
    arena: &'arena A,
) -> Result<(Code<'arena>, usize), ParseError>
where
    A: Arena,
{
    let mut i = start_index;
    let Token::Line { content, span } = &tokens[i] else {
        return Err(ParseError::ExpectedLine(tokens[i].span()));
    };

    let mut directives = Vec::new_in(arena);
    let rest = content[3..].trim_ascii();
    if !rest.is_empty() {
        directives = Vec::from_iter_in(rest.split(|&b| b == b',').map(<[u8]>::trim_ascii), arena);
    }

    let mut code_content: std::vec::Vec<u8> = std::vec::Vec::new();
    let mut end_span = *span;
    i += 1;

    let mut found_closing = false;
    while i < tokens.len() {
        match &tokens[i] {
            Token::Line { content, span } => {
                if content.starts_with(b"```") {
                    found_closing = true;
                    end_span = *span;
                    i += 1;
                    break;
                }
                if !code_content.is_empty() {
                    code_content.push(b'\n');
                }
                code_content.extend_from_slice(content);
                end_span = *span;
                i += 1;
            }
            Token::EmptyLine { span } => {
                code_content.push(b'\n');
                end_span = *span;
                i += 1;
            }
        }
    }

    let code_span = Span::new(span.file_id, span.start, end_span.end);
    if !found_closing {
        return Err(ParseError::UnclosedCodeBlock(code_span));
    }

    Ok((Code { span: code_span, directives: directives.leak(), content: arena.alloc_slice_copy(&code_content) }, i))
}

fn parse_indented_code<'arena, A>(
    tokens: &[Token<'arena>],
    start_index: usize,
    arena: &'arena A,
) -> Result<(Code<'arena>, usize), ParseError>
where
    A: Arena,
{
    let mut i = start_index;
    let Token::Line { content, span } = &tokens[i] else {
        return Err(ParseError::ExpectedLine(tokens[i].span()));
    };

    let indent_len = content.iter().take_while(|b| b.is_ascii_whitespace()).count();

    let mut code_content: std::vec::Vec<u8> = std::vec::Vec::new();
    let mut end_span = *span;

    while i < tokens.len() {
        match &tokens[i] {
            Token::Line { content, span } => {
                if content.starts_with(b"@") || content.starts_with(b"```") {
                    break;
                }
                let current_indent_len = content.iter().take_while(|b| b.is_ascii_whitespace()).count();
                if current_indent_len < indent_len {
                    break;
                }

                let line_content = &content[indent_len..];
                if !code_content.is_empty() {
                    code_content.push(b'\n');
                }
                code_content.extend_from_slice(line_content);
                end_span = *span;
                i += 1;
            }
            Token::EmptyLine { span } => {
                code_content.push(b'\n');
                end_span = *span;
                i += 1;
            }
        }
    }

    Ok((
        Code {
            span: Span::new(span.file_id, span.start, end_span.end),
            directives: &[],
            content: arena.alloc_slice_copy(&code_content),
        },
        i,
    ))
}

fn parse_text<'arena, A>(
    tokens: &[Token<'arena>],
    start_index: usize,
    arena: &'arena A,
) -> Result<(Text<'arena>, usize), ParseError>
where
    A: Arena,
{
    let mut i = start_index;
    let mut text_content: std::vec::Vec<u8> = std::vec::Vec::new();
    let start_span = tokens[start_index].span();

    let mut end_span = start_span;
    let mut open_braces: usize = 0;

    while i < tokens.len() {
        match &tokens[i] {
            Token::Line { content, span } => {
                if open_braces == 0
                    && (content.is_empty()
                        || content.trim_ascii().is_empty()
                        || content.starts_with(b"@")
                        || content.starts_with(b"```")
                        || is_indented_line(content))
                {
                    break;
                }

                if !text_content.is_empty() {
                    text_content.push(b'\n');
                }

                text_content.extend_from_slice(content);
                end_span = *span;
                i += 1;

                for &b in *content {
                    match b {
                        b'{' => open_braces += 1,
                        b'}' => open_braces = open_braces.saturating_sub(1),
                        _ => {}
                    }
                }
            }
            Token::EmptyLine { .. } => {
                if open_braces > 0 {
                    if !text_content.is_empty() {
                        text_content.push(b'\n');
                    }
                    end_span = tokens[i].span();
                    i += 1;
                } else {
                    break;
                }
            }
        }
    }

    let text_span = Span::new(start_span.file_id, start_span.start, end_span.end);
    let segments = parse_text_segments(arena.alloc_slice_copy(&text_content), text_span, arena)?;

    let text = Text { span: text_span, segments: segments.leak() };

    Ok((text, i))
}

fn parse_text_segments<'arena, A>(
    text_content: &'arena [u8],
    base_span: Span,
    arena: &'arena A,
) -> Result<Vec<'arena, TextSegment<'arena>, A>, ParseError>
where
    A: Arena,
{
    let mut segments = Vec::new_in(arena);
    let mut i = 0usize;
    let len = text_content.len();

    while i < len {
        let ch = text_content[i];

        if ch == b'`' {
            let is_start = i == 0;
            let is_prev_whitespace = i > 0 && text_content[i - 1].is_ascii_whitespace();

            if is_start || is_prev_whitespace {
                let mut backtick_count = 0;
                let mut end_pos = i;

                while end_pos < len && text_content[end_pos] == b'`' {
                    backtick_count += 1;
                    end_pos += 1;
                }

                let code_start_pos = end_pos;
                let backticks_slice = &text_content[i..end_pos];

                let mut search_pos = end_pos;
                let mut code_end_pos = None;
                while search_pos < len {
                    if text_content[search_pos..].starts_with(backticks_slice) {
                        code_end_pos = Some(search_pos);
                        break;
                    }
                    search_pos += 1;
                }

                if let Some(code_end) = code_end_pos {
                    let code_content = &text_content[code_start_pos..code_end];
                    let code_span = base_span.subspan(i as u32, code_end as u32 + backtick_count as u32);

                    let code = Code { span: code_span, directives: &[], content: code_content };

                    segments.push(TextSegment::InlineCode(code));
                    i = code_end + backtick_count;
                } else {
                    return Err(ParseError::UnclosedInlineCode(base_span.subspan(i as u32, base_span.length())));
                }
                continue;
            }
        }

        if text_content[i..].starts_with(b"{@") {
            let is_start = i == 0;
            let is_prev_whitespace = i > 0 && text_content[i - 1].is_ascii_whitespace();

            if is_start || is_prev_whitespace {
                let tag_start_pos = i;
                let tag_content_start = tag_start_pos + 2;
                let mut search_pos = tag_content_start;
                let mut tag_end_pos = None;
                while search_pos < len {
                    if text_content[search_pos] == b'}' {
                        tag_end_pos = Some(search_pos);
                        break;
                    }
                    search_pos += 1;
                }

                if let Some(tag_end) = tag_end_pos {
                    let tag_content = &text_content[tag_content_start..tag_end];
                    let tag_span = base_span.subspan(tag_start_pos as u32, tag_end as u32 + 1);
                    let tag = parse_inline_tag(tag_content, tag_span);
                    segments.push(TextSegment::InlineTag(tag));
                    i = tag_end + 1;
                } else {
                    return Err(ParseError::UnclosedInlineTag(base_span.subspan(i as u32, base_span.length())));
                }
                continue;
            }
        }

        let paragraph_start_pos = i;
        let mut paragraph_end_pos = i;

        while paragraph_end_pos < len {
            let idx = paragraph_end_pos;
            let cur = text_content[idx];

            let is_code_start = cur == b'`' && {
                let is_start = idx == 0;
                let is_prev_whitespace = idx > 0 && text_content[idx - 1].is_ascii_whitespace();
                is_start || is_prev_whitespace
            };

            let is_tag_start = text_content[idx..].starts_with(b"{@") && {
                let is_start = idx == 0;
                let is_prev_whitespace = idx > 0 && text_content[idx - 1].is_ascii_whitespace();
                is_start || is_prev_whitespace
            };

            if is_code_start || is_tag_start {
                break;
            }
            paragraph_end_pos += 1;
        }

        let paragraph_content = &text_content[paragraph_start_pos..paragraph_end_pos];

        segments.push(TextSegment::Paragraph { span: base_span, content: paragraph_content });
        i = paragraph_end_pos;
    }

    Ok(segments)
}

fn parse_inline_tag(tag_content: &[u8], span: Span) -> Tag<'_> {
    let trimmed = tag_content.trim_ascii();
    let (name, description) = match trimmed.iter().position(|b| b.is_ascii_whitespace()) {
        Some(idx) => (&trimmed[..idx], trimmed[idx + 1..].trim_ascii_start()),
        None => (trimmed, b"" as &[u8]),
    };

    Tag {
        span,
        name,
        kind: name.into(),
        metadata: None,
        description,
        description_span: Span::new(span.file_id, span.start.forward(name.len() as u32 + 1), span.end),
    }
}
