use std::borrow::Cow;

use mago_allocator::LocalArena;

use mago_database::file::File;
use mago_names::resolver::NameResolver;
use mago_php_version::PHPVersion;
use mago_semantics::SemanticsChecker;
use mago_syntax::parser::parse_file;

fn count_semantics(source: &str, version: PHPVersion) -> usize {
    let arena = LocalArena::new();
    let file = File::ephemeral(Cow::Borrowed(b"test.php"), Cow::Owned(source.as_bytes().to_vec()));
    let program = parse_file(&arena, &file);
    assert!(program.errors.is_empty(), "test source did not parse: {:?}", program.errors);

    let names = NameResolver::new(&arena).resolve(program);
    let issues = SemanticsChecker::new(version).check(&file, program, &names);

    issues.iter().filter(|issue| issue.code.as_deref() == Some("semantics")).count()
}

const ENUM_DEBUG_INFO: [&str; 4] = [
    "enum E { case A; public function __debugInfo(): array { return []; } }",
    "enum E: string { case A = 'a'; public function __debugInfo(): array { return ['a' => 1]; } }",
    "enum E { case A; public function __DEBUGINFO(): array { return []; } }",
    "interface I {} enum E implements I { case A; public function __debugInfo() { return []; } }",
];

#[test]
fn enum_debug_info_is_rejected_before_php_8_6() {
    for source in ENUM_DEBUG_INFO {
        assert_eq!(count_semantics(&format!("<?php {source}"), PHPVersion::PHP85), 1, "{source}");
    }
}

#[test]
fn enum_debug_info_is_accepted_from_php_8_6() {
    for source in ENUM_DEBUG_INFO {
        assert_eq!(count_semantics(&format!("<?php {source}"), PHPVersion::PHP86), 0, "{source}");
    }
}

#[test]
fn other_enum_magic_methods_are_still_rejected_in_php_8_6() {
    for source in [
        "enum E { case A; public function __toString(): string { return ''; } }",
        "enum E { case A; public function __get(string $name): mixed { return null; } }",
        "enum E { case A; public function __serialize(): array { return []; } }",
        "enum E { case A; public function __sleep(): array { return []; } }",
    ] {
        assert_eq!(count_semantics(&format!("<?php {source}"), PHPVersion::PHP86), 1, "{source}");
    }
}

#[test]
fn debug_info_outside_enums_and_allowed_enum_magic_methods_are_accepted() {
    for source in [
        "class C { public function __debugInfo(): array { return []; } }",
        "enum E { case A; public function __call(string $name, array $arguments): mixed { return null; } }",
        "enum E { case A; public function __invoke(): int { return 1; } }",
    ] {
        assert_eq!(count_semantics(&format!("<?php {source}"), PHPVersion::PHP85), 0, "{source}");
    }
}
