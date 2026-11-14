use std::fmt::Write;

use foldhash::HashMap;
use mago_database::file::File;
use mago_span::HasSpan;
use mago_span::Span;
use mago_text_edit::TextEdit;
use mago_text_edit::TextRange;

use crate::pragma::Pragma;

pub(super) fn directive_edits(
    file: &File,
    pragmas: &[Pragma<'_>],
    skipped: impl Fn(&Pragma<'_>) -> bool,
) -> HashMap<Span, TextEdit> {
    let mut groups: HashMap<Span, Vec<&Pragma<'_>>> = HashMap::default();
    for pragma in pragmas {
        groups.entry(pragma.span).or_default().push(pragma);
    }

    let mut edits = HashMap::default();
    for (span, group) in groups {
        if group.iter().all(|pragma| pragma.is_fulfilled() || skipped(pragma)) {
            continue;
        }

        let first = group[0];
        let mut kept = group.iter().filter(|pragma| pragma.matches > 0 || skipped(pragma)).peekable();
        let replacement = if kept.peek().is_none() {
            String::from(" ")
        } else {
            let mut replacement = format!("@mago-{} {}:", first.kind.as_str(), first.category);
            for (index, pragma) in kept.enumerate() {
                if index > 0 {
                    replacement.push_str(", ");
                }

                replacement.push_str(pragma.code);
                if pragma.count_span.is_some() {
                    let count = if pragma.matches > 0 && !pragma.is_fulfilled() {
                        pragma.matches
                    } else {
                        pragma.expected_matches
                    };
                    let _ = write!(replacement, "({count})");
                }
            }

            if !first.description.is_empty() {
                replacement.push(' ');
                replacement.push_str(first.description);
            }

            let original = &file.contents[span.start_offset() as usize..span.end_offset() as usize];
            let trailing = &original[original.trim_ascii_end().len()..];
            if let Ok(trailing) = std::str::from_utf8(trailing) {
                replacement.push_str(trailing);
            }

            replacement
        };

        edits.insert(span, TextEdit::replace(TextRange::new(span.start_offset(), span.end_offset()), replacement));
    }

    edits
}
