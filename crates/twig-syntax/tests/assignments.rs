#![allow(clippy::unwrap_used)]

use mago_allocator::LocalArena;
use mago_database::file::FileId;
use mago_twig_syntax::parser::parse_file_content;

#[test]
fn validates_variable_and_destructuring_assignment_targets() {
    for source in include_str!("fixtures/expressions/valid_assignment_targets.twig").lines() {
        let arena = LocalArena::new();
        let template = parse_file_content(&arena, FileId::new(b"assignment.twig"), source.as_bytes());
        assert!(!template.has_errors(), "Valid assignment rejected: {source}: {:?}", template.errors);
    }
    for source in include_str!("fixtures/expressions/invalid_assignment_targets.twig").lines() {
        let arena = LocalArena::new();
        let template = parse_file_content(&arena, FileId::new(b"assignment.twig"), source.as_bytes());
        assert!(template.has_errors(), "Invalid assignment accepted: {source}");
    }
}
