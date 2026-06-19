use std::borrow::Cow;
use std::sync::Arc;

use mago_allocator::LocalArena;
use mago_database::file::File;
use mago_linter::Linter;
use mago_linter::registry::RuleRegistry;
use mago_linter::settings::Settings;
use mago_names::resolver::NameResolver;
use mago_syntax::parser::parse_file;

fn count_issues(code: &str) -> usize {
    let arena = LocalArena::new();
    let file = File::ephemeral(Cow::Borrowed(b"test.php"), Cow::Owned(code.as_bytes().to_vec()));
    let program = parse_file(&arena, &file);

    let resolver = NameResolver::new(&arena);
    let resolved_names = resolver.resolve(program);

    let settings = Settings::default();
    let registry = RuleRegistry::build(&settings, Some(&["no-boolean-flag-parameter".to_string()]), true);
    let linter = Linter::from_registry(&arena, Arc::new(registry), settings.php_version);

    linter.lint(&file, program, &resolved_names).len()
}

#[test]
fn flags_boolean_parameter_used_in_condition() {
    let code = "<?php\nfunction f(bool $verbose): void {\n    if ($verbose) {\n        echo 'x';\n    }\n}\n";
    assert_eq!(count_issues(code), 1);
}

#[test]
fn flags_boolean_parameter_used_in_ternary() {
    let code = "<?php\nfunction f(bool $flag): string {\n    return $flag ? 'a' : 'b';\n}\n";
    assert_eq!(count_issues(code), 1);
}

#[test]
fn flags_boolean_parameter_passed_to_call() {
    let code = "<?php\nfunction f(bool $flag): void {\n    g($flag);\n}\n";
    assert_eq!(count_issues(code), 1);
}

#[test]
fn flags_boolean_parameter_used_in_compound_expression() {
    let code = "<?php\nfunction f(bool $flag): bool {\n    $result = $flag && do_thing();\n    return $result;\n}\n";
    assert_eq!(count_issues(code), 1);
}

// Reproduction of https://github.com/carthage-software/mago/issues/1988:
// a boolean parameter that is only stored should not be flagged.
#[test]
fn does_not_flag_boolean_parameter_only_assigned_to_property() {
    let code = "<?php\nclass A {\n    private bool $myBool;\n    public function update(bool $myBool): void {\n        $this->myBool = $myBool;\n    }\n}\n";
    assert_eq!(count_issues(code), 0);
}

#[test]
fn does_not_flag_boolean_parameter_only_assigned_to_local() {
    let code = "<?php\nfunction f(bool $flag): void {\n    $copy = $flag;\n}\n";
    assert_eq!(count_issues(code), 0);
}

#[test]
fn does_not_flag_unused_boolean_parameter() {
    let code = "<?php\nfunction f(bool $flag): void {\n    do_thing();\n}\n";
    assert_eq!(count_issues(code), 0);
}
