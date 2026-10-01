#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::borrow::Cow;

use mago_allocator::LocalArena;
use mago_collector::Collector;
use mago_collector::pragma::Pragma;
use mago_collector::pragma::PragmaKind;
use mago_database::file::File;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Position;
use mago_span::Span;
use mago_syntax::parser::parse_file;
use mago_text_edit::ApplyResult;
use mago_text_edit::TextEditor;

#[test]
fn parses_spaced_lists_and_repeated_categories() {
    let arena = LocalArena::new();
    let file = File::ephemeral(Cow::Borrowed(b"lists.php"), Cow::Borrowed(include_bytes!("cases/lists.php")));
    let program = parse_file(&arena, &file);
    assert!(!program.has_errors());

    let pragmas = Pragma::extract(&arena, &file, program.trivia.as_slice(), &["analysis", "analyzer", "analyser"]);
    assert_eq!(pragmas.len(), 21);

    for (index, group) in pragmas.as_chunks::<3>().0.iter().enumerate() {
        for (pragma, (code, count)) in group.iter().zip([("first", 1), ("second", 2), ("third", 3)]) {
            assert_eq!(pragma.kind, if index == 6 { PragmaKind::Ignore } else { PragmaKind::Expect });
            assert_eq!(pragma.code, code);
            assert_eq!(pragma.expected_matches, count);
            assert_eq!(pragma.description, "Expected issues, with counts.");

            let code_text =
                &file.contents[pragma.code_span.start_offset() as usize..pragma.code_span.end_offset() as usize];
            assert!(code_text.ends_with(format!("{code}({count})").as_bytes()));

            let count_span = pragma.count_span.unwrap();
            assert_eq!(
                &file.contents[count_span.start_offset() as usize..count_span.end_offset() as usize],
                format!("({count})").as_bytes()
            );
        }
    }
}

#[test]
fn fixes_unused_codes_and_partial_counts_in_spaced_lists() {
    let arena = LocalArena::new();
    let file = File::ephemeral(Cow::Borrowed(b"fixes.php"), Cow::Borrowed(include_bytes!("cases/fixes.php")));
    let program = parse_file(&arena, &file);
    assert!(!program.has_errors());

    let mut collector = Collector::new(&arena, &file, program, &["analysis", "analyzer", "analyser"]);
    for (call, code, count) in [
        ("first()", "used", 1),
        ("last()", "used", 1),
        ("middle()", "used", 1),
        ("middle()", "counted", 2),
        ("partial()", "used", 1),
        ("partial()", "counted", 2),
        ("shared()", "used", 1),
        ("alias()", "used", 1),
        ("ignored()", "used", 1),
    ] {
        let start = file.contents.windows(call.len()).position(|text| text == call.as_bytes()).unwrap() as u32;
        let span = Span::new(file.id, Position::new(start), Position::new(start + call.len() as u32));
        for _ in 0..count {
            assert!(
                !collector
                    .report(Issue::warning("Test issue").with_code(code).with_annotation(Annotation::primary(span)))
            );
        }
    }

    let issues = collector.finish();
    assert_eq!(issues.len(), 7);

    let mut editor = TextEditor::new(&file.contents);
    for issue in issues {
        for edit in issue.edits.into_values().flatten() {
            assert_eq!(editor.apply::<fn(&[u8]) -> bool>(edit, None), ApplyResult::Applied);
        }
    }

    assert_eq!(editor.finish(), include_bytes!("cases/fixes.fixed.php"));
}
