use bumpalo::Bump;

use mago_docblock::document::Element;
use mago_docblock::document::TagKind;
use mago_span::Span;

use crate::settings::FormatSettings;
use crate::settings::PhpdocAlign;
use crate::settings::PhpdocNullPosition;

/// A parsed tag line ready for reconstruction.
struct TagLine<'a> {
    /// The tag name including `@`, e.g. `@param`.
    tag_name: &'a str,
    /// The type portion, e.g. `string|null`. Already normalized.
    type_str: Option<String>,
    /// The variable name, e.g. `$name`.
    variable: Option<String>,
    /// The remainder description (single-line).
    description: String,
    /// Sort priority: @param=0, @return=1, @throws=2, other=3.
    priority: u8,
    /// Original index for stable sort.
    original_index: usize,
    /// If the tag has multi-line content, store the raw reconstructed lines
    /// (each line WITHOUT the ` * ` prefix). When set, type_str/variable/description are ignored.
    raw_lines: Option<Vec<String>>,
}

/// Attempt to reformat a docblock comment.
///
/// Returns `Some(reformatted)` allocated in the arena, or `None` if the docblock
/// should be removed entirely (empty docblock → `no_empty_phpdoc`).
///
/// On parse failure, returns `None` so the caller falls through to default formatting.
pub(crate) fn reformat_docblock<'arena>(
    arena: &'arena Bump,
    settings: &FormatSettings,
    content: &str,
    span: Span,
) -> Option<&'arena str> {
    // Only process multi-line docblocks
    if !content.starts_with("/**") || !content.contains('\n') {
        return None;
    }

    // Allocate content in the arena for the parser
    let arena_content = arena.alloc_str(content);

    let doc = mago_docblock::parse_phpdoc_with_span(arena, arena_content, span).ok()?;

    // Skip docblocks containing code blocks — their formatting is too complex
    // to reconstruct reliably and idempotently.
    let has_code_block = doc.elements.iter().any(|e| matches!(e, Element::Code(_)));
    if has_code_block {
        return None;
    }

    // Separate description elements from tag elements
    let mut description_lines: Vec<String> = Vec::new();
    let mut tag_lines: Vec<TagLine<'_>> = Vec::new();
    let mut tag_index: usize = 0;
    let mut seen_tag = false;

    for element in doc.elements.iter() {
        match element {
            Element::Text(text) => {
                if !seen_tag {
                    // Description text: reconstruct from segments
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
            Element::Code(code) => {
                if !seen_tag {
                    if code.directives.is_empty() {
                        description_lines.push(format!("```\n{}\n```", code.content));
                    } else {
                        let directives: Vec<&str> =
                            code.directives.iter().copied().collect();
                        description_lines
                            .push(format!("```{}\n{}\n```", directives.join(" "), code.content));
                    }
                }
            }
            Element::Tag(tag) => {
                seen_tag = true;

                let kind = tag.kind;
                let tag_name_str = tag.name;

                // Filter out @access and @package tags
                if matches!(kind, TagKind::Access | TagKind::Package) {
                    continue;
                }

                // Rename alias tags
                let effective_name = match kind {
                    TagKind::Type => "var",
                    _ => {
                        if tag_name_str == "link" {
                            "see"
                        } else {
                            tag_name_str
                        }
                    }
                };

                let priority = tag_priority(kind);

                let desc = tag.description.trim();

                // Check if the description contains newlines — if so, this is a multi-line
                // tag (e.g. @return array{\n *     id: int, ... }) that we should preserve
                // structurally but still apply transformations to the first line.
                if desc.contains('\n') {
                    let raw = build_multiline_tag_raw(effective_name, desc, settings);
                    tag_lines.push(TagLine {
                        tag_name: arena.alloc_str(&format!("@{}", effective_name)),
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

                let (type_str, variable, rest) = parse_tag_parts(desc, kind);

                // Apply scalar normalization to type
                let type_str = type_str.map(|t| {
                    let mut t = t;
                    if settings.phpdoc_scalar_types {
                        t = normalize_scalar_types(&t);
                    }
                    if settings.phpdoc_null_position == PhpdocNullPosition::Last {
                        t = move_null_to_end(&t);
                    }
                    t
                });

                tag_lines.push(TagLine {
                    tag_name: arena.alloc_str(&format!("@{}", effective_name)),
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
                if !seen_tag {
                    // Preserve blank lines in description area
                    // Only add if we already have description content (avoid leading blanks)
                    if !description_lines.is_empty() {
                        description_lines.push(String::new());
                    }
                }
                // In tag area, we handle spacing ourselves via group separators
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
        }
    }

    // Trim trailing empty lines from description
    while description_lines.last().map_or(false, |l| l.is_empty()) {
        description_lines.pop();
    }

    // Collapse consecutive blank lines in description (no more than one)
    let mut deduped_desc: Vec<String> = Vec::new();
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

    // If both description and tags are empty, signal removal (no_empty_phpdoc)
    if description_lines.is_empty() && tag_lines.is_empty() {
        return Some(""); // empty string signals removal to caller
    }

    // Sort tags: stable sort by priority
    tag_lines.sort_by(|a, b| a.priority.cmp(&b.priority).then(a.original_index.cmp(&b.original_index)));

    // Build the output
    let mut output = String::with_capacity(content.len());
    output.push_str("/**");

    // Description lines
    for line in &description_lines {
        output.push('\n');
        if line.is_empty() {
            output.push_str(" *");
        } else {
            output.push_str(" * ");
            output.push_str(line);
        }
    }

    // Separator between description and tags
    if !description_lines.is_empty() && !tag_lines.is_empty() {
        output.push('\n');
        output.push_str(" *");
    }

    // Tags — potentially with vertical alignment
    if !tag_lines.is_empty() {
        let formatted_tags = if settings.phpdoc_align == PhpdocAlign::Vertical {
            format_tags_aligned(&tag_lines)
        } else {
            format_tags_unaligned(&tag_lines)
        };

        for (i, line) in formatted_tags.iter().enumerate() {
            // Insert blank line between different tag groups
            if i > 0 && tag_lines[i].priority != tag_lines[i - 1].priority {
                output.push('\n');
                output.push_str(" *");
            }
            output.push('\n');
            output.push_str(" * ");
            output.push_str(line);
        }
    }

    output.push('\n');
    output.push_str(" */");

    let result = arena.alloc_str(&output);
    Some(result)
}

/// Build raw lines for a multi-line tag, preserving structure.
fn build_multiline_tag_raw(effective_name: &str, desc: &str, settings: &FormatSettings) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let first_line_desc: &str;
    let rest_lines: &str;

    if let Some(nl_pos) = desc.find('\n') {
        first_line_desc = &desc[..nl_pos];
        rest_lines = &desc[nl_pos + 1..];
    } else {
        first_line_desc = desc;
        rest_lines = "";
    }

    // Apply transformations to the first line's type portion
    let mut first = first_line_desc.to_string();
    if settings.phpdoc_scalar_types {
        first = normalize_scalar_types(&first);
    }
    if settings.phpdoc_null_position == PhpdocNullPosition::Last {
        first = move_null_to_end(&first);
    }

    lines.push(format!("@{} {}", effective_name, first.trim()));

    // Add continuation lines with proper ` *` prefix handling
    for line in rest_lines.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            // Preserve internal blank lines within the type
            lines.push(String::new());
        } else {
            // The parser strips ` * ` prefix, so the content is just the raw text
            lines.push(format!("  {}", trimmed));
        }
    }

    lines
}

/// Get sort priority for a tag kind.
fn tag_priority(kind: TagKind) -> u8 {
    match kind {
        TagKind::Param | TagKind::PsalmParam | TagKind::PhpstanParam => 0,
        TagKind::Return | TagKind::PsalmReturn | TagKind::PhpstanReturn => 1,
        TagKind::Throws => 2,
        _ => 3,
    }
}

/// Parse tag description into (type, variable, rest) parts.
fn parse_tag_parts(desc: &str, kind: TagKind) -> (Option<String>, Option<String>, String) {
    let desc = desc.trim();
    if desc.is_empty() {
        return (None, None, String::new());
    }

    match kind {
        TagKind::Param | TagKind::PsalmParam | TagKind::PhpstanParam => {
            let (type_str, rest) = split_type_from_desc(desc);
            if let Some(type_str) = type_str {
                let rest = rest.trim_start();
                let (var, remainder) = split_variable(rest);
                (Some(type_str), var, remainder.trim_start().to_string())
            } else {
                let (var, remainder) = split_variable(desc);
                (None, var, remainder.trim_start().to_string())
            }
        }
        TagKind::Return | TagKind::PsalmReturn | TagKind::PhpstanReturn | TagKind::Throws => {
            let (type_str, rest) = split_type_from_desc(desc);
            (type_str, None, rest.trim_start().to_string())
        }
        TagKind::Var | TagKind::PsalmVar | TagKind::PhpstanVar | TagKind::Type => {
            let (type_str, rest) = split_type_from_desc(desc);
            if let Some(type_str) = type_str {
                let rest = rest.trim_start();
                let (var, remainder) = split_variable(rest);
                (Some(type_str), var, remainder.trim_start().to_string())
            } else {
                (None, None, desc.to_string())
            }
        }
        TagKind::Property | TagKind::PropertyRead | TagKind::PropertyWrite => {
            let (type_str, rest) = split_type_from_desc(desc);
            if let Some(type_str) = type_str {
                let rest = rest.trim_start();
                let (var, remainder) = split_variable(rest);
                (Some(type_str), var, remainder.trim_start().to_string())
            } else {
                (None, None, desc.to_string())
            }
        }
        _ => (None, None, desc.to_string()),
    }
}

/// Split the leading type annotation from the remaining description.
fn split_type_from_desc(s: &str) -> (Option<String>, &str) {
    let s = s.trim_start();
    if s.is_empty() {
        return (None, s);
    }

    if s.starts_with('$') {
        return (None, s);
    }

    let bytes = s.as_bytes();
    let mut i = 0;
    let len = bytes.len();
    let mut depth = 0i32;

    while i < len {
        let b = bytes[i];
        match b {
            b'<' | b'(' | b'[' | b'{' => {
                depth += 1;
                i += 1;
            }
            b'>' | b')' | b']' | b'}' => {
                depth -= 1;
                i += 1;
            }
            b' ' | b'\t' if depth == 0 => {
                break;
            }
            b'|' | b'&' if depth == 0 => {
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    if i == 0 {
        return (None, s);
    }

    let type_part = &s[..i];
    let rest = &s[i..];
    (Some(type_part.to_string()), rest)
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

/// Normalize scalar type names: `integer` → `int`, `boolean` → `bool`, etc.
fn normalize_scalar_types(type_str: &str) -> String {
    let mut result = String::with_capacity(type_str.len());
    let mut last = 0;

    let mut i = 0;
    let byte_len = type_str.len();

    while i < byte_len {
        // Only check at word boundaries (start of string, or after non-alphanumeric)
        let at_word_boundary = i == 0 || {
            let prev = type_str.as_bytes()[i - 1];
            !prev.is_ascii_alphanumeric() && prev != b'_'
        };

        if at_word_boundary {
            let remaining = &type_str[i..];
            if let Some((old_len, new_word)) = check_scalar_word(remaining) {
                result.push_str(&type_str[last..i]);
                result.push_str(new_word);
                last = i + old_len;
                i += old_len;
                continue;
            }
        }

        i += 1;
    }

    result.push_str(&type_str[last..]);
    result
}

/// Check if a scalar type word starts at this position.
fn check_scalar_word(s: &str) -> Option<(usize, &'static str)> {
    let scalars: &[(&str, &str)] = &[
        ("integer", "int"),
        ("boolean", "bool"),
        ("double", "float"),
        ("real", "float"),
    ];

    for (old, new) in scalars {
        if s.starts_with(old) {
            let after = &s[old.len()..];
            let is_boundary = after
                .chars()
                .next()
                .map_or(true, |c| !c.is_ascii_alphanumeric() && c != '_');
            if is_boundary {
                return Some((old.len(), new));
            }
        }
    }
    None
}

/// Move `null` to the end of a union type string.
fn move_null_to_end(type_str: &str) -> String {
    if type_str.starts_with('?') {
        return type_str.to_string();
    }

    if !type_str.contains('|') {
        return type_str.to_string();
    }

    reorder_outer_union_null(type_str)
}

/// Reorder null to end in a union type, respecting nesting depth.
fn reorder_outer_union_null(type_str: &str) -> String {
    let parts = split_top_level(type_str, '|');
    if parts.len() <= 1 {
        return type_str.to_string();
    }

    let mut non_null: Vec<&str> = Vec::new();
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
fn split_top_level(s: &str, delim: char) -> Vec<&str> {
    let mut parts = Vec::new();
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
fn format_tags_unaligned(tags: &[TagLine<'_>]) -> Vec<String> {
    tags.iter()
        .map(|tag| {
            if let Some(ref raw) = tag.raw_lines {
                return raw.join("\n * ");
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
fn format_tags_aligned(tags: &[TagLine<'_>]) -> Vec<String> {
    if tags.is_empty() {
        return Vec::new();
    }

    let mut result: Vec<String> = Vec::with_capacity(tags.len());
    let mut group_start = 0;

    while group_start < tags.len() {
        let group_priority = tags[group_start].priority;
        let mut group_end = group_start + 1;
        while group_end < tags.len() && tags[group_end].priority == group_priority {
            group_end += 1;
        }

        let group = &tags[group_start..group_end];

        // Calculate column widths, excluding raw_lines tags from alignment
        let alignable: Vec<&TagLine<'_>> =
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
            // Multi-line tags: emit raw
            if let Some(ref raw) = tag.raw_lines {
                result.push(raw.join("\n * "));
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
