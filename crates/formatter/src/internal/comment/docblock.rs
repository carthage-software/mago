use bumpalo::Bump;
use bumpalo::collections::Vec as BumpVec;

use mago_docblock::document::Element;
use mago_docblock::document::TagKind;
use mago_docblock::tag::split_tag_content;
use mago_span::Span;

use crate::document::BreakMode;
use crate::document::Document;
use crate::document::Group;
use crate::document::Line;
use crate::settings::FormatSettings;
use crate::settings::PhpdocAlign;
use crate::settings::PhpdocNullPosition;

/// A parsed tag line ready for reconstruction.
struct TagLine<'a> {
    tag_name: &'a str,
    type_str: Option<String>,
    variable: Option<String>,
    description: String,
    priority: u8,
    original_index: usize,
    raw_lines: Option<std::vec::Vec<String>>,
}

/// Attempt to reformat a docblock comment, returning a `Document` IR.
///
/// Returns `Some(Document)` with the reformatted docblock, or `None` on parse
/// failure so the caller falls through to default formatting.
///
/// An empty `Document::String("")` signals an empty docblock.
pub(crate) fn reformat_docblock<'arena>(
    arena: &'arena Bump,
    settings: &FormatSettings,
    content: &str,
    span: Span,
) -> Option<Document<'arena>> {
    if !content.starts_with("/**") || !content.contains('\n') {
        return None;
    }

    let arena_content = arena.alloc_str(content);
    let doc = mago_docblock::parse_phpdoc_with_span(arena, arena_content, span).ok()?;

    let has_code_block = doc.elements.iter().any(|e| matches!(e, Element::Code(_)));
    if has_code_block {
        return None;
    }

    let mut description_lines: std::vec::Vec<String> = std::vec::Vec::new();
    let mut tag_lines: std::vec::Vec<TagLine<'_>> = std::vec::Vec::new();
    let mut tag_index: usize = 0;
    let mut seen_tag = false;

    for element in doc.elements.iter() {
        match element {
            Element::Text(text) => {
                if !seen_tag {
                    for segment in text.segments.iter() {
                        match segment {
                            mago_docblock::document::TextSegment::Paragraph { content: para, .. } => {
                                for line in para.lines() {
                                    let trimmed = line.trim();
                                    if !trimmed.is_empty() {
                                        description_lines.push(trimmed.to_string());
                                    }
                                }
                            }
                            mago_docblock::document::TextSegment::InlineCode(code) => {
                                description_lines.push(format!("`{}`", code.content));
                            }
                            mago_docblock::document::TextSegment::InlineTag(tag) => {
                                let desc = tag.description.trim();
                                if desc.is_empty() {
                                    description_lines.push(format!("{{@{}}}", tag.name));
                                } else {
                                    description_lines
                                        .push(format!("{{@{} {}}}", tag.name, desc));
                                }
                            }
                        }
                    }
                }
            }
            Element::Tag(tag) => {
                seen_tag = true;

                let tag_name_str = tag.name;
                let kind = tag.kind;
                let priority = tag_priority(kind);
                let desc = tag.description.trim();

                if desc.contains('\n') {
                    let raw = build_multiline_tag_raw(tag_name_str, desc, settings);
                    tag_lines.push(TagLine {
                        tag_name: arena.alloc_str(&format!("@{}", tag_name_str)),
                        type_str: None,
                        variable: None,
                        description: String::new(),
                        priority,
                        original_index: tag_index,
                        raw_lines: Some(raw),
                    });
                    tag_index += 1;
                    continue;
                }

                let (type_str, variable, rest) = parse_tag_parts(desc, kind, tag.description_span);

                let type_str = type_str.map(|t| {
                    if settings.phpdoc_null_position == PhpdocNullPosition::Last {
                        move_null_to_end(&t)
                    } else {
                        t
                    }
                });

                tag_lines.push(TagLine {
                    tag_name: arena.alloc_str(&format!("@{}", tag_name_str)),
                    type_str,
                    variable,
                    description: rest,
                    priority,
                    original_index: tag_index,
                    raw_lines: None,
                });
                tag_index += 1;
            }
            Element::Line(_) => {
                if !seen_tag && !description_lines.is_empty() {
                    description_lines.push(String::new());
                }
            }
            Element::Annotation(ann) => {
                if !seen_tag {
                    if let Some(args) = ann.arguments {
                        description_lines.push(format!("@{}({})", ann.name, args));
                    } else {
                        description_lines.push(format!("@{}", ann.name));
                    }
                }
            }
            _ => {}
        }
    }

    // Trim trailing empty lines from description
    while description_lines.last().is_some_and(|l| l.is_empty()) {
        description_lines.pop();
    }

    // Collapse consecutive blank lines
    let mut deduped_desc: std::vec::Vec<String> = std::vec::Vec::new();
    let mut prev_blank = false;
    for line in description_lines {
        if line.is_empty() {
            if !prev_blank {
                deduped_desc.push(line);
            }
            prev_blank = true;
        } else {
            deduped_desc.push(line);
            prev_blank = false;
        }
    }
    let description_lines = deduped_desc;

    if description_lines.is_empty() && tag_lines.is_empty() {
        return Some(Document::String(""));
    }

    // Sort tags by priority (stable)
    tag_lines.sort_by(|a, b| a.priority.cmp(&b.priority).then(a.original_index.cmp(&b.original_index)));

    // Build Document IR
    let mut parts = BumpVec::with_capacity_in(
        2 + description_lines.len() * 2 + tag_lines.len() * 2 + 4,
        arena,
    );

    parts.push(Document::String("/**"));

    for line in &description_lines {
        parts.push(Document::Line(Line::hard()));
        if line.is_empty() {
            parts.push(Document::String(" *"));
        } else {
            parts.push(Document::String(arena.alloc_str(&format!(" * {}", line))));
        }
    }

    // Separator between description and tags
    if !description_lines.is_empty() && !tag_lines.is_empty() {
        parts.push(Document::Line(Line::hard()));
        parts.push(Document::String(" *"));
    }

    if !tag_lines.is_empty() {
        let formatted_tags = if settings.phpdoc_align == PhpdocAlign::Vertical {
            format_tags_aligned(&tag_lines)
        } else {
            format_tags_unaligned(&tag_lines)
        };

        for (i, line) in formatted_tags.iter().enumerate() {
            if i > 0 && tag_lines[i].priority != tag_lines[i - 1].priority {
                parts.push(Document::Line(Line::hard()));
                parts.push(Document::String(" *"));
            }

            // Multi-line tag lines contain newlines; split into separate Document entries
            if line.contains('\n') {
                for (j, sub_line) in line.split('\n').enumerate() {
                    if j > 0 {
                        parts.push(Document::Line(Line::hard()));
                    } else {
                        parts.push(Document::Line(Line::hard()));
                    }
                    parts.push(Document::String(arena.alloc_str(&format!(" * {}", sub_line))));
                }
            } else {
                parts.push(Document::Line(Line::hard()));
                parts.push(Document::String(arena.alloc_str(&format!(" * {}", line))));
            }
        }
    }

    parts.push(Document::Line(Line::hard()));
    parts.push(Document::String(" */"));

    Some(Document::Group(Group::new(parts).with_break_mode(BreakMode::Force)))
}

/// Build raw lines for a multi-line tag, preserving structure.
fn build_multiline_tag_raw(tag_name: &str, desc: &str, settings: &FormatSettings) -> std::vec::Vec<String> {
    let mut lines: std::vec::Vec<String> = std::vec::Vec::new();

    let (first_line_desc, rest_lines) = if let Some(nl_pos) = desc.find('\n') {
        (&desc[..nl_pos], &desc[nl_pos + 1..])
    } else {
        (desc, "")
    };

    let mut first = first_line_desc.to_string();
    if settings.phpdoc_null_position == PhpdocNullPosition::Last {
        first = move_null_to_end(&first);
    }

    lines.push(format!("@{} {}", tag_name, first.trim()));

    for line in rest_lines.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            lines.push(String::new());
        } else {
            lines.push(format!("  {}", trimmed));
        }
    }

    lines
}

fn tag_priority(kind: TagKind) -> u8 {
    match kind {
        TagKind::Param | TagKind::PsalmParam | TagKind::PhpstanParam => 0,
        TagKind::Return | TagKind::PsalmReturn | TagKind::PhpstanReturn => 1,
        TagKind::Throws => 2,
        _ => 3,
    }
}

/// Parse tag description into (type, variable, rest) parts using mago_docblock's
/// `split_tag_content` for type extraction.
fn parse_tag_parts(desc: &str, kind: TagKind, desc_span: Span) -> (Option<String>, Option<String>, String) {
    let desc = desc.trim();
    if desc.is_empty() {
        return (None, None, String::new());
    }

    match kind {
        TagKind::Param | TagKind::PsalmParam | TagKind::PhpstanParam => {
            if let Some((type_string, rest)) = split_tag_content(desc, desc_span) {
                let rest = rest.trim_start();
                let (var, remainder) = split_variable(rest);
                (Some(type_string.value), var, remainder.trim_start().to_string())
            } else {
                let (var, remainder) = split_variable(desc);
                (None, var, remainder.trim_start().to_string())
            }
        }
        TagKind::Return | TagKind::PsalmReturn | TagKind::PhpstanReturn | TagKind::Throws => {
            if let Some((type_string, rest)) = split_tag_content(desc, desc_span) {
                (Some(type_string.value), None, rest.trim_start().to_string())
            } else {
                (None, None, desc.to_string())
            }
        }
        TagKind::Var | TagKind::PsalmVar | TagKind::PhpstanVar | TagKind::Type => {
            if let Some((type_string, rest)) = split_tag_content(desc, desc_span) {
                let rest = rest.trim_start();
                let (var, remainder) = split_variable(rest);
                (Some(type_string.value), var, remainder.trim_start().to_string())
            } else {
                (None, None, desc.to_string())
            }
        }
        TagKind::Property | TagKind::PropertyRead | TagKind::PropertyWrite => {
            if let Some((type_string, rest)) = split_tag_content(desc, desc_span) {
                let rest = rest.trim_start();
                let (var, remainder) = split_variable(rest);
                (Some(type_string.value), var, remainder.trim_start().to_string())
            } else {
                (None, None, desc.to_string())
            }
        }
        _ => (None, None, desc.to_string()),
    }
}

/// Split a leading `$variable` (possibly with `...` or `&` prefix) from the rest.
fn split_variable(s: &str) -> (Option<String>, &str) {
    let s = s.trim_start();

    let mut start = 0;
    let bytes = s.as_bytes();
    let len = bytes.len();

    while start < len && (bytes[start] == b'.' || bytes[start] == b'&') {
        start += 1;
    }

    if start >= len || bytes[start] != b'$' {
        return (None, s);
    }

    let mut end = start + 1;
    while end < len && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
        end += 1;
    }

    let var = &s[..end];
    let rest = &s[end..];
    (Some(var.to_string()), rest)
}

/// Move `null` to the end of a union type string.
fn move_null_to_end(type_str: &str) -> String {
    if type_str.starts_with('?') || !type_str.contains('|') {
        return type_str.to_string();
    }

    let parts = split_top_level(type_str, '|');
    if parts.len() <= 1 {
        return type_str.to_string();
    }

    let mut non_null: std::vec::Vec<&str> = std::vec::Vec::new();
    let mut has_null = false;

    for part in &parts {
        if part.trim() == "null" {
            has_null = true;
        } else {
            non_null.push(part.trim());
        }
    }

    if !has_null {
        return type_str.to_string();
    }

    non_null.push("null");
    non_null.join("|")
}

/// Split a string on a delimiter, but only at the top level (depth 0).
fn split_top_level(s: &str, delim: char) -> std::vec::Vec<&str> {
    let mut parts = std::vec::Vec::new();
    let mut depth = 0i32;
    let mut start = 0;

    for (i, c) in s.char_indices() {
        match c {
            '<' | '(' | '[' | '{' => depth += 1,
            '>' | ')' | ']' | '}' => depth -= 1,
            c if c == delim && depth == 0 => {
                parts.push(&s[start..i]);
                start = i + c.len_utf8();
            }
            _ => {}
        }
    }
    parts.push(&s[start..]);
    parts
}

/// Format tag lines without alignment (single spaces).
fn format_tags_unaligned(tags: &[TagLine<'_>]) -> std::vec::Vec<String> {
    tags.iter()
        .map(|tag| {
            if let Some(ref raw) = tag.raw_lines {
                return raw.join("\n");
            }
            let mut line = tag.tag_name.to_string();
            if let Some(ref t) = tag.type_str {
                line.push(' ');
                line.push_str(t);
            }
            if let Some(ref v) = tag.variable {
                line.push(' ');
                line.push_str(v);
            }
            if !tag.description.is_empty() {
                line.push(' ');
                line.push_str(&tag.description);
            }
            line
        })
        .collect()
}

/// Format tag lines with vertical alignment of type, variable, and description columns.
fn format_tags_aligned(tags: &[TagLine<'_>]) -> std::vec::Vec<String> {
    if tags.is_empty() {
        return std::vec::Vec::new();
    }

    let mut result: std::vec::Vec<String> = std::vec::Vec::with_capacity(tags.len());
    let mut group_start = 0;

    while group_start < tags.len() {
        let group_priority = tags[group_start].priority;
        let mut group_end = group_start + 1;
        while group_end < tags.len() && tags[group_end].priority == group_priority {
            group_end += 1;
        }

        let group = &tags[group_start..group_end];

        let alignable: std::vec::Vec<&TagLine<'_>> =
            group.iter().filter(|t| t.raw_lines.is_none()).collect();

        let max_tag_width = alignable.iter().map(|t| t.tag_name.len()).max().unwrap_or(0);
        let max_type_width = alignable
            .iter()
            .map(|t| t.type_str.as_ref().map_or(0, |s| s.len()))
            .max()
            .unwrap_or(0);
        let max_var_width = alignable
            .iter()
            .map(|t| t.variable.as_ref().map_or(0, |s| s.len()))
            .max()
            .unwrap_or(0);

        let has_types = alignable.iter().any(|t| t.type_str.is_some());
        let has_vars = alignable.iter().any(|t| t.variable.is_some());

        for tag in group {
            if let Some(ref raw) = tag.raw_lines {
                result.push(raw.join("\n"));
                continue;
            }

            let mut line = String::with_capacity(80);
            line.push_str(tag.tag_name);

            if has_types {
                let padding = max_tag_width - tag.tag_name.len();
                line.push(' ');
                if let Some(ref t) = tag.type_str {
                    line.push_str(t);
                    if has_vars || !tag.description.is_empty() {
                        let type_padding = max_type_width - t.len();
                        for _ in 0..type_padding {
                            line.push(' ');
                        }
                    }
                } else if has_vars || !tag.description.is_empty() {
                    for _ in 0..max_type_width {
                        line.push(' ');
                    }
                }
                for _ in 0..padding {
                    line.push(' ');
                }
            }

            if has_vars {
                line.push(' ');
                if let Some(ref v) = tag.variable {
                    line.push_str(v);
                    if !tag.description.is_empty() {
                        let var_padding = max_var_width - v.len();
                        for _ in 0..var_padding {
                            line.push(' ');
                        }
                    }
                } else if !tag.description.is_empty() {
                    for _ in 0..max_var_width {
                        line.push(' ');
                    }
                }
            }

            if !tag.description.is_empty() {
                line.push(' ');
                line.push_str(&tag.description);
            }

            let trimmed = line.trim_end().to_string();
            result.push(trimmed);
        }

        group_start = group_end;
    }

    result
}
