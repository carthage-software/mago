#![allow(clippy::unwrap_used)]

use std::borrow::Cow;

use mago_allocator::LocalArena;
use mago_database::file::File;
use mago_span::HasSpan;
use mago_twig_syntax::comments::CommentScopes;
use mago_twig_syntax::parser::parse_file;

#[test]
fn finds_nested_statements_through_comment_and_whitespace_gaps() {
    let source = include_bytes!("fixtures/comment_scopes.twig");
    let arena = LocalArena::new();
    let file = File::ephemeral(Cow::Borrowed(b"scope.twig"), Cow::Borrowed(source));
    let template = parse_file(&arena, &file);
    assert!(!template.has_errors());
    let comments: Vec<_> = template.trivia.iter().filter(|trivia| trivia.kind.is_comment()).collect();
    assert_eq!(comments.len(), 4);
    let scopes = CommentScopes::new(template);
    let first = scopes.following_statement(comments[0].span).unwrap();
    let second = scopes.following_statement(comments[1].span).unwrap();
    assert_eq!(first, second);
    assert!(source[first.start_offset() as usize..first.end_offset() as usize].starts_with(b"{% if"));
    let inner = scopes.following_statement(comments[2].span).unwrap();
    assert_eq!(&source[inner.start_offset() as usize..inner.end_offset() as usize], b"{{ value }}");
    assert!(scopes.following_statement(comments[3].span).is_none());
}

#[test]
fn inline_comments_stop_at_opening_branch_and_closing_code_tags() {
    let source = include_bytes!("fixtures/comment_inline_scopes.twig");
    let arena = LocalArena::new();
    let file = File::ephemeral(Cow::Borrowed(b"inline.twig"), Cow::Borrowed(source));
    let template = parse_file(&arena, &file);
    assert!(!template.has_errors(), "{:?}", template.errors);
    let scopes = CommentScopes::new(template);
    let mut count = 0;
    for comment in template.trivia.iter().filter(|trivia| trivia.kind.is_comment()) {
        let scope = scopes.inline_scope(comment.span).unwrap();
        assert_eq!(scope.start, comment.span.end);
        let after = &source[scope.start_offset() as usize..];
        let close = after
            .split_inclusive(|byte| *byte == b'\n')
            .scan(0, |length, line| {
                *length += line.len();
                Some((*length, line.trim_ascii()))
            })
            .find(|(_, line)| matches!(*line, b"%}" | b"~}}" | b") %}"));
        let (length, _) = close.unwrap();
        assert_eq!(scope.end_offset(), scope.start_offset() + length as u32 - 1);
        count += 1;
    }
    assert_eq!(count, 19);
}
