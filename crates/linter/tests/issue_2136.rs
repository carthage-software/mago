//! Regression tests for https://github.com/carthage-software/mago/issues/2136
//!
//! When namespace `A` already exposes `SomeInterface` (including via another file),
//! `no-fully-qualified-global-class-like --fix` must not import `\B\SomeInterface` as
//! `SomeInterface`, which would shadow the namespace symbol.

use std::borrow::Cow;

use mago_allocator::LocalArena;
use mago_database::file::File;
use mago_linter::Linter;
use mago_linter::registry::RuleRegistry;
use mago_linter::settings::Settings;
use mago_names::resolver::NameResolver;
use mago_reporting::IssueCollection;
use mago_syntax::parser::parse_file;
use mago_text_edit::TextEditor;

const RULE: &str = "no-fully-qualified-global-class-like";

/// Models `A/X.php` while `SomeInterface` is declared in a separate `A/SomeInterface.php`.
const INPUT_INTERFACE_IN_OTHER_FILE: &str = r#"<?php

namespace A;

class X implements SomeInterface
{
    public function __construct(private \B\SomeInterface $some) {}

    public function foo(): void
    {
        echo 'Hello';
    }
}
"#;

/// Same-file variant; conflict detection via `local_classes` already covers this case.
const INPUT_INTERFACE_IN_SAME_FILE: &str = r#"<?php

namespace A;

interface SomeInterface
{
    public function foo(): void;
}

class X implements SomeInterface
{
    public function __construct(private \B\SomeInterface $some) {}

    public function foo(): void
    {
        echo 'Hello';
    }
}
"#;

const INPUT_FUNCTION_WITH_SAME_NAME: &str = r#"<?php

namespace A;

function SomeInterface(): void {}

class X
{
    public function __construct(private \B\SomeInterface $some) {}
}
"#;

const FIXED_FUNCTION_WITH_SAME_NAME: &str = r#"<?php

namespace A;

use B\SomeInterface;

function SomeInterface(): void {}

class X
{
    public function __construct(private SomeInterface $some) {}
}
"#;

const INPUT_REFERENCE_IN_SEPARATE_NAMED_NAMESPACE_BLOCK: &str = r#"<?php

namespace A {
    class X
    {
        public function __construct(private \B\Foo $foo) {}
    }
}

namespace A {
    function handle(Foo $foo): void {}
}
"#;

const INPUT_REFERENCE_IN_SEPARATE_GLOBAL_NAMESPACE_BLOCK: &str = r#"<?php

namespace {
    class X
    {
        public function __construct(private \B\Foo $foo) {}
    }
}

namespace {
    function handle(Foo $foo): void {}
}
"#;

const INPUT_QUALIFIED_REFERENCE_IN_SAME_SCOPE: &str = r#"<?php

namespace A;

function handle(Foo\Bar $foo): void {}

class X
{
    public function __construct(private \B\Foo $foo) {}
}
"#;

const INPUT_FULLY_QUALIFIED_REFERENCE_IN_SAME_SCOPE: &str = r#"<?php

namespace A;

class X
{
    public function __construct(private \B\Foo $foo) {}
}

function handle(\A\Foo $foo): void {}
"#;

const INPUT_ALIAS_COLLISION: &str = r#"<?php

namespace A;

class X implements SomeInterface
{
    public function __construct(
        private \B\SomeInterface $some,
        private BSomeInterface $other,
    ) {}
}
"#;

fn lint(code: &str) -> IssueCollection {
    let arena = LocalArena::new();
    let file = File::ephemeral(Cow::Owned(b"test.php".to_vec()), Cow::Owned(code.as_bytes().to_vec()));
    let program = parse_file(&arena, &file);

    let resolver = NameResolver::new(&arena);
    let resolved_names = resolver.resolve(program);

    let settings = Settings::default();
    let registry = RuleRegistry::build(&settings, Some(&[RULE.to_string()]), true);
    let linter = Linter::from_registry(&arena, std::sync::Arc::new(registry), settings.php_version);

    linter.lint(&file, program, &resolved_names)
}

fn lint_and_fix(code: &str) -> String {
    let mut issues = lint(code);

    let mut editor = TextEditor::new(code.as_bytes());
    for (_, edits) in issues.take_edits() {
        for edit in edits {
            editor.apply(edit, None::<fn(&[u8]) -> bool>);
        }
    }

    String::from_utf8_lossy(&editor.finish()).into_owned()
}

fn assert_fix_declined(code: &str) {
    let fixed = lint_and_fix(code);

    assert_eq!(fixed, code, "auto-fix must be declined when importing would shadow a same-namespace interface");
    assert!(!fixed.contains("use B\\SomeInterface"), "auto-fix must not add `use B\\SomeInterface`, got:\n{fixed}");
    assert!(
        fixed.contains(r"private \B\SomeInterface $some"),
        "the constructor type hint must remain fully qualified, got:\n{fixed}"
    );
}

fn assert_foo_import_applied(code: &str) -> String {
    let fixed = lint_and_fix(code);

    assert!(fixed.contains("use B\\Foo;"), "auto-fix must add `use B\\Foo`, got:\n{fixed}");
    assert!(
        fixed.contains("private Foo $foo"),
        "the constructor type hint must use the imported short name, got:\n{fixed}"
    );

    fixed
}

#[test]
fn no_fully_qualified_global_class_like_does_not_shadow_namespace_interface_in_other_file_on_fix() {
    assert_fix_declined(INPUT_INTERFACE_IN_OTHER_FILE);
}

#[test]
fn no_fully_qualified_global_class_like_does_not_shadow_same_file_interface_on_fix() {
    assert_fix_declined(INPUT_INTERFACE_IN_SAME_FILE);
}

#[test]
fn same_named_function_does_not_block_class_like_import() {
    assert_eq!(lint_and_fix(INPUT_FUNCTION_WITH_SAME_NAME), FIXED_FUNCTION_WITH_SAME_NAME);
}

#[test]
fn reference_in_separate_named_namespace_block_does_not_block_import() {
    let fixed = assert_foo_import_applied(INPUT_REFERENCE_IN_SEPARATE_NAMED_NAMESPACE_BLOCK);

    assert!(fixed.contains("function handle(Foo $foo): void {}"));
}

#[test]
fn reference_in_separate_global_namespace_block_does_not_block_import() {
    let fixed = assert_foo_import_applied(INPUT_REFERENCE_IN_SEPARATE_GLOBAL_NAMESPACE_BLOCK);

    assert!(fixed.contains("function handle(Foo $foo): void {}"));
}

#[test]
fn qualified_reference_in_same_scope_blocks_import_of_its_first_segment() {
    assert_eq!(lint_and_fix(INPUT_QUALIFIED_REFERENCE_IN_SAME_SCOPE), INPUT_QUALIFIED_REFERENCE_IN_SAME_SCOPE);
}

#[test]
fn fully_qualified_reference_in_same_scope_does_not_block_import() {
    let fixed = assert_foo_import_applied(INPUT_FULLY_QUALIFIED_REFERENCE_IN_SAME_SCOPE);

    assert!(fixed.contains(r"function handle(\A\Foo $foo): void {}"));
}

#[test]
fn conflicting_import_warning_suggests_safe_alias() {
    let issues = lint(INPUT_INTERFACE_IN_OTHER_FILE);
    let issues = issues.iter().collect::<Vec<_>>();
    assert!(!issues.is_empty(), "expected lint issue");
    let issue = issues[0];

    assert_eq!(
        issue.help.as_deref(),
        Some(
            "Import with a non-conflicting alias: `use B\\SomeInterface as BSomeInterface;`, then reference `BSomeInterface` directly."
        )
    );
}

#[test]
fn conflicting_import_warning_avoids_alias_already_in_use() {
    let issues = lint(INPUT_ALIAS_COLLISION);
    let issues = issues
        .iter()
        .filter(|issue| {
            issue.annotations.iter().any(|annotation| {
                annotation.message.as_deref().is_some_and(|message| message.contains(r"\B\SomeInterface"))
            })
        })
        .collect::<Vec<_>>();
    assert_eq!(issues.len(), 1, "expected one lint issue for \\B\\SomeInterface");
    let issue = issues[0];

    assert_eq!(
        issue.help.as_deref(),
        Some(
            "Import with a non-conflicting alias: `use B\\SomeInterface as BSomeInterface2;`, then reference `BSomeInterface2` directly."
        )
    );
}
