use mago_php_version::PHPVersion;

/// Configuration settings that control the behavior of the Mago analyzer.
///
/// This struct allows you to enable/disable specific checks, suppress categories of issues,
/// and tune the analyzer's performance and strictness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settings {
    /// The target PHP version for the analysis.
    pub version: PHPVersion,

    /// Report all issues related to the use of `mixed` types. Defaults to `true`.
    pub mixed_issues: bool,

    /// Report all issues related to possibly `false` values. Defaults to `true`.
    pub falsable_issues: bool,

    /// Report all issues related to possibly `null` values. Defaults to `true`.
    pub nullable_issues: bool,

    /// Report all issues related to redundant code. Defaults to `true`.
    pub redundancy_issues: bool,

    /// Report all issues related to by-reference variables. Defaults to `true`.
    pub reference_issues: bool,

    /// Report all issues related to unreachable code. Defaults to `true`.
    pub unreachable_issues: bool,

    /// Report all issues related to using deprecated code. Defaults to `true`.
    pub deprecation_issues: bool,

    /// Report all issues related to logically impossible conditions. Defaults to `true`.
    pub impossibility_issues: bool,

    /// Report all issues related to ambiguous code constructs. Defaults to `true`.
    pub ambiguity_issues: bool,

    /// Report all issues related to the existence of symbols (e.g., classes, functions, constants). Defaults to `true`.
    pub existence_issues: bool,

    /// Report all issues related to generic template types and their usage. Defaults to `true`.
    pub template_issues: bool,

    /// Report all issues related to function arguments. Defaults to `true`.
    pub argument_issues: bool,

    /// Report all issues related to operands in expressions. Defaults to `true`.
    pub operand_issues: bool,

    /// Report all issues related to properties and their usage. Defaults to `true`.
    pub property_issues: bool,

    /// Report all issues related to the use of generators. Defaults to `true`.
    pub generator_issues: bool,

    /// Report all issues related to array operations and usage. Defaults to `true`.
    pub array_issues: bool,

    /// Report issues related to the return type of functions and methods. Defaults to `true`.
    pub return_issues: bool,

    /// Report issues related to methods and their usage. Defaults to `true`.
    pub method_issues: bool,

    /// Report issues related to iterators and their usage. Defaults to `true`.
    pub iterator_issues: bool,

    /// Find and report expressions whose results are not used (e.g., `$a + $b;`). Defaults to `false`.
    pub find_unused_expressions: bool,

    /// Find and report unused definitions (e.g., private methods that are never called). Defaults to `false`.
    pub find_unused_definitions: bool,

    /// Analyze code that appears to be unreachable. Defaults to `false`.
    pub analyze_dead_code: bool,

    /// Track the literal values of class properties when they are assigned.
    /// This improves type inference but may increase memory usage. Defaults to `true`.
    pub memoize_properties: bool,

    /// Allow accessing array keys that may not be defined without reporting an issue. Defaults to `true`.
    pub allow_possibly_undefined_array_keys: bool,

    /// Enable checking for unhandled thrown exceptions.
    ///
    /// When `true`, the analyzer will report any exception that is thrown but not caught
    /// in a `try-catch` block or documented in a `@throws` tag.
    ///
    /// This check is disabled by default (`false`) as it can be computationally expensive.
    pub check_throws: bool,

    /// Perform heuristic checks to identify potential issues in the code.
    ///
    /// This includes checks that are not strictly type-related but can help catch common mistakes.
    /// Defaults to `true`.
    pub perform_heuristic_checks: bool,

    /// Enforce strict checks when accessing list elements by index.
    ///
    /// When `true`, the analyzer requires that any integer used to access a `list`
    /// element is provably non-negative (e.g., of type `int<0, max>`). This helps
    /// prevent potential runtime errors from using a negative index.
    ///
    /// When `false` (the default), any `int` is permitted as an index, offering
    /// more flexibility at the cost of type safety.
    pub strict_list_index_checks: bool,

    /// Disable comparisons to boolean literals (`true`/`false`).
    ///
    /// When enabled, comparisons to boolean literals will not be reported as issues.
    ///
    /// Defaults to `false`.
    pub no_boolean_literal_comparison: bool,

    /// Check for missing type hints on parameters, properties, and return types.
    ///
    /// When enabled, the analyzer will report warnings for function parameters, class properties,
    /// and function return types that lack explicit type declarations. The analyzer uses its
    /// type system knowledge to avoid false positives - for instance, it won't require a type hint
    /// on a property if adding one would conflict with a parent class or trait that has no type hint.
    ///
    /// Defaults to `false`.
    pub check_missing_type_hints: bool,

    /// Check for missing type hints (both parameters and return types) in closures when `check_missing_type_hints` is enabled.
    ///
    /// When `true`, closures (anonymous functions declared with `function() {}`) will be
    /// checked for missing type hints. When `false`, closures are ignored, which is useful
    /// because closures often rely on type inference.
    ///
    /// Defaults to `false`.
    pub check_closure_missing_type_hints: bool,

    /// Check for missing type hints (both parameters and return types) in arrow functions when `check_missing_type_hints` is enabled.
    ///
    /// When `true`, arrow functions (declared with `fn() => ...`) will be checked for missing
    /// type hints. When `false`, arrow functions are ignored, which is useful because arrow
    /// functions often rely on type inference and are typically short, making types obvious.
    ///
    /// Defaults to `false`.
    pub check_arrow_function_missing_type_hints: bool,

    /// Register superglobals (e.g., `$_GET`, `$_POST`, `$_SERVER`) in the analysis context.
    ///
    /// If disabled, super globals won't be available unless explicitly imported using
    /// the `global` keyword.
    ///
    /// Defaults to `true`.
    pub register_super_globals: bool,

    /// Enable colored output in terminal environments that support it. Defaults to `true`.
    ///
    /// This setting is primarily used for enabling/disabling colored diffs in
    /// issue reports.
    pub use_colors: bool,

    /// **Internal use only.**
    ///
    /// Enables a diffing mode for incremental analysis, used by integrations like LSPs.
    /// This avoids re-analyzing unchanged code in the same session. Defaults to `false`.
    pub diff: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self::new(PHPVersion::LATEST)
    }
}

impl Settings {
    pub fn new(version: PHPVersion) -> Self {
        Self {
            version,
            mixed_issues: true,
            falsable_issues: true,
            nullable_issues: true,
            redundancy_issues: true,
            reference_issues: true,
            unreachable_issues: true,
            deprecation_issues: true,
            impossibility_issues: true,
            existence_issues: true,
            template_issues: true,
            argument_issues: true,
            operand_issues: true,
            ambiguity_issues: true,
            property_issues: true,
            generator_issues: true,
            array_issues: true,
            return_issues: true,
            method_issues: true,
            iterator_issues: true,
            find_unused_expressions: false,
            find_unused_definitions: false,
            analyze_dead_code: false,
            memoize_properties: true,
            allow_possibly_undefined_array_keys: true,
            check_throws: false,
            use_colors: true,
            // TODO(azjezz): enable heuristic checks in the future,
            // need optimizations first
            perform_heuristic_checks: false,
            strict_list_index_checks: false,
            no_boolean_literal_comparison: false,
            check_missing_type_hints: false,
            check_closure_missing_type_hints: false,
            check_arrow_function_missing_type_hints: false,
            register_super_globals: true,
            diff: false,
        }
    }
}
