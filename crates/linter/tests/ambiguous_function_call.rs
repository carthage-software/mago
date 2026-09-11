#![allow(clippy::needless_raw_strings, clippy::needless_raw_string_hashes)]

use std::borrow::Cow;
use std::sync::Arc;

use mago_allocator::LocalArena;
use mago_database::file::File;
use mago_linter::Linter;
use mago_linter::integration::IntegrationSet;
use mago_linter::registry::RuleRegistry;
use mago_linter::settings::RulesSettings;
use mago_linter::settings::Settings;
use mago_names::resolver::NameResolver;
use mago_syntax::parser::parse_file;

fn codes(code: &str) -> Vec<String> {
    let arena = LocalArena::new();
    let file = File::ephemeral(Cow::Owned(b"test.php".to_vec()), Cow::Owned(code.as_bytes().to_vec()));
    let program = parse_file(&arena, &file);
    let resolved_names = NameResolver::new(&arena).resolve(program);
    let settings =
        Settings { integrations: IntegrationSet::all(), rules: RulesSettings::default(), ..Settings::default() };
    let php_version = settings.php_version;
    let registry = RuleRegistry::build(&settings, Some(&["ambiguous-function-call".to_string()]), true);
    let linter = Linter::from_registry(&arena, Arc::new(registry), php_version);

    linter.lint(&file, program, &resolved_names).iter().filter_map(|issue| issue.code.clone()).collect()
}

#[test]
fn a_function_declared_in_the_same_namespace_is_not_ambiguous() {
    let issues = codes(
        r#"<?php

namespace App;

function shout(string $value): string { return $value; }

echo shout('hi');
"#,
    );

    assert!(issues.is_empty(), "expected no issue, got {issues:?}");
}

#[test]
fn a_function_the_file_does_not_declare_is_still_ambiguous() {
    let issues = codes(
        r#"<?php

namespace App;

echo other('hi');
"#,
    );

    assert_eq!(issues, vec!["ambiguous-function-call".to_string()]);
}

#[test]
fn a_declaration_in_another_namespace_does_not_silence_the_call() {
    // The guard compares resolved names, so `B\shout` is not answered by a
    // declaration of `A\shout` sitting in the same file.
    let issues = codes(
        r#"<?php

namespace A {
    function shout(string $value): string { return $value; }
}

namespace B {
    echo shout('hi');
}
"#,
    );

    assert_eq!(issues, vec!["ambiguous-function-call".to_string()]);
}
