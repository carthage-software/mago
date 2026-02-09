use std::borrow::Cow;

use bumpalo::Bump;
use mago_database::file::File;
use mago_linter::Linter;
use mago_linter::integration::IntegrationSet;
use mago_linter::registry::RuleRegistry;
use mago_linter::settings::RulesSettings;
use mago_linter::settings::Settings;
use mago_names::resolver::NameResolver;
use mago_syntax::parser::parse_file;
use mago_text_edit::TextEditor;

/// Helper to lint code, apply fixes, and return the fixed code.
fn lint_and_fix(code: &str) -> String {
    let arena = Bump::new();

    let file = File::ephemeral(Cow::Owned("test.php".to_string()), Cow::Owned(code.to_string()));

    let program = parse_file(&arena, &file);

    let resolver = NameResolver::new(&arena);
    let resolved_names = resolver.resolve(program);

    let settings =
        Settings { integrations: IntegrationSet::all(), rules: RulesSettings::default(), ..Settings::default() };

    let php_version = settings.php_version;
    let registry = RuleRegistry::build(&settings, Some(&["prefer-early-continue".to_string()]), true);

    let linter = Linter::from_registry(&arena, std::sync::Arc::new(registry), php_version);

    let mut issues = linter.lint(&file, program, &resolved_names);

    // Collect all edits for our file
    let mut editor = TextEditor::new(code);
    for (file_id, edits) in issues.take_edits() {
        if file_id == file.id {
            for edit in edits {
                let _ = editor.apply(edit, None::<fn(&str) -> bool>);
            }
        }
    }

    editor.finish()
}

#[test]
fn test_fix_with_block_body() {
    let input = r#"<?php

foreach ($items as $item) {
    if ($item->isValid()) {
        doSomething($item);
    }
}
"#;

    // The rule wraps the negated condition in parentheses and places the
    // body statements after the continue block.
    let expected = r#"<?php

foreach ($items as $item) {
    if (!($item->isValid())) { continue; }

doSomething($item);
}
"#;

    let result = lint_and_fix(input);
    assert_eq!(result, expected);
}

#[test]
fn test_fix_with_non_block_body() {
    // When the if body is a single statement without braces, the rule should
    // still properly transform it to the early continue pattern.
    let input = r#"<?php

foreach ($memberships as $membership) {
    if ($membership->getClassId() === $classId)
        $membership->markAsFound();
}
"#;

    let expected = r#"<?php

foreach ($memberships as $membership) {
    if ($membership->getClassId() !== $classId) { continue; }

$membership->markAsFound();
}
"#;

    let result = lint_and_fix(input);
    assert_eq!(result, expected);
}

#[test]
fn test_no_fix_when_body_is_continue() {
    // If the if body is already a continue statement, the rule should NOT
    // suggest transforming it - that would create redundant code like:
    // `if (!$x) { continue; } continue;`
    let input = r#"<?php

foreach ($items as $item) {
    if (!$item->isValid())
        continue;
}
"#;

    // Should remain unchanged - no fix applied
    let result = lint_and_fix(input);
    assert_eq!(result, input, "Code with 'if (x) continue;' should not be transformed");
}

#[test]
fn test_no_fix_when_body_is_continue_with_block() {
    // Same for block-style continue
    let input = r#"<?php

foreach ($items as $item) {
    if (!$item->isValid()) {
        continue;
    }
}
"#;

    // Should remain unchanged
    let result = lint_and_fix(input);
    assert_eq!(result, input, "Code with 'if (x) {{ continue; }}' should not be transformed");
}

#[test]
fn test_fix_with_function_call() {
    let input = r#"<?php

foreach ($users as $user) {
    if ($user->isActive())
        $user->sendNotification();
}
"#;

    let expected = r#"<?php

foreach ($users as $user) {
    if (!($user->isActive())) { continue; }

$user->sendNotification();
}
"#;

    let result = lint_and_fix(input);
    assert_eq!(result, expected);
}

#[test]
fn test_fix_with_assignment() {
    let input = r#"<?php

foreach ($items as $item) {
    if ($item->hasValue())
        $total += $item->getValue();
}
"#;

    let expected = r#"<?php

foreach ($items as $item) {
    if (!($item->hasValue())) { continue; }

$total += $item->getValue();
}
"#;

    let result = lint_and_fix(input);
    assert_eq!(result, expected);
}

#[test]
fn test_fix_with_echo() {
    let input = r#"<?php

foreach ($messages as $message) {
    if ($message->isImportant())
        echo $message->getText();
}
"#;

    let expected = r#"<?php

foreach ($messages as $message) {
    if (!($message->isImportant())) { continue; }

echo $message->getText();
}
"#;

    let result = lint_and_fix(input);
    assert_eq!(result, expected);
}

#[test]
fn test_no_fix_when_body_is_throw() {
    // Throw is an early exit - no nesting to reduce, skip transformation
    let input = r#"<?php

foreach ($inputs as $input) {
    if ($input === null)
        throw new InvalidArgumentException("Input cannot be null");
}
"#;

    let result = lint_and_fix(input);
    assert_eq!(result, input);
}

#[test]
fn test_no_fix_when_body_is_break() {
    // Break is an early exit - no nesting to reduce, skip transformation
    let input = r#"<?php

foreach ($items as $item) {
    if ($item->isTarget())
        break;
}
"#;

    let result = lint_and_fix(input);
    assert_eq!(result, input);
}

#[test]
fn test_no_fix_when_body_is_return() {
    // Return is an early exit - no nesting to reduce, skip transformation
    let input = r#"<?php

foreach ($items as $item) {
    if ($item->isMatch())
        return $item;
}
"#;

    let result = lint_and_fix(input);
    assert_eq!(result, input);
}

#[test]
fn test_fix_with_nested_if() {
    // When the if body is another if statement (without braces),
    // the rule should still transform the outer if.
    let input = r#"<?php

foreach ($items as $item) {
    if ($item->isValid())
        if ($item->hasPermission())
            $item->process();
}
"#;

    let expected = r#"<?php

foreach ($items as $item) {
    if (!($item->isValid())) { continue; }

if ($item->hasPermission())
            $item->process();
}
"#;

    let result = lint_and_fix(input);
    assert_eq!(result, expected);
}

#[test]
fn test_fix_with_binary_greater_than() {
    // Test that other binary operators are correctly negated
    let input = r#"<?php

foreach ($numbers as $num) {
    if ($num > 10)
        processLargeNumber($num);
}
"#;

    let expected = r#"<?php

foreach ($numbers as $num) {
    if ($num <= 10) { continue; }

processLargeNumber($num);
}
"#;

    let result = lint_and_fix(input);
    assert_eq!(result, expected);
}

#[test]
fn test_fix_with_not_equal() {
    let input = r#"<?php

foreach ($values as $value) {
    if ($value != null)
        doSomething($value);
}
"#;

    let expected = r#"<?php

foreach ($values as $value) {
    if ($value == null) { continue; }

doSomething($value);
}
"#;

    let result = lint_and_fix(input);
    assert_eq!(result, expected);
}
