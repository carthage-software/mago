#![allow(clippy::unwrap_used)]

mod support;

use std::borrow::Cow;

use mago_allocator::LocalArena;
use mago_database::file::File;
use mago_span::HasSpan;
use mago_twig_linter::settings::Settings;

#[test]
fn trailing_pragmas_apply_to_their_line_not_the_following_statement() {
    for newline in ["\n", "\r\n", "\r"] {
        let source = include_str!("fixtures/trailing_pragma.twig").replace('\n', newline);
        let arena = LocalArena::new();
        let only = ["twig-no-dump"];
        let file = File::ephemeral(Cow::Borrowed(b"trailing.twig"), Cow::Owned(source.as_bytes().to_vec()));
        let issues = support::lint(&arena, &file, &Settings::default(), &only);
        assert_eq!(issues.len(), 1, "{issues:?}");
        let issue = issues.iter().next().unwrap();
        assert_eq!(issue.code.as_deref(), Some("twig-no-dump"));
        let span = issue.annotations.iter().find(|annotation| annotation.is_primary()).unwrap().span;
        assert_eq!(span.start_offset(), source.find("dump(second)").unwrap() as u32);
    }
}

#[test]
fn consecutive_comments_share_the_next_statement_scope_without_rescanning_the_tail() {
    let source = format!("{}{{{{ value }}}}", "{# @mago-expect lint:twig-no-dump #}\n".repeat(5000));
    let arena = LocalArena::new();
    let only = ["twig-no-dump"];
    let file = File::ephemeral(Cow::Borrowed(b"comments.twig"), Cow::Owned(source.into_bytes()));
    let issues = support::lint(&arena, &file, &Settings::default(), &only);
    assert_eq!(issues.len(), 5000);
    assert!(issues.iter().all(|issue| issue.code.as_deref() == Some("unfulfilled-expect")));
}

#[test]
fn orphan_own_line_pragmas_do_not_reach_past_text_or_closing_tags() {
    for newline in ["\n", "\r\n", "\r"] {
        let source = include_str!("fixtures/orphan_pragmas.twig").replace('\n', newline);
        let arena = LocalArena::new();
        let only = ["twig-no-dump"];
        let file = File::ephemeral(Cow::Borrowed(b"orphan.twig"), Cow::Owned(source.as_bytes().to_vec()));
        let issues = support::lint(&arena, &file, &Settings::default(), &only);
        assert_eq!(issues.len(), 4, "{issues:?}");
        let reported: Vec<_> = issues.iter().filter(|issue| issue.code.as_deref() == Some("twig-no-dump")).collect();
        assert_eq!(reported.len(), 2);
        for (issue, name) in reported.into_iter().zip(["first", "second"]) {
            let primary = issue.annotations.iter().find(|annotation| annotation.is_primary()).unwrap();
            assert_eq!(primary.span.start_offset(), source.find(&format!("dump({name})")).unwrap() as u32);
            assert_eq!(primary.span.end_offset(), primary.span.start_offset() + 4);
        }
        assert_eq!(issues.iter().filter(|issue| issue.code.as_deref() == Some("unfulfilled-expect")).count(), 1);
        assert_eq!(issues.iter().filter(|issue| issue.code.as_deref() == Some("unused-pragma")).count(), 1);
    }
}

#[test]
fn inline_pragmas_stop_at_their_code_tag_not_the_statement_body() {
    for newline in ["\n", "\r\n", "\r"] {
        let source = include_str!("fixtures/inline_pragmas.twig").replace('\n', newline);
        let arena = LocalArena::new();
        let only = ["twig-single-quotes"];
        let file = File::ephemeral(Cow::Borrowed(b"inline.twig"), Cow::Owned(source.as_bytes().to_vec()));
        let issues = support::lint(&arena, &file, &Settings::default(), &only);
        assert_eq!(issues.len(), 4, "{issues:?}");
        let reported: Vec<_> =
            issues.iter().filter(|issue| issue.code.as_deref() == Some("twig-single-quotes")).collect();
        assert_eq!(reported.len(), 3);
        for (issue, text) in reported.into_iter().zip(["\"outside\"", "\"body\"", "\"after_close\""]) {
            let primary = issue.annotations.iter().find(|annotation| annotation.is_primary()).unwrap();
            let start = source.find(text).unwrap() as u32;
            assert_eq!((primary.span.start_offset(), primary.span.end_offset()), (start, start + text.len() as u32));
        }
        assert_eq!(issues.iter().filter(|issue| issue.code.as_deref() == Some("unfulfilled-expect")).count(), 1);
    }
}
