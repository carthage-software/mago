//! Choose source locations and message text for diagnostics.

use std::iter;
use std::ops::Range;

use mago_reporting::Annotation;
use mago_reporting::AnnotationKind;
use mago_reporting::Issue;

const DETAIL: &str = "\n = ";
const CONTINUATION: &str = "\n   ";
const HELP: &str = "Help: ";

/// Put the headline first, then each distinct detail, note, and help on its own ` = ` line.
///
/// Code spans keep their text as written, line breaks included.
pub fn diagnostic_message(issue: &Issue, annotation: &Annotation) -> String {
    let detail = annotation.message.as_deref().map(str::trim).filter(|message| !message.is_empty());
    // Parser issues keep their specific explanation in the primary annotation.
    let title = if matches!(issue.code.as_deref(), Some("parse" | "syntax")) {
        detail.unwrap_or(issue.message.trim())
    } else {
        issue.message.trim()
    };
    let help = issue.help.as_deref().map(str::trim);
    let mut out = String::with_capacity(
        title.len()
            + detail.map_or(0, |detail| detail.len() + DETAIL.len())
            + issue.notes.iter().map(|note| note.len() + DETAIL.len()).sum::<usize>()
            + help.map_or(0, |help| help.len() + DETAIL.len() + HELP.len()),
    );
    let mut headline = message_lines(title).map(str::trim_start);
    if let Some(first) = headline.next() {
        out.push_str(first);
        for line in headline {
            out.push(' ');
            out.push_str(line);
        }
    }
    if let Some(detail) = detail
        && detail != title
    {
        push_detail(&mut out, "", detail);
    }
    for (index, note) in issue.notes.iter().enumerate() {
        let note = note.trim();
        if !note.is_empty()
            && note != title
            && Some(note) != detail
            && !issue.notes[..index].iter().any(|previous| previous.trim() == note)
        {
            push_detail(&mut out, "", note);
        }
    }
    if let Some(help) = help
        && !help.is_empty()
        && help != title
        && Some(help) != detail
        && !issue.notes.iter().any(|note| note.trim() == help)
    {
        push_detail(&mut out, HELP, help);
    }
    out
}

fn push_detail(out: &mut String, label: &str, text: &str) {
    let mut lines = message_lines(text);
    let Some(first) = lines.next() else {
        return;
    };
    out.push_str(DETAIL);
    out.push_str(label);
    out.push_str(first);
    for line in lines {
        out.push_str(CONTINUATION);
        out.push_str(line);
    }
}

/// Split at the line breaks outside backtick code spans, without trailing whitespace or blank lines.
pub fn message_lines(text: &str) -> impl Iterator<Item = &str> {
    let mut spans = code_spans(text).peekable();
    let mut start = Some(0);
    let mut search = 0;
    iter::from_fn(move || {
        let from = start?;
        loop {
            let Some(offset) = text[search..].find(['\r', '\n']) else {
                start = None;
                return Some(&text[from..]);
            };
            let at = search + offset;
            while spans.next_if(|span| span.end <= at).is_some() {}
            match spans.peek() {
                Some(span) if span.start < at => search = span.end,
                _ => {
                    start = Some(at + 1);
                    search = at + 1;
                    return Some(&text[from..at]);
                }
            }
        }
    })
    .map(str::trim_end)
    .filter(|line| !line.is_empty())
}

/// Byte ranges of the backtick code spans in `text`, from the opening run to the end of the closing run.
///
/// A run closes at the next run of the same length; a run without one is plain text.
pub fn code_spans(text: &str) -> impl Iterator<Item = Range<usize>> + '_ {
    let bytes = text.as_bytes();
    let mut cursor = 0;
    iter::from_fn(move || {
        while let Some((open, width)) = backtick_run(bytes, cursor) {
            cursor = open + width;
            let mut search = cursor;
            while let Some((close, length)) = backtick_run(bytes, search) {
                search = close + length;
                if length == width {
                    cursor = search;
                    return Some(open..search);
                }
            }
        }
        None
    })
}

fn backtick_run(bytes: &[u8], from: usize) -> Option<(usize, usize)> {
    let start = from + bytes[from..].iter().position(|&byte| byte == b'`')?;
    Some((start, bytes[start..].iter().take_while(|&&byte| byte == b'`').count()))
}

/// Keep distinct source explanations, including other primary locations.
#[must_use]
pub fn related_annotations<'a>(issue: &'a Issue, primary: &Annotation) -> Vec<&'a Annotation> {
    let mut annotations: Vec<_> = issue
        .annotations
        .iter()
        .filter(|annotation| {
            annotation.message.as_deref().is_some_and(|message| !message.trim().is_empty())
                && (annotation.span != primary.span
                    || annotation.message.as_deref().map(str::trim) != primary.message.as_deref().map(str::trim))
        })
        .collect();
    annotations.sort_unstable_by_key(|annotation| (annotation.span, annotation.message.as_deref().map(str::trim)));
    annotations.dedup_by_key(|annotation| (annotation.span, annotation.message.as_deref().map(str::trim)));
    annotations
}

/// Known issue kinds that describe obsolete or unused code.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DiagnosticTags {
    pub deprecated: bool,
    pub unnecessary: bool,
}

#[must_use]
pub fn diagnostic_tags(issue: &Issue) -> DiagnosticTags {
    let code = issue.code.as_deref();
    DiagnosticTags {
        deprecated: matches!(
            code,
            Some(
                "deprecated-class"
                    | "deprecated-class-constant"
                    | "deprecated-closure"
                    | "deprecated-constant"
                    | "deprecated-feature"
                    | "deprecated-function"
                    | "deprecated-method"
                    | "deprecated-property"
                    | "deprecated-trait"
                    | "deprecated-cast"
                    | "deprecated-shell-execute-string"
                    | "deprecated-switch-semicolon"
                    | "explicit-nullable-param"
                    | "no-underscore-class"
                    | "no-void-reference-return"
                    | "optional-param-order"
            )
        ),
        unnecessary: matches!(
            code,
            Some(
                "dead-catch"
                    | "unevaluated-code"
                    | "unreachable-else-clause"
                    | "unreachable-match-arm"
                    | "unreachable-match-default-arm"
                    | "unreachable-switch-case"
                    | "unreachable-switch-default"
                    | "unused-function-call"
                    | "unused-method"
                    | "unused-method-call"
                    | "unused-parameter"
                    | "unused-property"
                    | "unused-statement"
                    | "unused-template-parameter"
                    | "useless-control-flow"
                    | "no-noop"
            )
        ),
    }
}

/// Pick a stable primary location and explanation, with the first annotation as a fallback.
#[must_use]
pub fn primary_annotation(issue: &Issue) -> Option<&Annotation> {
    issue
        .annotations
        .iter()
        .filter(|annotation| matches!(annotation.kind, AnnotationKind::Primary))
        .min_by_key(|annotation| {
            let message = annotation.message.as_deref().unwrap_or_default().trim();
            (annotation.span, message.is_empty(), message)
        })
        .or_else(|| issue.annotations.first())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use mago_span::Position;
    use mago_span::Span;
    use mago_syntax::error::Expected;
    use mago_syntax::error::ParseError;
    use mago_syntax::error::SyntaxError;
    use mago_syntax::token::TokenKind;

    use super::*;

    #[test]
    fn primary_location_does_not_depend_on_exception_set_order() {
        let early = Annotation::primary(Span::dummy(1, 4));
        let late = Annotation::primary(Span::dummy(8, 12));
        for annotations in [[early.clone(), late.clone()], [late, early.clone()]] {
            let issue = Issue::error("unhandled exception").with_annotations(annotations);
            assert_eq!(primary_annotation(&issue), Some(&early));
        }
    }

    #[test]
    fn equal_span_primaries_prefer_stable_details_and_keep_other_explanations() {
        let span = Span::dummy(1, 4);
        let blank = Annotation::primary(span).with_message(" ");
        let absent = Annotation::primary(span);
        let first = Annotation::primary(span).with_message(" A specific explanation. ");
        let other = Annotation::primary(span).with_message("Another explanation.");
        for (annotations, related) in [
            (vec![absent.clone(), blank.clone(), first.clone()], vec![]),
            (vec![other, first.clone(), blank, absent], vec!["Another explanation."]),
        ] {
            for reverse in [false, true] {
                let mut annotations = annotations.clone();
                if reverse {
                    annotations.reverse();
                }
                let issue = Issue::error("The issue.").with_annotations(annotations);
                let primary = primary_annotation(&issue).unwrap();
                assert_eq!(primary, &first);
                assert_eq!(diagnostic_message(&issue, primary), "The issue.\n = A specific explanation.");
                assert_eq!(
                    related_annotations(&issue, primary)
                        .iter()
                        .map(|annotation| annotation.message.as_deref().unwrap().trim())
                        .collect::<Vec<_>>(),
                    related
                );
            }
        }
    }

    #[test]
    fn parser_messages_start_with_the_specific_error_and_keep_notes_and_help() {
        let span = Span::dummy(1, 2);
        let errors = [
            ParseError::UnexpectedToken(Expected::Exactly(TokenKind::RightParenthesis), TokenKind::Semicolon, span),
            ParseError::SyntaxError(SyntaxError::UnrecognizedToken(span.file_id, 0, Position::new(1))),
        ];
        for error in errors {
            let issue = Issue::from(&error);
            let message = diagnostic_message(&issue, primary_annotation(&issue).unwrap());
            assert_eq!(message.lines().next(), Some(error.to_string().as_str()));
            for note in &issue.notes {
                assert!(message.contains(note.as_ref()));
            }
            assert!(message.contains(issue.help.as_deref().unwrap()));
            assert!(!message.contains(issue.message.as_ref()));
            assert_single_spaced(&message);
        }
    }

    #[test]
    fn diagnostic_messages_keep_distinct_primary_details_without_repeating_the_title() {
        let issue = Issue::error("Invalid return type").with_note("Expected int.").with_help("Return an integer.");
        for (detail, expected) in [
            (None, "Invalid return type"),
            (Some("  "), "Invalid return type"),
            (Some("Invalid return type"), "Invalid return type"),
            (Some("This expression has type string."), "Invalid return type\n = This expression has type string."),
        ] {
            let annotation = Annotation { message: detail.map(Into::into), ..Annotation::primary(Span::dummy(0, 1)) };
            assert_eq!(
                diagnostic_message(&issue, &annotation),
                format!("{expected}\n = Expected int.\n = Help: Return an integer.")
            );
        }
    }

    #[test]
    fn blank_and_repeated_blocks_do_not_hide_distinct_details() {
        let annotation = Annotation::primary(Span::dummy(0, 1)).with_message(" Specific detail. ");
        let issue = Issue::error(" A title. ")
            .with_note(" ")
            .with_note(" A title. ")
            .with_note(" Specific detail. ")
            .with_note(" A useful note. ")
            .with_note("A useful note.")
            .with_help(" A next step. ");
        assert_eq!(
            diagnostic_message(&issue, &annotation),
            "A title.\n = Specific detail.\n = A useful note.\n = Help: A next step."
        );
        for help in [" ", "A title.", "Specific detail.", "A useful note."] {
            assert!(!diagnostic_message(&issue.clone().with_help(help), &annotation).contains("Help:"));
        }
    }

    #[test]
    fn multi_line_blocks_use_a_hanging_indent_and_drop_blank_lines() {
        let annotation = Annotation::primary(Span::dummy(0, 1)).with_message("detail\r\r  continued");
        let issue = Issue::error(" Disallowed\r\n\n  type. ")
            .with_note("first line\r\n\n  indented second\n \t\n")
            .with_help("multi\rline help\r\n");
        let message = diagnostic_message(&issue, &annotation);
        assert_eq!(
            message,
            "Disallowed type.\n = detail\n     continued\n = first line\n     indented second\n = Help: multi\n   line help"
        );
        assert_single_spaced(&message);
    }

    #[test]
    fn messages_never_contain_blank_lines() {
        let span = Span::dummy(1, 2);
        let mut issues: Vec<_> = [
            ParseError::UnexpectedToken(Expected::Exactly(TokenKind::RightParenthesis), TokenKind::Semicolon, span),
            ParseError::SyntaxError(SyntaxError::UnrecognizedToken(span.file_id, 0, Position::new(1))),
        ]
        .iter()
        .map(Issue::from)
        .collect();
        issues.push(
            Issue::error("A title.\n\n")
                .with_annotation(Annotation::primary(span).with_message("\n\nA detail.\n\n"))
                .with_note("A note.\r\n\r\nWith more.")
                .with_help("\n\nA next step."),
        );
        issues.push(Issue::error("Only a title."));
        for issue in issues {
            let message = diagnostic_message(&issue, primary_annotation(&issue).unwrap_or(&Annotation::primary(span)));
            assert_single_spaced(&message);
        }
    }

    #[test]
    fn code_spans_keep_their_line_breaks_as_written() {
        let annotation = Annotation::primary(Span::dummy(0, 1))
            .with_message("This has type `string('SELECT id,\n       name\nFROM users\n\nWHERE id = 1')`");
        let issue = Issue::error("expected `string('a\\nb')`, but found `string('a\nb')`.\nSee the\n  docs.")
            .with_note("`string('x\n\n  y')` here")
            .with_note("Odd ``` tick\r\nthen ``a`\r\nb`` and `c\rd`\r\n\r\n  done")
            .with_help("Use `'\r\n'`:\n`return [\n        1,\n    ];`");
        let message = diagnostic_message(&issue, &annotation);
        assert_eq!(
            message,
            "expected `string('a\\nb')`, but found `string('a\nb')`. See the docs.\n = This has type `string('SELECT id,\n       name\nFROM users\n\nWHERE id = 1')`\n = `string('x\n\n  y')` here\n = Odd ``` tick\n   then ``a`\r\nb`` and `c\rd`\n     done\n = Help: Use `'\r\n'`:\n   `return [\n        1,\n    ];`"
        );
        let headline: Vec<_> = code_spans(&message).take(2).map(|span| &message[span]).collect();
        assert_eq!(headline, ["`string('a\\nb')`", "`string('a\nb')`"]);
    }

    #[test]
    fn lines_break_only_outside_code_spans() {
        let spans = |text| code_spans(text).map(|span| &text[span]).collect::<Vec<_>>();
        assert_eq!(spans("`a`\n`b`"), ["`a`", "`b`"]);
        assert_eq!(spans("Odd ``` tick `a\nb` and ``c`d``"), ["`a\nb`", "``c`d``"]);
        for text in ["plain", "Odd ` tick\nand prose", "``a`\nb", "`a\nb"] {
            assert!(spans(text).is_empty(), "{text:?}");
        }
        let lines = |text| message_lines(text).collect::<Vec<_>>();
        assert_eq!(lines("first\r\n\r\n  `a\r\n\r\nb` then  \rlast  "), ["first", "  `a\r\n\r\nb` then", "last"]);
        assert_eq!(lines("`a`\n`b`"), ["`a`", "`b`"]);
        assert_eq!(lines("Odd ` tick\nand ``a``\nb"), ["Odd ` tick", "and ``a``", "b"]);
        assert_eq!(lines("Odd ` tick\nand ``a`\nb"), ["Odd ` tick\nand ``a`", "b"]);
        assert_eq!(lines("one line"), ["one line"]);
        assert!(lines(" \r\n\t").is_empty());
    }

    fn assert_single_spaced(message: &str) {
        assert!(!message.contains("\n\n") && !message.contains('\r'), "{message:?}");
        assert!(!message.lines().next().unwrap().starts_with(' '), "{message:?}");
        for line in message.lines().skip(1) {
            assert!(line.starts_with(" = ") || line.starts_with("   "), "{message:?}");
            assert!(!line.trim().is_empty() && line.trim_end() == line, "{message:?}");
        }
    }

    #[test]
    fn related_locations_keep_other_primaries_in_source_order_without_duplicates() {
        let primary = Annotation::primary(Span::dummy(1, 2)).with_message(" First cause. ");
        let early = Annotation::secondary(Span::dummy(3, 4)).with_message(" Earlier context. ");
        let late = Annotation::primary(Span::dummy(8, 9)).with_message("Another cause.");
        let duplicate = Annotation::secondary(late.span).with_message(" Another cause. ");
        let blank = Annotation::secondary(Span::dummy(5, 6)).with_message(" ");
        let same_span = Annotation::secondary(primary.span).with_message("A distinct explanation.");
        for annotations in [
            vec![late.clone(), primary.clone(), duplicate.clone(), blank.clone(), early.clone(), same_span.clone()],
            vec![same_span, early.clone(), blank, duplicate, primary.clone(), late.clone()],
        ] {
            let issue = Issue::error("Multiple causes.").with_annotations(annotations);
            let related = related_annotations(&issue, primary_annotation(&issue).unwrap());
            assert_eq!(related.len(), 3);
            assert_eq!([related[0].span, related[1].span, related[2].span], [primary.span, early.span, late.span]);
        }
    }

    #[test]
    fn tags_require_a_known_semantic_issue_kind() {
        for code in ["deprecated-function", "explicit-nullable-param"] {
            assert_eq!(
                diagnostic_tags(&Issue::warning("old").with_code(code)),
                DiagnosticTags { deprecated: true, unnecessary: false }
            );
        }
        for code in ["unused-statement", "unevaluated-code", "no-noop"] {
            assert_eq!(
                diagnostic_tags(&Issue::note("unused").with_code(code)),
                DiagnosticTags { deprecated: false, unnecessary: true }
            );
        }
        for code in ["invalid-return-statement", "no-empty-loop", "unused-custom-rule", "deprecated-custom-rule"] {
            assert_eq!(diagnostic_tags(&Issue::warning("plain").with_code(code)), DiagnosticTags::default());
        }
    }
}
