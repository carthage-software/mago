//! Test infrastructure for lint rules.

use std::borrow::Cow;
use std::sync::Arc;

use mago_allocator::prelude::*;
use mago_database::file::File;
use mago_twig_syntax::parser::Parser;
use mago_twig_syntax::settings::ParserSettings;

use crate::Linter;
use crate::registry::RuleRegistry;
use crate::rule::LintRule;
use crate::settings::Settings;

pub(super) fn collect_issues<R, F>(
    code: &str,
    filename: Option<&'static str>,
    settings_fn: Option<F>,
) -> mago_reporting::IssueCollection
where
    F: FnOnce(&mut Settings),
    R: LintRule,
{
    let arena = LocalArena::new();
    let file = File::ephemeral(
        Cow::Borrowed(filename.unwrap_or("test.twig").as_bytes()),
        Cow::Owned(code.as_bytes().to_vec()),
    );

    let template =
        Parser::new(&arena, file.id, &file.contents, ParserSettings::default()).parse(&file.contents, file.id);
    if template.has_errors() {
        panic!("Failed to parse code for lint test, errors: {:?}", template.errors);
    }

    let mut settings = Settings::default();
    if let Some(f) = settings_fn {
        f(&mut settings);
    }

    let rule_code = R::meta().code;
    let registry = RuleRegistry::build(&settings, Some(&[rule_code.to_string()]), true);

    assert!(!registry.rules().is_empty(), "No rules loaded for code '{rule_code}'");

    let linter = Linter::from_registry(&arena, Arc::new(registry));
    let issues = linter.lint(&file, template);
    for issue in issues.iter().filter(|issue| issue.code.as_deref() == Some(rule_code)) {
        assert_explained(issue);
    }

    issues
}

/// Asserts that an issue tells the reader what each highlighted span is, why it matters, and
/// what to do about it.
///
/// # Panics
///
/// Panics if an annotation has no message, or if the issue has no note or no help.
pub fn assert_explained(issue: &mago_reporting::Issue) {
    let code = issue.code.as_deref().unwrap_or_default();
    assert!(!issue.annotations.is_empty(), "Issue '{code}' highlights nothing: {issue:#?}");
    assert!(
        issue
            .annotations
            .iter()
            .all(|annotation| annotation.message.as_ref().is_some_and(|message| !message.is_empty())),
        "Issue '{code}' has an annotation without a message: {issue:#?}"
    );
    assert!(!issue.notes.is_empty(), "Issue '{code}' has no note: {issue:#?}");
    assert!(issue.help.as_ref().is_some_and(|help| !help.is_empty()), "Issue '{code}' has no help: {issue:#?}");
}

/// Runs a lint test for a specific rule type.
///
/// This is the core testing function that all test macros use.
///
/// # Arguments
///
/// * `code` - The Twig code to lint
/// * `filename` - Optional custom file name (defaults to "test.twig")
/// * `expected_count` - Expected number of issues:
///   - `Some(0)` - Success test: code should produce no issues
///   - `Some(n)` - Exact count test: code should produce exactly n issues
///   - `None` - Failure test: code should produce at least one issue
/// * `settings_fn` - Optional closure to customize settings before running the test
///
/// # Panics
///
/// Panics if:
/// - The code fails to parse
/// - The actual issue count doesn't match the expected count
pub fn run_lint_test<R, F>(
    code: &str,
    filename: Option<&'static str>,
    expected_count: Option<usize>,
    settings_fn: Option<F>,
) where
    F: FnOnce(&mut Settings),
    R: LintRule,
{
    let issues = collect_issues::<R, F>(code, filename, settings_fn);
    let rule_code = R::meta().code;

    match expected_count {
        Some(0) => {
            // Success test - should have no issues
            assert!(
                issues.is_empty(),
                "Test failed for rule '{}': Expected code to NOT produce lint issues, but found {} issue(s):\n{:#?}",
                rule_code,
                issues.len(),
                issues
            );
        }
        Some(n) => {
            // Exact count test
            assert!(
                issues.len() == n,
                "Test failed for rule '{}': Expected {} issue(s), but found {} issue(s):\n{:#?}",
                rule_code,
                n,
                issues.len(),
                issues
            );
        }
        None => {
            // Failure test - should have at least one issue
            assert!(
                !issues.is_empty(),
                "Test failed for rule '{rule_code}': Expected code to produce lint issues, but none were found."
            );
        }
    }
}

/// Runs a lint fix test for a specific rule type.
///
/// Lints the given code, collects all text edits from the resulting issues,
/// applies them, and asserts the result matches `expected_fixed`.
///
/// # Panics
///
/// Panics if:
/// - The code fails to parse
/// - No issues are produced
/// - No edits are attached to any issue
/// - The fixed output doesn't match `expected_fixed`
pub fn run_lint_fix_test<R, F>(code: &str, filename: Option<&'static str>, expected_fixed: &str, settings_fn: Option<F>)
where
    F: FnOnce(&mut Settings),
    R: LintRule,
{
    use mago_text_edit::TextEditor;

    let issues = collect_issues::<R, F>(code, filename, settings_fn);
    let rule_code = R::meta().code;

    assert!(
        !issues.is_empty(),
        "Test failed for rule '{rule_code}': Expected code to produce lint issues, but none were found."
    );

    let mut editor = TextEditor::new(code.as_bytes());
    let mut has_edits = false;
    for batches in issues.to_edit_batches().into_values() {
        for (_, edits) in batches {
            has_edits |= !edits.is_empty();
            editor.apply_unique_batch(edits, None::<fn(&[u8]) -> bool>);
        }
    }

    assert!(has_edits, "Test failed for rule '{rule_code}': Issues were produced but none had auto-fix edits.");

    let fixed_bytes = editor.finish();
    let fixed = String::from_utf8_lossy(&fixed_bytes);
    assert_eq!(
        fixed, expected_fixed,
        "Test failed for rule '{rule_code}': Fixed output doesn't match expected.\n\nGot:\n{fixed}\n\nExpected:\n{expected_fixed}"
    );
}

/// Test macro for code that should NOT produce lint issues.
///
/// # Examples
///
/// Basic usage:
/// ```ignore
/// test_lint_success! {
///     name = plain_output_is_fine,
///     rule = NoDumpRule,
///     code = r#"{{ user.name }}"#,
/// }
/// ```
///
/// With custom settings:
/// ```ignore
/// test_lint_success! {
///     name = allowed_function_is_fine,
///     rule = DisallowedFunctionsRule,
///     settings = |s: &mut Settings| s.rules.disallowed_functions.config.names = vec!["source".into()],
///     code = r#"{{ range(1, 3) }}"#,
/// }
/// ```
#[macro_export]
macro_rules! test_lint_success {
    {
        name = $test_name:ident,
        rule = $rule:ty,
        $(filename = $filename:expr,)?
        settings = $settings:expr,
        code = $code:expr $(,)?
    } => {
        #[test]
        fn $test_name() {
            $crate::rule::tests::run_lint_test::<$rule, _>(
                $code,
                None $(.or(Some($filename)))?,
                Some(0),
                Some($settings),
            );
        }
    };
    {
        name = $test_name:ident,
        rule = $rule:ty,
        $(filename = $filename:expr,)?
        code = $code:expr $(,)?
    } => {
        #[test]
        fn $test_name() {
            $crate::rule::tests::run_lint_test::<$rule, fn(&mut $crate::settings::Settings)>(
                $code,
                None $(.or(Some($filename)))?,
                Some(0),
                None,
            );
        }
    };
}

/// Test macro for code that SHOULD produce lint issues.
///
/// # Examples
///
/// Basic usage - just check that code fails:
/// ```ignore
/// test_lint_failure! {
///     name = dump_is_reported,
///     rule = NoDumpRule,
///     code = r#"{{ dump(user) }}"#,
/// }
/// ```
///
/// With a specific issue count and custom settings:
/// ```ignore
/// test_lint_failure! {
///     name = each_disallowed_call_is_reported,
///     rule = DisallowedFunctionsRule,
///     count = 2,
///     settings = |s: &mut Settings| s.rules.disallowed_functions.config.names = vec!["source".into()],
///     code = r#"{{ source('a') ~ source('b') }}"#,
/// }
/// ```
#[macro_export]
macro_rules! test_lint_failure {
    {
        name = $test_name:ident,
        rule = $rule:ty,
        $(filename = $filename:expr,)?
        $(count = $count:expr,)?
        settings = $settings:expr,
        code = $code:expr $(,)?
    } => {
        #[test]
        fn $test_name() {
            $crate::rule::tests::run_lint_test::<$rule, _>(
                $code,
                None $(.or(Some($filename)))?,
                None $(.or(Some($count)))?,
                Some($settings),
            );
        }
    };
    {
        name = $test_name:ident,
        rule = $rule:ty,
        $(filename = $filename:expr,)?
        $(count = $count:expr,)?
        code = $code:expr $(,)?
    } => {
        #[test]
        fn $test_name() {
            $crate::rule::tests::run_lint_test::<$rule, fn(&mut $crate::settings::Settings)>(
                $code,
                None $(.or(Some($filename)))?,
                None $(.or(Some($count)))?,
                None,
            );
        }
    };
}

/// Test macro for verifying auto-fix behavior of lint rules.
///
/// Lints the given `code`, applies all auto-fix edits from the resulting issues,
/// and asserts the output matches `fixed`.
///
/// # Examples
///
/// ```ignore
/// test_lint_fix! {
///     name = double_quotes_become_single_quotes,
///     rule = SingleQuotesRule,
///     code = r#"{{ "title" }}"#,
///     fixed = r#"{{ 'title' }}"#,
/// }
/// ```
#[macro_export]
macro_rules! test_lint_fix {
    {
        name = $test_name:ident,
        rule = $rule:ty,
        $(filename = $filename:expr,)?
        settings = $settings:expr,
        code = $code:expr,
        fixed = $fixed:expr $(,)?
    } => {
        #[test]
        fn $test_name() {
            $crate::rule::tests::run_lint_fix_test::<$rule, _>(
                $code,
                None $(.or(Some($filename)))?,
                $fixed,
                Some($settings),
            );
        }
    };
    {
        name = $test_name:ident,
        rule = $rule:ty,
        $(filename = $filename:expr,)?
        code = $code:expr,
        fixed = $fixed:expr $(,)?
    } => {
        #[test]
        fn $test_name() {
            $crate::rule::tests::run_lint_fix_test::<$rule, fn(&mut $crate::settings::Settings)>(
                $code,
                None $(.or(Some($filename)))?,
                $fixed,
                None,
            );
        }
    };
}
