use bumpalo::Bump;
use bumpalo::collections::Vec;

use mago_span::Span;

use crate::document::Code;
use crate::document::Document;
use crate::document::Element;
use crate::document::Tag;
use crate::document::Text;
use crate::document::TextSegment;
use crate::error::ParseError;

use super::token::Token;

pub fn parse_document<'arena>(
    span: Span,
    tokens: &[Token<'arena>],
    arena: &'arena Bump,
) -> Result<Document<'arena>, ParseError> {
    let mut elements = Vec::new_in(arena);
    let mut i = 0;

    while i < tokens.len() {
        match &tokens[i] {
            Token::Line { content, .. } => {
                if content.starts_with('@') {
                    let (tag, new_i) = parse_tag(tokens, i, arena)?;
                    elements.push(Element::Tag(tag));
                    i = new_i;
                } else if content.starts_with("```") {
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

    Ok(Document { span, elements })
}

fn is_indented_line(content: &str) -> bool {
    content.starts_with(' ') || content.starts_with('\t')
}

fn parse_tag<'arena>(
    tokens: &[Token<'arena>],
    start_index: usize,
    arena: &'arena Bump,
) -> Result<(Tag<'arena>, usize), ParseError> {
    let mut i = start_index;
    let Token::Line { content, span } = &tokens[i] else {
        return Err(ParseError::ExpectedLine(tokens[i].span()));
    };

    let mut name_end = None;
    for (byte_pos, ch) in content[1..].char_indices() {
        if ch.is_whitespace() || ch == '(' {
            name_end = Some(1 + byte_pos);
            break;
        }
    }

    let tag_name = if let Some(end) = name_end { &content[1..end] } else { &content[1..] };

    if tag_name.is_empty()
        || !tag_name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-' || b == b':' || b == b'\\' || b >= 0x80)
    {
        return Err(ParseError::InvalidTagName(span.subspan(0, tag_name.len() as u32 + 1)));
    }

    let after_name = &content[1 + tag_name.len()..];

    let mut metadata: Option<&'arena str> = None;
    let mut after_metadata = after_name;
    if after_name.starts_with('(') {
        let mut depth: usize = 0;
        let mut close_pos = None;
        for (pos, ch) in after_name.char_indices() {
            match ch {
                '(' => depth += 1,
                ')' => {
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
            metadata = Some(arena.alloc_str(&after_name[..=close]));
            after_metadata = &after_name[close + 1..];
        }
    }

    let description_part;
    let description_start;
    let trimmed = after_metadata.trim_start();
    if trimmed.is_empty() {
        description_part = "";
        description_start = span.start.forward(content.len() as u32);
    } else {
        let offset = content.len() - trimmed.len();
        description_part = trimmed;
        description_start = span.start.forward(offset as u32);
    }

    let mut description = String::from(description_part);
    let mut end_span = *span;

    i += 1;
    while i < tokens.len() {
        match &tokens[i] {
            Token::Line { content, span } => {
                if content.is_empty()
                    || content.trim().is_empty()
                    || content.starts_with('@')
                    || content.starts_with("```")
                {
                    break;
                }
                description.push('\n');
                description.push_str(content);
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
        description: arena.alloc_str(&description),
        description_span: Span::new(span.file_id, description_start, end_span.end),
    };

    Ok((tag, i))
}

fn parse_code_block<'arena>(
    tokens: &[Token<'arena>],
    start_index: usize,
    arena: &'arena Bump,
) -> Result<(Code<'arena>, usize), ParseError> {
    let mut i = start_index;
    let Token::Line { content, span } = &tokens[i] else {
        return Err(ParseError::ExpectedLine(tokens[i].span()));
    };

    let mut directives = Vec::new_in(arena);
    let rest = &content[3..].trim();
    if !rest.is_empty() {
        directives = Vec::from_iter_in(rest.split(',').map(str::trim), arena);
    }

    let mut code_content = String::new();
    let mut end_span = *span;
    i += 1;

    let mut found_closing = false;
    while i < tokens.len() {
        match &tokens[i] {
            Token::Line { content, span } => {
                if content.starts_with("```") {
                    found_closing = true;
                    end_span = *span;
                    i += 1;
                    break;
                }
                if !code_content.is_empty() {
                    code_content.push('\n');
                }
                code_content.push_str(content);
                end_span = *span;
                i += 1;
            }
            Token::EmptyLine { span } => {
                code_content.push('\n');
                end_span = *span;
                i += 1;
            }
        }
    }

    let code_span = Span::new(span.file_id, span.start, end_span.end);
    if !found_closing {
        return Err(ParseError::UnclosedCodeBlock(code_span));
    }

    Ok((Code { span: code_span, directives, content: arena.alloc_str(&code_content) }, i))
}

fn parse_indented_code<'arena>(
    tokens: &[Token<'arena>],
    start_index: usize,
    arena: &'arena Bump,
) -> Result<(Code<'arena>, usize), ParseError> {
    let mut i = start_index;
    let Token::Line { content, span } = &tokens[i] else {
        return Err(ParseError::ExpectedLine(tokens[i].span()));
    };

    let indent_len = content.chars().take_while(|c| c.is_whitespace()).count();

    let mut code_content = String::new();
    let mut end_span = *span;

    while i < tokens.len() {
        match &tokens[i] {
            Token::Line { content, span } => {
                if content.starts_with('@') || content.starts_with("```") {
                    break;
                }
                let current_indent_len = content.chars().take_while(|c| c.is_whitespace()).count();
                if current_indent_len < indent_len {
                    break;
                }

                // Calculate byte offset from original indent character count
                let current_indent_bytes: usize = content.chars().take(indent_len).map(|c| c.len_utf8()).sum();
                let line_content = &content[current_indent_bytes..];
                if !code_content.is_empty() {
                    code_content.push('\n');
                }
                code_content.push_str(line_content);
                end_span = *span;
                i += 1;
            }
            Token::EmptyLine { span } => {
                code_content.push('\n');
                end_span = *span;
                i += 1;
            }
        }
    }

    Ok((
        Code {
            span: Span::new(span.file_id, span.start, end_span.end),
            directives: Vec::new_in(arena),
            content: arena.alloc_str(&code_content),
        },
        i,
    ))
}

fn parse_text<'arena>(
    tokens: &[Token<'arena>],
    start_index: usize,
    arena: &'arena Bump,
) -> Result<(Text<'arena>, usize), ParseError> {
    let mut i = start_index;
    let mut text_content = String::new();
    let start_span = tokens[start_index].span();

    let mut end_span = start_span;
    let mut open_braces: usize = 0;

    while i < tokens.len() {
        match &tokens[i] {
            Token::Line { content, span } => {
                if open_braces == 0
                    && (content.is_empty()
                        || content.trim().is_empty()
                        || content.starts_with('@')
                        || content.starts_with("```")
                        || is_indented_line(content))
                {
                    break;
                }

                if !text_content.is_empty() {
                    text_content.push('\n');
                }

                text_content.push_str(content);
                end_span = *span;
                i += 1;

                for ch in content.chars() {
                    match ch {
                        '{' => open_braces += 1,
                        '}' => open_braces = open_braces.saturating_sub(1),
                        _ => {}
                    }
                }
            }
            Token::EmptyLine { .. } => {
                if open_braces > 0 {
                    if !text_content.is_empty() {
                        text_content.push('\n');
                    }
                    end_span = tokens[i].span();
                    i += 1;
                } else {
                    break;
                }
            }
        }
    }

    // Now parse text_content into TextSegments
    let text_span = Span::new(start_span.file_id, start_span.start, end_span.end);
    let segments = parse_text_segments(arena.alloc_str(&text_content), text_span, arena)?;

    let text = Text { span: text_span, segments };

    Ok((text, i))
}

fn parse_text_segments<'arena>(
    text_content: &'arena str,
    base_span: Span,
    arena: &'arena Bump,
) -> Result<Vec<'arena, TextSegment<'arena>>, ParseError> {
    let mut segments = Vec::new_in(arena);
    let mut char_indices = text_content.char_indices().peekable();

    while let Some((start_pos, ch)) = char_indices.peek().copied() {
        if ch == '`' {
            let is_start = start_pos == 0;
            let is_prev_whitespace = if start_pos > 0 {
                text_content[..start_pos].chars().next_back().is_some_and(|c| c.is_ascii_whitespace())
            } else {
                false
            };

            if is_start || is_prev_whitespace {
                let mut backtick_count = 0;
                let mut end_pos = start_pos;

                while let Some((idx, ch)) = char_indices.peek() {
                    if *ch == '`' {
                        backtick_count += 1;
                        end_pos = *idx + ch.len_utf8();
                        char_indices.next();
                    } else {
                        break;
                    }
                }

                let backticks = "`".repeat(backtick_count);
                let code_start_pos = end_pos;

                let mut code_end_pos = None;
                while let Some((idx, _)) = char_indices.peek() {
                    if text_content[*idx..].starts_with(&backticks) {
                        code_end_pos = Some(*idx);
                        for _ in 0..backtick_count {
                            char_indices.next();
                        }
                        break;
                    }
                    char_indices.next();
                }

                if let Some(code_end_pos) = code_end_pos {
                    let code_content = &text_content[code_start_pos..code_end_pos];
                    let code_span = base_span.subspan(start_pos as u32, code_end_pos as u32 + backtick_count as u32);

                    let code = Code { span: code_span, directives: Vec::new_in(arena), content: code_content };

                    segments.push(TextSegment::InlineCode(code));
                } else {
                    return Err(ParseError::UnclosedInlineCode(
                        base_span.subspan(start_pos as u32, base_span.length()),
                    ));
                }
                continue;
            }
        }

        if text_content[start_pos..].starts_with("{@") {
            let is_start = start_pos == 0;
            let is_prev_whitespace = if start_pos > 0 {
                text_content[..start_pos].chars().next_back().is_some_and(|c| c.is_ascii_whitespace())
            } else {
                false
            };

            if is_start || is_prev_whitespace {
                let tag_start_pos = start_pos;
                char_indices.next(); // Skip '{'
                char_indices.next(); // Skip '@'

                let tag_content_start = tag_start_pos + 2;
                let mut tag_end_pos = None;
                for (idx, ch) in char_indices.by_ref() {
                    if ch == '}' {
                        tag_end_pos = Some(idx);
                        break;
                    }
                }

                if let Some(tag_end_pos) = tag_end_pos {
                    let tag_content = &text_content[tag_content_start..tag_end_pos];
                    let tag_span = base_span.subspan(tag_start_pos as u32, tag_end_pos as u32 + 1);
                    let tag = parse_inline_tag(tag_content, tag_span);
                    segments.push(TextSegment::InlineTag(tag));
                } else {
                    // Unclosed inline tag
                    return Err(ParseError::UnclosedInlineTag(base_span.subspan(start_pos as u32, base_span.length())));
                }
                continue;
            }
        }

        let paragraph_start_pos = start_pos;
        let mut paragraph_end_pos = start_pos;

        while let Some((idx, ch)) = char_indices.peek().copied() {
            let is_code_start = ch == '`' && {
                let is_start = idx == 0;
                let is_prev_whitespace = if idx > 0 {
                    text_content[..idx].chars().next_back().is_some_and(|c| c.is_ascii_whitespace())
                } else {
                    false
                };

                is_start || is_prev_whitespace
            };

            let is_tag_start = text_content[idx..].starts_with("{@") && {
                let is_start = idx == 0;
                let is_prev_whitespace = if idx > 0 {
                    text_content[..idx].chars().next_back().is_some_and(|c| c.is_ascii_whitespace())
                } else {
                    false
                };

                is_start || is_prev_whitespace
            };

            if is_code_start || is_tag_start {
                break;
            }
            char_indices.next();
            paragraph_end_pos = idx + ch.len_utf8();
        }

        let paragraph_content = &text_content[paragraph_start_pos..paragraph_end_pos];

        segments.push(TextSegment::Paragraph { span: base_span, content: paragraph_content });
    }

    Ok(segments)
}

fn parse_inline_tag(tag_content: &str, span: Span) -> Tag<'_> {
    let mut parts = tag_content.trim().splitn(2, char::is_whitespace);
    let name = parts.next().unwrap_or("");
    let description = parts.next().unwrap_or("");

    Tag {
        span,
        name,
        kind: name.into(),
        metadata: None,
        description,
        description_span: Span::new(span.file_id, span.start.forward(name.len() as u32 + 1), span.end),
    }
}
