use std::borrow::Cow;
use std::sync::Arc;

use bumpalo::Bump;
use mago_database::file::File;
use mago_linter::Linter;
use mago_linter::registry::RuleRegistry;
use mago_linter::settings::Settings;
use mago_names::resolver::NameResolver;
use mago_syntax::parser::parse_file;

const CODE: &str = "<?php\n\n// comment with trailing space   \n";

fn lint_with_excludes(filename: &'static str, excludes: &[&str]) -> usize {
    let arena = Bump::new();
    let file = File::ephemeral(Cow::Borrowed(filename), Cow::Borrowed(CODE));
    let program = parse_file(&arena, &file);

    let resolver = NameResolver::new(&arena);
    let resolved_names = resolver.resolve(program);

    let mut settings = Settings::default();
    settings.rules.no_trailing_space.exclude = excludes.iter().map(|s| (*s).to_string()).collect();

    let registry = RuleRegistry::build(&settings, Some(&["no-trailing-space".to_string()]), true);
    let linter = Linter::from_registry(&arena, Arc::new(registry), settings.php_version);
    let issues = linter.lint(&file, program, &resolved_names);

    issues.len()
}

#[test]
fn plain_directory_prefix_still_excludes() {
    // Historic behaviour: bare "tests" excludes "tests/...".
    assert_eq!(lint_with_excludes("tests/Foo.php", &["tests"]), 0);
    assert_ne!(lint_with_excludes("src/Foo.php", &["tests"]), 0);
}

#[test]
fn trailing_slash_prefix_still_excludes() {
    assert_eq!(lint_with_excludes("tests/Foo.php", &["tests/"]), 0);
    assert_ne!(lint_with_excludes("src/Foo.php", &["tests/"]), 0);
}

#[test]
fn exact_file_prefix_still_excludes() {
    assert_eq!(lint_with_excludes("src/Foo.php", &["src/Foo.php"]), 0);
    assert_ne!(lint_with_excludes("src/Bar.php", &["src/Foo.php"]), 0);
}

#[test]
fn double_star_glob_excludes_nested_files() {
    assert_eq!(lint_with_excludes("src/a/b/Foo.php", &["src/**/*.php"]), 0);
    assert_eq!(lint_with_excludes("src/Foo.php", &["src/**/*.php"]), 0);
    assert_ne!(lint_with_excludes("tests/Foo.php", &["src/**/*.php"]), 0);
}

#[test]
fn suffix_glob_excludes_test_files() {
    assert_eq!(lint_with_excludes("tests/Unit/FooTest.php", &["**/*Test.php"]), 0);
    assert_ne!(lint_with_excludes("src/Foo.php", &["**/*Test.php"]), 0);
}

#[test]
fn mixed_glob_and_prefix_both_work() {
    let excludes = &["vendor", "tests/**/*.php"];
    assert_eq!(lint_with_excludes("vendor/lib/Foo.php", excludes), 0);
    assert_eq!(lint_with_excludes("tests/Unit/Foo.php", excludes), 0);
    assert_ne!(lint_with_excludes("src/Foo.php", excludes), 0);
}
