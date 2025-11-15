//! Test infrastructure for lint rules.

use std::borrow::Cow;
use std::sync::Arc;

use bumpalo::Bump;
use mago_database::file::File;
use mago_names::resolver::NameResolver;
use mago_syntax::parser::parse_file;

use crate::Linter;
use crate::registry::RuleRegistry;
use crate::rule::LintRule;
use crate::settings::Settings;

/// Runs a lint test for a specific rule type.
///
/// This is the core testing function that all test macros use.
///
/// # Arguments
///
/// * `code` - The PHP code to lint
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
pub fn run_lint_test<R: LintRule, F>(code: &str, expected_count: Option<usize>, settings_fn: Option<F>)
where
    F: FnOnce(&mut Settings),
{
    let arena = Bump::new();
    let file = File::ephemeral(Cow::Owned("test.php".to_string()), Cow::Owned(code.to_string()));

    let (program, parse_error) = parse_file(&arena, &file);
    if let Some(err) = parse_error {
        panic!("Parse error in test code: {:?}", err);
    }

    let resolver = NameResolver::new(&arena);
    let resolved_names = resolver.resolve(program);

    let mut settings = Settings::default();
    if let Some(f) = settings_fn {
        f(&mut settings);
    }

    let rule_code = R::meta().code;
    let registry = RuleRegistry::build(&settings, Some(&[rule_code.to_string()]), true);

    if registry.rules().is_empty() {
        panic!("No rules loaded for code '{}'", rule_code);
    }

    let linter = Linter::from_registry(&arena, Arc::new(registry), settings.php_version);
    let issues = linter.lint(&file, program, &resolved_names);

    match expected_count {
        Some(0) => {
            // Success test - should have no issues
            if !issues.is_empty() {
                panic!(
                    "Test failed for rule '{}': Expected code to NOT produce lint issues, but found {} issue(s):\n{:#?}",
                    rule_code,
                    issues.len(),
                    issues
                );
            }
        }
        Some(n) => {
            // Exact count test
            if issues.len() != n {
                panic!(
                    "Test failed for rule '{}': Expected {} issue(s), but found {} issue(s):\n{:#?}",
                    rule_code,
                    n,
                    issues.len(),
                    issues
                );
            }
        }
        None => {
            // Failure test - should have at least one issue
            if issues.is_empty() {
                panic!(
                    "Test failed for rule '{}': Expected code to produce lint issues, but none were found.",
                    rule_code
                );
            }
        }
    }
}

/// Test macro for code that should NOT produce lint issues.
///
/// # Examples
///
/// Basic usage:
/// ```ignore
/// test_lint_success! {
///     name = json_decode_is_safe,
///     rule = NoEvalRule,
///     code = indoc! {r#"
///         <?php
///         $result = json_decode($data);
///     "#}
/// }
/// ```
///
/// With custom settings:
/// ```ignore
/// test_lint_success! {
///     name = wp_debug_is_fine,
///     rule = NoIniSetRule,
///     settings = |s| s.rules.no_ini_set.level = Level::Warning,
///     code = indoc! {r#"
///         <?php
///         define('WP_DEBUG', true);
///     "#}
/// }
/// ```
#[macro_export]
macro_rules! test_lint_success {
    // Without settings
    {
        name = $test_name:ident,
        rule = $rule:ty,
        code = $code:expr $(,)?
    } => {
        #[test]
        fn $test_name() {
            $crate::rule::tests::run_lint_test::<$rule, fn(&mut $crate::settings::Settings)>(
                $code,
                Some(0),
                None,
            );
        }
    };
    // With settings
    {
        name = $test_name:ident,
        rule = $rule:ty,
        settings = $settings:expr,
        code = $code:expr $(,)?
    } => {
        #[test]
        fn $test_name() {
            $crate::rule::tests::run_lint_test::<$rule, _>($code, Some(0), Some($settings));
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
///     name = eval_is_forbidden,
///     rule = NoEvalRule,
///     code = indoc! {r#"
///         <?php
///         eval($code);
///     "#}
/// }
/// ```
///
/// With specific issue count:
/// ```ignore
/// test_lint_failure! {
///     name = multiple_evals_detected,
///     rule = NoEvalRule,
///     count = 3,
///     code = indoc! {r#"
///         <?php
///         eval($a);
///         eval($b);
///         eval($c);
///     "#}
/// }
/// ```
///
/// With custom settings:
/// ```ignore
/// test_lint_failure! {
///     name = eval_with_custom_level,
///     rule = NoEvalRule,
///     settings = |s| s.rules.no_eval.level = Level::Warning,
///     code = indoc! {r#"
///         <?php
///         eval($code);
///     "#}
/// }
/// ```
///
/// With both count and settings:
/// ```ignore
/// test_lint_failure! {
///     name = multiple_evals_with_custom_settings,
///     rule = NoEvalRule,
///     count = 2,
///     settings = |s| s.rules.no_eval.level = Level::Warning,
///     code = indoc! {r#"
///         <?php
///         eval($a);
///         eval($b);
///     "#}
/// }
/// ```
#[macro_export]
macro_rules! test_lint_failure {
    // Basic failure test - at least one issue expected
    {
        name = $test_name:ident,
        rule = $rule:ty,
        code = $code:expr $(,)?
    } => {
        #[test]
        fn $test_name() {
            $crate::rule::tests::run_lint_test::<$rule, fn(&mut $crate::settings::Settings)>(
                $code,
                None,
                None,
            );
        }
    };
    // Failure test with exact count
    {
        name = $test_name:ident,
        rule = $rule:ty,
        count = $count:expr,
        code = $code:expr $(,)?
    } => {
        #[test]
        fn $test_name() {
            $crate::rule::tests::run_lint_test::<$rule, fn(&mut $crate::settings::Settings)>(
                $code,
                Some($count),
                None,
            );
        }
    };
    // Failure test with settings
    {
        name = $test_name:ident,
        rule = $rule:ty,
        settings = $settings:expr,
        code = $code:expr $(,)?
    } => {
        #[test]
        fn $test_name() {
            $crate::rule::tests::run_lint_test::<$rule, _>($code, None, Some($settings));
        }
    };
    // Failure test with both count and settings
    {
        name = $test_name:ident,
        rule = $rule:ty,
        count = $count:expr,
        settings = $settings:expr,
        code = $code:expr $(,)?
    } => {
        #[test]
        fn $test_name() {
            $crate::rule::tests::run_lint_test::<$rule, _>($code, Some($count), Some($settings));
        }
    };
}
