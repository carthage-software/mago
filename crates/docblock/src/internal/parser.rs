use bumpalo::Bump;
use bumpalo::collections::Vec;

use mago_span::Span;

use crate::document::*;
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
                if let Some(stripped) = content.strip_prefix('@') {
                    if is_annotation_start(stripped) {
                        let (annotation, new_i) = parse_annotation(tokens, i, arena)?;
                        elements.push(Element::Annotation(annotation));
                        i = new_i;
                    } else {
                        let (tag, new_i) = parse_tag(tokens, i, arena)?;
                        elements.push(Element::Tag(tag));
                        i = new_i;
                    }
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

    Ok(Document { elements, span })
}

fn is_indented_line(content: &str) -> bool {
    content.starts_with(' ') || content.starts_with('\t')
}

fn is_annotation_start(s: &str) -> bool {
    if s.starts_with('\\') {
        true
    } else if let Some(first_char) = s.chars().next() {
        first_char.is_ascii_uppercase() || first_char == '_'
    } else {
        false
    }
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

    let next_whitespace = content[1..].find(char::is_whitespace);

    let tag_name;
    let description_part;
    let description_start;
    if let Some(position) = next_whitespace {
        tag_name = &content[1..position + 1];
        description_part = &content[position + 2..];
        description_start = span.start.forward(2 + position as u32); // 1 for '@' and 1 for whitespace
    } else {
        tag_name = &content[1..];
        description_part = "";
        description_start = span.start.forward(1 + tag_name.len() as u32);
    }

    if tag_name.is_empty() || !tag_name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == ':') {
        return Err(ParseError::InvalidTagName(span.subspan(0, tag_name.len() as u32 + 1)));
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
                } else {
                    description.push('\n');
                    description.push_str(content);
                    end_span = *span;
                    i += 1;
                }
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
                } else {
                    if !code_content.is_empty() {
                        code_content.push('\n');
                    }
                    code_content.push_str(content);
                    end_span = *span;
                    i += 1;
                }
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
                } else {
                    let line_content = &content[indent_len..];
                    if !code_content.is_empty() {
                        code_content.push('\n');
                    }
                    code_content.push_str(line_content);
                    end_span = *span;
                    i += 1;
                }
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

    while i < tokens.len() {
        match &tokens[i] {
            Token::Line { content, span } => {
                if content.is_empty()
                    || content.trim().is_empty()
                    || content.starts_with('@')
                    || content.starts_with("```")
                    || is_indented_line(content)
                {
                    break;
                } else {
                    if !text_content.is_empty() {
                        text_content.push('\n');
                    }
                    text_content.push_str(content);
                    end_span = *span;
                    i += 1;
                }
            }
            Token::EmptyLine { .. } => {
                break;
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

    while let Some((start_pos, ch)) = char_indices.peek().cloned() {
        if ch == '`' {
            let is_start = start_pos == 0;
            let is_prev_whitespace = if start_pos > 0 {
                text_content[..start_pos].chars().next_back().map(|c| c.is_ascii_whitespace()).unwrap_or(false)
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
                    } else {
                        char_indices.next();
                    }
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
                text_content[..start_pos].chars().next_back().map(|c| c.is_ascii_whitespace()).unwrap_or(false)
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
                    let tag = parse_inline_tag(tag_content, tag_span)?;
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

        while let Some((idx, ch)) = char_indices.peek().cloned() {
            let is_code_start = ch == '`' && {
                let is_start = idx == 0;
                let is_prev_whitespace = if idx > 0 {
                    text_content[..idx].chars().next_back().map(|c| c.is_ascii_whitespace()).unwrap_or(false)
                } else {
                    false
                };

                is_start || is_prev_whitespace
            };

            let is_tag_start = text_content[idx..].starts_with("{@") && {
                let is_start = idx == 0;
                let is_prev_whitespace = if idx > 0 {
                    text_content[..idx].chars().next_back().map(|c| c.is_ascii_whitespace()).unwrap_or(false)
                } else {
                    false
                };

                is_start || is_prev_whitespace
            };

            if is_code_start || is_tag_start {
                break;
            } else {
                char_indices.next();
                paragraph_end_pos = idx + ch.len_utf8();
            }
        }

        let paragraph_content = &text_content[paragraph_start_pos..paragraph_end_pos];

        segments.push(TextSegment::Paragraph { span: base_span, content: paragraph_content });
    }

    Ok(segments)
}

fn parse_inline_tag<'arena>(tag_content: &'arena str, span: Span) -> Result<Tag<'arena>, ParseError> {
    let mut parts = tag_content.trim().splitn(2, char::is_whitespace);
    let name = parts.next().unwrap_or("");
    let description = parts.next().unwrap_or("");

    Ok(Tag {
        span,
        name,
        kind: name.into(),
        description,
        description_span: Span::new(span.file_id, span.start.forward(name.len() as u32 + 1), span.end),
    })
}

fn parse_annotation<'arena>(
    tokens: &[Token<'arena>],
    start_index: usize,
    arena: &'arena Bump,
) -> Result<(Annotation<'arena>, usize), ParseError> {
    let mut i = start_index;
    let Token::Line { content, span } = &tokens[i] else {
        return Err(ParseError::ExpectedLine(tokens[i].span()));
    };

    let content_after_at = &content[1..]; // Skip '@'

    let (name, name_len) = parse_annotation_name(content_after_at, *span)?;

    let content_rest = &content[1 + name_len..];
    let mut arguments: Option<&'arena str> = None;
    let mut end_span = *span;

    if content_rest.trim_start().starts_with('(') {
        let mut args = String::new();
        let mut open_parens = 0;

        let paren_start_pos = content_rest.find('(').unwrap();
        let line_content = content_rest[paren_start_pos..].trim_end();

        args.push_str(line_content);
        open_parens += line_content.chars().filter(|&c| c == '(').count();
        open_parens -= line_content.chars().filter(|&c| c == ')').count();

        i += 1;
        end_span = *span;

        while open_parens > 0 && i < tokens.len() {
            match &tokens[i] {
                Token::Line { content, span } => {
                    args.push('\n');
                    args.push_str(content);
                    end_span = *span;
                    open_parens += content.chars().filter(|&c| c == '(').count();
                    open_parens -= content.chars().filter(|&c| c == ')').count();
                    i += 1;
                }
                Token::EmptyLine { .. } => {
                    args.push('\n');
                    i += 1;
                }
            }
        }

        if open_parens != 0 {
            return Err(ParseError::UnclosedAnnotationArguments(Span::new(span.file_id, span.start, end_span.end)));
        }

        arguments = Some(arena.alloc_str(&args));
    } else {
        i += 1;
    }

    let annotation_span = Span::new(span.file_id, span.start, end_span.end);

    let annotation = Annotation { span: annotation_span, name, arguments };

    Ok((annotation, i))
}

/// Parses an annotation name from the beginning of a string slice.
///
/// This is a zero-allocation function. It returns a slice (`&'a str`) that
/// points directly into the input `content` without creating a new String.
///
/// # Returns
///
/// A `Result` containing a tuple of the name slice and the number of bytes consumed,
/// or a `ParseError` if the name is invalid.
pub fn parse_annotation_name(content: &str, span: Span) -> Result<(&str, usize), ParseError> {
    let mut chars = content.char_indices();
    let mut last_valid_byte_index = 0;

    if let Some((_, c)) = chars.next() {
        if c == '\\' || c.is_ascii_uppercase() || c == '_' {
            last_valid_byte_index += c.len_utf8();
        } else {
            // The first character is invalid.
            return Err(ParseError::InvalidAnnotationName(span.subspan(1, 1)));
        }
    } else {
        // The input string is empty.
        return Err(ParseError::InvalidAnnotationName(span.subspan(1, 1)));
    }

    for (i, c) in chars {
        if c.is_ascii_alphanumeric() || c == '_' || c == '\\' || c as u8 >= 0x80 {
            // If the character is valid, update the index to the position *after* it.
            last_valid_byte_index = i + c.len_utf8();
        } else {
            // Stop at the first invalid character.
            break;
        }
    }

    // Create a slice of the input string from the start to the last valid position.
    let name_slice = &content[..last_valid_byte_index];

    Ok((name_slice, last_valid_byte_index))
}
