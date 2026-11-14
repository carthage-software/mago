#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::borrow::Cow;

use mago_allocator::LocalArena;
use mago_allocator::prelude::*;
use mago_collector::Collector;
use mago_collector::pragma::Pragma;
use mago_database::file::File;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::Span;
use mago_text_edit::ApplyResult;
use mago_text_edit::TextEditor;

fn pragmas<'arena>(arena: &'arena LocalArena, file: &File) -> Vec<'arena, Pragma<'arena>, LocalArena> {
    let source = arena.alloc_slice_copy(&file.contents);
    let mut pragmas = Vec::new_in(arena);
    let mut cursor = 0;
    while let Some(start) = source[cursor..].windows(2).position(|text| text == b"{#") {
        let start = cursor + start;
        let end = start + source[start..].windows(2).position(|text| text == b"#}").unwrap() + 2;
        let body_start = start + 2 + usize::from(matches!(source.get(start + 2), Some(b'-' | b'~')));
        let body_end = end - 2 - usize::from(matches!(source.get(end - 3), Some(b'-' | b'~')));
        let lines = source[body_start..body_end]
            .split(|byte| matches!(byte, b'\n' | b'\r'))
            .map(|line| ((line.as_ptr() as usize - source.as_ptr() as usize - start) as u32, line));
        let span = Span::new(file.id, (start as u32).into(), (end as u32).into());
        pragmas.extend(Pragma::parse_comment(arena, file, span, lines, &["lint", "linter"]));
        cursor = end;
    }

    pragmas
}

#[test]
fn preserves_comments_and_text_with_non_overlapping_grouped_edits() {
    for newline in ["\n", "\r\n", "\r"] {
        for marker in ["", "-", "~"] {
            let arena = LocalArena::new();
            let source = format!(
                "A \n{{#{marker}\n @mago-expect lint:first, linter:second, kept(3) Keep this reason.\n @mago-expect lint:third, fourth\n ordinary text\n{marker}#}} \n{{{{ value }}}}\n{{#{marker} @mago-ignore lint:fifth, sixth {marker}#}}\nB"
            ).replace('\n', newline);
            let file = File::ephemeral(Cow::Borrowed(b"test.twig"), Cow::Owned(source.as_bytes().to_vec()));
            let mut collector = Collector::from_pragmas(&arena, &file, pragmas(&arena, &file));
            let start = source.find("value").unwrap() as u32;
            let span = Span::new(file.id, start.into(), (start + 5).into());
            for _ in 0..2 {
                assert!(
                    !collector
                        .report(Issue::warning("test").with_code("kept").with_annotation(Annotation::primary(span)))
                );
            }

            let issues = collector.finish();
            let edits: std::vec::Vec<_> =
                issues.into_iter().flat_map(|issue| issue.edits.into_values().flatten()).collect();
            assert_eq!(edits.len(), 3);
            let mut editor = TextEditor::new(&file.contents);
            for edit in edits {
                assert_eq!(editor.apply::<fn(&[u8]) -> bool>(edit, None), ApplyResult::Applied);
            }

            let expected = format!(
                "A \n{{#{marker}\n @mago-expect lint:kept(2) Keep this reason.\n  \n ordinary text\n{marker}#}} \n{{{{ value }}}}\n{{#{marker}  {marker}#}}\nB"
            ).replace('\n', newline);
            assert_eq!(editor.finish(), expected.as_bytes());
        }
    }
}

#[test]
fn empty_directives_do_not_join_trim_markers_to_the_other_delimiter() {
    for newline in ["\n", "\r\n", "\r"] {
        for left in ["", "-", "~"] {
            for right in ["", "-", "~"] {
                let arena = LocalArena::new();
                let source = format!("A  {newline}{{#{left}@mago-expect lint:unused{right}#}}  {newline}B");
                let file = File::ephemeral(Cow::Borrowed(b"trim.twig"), Cow::Owned(source.into_bytes()));
                let issues = Collector::from_pragmas(&arena, &file, pragmas(&arena, &file)).finish();
                assert_eq!(issues.len(), 1);
                let edits: std::vec::Vec<_> =
                    issues.into_iter().flat_map(|issue| issue.edits.into_values().flatten()).collect();
                assert_eq!(edits.len(), 1);
                let mut editor = TextEditor::new(&file.contents);
                assert_eq!(editor.apply::<fn(&[u8]) -> bool>(edits[0].clone(), None), ApplyResult::Applied);
                assert_eq!(editor.finish(), format!("A  {newline}{{#{left} {right}#}}  {newline}B").as_bytes());
            }
        }
    }
}

#[test]
fn retains_inactive_expectations_and_reconciles_deferred_counts() {
    let arena = LocalArena::new();
    let source = b"{# @mago-expect lint:inactive(2), kept(3), unused A reason. #}\n{{ value }}";
    let file = File::ephemeral(Cow::Borrowed(b"test.twig"), Cow::Borrowed(source));
    let mut collector = Collector::from_pragmas(&arena, &file, pragmas(&arena, &file));
    collector.set_active_codes(&["kept".into(), "unused".into()]);
    let start = source.windows(5).position(|text| text == b"value").unwrap() as u32;
    let span = Span::new(file.id, start.into(), (start + 5).into());
    assert!(!collector.report(Issue::warning("test").with_code("kept").with_annotation(Annotation::primary(span))));
    let (_, state) = collector.defer();
    let issues = state.unwrap().finish(&file);
    let edits: std::vec::Vec<_> = issues.into_iter().flat_map(|issue| issue.edits.into_values().flatten()).collect();
    assert_eq!(edits.len(), 1);
    let mut editor = TextEditor::new(source);
    assert_eq!(editor.apply::<fn(&[u8]) -> bool>(edits[0].clone(), None), ApplyResult::Applied);
    assert_eq!(editor.finish(), b"{# @mago-expect lint:inactive(2), kept(1) A reason. #}\n{{ value }}");
}

#[test]
fn retains_explicit_one_count_when_a_scoped_count_fix_is_reparsed() {
    let arena = LocalArena::new();
    let source = b"{# @mago-expect lint:kept(3) #}\n{{ value }}";
    let file = File::ephemeral(Cow::Borrowed(b"test.twig"), Cow::Borrowed(source));
    let start = source.windows(5).position(|text| text == b"value").unwrap() as u32;
    let span = Span::new(file.id, start.into(), (start + 5).into());
    let mut parsed = pragmas(&arena, &file);
    parsed[0].scope_span = Some(span);
    let mut collector = Collector::from_pragmas(&arena, &file, parsed);
    let issue = || Issue::warning("test").with_code("kept").with_annotation(Annotation::primary(span));
    assert!(!collector.report(issue()));
    let mut editor = TextEditor::new(source);
    for edit in collector.finish().into_iter().flat_map(|issue| issue.edits.into_values().flatten()) {
        assert_eq!(editor.apply::<fn(&[u8]) -> bool>(edit, None), ApplyResult::Applied);
    }

    let fixed = File::ephemeral(Cow::Borrowed(b"test.twig"), Cow::Owned(editor.finish()));
    let mut parsed = pragmas(&arena, &fixed);
    parsed[0].scope_span = Some(span);
    assert_eq!(parsed[0].expected_matches, 1);
    assert!(parsed[0].count_span.is_some());
    let mut collector = Collector::from_pragmas(&arena, &fixed, parsed);
    assert!(!collector.report(issue()));
    assert!(collector.report(issue()));
}

#[test]
fn skips_non_utf8_pragma_text_without_touching_template_bytes() {
    let arena = LocalArena::new();
    let source = b"\xff \n{# @mago-expect lint:unknown reason\xff #}\n{{ value }}\xfe";
    let file = File::ephemeral(Cow::Borrowed(b"test.twig"), Cow::Borrowed(source));
    let parsed = pragmas(&arena, &file);
    assert!(parsed.is_empty());
    assert!(Collector::from_pragmas(&arena, &file, parsed).finish().is_empty());
}
