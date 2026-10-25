//! Render diagnostic text as a headline paragraph and hard-broken detail lines.
//!
//! Code spans stay inline, except those with line breaks, which become fenced code blocks.

use std::iter;

use mago_server::diagnostics::code_spans;
use mago_server::diagnostics::message_lines;

const PARAGRAPH: &str = "\n\n";
const HARD_BREAK: &str = "\\\n";
const LINE: &str = "\n";
const DETAIL: &str = "\n = ";

pub(super) fn render(message: &str) -> String {
    let mut markdown = Markdown { rendered: String::with_capacity(message.len()), separator: "", fenced: false };
    let mut start = 0;
    let blocks = message.match_indices(DETAIL).map(|(end, _)| end).chain(iter::once(message.len())).map(|end| {
        let block = &message[start..end];
        start = end + 1;
        block
    });
    let mut separator = "";
    for line in blocks.flat_map(message_lines).map(str::trim_start) {
        markdown.line(separator, line);
        separator = if separator.is_empty() { PARAGRAPH } else { HARD_BREAK };
    }

    markdown.rendered
}

pub(super) fn detail_separator(rendered: &str) -> &'static str {
    let mut fence = 0;
    let mut details = false;
    let mut fenced = false;
    for line in rendered.split_inclusive('\n') {
        let line = line.strip_suffix('\n').unwrap_or(line);
        let ticks = line.len() >= 3 && line.bytes().all(|byte| byte == b'`');
        fenced = fence > 0 && ticks && line.len() >= fence;
        if fenced {
            fence = 0;
        } else if fence == 0 {
            if ticks {
                fence = line.len();
            } else {
                details |= line.is_empty();
            }
        }
    }

    match (details, fenced) {
        (false, _) => PARAGRAPH,
        (true, true) => LINE,
        (true, false) => HARD_BREAK,
    }
}

struct Markdown {
    rendered: String,
    separator: &'static str,
    fenced: bool,
}

impl Markdown {
    fn line(&mut self, separator: &'static str, line: &str) {
        self.separator = separator;
        let mut end = 0;
        for span in code_spans(line) {
            self.text(&line[end..span.start]);
            self.code(&line[span.start..span.end]);
            end = span.end;
        }

        self.text(&line[end..]);
    }

    fn text(&mut self, text: &str) {
        let text = if self.fenced { text.trim_start() } else { text };
        if !text.is_empty() {
            self.open();
            escape(&mut self.rendered, text);
        }
    }

    fn code(&mut self, span: &str) {
        let width = span.bytes().take_while(|&byte| byte == b'`').count();
        let code = &span[width..span.len() - width];
        if !code.contains(['\r', '\n']) {
            self.open();
            self.rendered.push_str(span);
            return;
        }

        let code = code.strip_prefix(' ').and_then(|code| code.strip_suffix(' ')).unwrap_or(code);
        let fence = code.split(|character| character != '`').map(str::len).max().unwrap_or(0).max(2) + 1;
        if !self.rendered.is_empty() {
            self.rendered.truncate(self.rendered.trim_end_matches([' ', '\t']).len());
            self.rendered.push_str(if self.separator == PARAGRAPH { PARAGRAPH } else { LINE });
        }

        self.rendered.extend(iter::repeat_n('`', fence));
        self.rendered.push('\n');
        self.rendered.push_str(code);
        if !code.ends_with('\n') {
            self.rendered.push('\n');
        }

        self.rendered.extend(iter::repeat_n('`', fence));
        self.separator = "";
        self.fenced = true;
    }

    fn open(&mut self) {
        self.rendered.push_str(if self.fenced && self.separator != PARAGRAPH { LINE } else { self.separator });
        self.separator = "";
        self.fenced = false;
    }
}

fn escape(rendered: &mut String, text: &str) {
    for character in text.chars() {
        if character.is_ascii_punctuation() {
            rendered.push('\\');
        }

        rendered.push(character);
    }
}

#[cfg(test)]
mod tests {
    use pulldown_cmark::CodeBlockKind;
    use pulldown_cmark::Event;
    use pulldown_cmark::Parser;
    use pulldown_cmark::Tag;

    use super::*;

    #[test]
    fn keeps_code_and_paragraphs_without_turning_source_text_into_markup() {
        let prose = "Expected `array<string, Foo\\Bar>`.\n = Help: <script> *value* [link](https://example.com).";
        let rendered = render(prose);
        let text: String = Parser::new(&rendered)
            .filter_map(|event| match event {
                Event::Text(text) | Event::Code(text) => Some(text.into_string()),
                _ => None,
            })
            .collect();
        assert_eq!(text, "Expected array<string, Foo\\Bar>.= Help: <script> *value* [link](https://example.com).");
        assert!(rendered.contains("`array<string, Foo\\Bar>`"));
        assert!(rendered.contains("\n\n"));
        assert!(!Parser::new(&rendered).any(|event| matches!(event, Event::Html(_) | Event::InlineHtml(_))));
        assert_eq!(render("Literal `` ` ``."), "Literal `` ` ``\\.");
        assert_eq!(render("Whitespace `   `."), "Whitespace `   `\\.");
    }

    #[test]
    fn headline_is_a_paragraph_and_details_use_hard_breaks() {
        let rendered = render("Title `a`.\n = Detail `b`.\n = Help: Next.\n   More # 1. - > text.");
        assert_eq!(
            rendered,
            "Title `a`\\.\n\n\\= Detail `b`\\.\\\n\\= Help\\: Next\\.\\\nMore \\# 1\\. \\- \\> text\\."
        );
        let events: Vec<_> = Parser::new(&rendered).collect();
        assert_eq!(events.iter().filter(|event| matches!(event, Event::Start(Tag::Paragraph))).count(), 2);
        assert_eq!(events.iter().filter(|event| matches!(event, Event::HardBreak)).count(), 2);
        assert!(!events.iter().any(|event| matches!(event, Event::SoftBreak | Event::Html(_) | Event::InlineHtml(_))));
        assert_eq!(
            events
                .iter()
                .filter_map(|event| if let Event::Code(code) = event { Some(code.as_ref()) } else { None })
                .collect::<Vec<_>>(),
            ["a", "b"]
        );
        assert_eq!(render("Title."), "Title\\.");
        assert_eq!(detail_separator(&render("Title.")), PARAGRAPH);
        assert_eq!(detail_separator(&rendered), HARD_BREAK);
    }

    #[test]
    fn backticks_never_pair_across_lines() {
        let rendered = render("Odd ` tick\n = Help: `code` and `\n = Other ` end");
        let events: Vec<_> = Parser::new(&rendered).collect();
        assert_eq!(
            events
                .iter()
                .filter_map(|event| if let Event::Code(code) = event { Some(code.as_ref()) } else { None })
                .collect::<Vec<_>>(),
            ["code"]
        );
        let text: String = events
            .iter()
            .filter_map(|event| match event {
                Event::Text(text) | Event::Code(text) => Some(text.as_ref()),
                _ => None,
            })
            .collect();
        assert_eq!(text, "Odd ` tick= Help: code and `= Other ` end");
    }

    #[test]
    fn multi_line_code_spans_become_fenced_blocks_and_single_line_spans_stay_inline() {
        let message = "expected `string('a\\nb')`, but found `string('a\nb')`.\n = This has type `string('SELECT id,\n\nWHERE id = 1')`\n = Help: Use:\n   `return [\r\n        1,\r\n    ];`";
        let rendered = render(message);
        assert_eq!(
            rendered,
            "expected `string('a\\nb')`\\, but found\n```\nstring('a\nb')\n```\n\\.\n\n\\= This has type\n```\nstring('SELECT id,\n\nWHERE id = 1')\n```\n\\= Help\\: Use\\:\n```\nreturn [\r\n        1,\r\n    ];\n```"
        );
        assert_eq!(
            blocks(&rendered),
            [
                "expected `string('a\\nb')`, but found",
                "```\nstring('a\nb')\n",
                ".",
                "= This has type",
                "```\nstring('SELECT id,\n\nWHERE id = 1')\n",
                "= Help: Use:",
                "```\nreturn [\n        1,\n    ];\n",
            ]
        );
        assert_eq!(detail_separator(&rendered), LINE);
        let padded = render("Use `` `a\nb` `` or ```` ```\nx ````.");
        assert_eq!(padded, "Use\n```\n`a\nb`\n```\nor\n````\n```\nx\n````\n\\.");
        assert_eq!(blocks(&padded), ["Use", "```\n`a\nb`\n", "or", "```\n```\nx\n", "."]);
    }

    #[test]
    fn detail_separators_ignore_blank_lines_and_fences_inside_code_blocks() {
        let link = "= See: https://example.com/";
        for (message, separator) in [
            ("found `a\n\nb`", PARAGRAPH),
            ("found `a\n\nb`.", PARAGRAPH),
            ("found `a\nb`.\n = Detail.", HARD_BREAK),
            ("Title.\n = found `a\n\n````\nb`", LINE),
            ("Title.\n = found `a\r`", LINE),
        ] {
            let rendered = render(message);
            assert_eq!(detail_separator(&rendered), separator, "{rendered:?}");
            let mut expected = blocks(&rendered);
            if separator == HARD_BREAK {
                let last = expected.last_mut().unwrap();
                last.push('\n');
                last.push_str(link);
            } else {
                expected.push(link.into());
            }
            let linked = format!("{rendered}{separator}\\= See\\: <https://example.com/>");
            assert_eq!(blocks(&linked), expected, "{linked:?}");
        }
    }

    fn blocks(markdown: &str) -> Vec<String> {
        let mut blocks: Vec<String> = Vec::new();
        for event in Parser::new(markdown) {
            match event {
                Event::Start(Tag::Paragraph) => blocks.push(String::new()),
                Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))) if info.is_empty() => {
                    blocks.push("```\n".into());
                }
                Event::Text(text) => blocks.last_mut().unwrap().push_str(&text.replace("\r\n", "\n")),
                Event::Code(code) => {
                    let block = blocks.last_mut().unwrap();
                    block.push('`');
                    block.push_str(&code);
                    block.push('`');
                }
                Event::HardBreak => blocks.last_mut().unwrap().push('\n'),
                Event::Start(Tag::Link { .. }) | Event::End(_) => {}
                event => panic!("unexpected {event:?} in {markdown:?}"),
            }
        }

        blocks
    }
}
