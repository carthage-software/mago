use mago_allocator::LocalArena;
use mago_database::file::File;
use mago_span::HasSpan;
use mago_syntax::cst::{ClassLikeMember, MethodBody, Statement};
use mago_syntax::parser::parse_file_borrowed_with_settings;
use mago_syntax::settings::ParserSettings;

#[test]
fn editor_recovery_keeps_nested_bodies_and_reports_missing_delimiters() {
    let source = b"<?php class C { function work($value) { take([1, 2], $value  ";
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_slice().into());
    let arena = LocalArena::new();
    let strict = parse_file_borrowed_with_settings(&arena, &file, ParserSettings::default());
    assert!(!strict.statements.iter().any(|statement| matches!(statement, Statement::Class(_))));
    assert!(!strict.errors.is_empty());
    let recovered = parse_file_borrowed_with_settings(
        &arena,
        &file,
        ParserSettings { recover_incomplete: true, ..Default::default() },
    );
    let Some(class) = recovered.statements.iter().find_map(|statement| match statement {
        Statement::Class(class) => Some(class),
        _ => None,
    }) else {
        panic!("missing class")
    };
    let ClassLikeMember::Method(method) = &class.members[0] else { panic!("missing method") };
    let MethodBody::Concrete(body) = &method.body else { panic!("missing body") };
    assert_eq!(body.statements.len(), 1);
    assert_eq!(class.end_offset(), file.size);
    assert!(recovered.errors.len() >= 3);
}

#[test]
fn editor_recovery_handles_each_truncated_prefix_without_changing_valid_trees() {
    for source in [
        "<?php namespace App { class C { function work($value) { return take([1, 2], ($value)); } } }",
        "<?php $f = fn($x) => [$x, 1]; $g = function($y) use ($f) { return $f($y); };",
        "<?php interface I { function f(): void; } trait T { function f(): void {} } enum E { case A; }",
    ] {
        let settings = ParserSettings { recover_incomplete: true, ..Default::default() };
        for end in 0..=source.len() {
            let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes()[..end].into());
            let arena = LocalArena::new();
            let recovered = parse_file_borrowed_with_settings(&arena, &file, settings);
            assert!(recovered.statements.iter().all(|statement| statement.end_offset() <= file.size));
            if end == source.len() {
                let strict = parse_file_borrowed_with_settings(&arena, &file, ParserSettings::default());
                assert!(recovered.errors.is_empty());
                assert_eq!(format!("{strict:?}"), format!("{recovered:?}"));
            }
        }
    }
}
