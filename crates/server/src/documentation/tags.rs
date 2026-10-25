//! Read PHPDoc tag headers while retaining the source text around them.

use mago_allocator::LocalArena;
use mago_database::file::FileId;
use mago_phpdoc_syntax::PHPDocParser;
use mago_phpdoc_syntax::cst::{Document, Tag, TagValue};
use mago_span::{HasSpan, Span};
use pulldown_cmark::{Event, Parser, Tag as MarkdownTag, TagEnd};

use crate::domain::{DocumentationTag, DocumentationTagKind, Range, SourceDocumentation};

pub(super) fn parse(text: String, file_id: FileId) -> SourceDocumentation {
    let arena = LocalArena::new();
    let input = without_code(&text);
    let document = PHPDocParser::parse(&arena, file_id, &input);
    let tags = document.tags().filter_map(|tag| checked_header(&text, &input, &document, tag)).collect();
    SourceDocumentation { text, tags }
}

pub(super) fn parameter_description(text: &str, file_id: FileId, parameter: &[u8]) -> Option<String> {
    let arena = LocalArena::new();
    let input = without_code(text);
    let document = PHPDocParser::parse(&arena, file_id, &input);
    let mut tags = document.tags().filter(|tag| starts_line(text, tag)).peekable();
    while let Some(tag) = tags.next() {
        let name = match tag.value {
            TagValue::Param(value) => value.parameter.map(|name| name.value),
            TagValue::TypelessParam(value) => Some(value.parameter.value),
            _ => None,
        };
        if name != Some(parameter) {
            continue;
        }
        let Some(header) = checked_header(text, &input, &document, tag) else { continue };
        if !matches!(header.kind, DocumentationTagKind::Parameter { .. }) {
            continue;
        }
        // Every block tag ends the description, including unknown or invalid tags.
        let end = tags.peek().map_or(text.len(), |next| next.at.start.offset as usize);
        return description(&text[header.range.end as usize..end]);
    }
    None
}

pub(super) fn property_description(text: &str, file_id: FileId, name: &[u8], write_only: bool) -> Option<String> {
    let arena = LocalArena::new();
    let input = without_code(text);
    let document = PHPDocParser::parse(&arena, file_id, &input);
    let mut tags = document.tags().filter(|tag| starts_line(text, tag)).peekable();
    while let Some(tag) = tags.next() {
        let property = match tag.value {
            TagValue::Property(property) => property,
            TagValue::PropertyRead(property) if !write_only => property,
            TagValue::PropertyWrite(property) if write_only => property,
            _ => continue,
        };
        let range = Range::new(tag.at.start.offset, property.variable.span.end.offset);
        if property.variable.value != name || !valid_header(text, &input, &document, tag, range) {
            continue;
        }
        let end = tags.peek().map_or(text.len(), |next| next.at.start.offset as usize);
        return description(&text[range.end as usize..end]);
    }
    None
}

fn description(text: &str) -> Option<String> {
    let mut lines: Vec<_> = text.lines().collect();
    let first = lines.first_mut()?;
    *first = first.trim_start();
    let start = lines.iter().position(|line| !line.trim().is_empty())?;
    let end = lines.iter().rposition(|line| !line.trim().is_empty())?;
    Some(lines[start..=end].join("\n"))
}

pub(super) fn method_description(text: &str, file_id: FileId, name: &[u8]) -> Option<String> {
    let arena = LocalArena::new();
    let input = without_code(text);
    let document = PHPDocParser::parse(&arena, file_id, &input);
    let mut tags = document.tags().filter(|tag| starts_line(text, tag)).peekable();
    let mut result = None;
    while let Some(tag) = tags.next() {
        let TagValue::Method(method) = tag.value else { continue };
        let range = Range::new(tag.at.start.offset, method.parameters.right_parenthesis.end.offset);
        if !method.name.value.eq_ignore_ascii_case(name) || !valid_header(text, &input, &document, tag, range) {
            continue;
        }
        // The scanner keeps the last declaration of a method name, including prefixed tags.
        let end = tags.peek().map_or(text.len(), |next| next.at.start.offset as usize);
        result = description(&text[range.end as usize..end]);
    }
    result
}

fn starts_line(text: &str, tag: &Tag<'_>) -> bool {
    text[..tag.at.start.offset as usize]
        .rsplit('\n')
        .next()
        .unwrap_or("")
        .bytes()
        .all(|byte| matches!(byte, b' ' | b'\t'))
}

fn checked_header(text: &str, input: &[u8], document: &Document<'_>, tag: &Tag<'_>) -> Option<DocumentationTag> {
    let header = header(text, tag)?;
    valid_header(text, input, document, tag, header.range).then_some(header)
}

fn valid_header(text: &str, input: &[u8], document: &Document<'_>, tag: &Tag<'_>, range: Range) -> bool {
    let span = tag.span();
    let range = range.start as usize..range.end as usize;
    starts_line(text, tag)
        && !document.errors.iter().any(|error| {
            let error = error.span();
            error.start.offset <= span.end.offset && error.end.offset >= span.start.offset
        })
        && text.as_bytes()[range.clone()] == input[range]
}

fn header(text: &str, tag: &Tag<'_>) -> Option<DocumentationTag> {
    if tag.vendor.is_some() {
        return None;
    }
    let slice = |span: Span| text[span.start.offset as usize..span.end.offset as usize].to_owned();
    let (kind, end) = match tag.value {
        TagValue::Param(value) if tag.name.value.eq_ignore_ascii_case(b"param") => {
            let parameter = value.parameter.map(|parameter| {
                let start = value.ampersand.or(value.ellipsis).unwrap_or(parameter.span);
                slice(start.join(parameter.span))
            });
            let end = value
                .parameter
                .map(|parameter| parameter.span)
                .or(value.ellipsis)
                .or(value.ampersand)
                .unwrap_or_else(|| value.r#type.span());
            (DocumentationTagKind::Parameter { parameter, type_name: Some(slice(value.r#type.span())) }, end.end.offset)
        }
        TagValue::TypelessParam(value) if tag.name.value.eq_ignore_ascii_case(b"param") => {
            let start = value.ampersand.or(value.ellipsis).unwrap_or(value.parameter.span);
            (
                DocumentationTagKind::Parameter {
                    parameter: Some(slice(start.join(value.parameter.span))),
                    type_name: None,
                },
                value.parameter.span.end.offset,
            )
        }
        TagValue::Return(value) if tag.name.value.eq_ignore_ascii_case(b"return") => {
            (DocumentationTagKind::Returns { type_name: slice(value.r#type.span()) }, value.r#type.span().end.offset)
        }
        TagValue::Throws(value) if tag.name.value.eq_ignore_ascii_case(b"throws") => {
            (DocumentationTagKind::Throws { type_name: slice(value.r#type.span()) }, value.r#type.span().end.offset)
        }
        TagValue::Deprecated(_) if tag.name.value.eq_ignore_ascii_case(b"deprecated") => {
            (DocumentationTagKind::Deprecated, tag.name.span.end.offset)
        }
        _ => return None,
    };
    Some(DocumentationTag { range: Range::new(tag.at.start.offset, end), kind })
}

fn without_code(text: &str) -> Vec<u8> {
    let mut input = text.as_bytes().to_vec();
    let mut block_start = None;
    for (event, range) in Parser::new(text).into_offset_iter() {
        let range = match event {
            Event::Start(MarkdownTag::CodeBlock(_)) => {
                block_start = Some(range.start);
                continue;
            }
            Event::End(TagEnd::CodeBlock) => block_start.take().map(|start| start..range.end),
            Event::Code(_) => Some(range),
            _ => None,
        };
        if let Some(range) = range {
            for byte in &mut input[range] {
                if !matches!(byte, b'\r' | b'\n') {
                    *byte = b' ';
                }
            }
        }
    }
    input
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn method_descriptions_follow_the_last_matching_tag_without_borrowing_class_docs() {
        let text = "Class description.\n@method string greet(string $name) First.\n@psalm-method string GREET(string $name) The **greeting**.\n  More details.\n@custom Other.";
        assert_eq!(
            method_description(text, FileId::zero(), b"greet").as_deref(),
            Some("The **greeting**.\n  More details.")
        );
        assert_eq!(method_description(text, FileId::zero(), b"other"), None);
        for text in [
            "Class description.\n@method string greet(string $name)",
            "@method string greet() First.\n@phpstan-method string greet()",
            "```php\n@method string greet() Example.\n```",
            "@method `string` greet() Broken.",
        ] {
            assert_eq!(method_description(text, FileId::zero(), b"greet"), None, "{text}");
        }
    }

    #[test]
    fn property_descriptions_follow_the_read_or_write_tag_and_keep_markdown() {
        let text = "@property-write int $value Setter.\n@psalm-property-read string $value The **value**.\n  More details.\n@custom Other.";
        assert_eq!(
            property_description(text, FileId::zero(), b"$value", false).as_deref(),
            Some("The **value**.\n  More details.")
        );
        assert_eq!(property_description(text, FileId::zero(), b"$value", true).as_deref(), Some("Setter."));
        assert_eq!(property_description(text, FileId::zero(), b"$Value", false), None);
        for tag in ["property", "property-read", "psalm-property-read", "phpstan-property"] {
            let text = format!("@{tag} string $value Description.");
            assert_eq!(property_description(&text, FileId::zero(), b"$value", false).as_deref(), Some("Description."));
        }
        for (first, second) in [("property", "psalm-property"), ("phpstan-property", "property")] {
            let text = format!("@{first} string $value First.\n@{second} int $value Second.");
            assert_eq!(property_description(&text, FileId::zero(), b"$value", false).as_deref(), Some("First."));
        }
        for text in [
            "@property-read | $value Broken.",
            "@property-read `string` $value Broken.",
            "```php\n@property string $value Example.\n```",
        ] {
            assert_eq!(property_description(text, FileId::zero(), b"$value", false), None, "{text}");
        }
        let text = "@property string $value\n```php\n@property string $fake\n```\n\nUse `@property`.\n@custom Other.";
        assert_eq!(
            property_description(text, FileId::zero(), b"$value", false).as_deref(),
            Some("```php\n@property string $fake\n```\n\nUse `@property`.")
        );
    }

    #[test]
    fn reads_tag_headers_without_rewriting_descriptions() {
        let text = "Keeps **Markdown**.\n\n@param list<string> &...$items First line.\n  Second line.\n@return string Result.\n@throws \\RuntimeException Failure.\n@deprecated Use `next()`.";
        let document = parse(text.into(), FileId::zero());
        assert_eq!(document.text, text);
        assert_eq!(document.tags.len(), 4);
        assert_eq!(
            document.tags[0].kind,
            DocumentationTagKind::Parameter {
                parameter: Some("&...$items".into()),
                type_name: Some("list<string>".into()),
            },
        );
        assert_eq!(&text[document.tags[0].range.end as usize..][..12], " First line.");
    }

    #[test]
    fn leaves_unknown_vendor_and_malformed_tags_in_source() {
        let text = "@custom **details**\n@param | broken\n@phpstan-return non-empty-string\n@return int Valid.";
        let document = parse(text.into(), FileId::zero());
        assert_eq!(document.text, text);
        assert_eq!(document.tags.len(), 1);
        assert!(matches!(document.tags[0].kind, DocumentationTagKind::Returns { .. }));
    }

    #[test]
    fn ignores_tags_in_markdown_code_regions() {
        let text = "Example `@return int` and {@see Name}.\n\n````php\n```\n@param int $fake\n````\n\n~~~php\n@return string\n~~~\n\n    @throws Error\n\n@return bool Real.";
        let document = parse(text.into(), FileId::zero());
        assert_eq!(document.text, text);
        assert_eq!(document.tags.len(), 1);
        assert_eq!(document.tags[0].kind, DocumentationTagKind::Returns { type_name: "bool".into() });
    }

    #[test]
    fn accepts_typeless_parameters_and_empty_deprecation() {
        let document = parse("@param $name Description.\n@deprecated".into(), FileId::zero());
        assert_eq!(document.tags.len(), 2);
        assert_eq!(
            document.tags[0].kind,
            DocumentationTagKind::Parameter { parameter: Some("$name".into()), type_name: None },
        );
        assert_eq!(document.tags[1].kind, DocumentationTagKind::Deprecated);
    }
    #[test]
    fn code_in_a_tag_header_cannot_be_removed_by_parsing() {
        let text = "@param `string` $name Description.\n@return bool Valid.";
        let document = parse(text.into(), FileId::zero());
        assert_eq!(document.text, text);
        assert_eq!(document.tags.len(), 1);
        assert!(matches!(document.tags[0].kind, DocumentationTagKind::Returns { .. }));
    }

    #[test]
    fn parameter_descriptions_match_exact_names_and_keep_markdown() {
        let text = "@param string $names Other.\n@param string &...$name The **name**.\n  More details.\n@return string Result.";
        assert_eq!(
            parameter_description(text, FileId::zero(), b"$name").as_deref(),
            Some("The **name**.\n  More details.")
        );
        assert_eq!(parameter_description(text, FileId::zero(), b"$Name"), None);
        assert_eq!(parameter_description("@param $name A name.", FileId::zero(), b"$name").as_deref(), Some("A name."));
        assert_eq!(parameter_description("@param string $name", FileId::zero(), b"$name"), None);
    }

    #[test]
    fn parameter_descriptions_stop_at_unknown_vendor_and_invalid_tags() {
        for next in ["@custom Other.", "@phpstan-param int $other Other.", "@param | broken", "@return | broken"] {
            let text = format!("@param string $name The name.\n{next}");
            assert_eq!(parameter_description(&text, FileId::zero(), b"$name").as_deref(), Some("The name."), "{text}");
        }
        for text in ["@phpstan-param string $name Other.", "@param | $name Broken.", "@param `string` $name Broken."] {
            assert_eq!(parameter_description(text, FileId::zero(), b"$name"), None, "{text}");
        }
    }

    #[test]
    fn parameter_descriptions_keep_code_and_ignore_its_tags() {
        let description = "```php\n@param int $other\n```\n\n~~~php\n@return int\n~~~\n\n    @param int $hidden\n\nUse `@param string $inline`.";
        let text = format!("@param string $name\n{description}\n@custom Other.");
        assert_eq!(parameter_description(&text, FileId::zero(), b"$name").as_deref(), Some(description));
        for name in [b"$other".as_slice(), b"$hidden", b"$inline"] {
            assert_eq!(parameter_description(&text, FileId::zero(), name), None);
        }
        let text = "@param string $name\n\n    sample($name);\n@custom Other.";
        assert_eq!(parameter_description(text, FileId::zero(), b"$name").as_deref(), Some("    sample($name);"));
    }
}
