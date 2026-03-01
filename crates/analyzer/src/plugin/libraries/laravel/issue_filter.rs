//! Issue filter hook for suppressing false-positive diagnostics on Laravel Eloquent classes.
//!
//! Eloquent relies heavily on magic methods (`__call`, `__callStatic`, `__get`, `__set`)
//! and runtime conventions that static analysis cannot see. This filter suppresses
//! diagnostics that would be false positives on Model, Builder, and Factory subclasses.
//!
//! Derived from phpantom_lsp's `LaravelModelProvider` and `LaravelFactoryProvider`.
//! In phpantom these false positives are avoided by synthesizing virtual members directly;
//! in Mago's architecture, this issue filter serves as the equivalent mechanism until
//! full virtual member synthesis (Phases 3–6) is implemented.

use mago_codex::metadata::CodebaseMetadata;
use mago_database::file::File;
use mago_reporting::Issue;

use crate::plugin::hook::HookResult;
use crate::plugin::hook::IssueFilterDecision;
use crate::plugin::hook::IssueFilterHook;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;

use super::utils::ELOQUENT_BUILDER;
use super::utils::ELOQUENT_FACTORY;
use super::utils::ELOQUENT_MODEL;

/// Issue codes to suppress on Eloquent Model subclasses.
const MODEL_SUPPRESSED_CODES: &[&str] = &[
    "non-existent-property",
    "non-documented-property",
    "possibly-non-existent-property",
    "non-existent-method",
    "non-documented-method",
    "possibly-non-existent-method",
    "uninitialized-property",
    "mixed-property-access",
];

/// Issue codes to suppress on Eloquent Builder subclasses.
const BUILDER_SUPPRESSED_CODES: &[&str] =
    &["non-existent-method", "non-documented-method", "possibly-non-existent-method", "uninitialized-property"];

/// Issue codes to suppress on Factory subclasses.
const FACTORY_SUPPRESSED_CODES: &[&str] = &["uninitialized-property", "non-existent-method", "non-documented-method"];

static META: ProviderMeta = ProviderMeta::new(
    "laravel-issue-filter",
    "Laravel Issue Filter",
    "Suppresses false-positive diagnostics on Eloquent Model, Builder, and Factory classes",
);

/// Issue filter hook that suppresses false-positive diagnostics on
/// Eloquent Model, Builder, and Factory subclasses.
#[derive(Default)]
pub struct LaravelIssueFilter;

impl LaravelIssueFilter {
    pub fn new() -> Self {
        Self
    }
}

impl Provider for LaravelIssueFilter {
    fn meta() -> &'static ProviderMeta
    where
        Self: Sized,
    {
        &META
    }
}

impl IssueFilterHook for LaravelIssueFilter {
    fn filter_issue(
        &self,
        _file: &File,
        issue: &Issue,
        codebase: &CodebaseMetadata,
    ) -> HookResult<IssueFilterDecision> {
        let Some(ref code) = issue.code else {
            return Ok(IssueFilterDecision::Keep);
        };

        // Quick check: is this an issue code we care about at all?
        if !is_potentially_suppressible(code) {
            return Ok(IssueFilterDecision::Keep);
        }

        // Extract the class name from the issue message.
        let class_name = match extract_class_name(&issue.message) {
            Some(name) => name,
            None => return Ok(IssueFilterDecision::Keep),
        };

        // Check if the class is an Eloquent Model, Builder, or Factory subclass,
        // and whether this issue code should be suppressed for that class kind.
        if should_suppress(code, class_name, codebase) {
            Ok(IssueFilterDecision::Remove)
        } else {
            Ok(IssueFilterDecision::Keep)
        }
    }
}

/// Quick check: is this code in any of the suppression lists?
fn is_potentially_suppressible(code: &str) -> bool {
    MODEL_SUPPRESSED_CODES.contains(&code)
        || BUILDER_SUPPRESSED_CODES.contains(&code)
        || FACTORY_SUPPRESSED_CODES.contains(&code)
}

/// Determines whether to suppress an issue with the given code for the given class.
fn should_suppress(code: &str, class_name: &str, codebase: &CodebaseMetadata) -> bool {
    // Check Model hierarchy
    if MODEL_SUPPRESSED_CODES.contains(&code) && codebase.is_instance_of(class_name, ELOQUENT_MODEL) {
        return true;
    }

    // Check Builder hierarchy
    if BUILDER_SUPPRESSED_CODES.contains(&code) && codebase.is_instance_of(class_name, ELOQUENT_BUILDER) {
        return true;
    }

    // Check Factory hierarchy
    if FACTORY_SUPPRESSED_CODES.contains(&code) && codebase.is_instance_of(class_name, ELOQUENT_FACTORY) {
        return true;
    }

    false
}

/// Extracts a class name from an issue message.
///
/// Tries two strategies:
///
/// 1. **`ClassName::member` pattern** — e.g. message contains backtick-wrapped
///    `App\Models\User::$foo` or `App\Models\User::doSomething`
///
/// 2. **Backtick-wrapped class name** — looks for a class-like FQN inside backticks
///    in patterns like "on class `ClassName`", "on type `ClassName`",
///    "of type `ClassName`", "of class `ClassName`", "instance of `ClassName`",
///    or "class `ClassName`".
fn extract_class_name(message: &str) -> Option<&str> {
    // Strategy 1: look for `ClassName::member` inside backticks
    if let Some(name) = extract_class_from_double_colon(message) {
        return Some(name);
    }

    // Strategy 2: look for class name after known patterns
    if let Some(name) = extract_class_from_context(message) {
        return Some(name);
    }

    None
}

/// Extracts a class name from a `ClassName::member` pattern inside backticks.
///
/// Looks for `` `SomeClass::$prop` `` or `` `SomeClass::method` `` and
/// returns `SomeClass`.
fn extract_class_from_double_colon(message: &str) -> Option<&str> {
    // Find backtick-delimited segments containing `::`
    let mut search_from = 0;
    while let Some(start) = message[search_from..].find('`') {
        let abs_start = search_from + start + 1;
        if let Some(end) = message[abs_start..].find('`') {
            let abs_end = abs_start + end;
            let segment = &message[abs_start..abs_end];

            if let Some(colon_pos) = segment.find("::") {
                let class_part = &segment[..colon_pos];
                // Must look like a class name (contains at least one letter, possibly backslashes)
                if looks_like_class_name(class_part) {
                    return Some(class_part);
                }
            }

            search_from = abs_end + 1;
        } else {
            break;
        }
    }

    None
}

/// Extracts a class name from contextual patterns in the message.
///
/// Looks for patterns like:
/// - "on class `ClassName`"
/// - "on type `ClassName`"
/// - "of type `ClassName`"
/// - "of class `ClassName`"
/// - "instance of `ClassName`"
/// - "In class `ClassName`"
/// - "class `ClassName`"
/// - "on sealed object type `ClassName`"
/// - "on object `ClassName`"
fn extract_class_from_context(message: &str) -> Option<&str> {
    // Patterns that precede a backtick-wrapped class name
    const PATTERNS: &[&str] = &[
        "on class `",
        "on type `",
        "of type `",
        "of class `",
        "instance of `",
        "In class `",
        "class `",
        "on sealed object type `",
        "on object `",
        "final type `",
        "final class `",
    ];

    for pattern in PATTERNS {
        if let Some(pos) = message.find(pattern) {
            let after_pattern = pos + pattern.len();
            if let Some(end) = message[after_pattern..].find('`') {
                let class_name = &message[after_pattern..after_pattern + end];
                if looks_like_class_name(class_name) {
                    return Some(class_name);
                }
            }
        }
    }

    None
}

/// Checks if a string looks like a PHP class name (or FQN).
///
/// Must contain at least one letter and consist only of alphanumeric chars,
/// backslashes, and underscores.
fn looks_like_class_name(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let has_letter = s.chars().any(|c| c.is_alphabetic());
    let all_valid = s.chars().all(|c| c.is_alphanumeric() || c == '\\' || c == '_');

    has_letter && all_valid
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_class_from_double_colon() {
        assert_eq!(
            extract_class_from_double_colon("Property `App\\Models\\User::$foo` is not accessible"),
            Some("App\\Models\\User"),
        );
        assert_eq!(extract_class_from_double_colon("Method `User::doSomething` not found"), Some("User"),);
        assert_eq!(extract_class_from_double_colon("No double colon here"), None,);
    }

    #[test]
    fn test_extract_class_from_context() {
        assert_eq!(
            extract_class_from_context("Property `$foo` does not exist on class `App\\Models\\User`."),
            Some("App\\Models\\User"),
        );
        assert_eq!(
            extract_class_from_context("Method might not exist on type `App\\Models\\Post` at runtime."),
            Some("App\\Models\\Post"),
        );
        assert_eq!(extract_class_from_context("On an object of type `App\\Models\\User`"), Some("App\\Models\\User"),);
        assert_eq!(extract_class_from_context("In class `App\\Models\\User`"), Some("App\\Models\\User"),);
        assert_eq!(extract_class_from_context("no class name here"), None,);
    }

    #[test]
    fn test_extract_class_name_prefers_double_colon() {
        // When both patterns are present, double colon wins
        let msg = "Property `App\\Models\\User::$name` does not exist on class `App\\Models\\User`.";
        assert_eq!(extract_class_name(msg), Some("App\\Models\\User"));
    }

    #[test]
    fn test_extract_class_name_falls_back_to_context() {
        let msg = "Property `$name` does not exist on class `App\\Models\\User`.";
        assert_eq!(extract_class_name(msg), Some("App\\Models\\User"));
    }

    #[test]
    fn test_looks_like_class_name() {
        assert!(looks_like_class_name("User"));
        assert!(looks_like_class_name("App\\Models\\User"));
        assert!(looks_like_class_name("My_Class"));
        assert!(!looks_like_class_name(""));
        assert!(!looks_like_class_name("123"));
        assert!(!looks_like_class_name("$foo"));
    }

    #[test]
    fn test_is_potentially_suppressible() {
        assert!(is_potentially_suppressible("non-existent-property"));
        assert!(is_potentially_suppressible("non-existent-method"));
        assert!(is_potentially_suppressible("uninitialized-property"));
        assert!(is_potentially_suppressible("mixed-property-access"));
        assert!(is_potentially_suppressible("non-documented-method"));
        assert!(!is_potentially_suppressible("unused-variable"));
        assert!(!is_potentially_suppressible("some-other-code"));
    }

    #[test]
    fn test_extract_class_from_non_existent_property_message() {
        // This is the actual message format from Mago's property resolver
        let msg = "Property `$name` does not exist on class `App\\Models\\User`.";
        assert_eq!(extract_class_name(msg), Some("App\\Models\\User"));
    }

    #[test]
    fn test_extract_class_from_non_existent_method_message() {
        let msg = "Method `doSomething` does not exist on type `App\\Models\\User`.";
        assert_eq!(extract_class_name(msg), Some("App\\Models\\User"));
    }

    #[test]
    fn test_extract_class_from_non_documented_property_message() {
        let msg = "Ambiguous property access: $name on class `App\\Models\\User`.";
        assert_eq!(extract_class_name(msg), Some("App\\Models\\User"));
    }

    #[test]
    fn test_extract_class_from_non_documented_method_message() {
        let msg = "Ambiguous method call to `doSomething` on class `App\\Models\\User`.";
        assert_eq!(extract_class_name(msg), Some("App\\Models\\User"));
    }

    #[test]
    fn test_extract_class_from_uninitialized_property_message() {
        let msg = "Property `$name` is not initialized in the constructor of class `App\\Models\\User`.";
        assert_eq!(extract_class_name(msg), Some("App\\Models\\User"));
    }

    #[test]
    fn test_extract_class_from_possibly_non_existent_property_message() {
        let msg = "Property `$name` might not exist on object `App\\Models\\User`.";
        assert_eq!(extract_class_name(msg), Some("App\\Models\\User"));
    }

    #[test]
    fn test_extract_class_from_possibly_non_existent_method_message() {
        let msg = "Method `doSomething` might not exist on type `App\\Models\\User` at runtime.";
        assert_eq!(extract_class_name(msg), Some("App\\Models\\User"));
    }

    #[test]
    fn test_extract_class_from_sealed_object_message() {
        let msg = "Property `$name` does not exist on sealed object type `App\\Models\\User`.";
        assert_eq!(extract_class_name(msg), Some("App\\Models\\User"));
    }

    #[test]
    fn test_extract_class_from_mixin_property_message() {
        let msg = "Property `$name` might not exist on type `App\\Models\\User` at runtime.";
        assert_eq!(extract_class_name(msg), Some("App\\Models\\User"));
    }

    #[test]
    fn test_extract_class_from_final_type_message() {
        let msg = "Method `doSomething` does not exist on final type `App\\Models\\User`.";
        assert_eq!(extract_class_name(msg), Some("App\\Models\\User"));
    }
}
