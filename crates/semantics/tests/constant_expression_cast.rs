use std::borrow::Cow;

use mago_allocator::LocalArena;

use mago_database::file::File;
use mago_names::resolver::NameResolver;
use mago_php_version::PHPVersion;
use mago_semantics::SemanticsChecker;
use mago_syntax::parser::parse_file;

fn has_semantics_issue(source: &str, version: PHPVersion) -> bool {
    let arena = LocalArena::new();
    let file = File::ephemeral(Cow::Borrowed(b"test.php"), Cow::Owned(source.as_bytes().to_vec()));
    let program = parse_file(&arena, &file);
    assert!(program.errors.is_empty(), "test source did not parse: {:?}", program.errors);

    let names = NameResolver::new(&arena).resolve(program);
    let issues = SemanticsChecker::new(version).check(&file, program, &names);

    issues.iter().any(|issue| issue.code.as_deref() == Some("semantics"))
}

const SOURCE: &str = "<?php final class K { public const FIVE = (int) '5'; public static int $seven = (int) '7'; }";

#[test]
fn cast_in_constant_expression_is_allowed_on_php_8_5() {
    assert!(!has_semantics_issue(SOURCE, PHPVersion::new(8, 5, 0)));
}

#[test]
fn cast_in_constant_expression_is_rejected_before_php_8_5() {
    assert!(has_semantics_issue(SOURCE, PHPVersion::new(8, 4, 0)));
}
