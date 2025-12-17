use std::io::IsTerminal;
use std::path::PathBuf;

use clap::ColorChoice;
use mago_atom::ascii_lowercase_atom;
use mago_atom::atom;
use mago_reporting::baseline::BaselineVariant;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_analyzer::settings::Settings;
use mago_php_version::PHPVersion;

/// Configuration options for the static analyzer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct AnalyzerConfiguration {
    /// A list of patterns to exclude from analysis.
    pub excludes: Vec<String>,

    /// Ignore specific issues based on their code.
    pub ignore: Vec<String>,

    /// Path to a baseline file to ignore listed issues.
    pub baseline: Option<PathBuf>,

    /// The baseline variant to use when generating new baselines.
    ///
    /// Options:
    ///
    /// - `"strict"`: Exact line matching with start/end line numbers
    /// - `"loose"`: Count-based matching by (file, code, message) tuple (default)
    ///
    /// The loose variant is more resilient to code changes as line number shifts
    /// don't affect the baseline.
    pub baseline_variant: BaselineVariant,

    /// Whether to find unused expressions.
    pub find_unused_expressions: bool,

    /// Whether to find unused definitions.
    pub find_unused_definitions: bool,

    /// Whether to analyze dead code.
    pub analyze_dead_code: bool,

    /// Whether to memoize properties.
    pub memoize_properties: bool,

    /// Allow accessing array keys that may not be defined without reporting an issue.
    pub allow_possibly_undefined_array_keys: bool,

    /// Whether to check for thrown exceptions.
    pub check_throws: bool,

    /// Exceptions to ignore including all subclasses (hierarchy-aware).
    ///
    /// When an exception class is listed here, any exception of that class or any of its
    /// subclasses will be ignored during `check_throws` analysis.
    ///
    /// For example, adding `LogicException` will ignore `LogicException`, `InvalidArgumentException`,
    /// `OutOfBoundsException`, and all other subclasses.
    pub unchecked_exceptions: Vec<String>,

    /// Exceptions to ignore (exact class match only, not subclasses).
    ///
    /// When an exception class is listed here, only that exact class will be ignored
    /// during `check_throws` analysis. Parent classes and subclasses are not affected.
    pub unchecked_exception_classes: Vec<String>,

    /// Enforce strict checks when accessing list elements by index.
    ///
    /// When `true`, the analyzer requires that any integer used to access a `list`
    /// element is provably non-negative (e.g., of type `int<0, max>`). This helps
    /// prevent potential runtime errors from using a negative index.
    ///
    /// When `false` (the default), any `int` is permitted as an index, offering
    /// more flexibility at the cost of type safety.
    pub strict_list_index_checks: bool,

    /// Disallow comparisons where a boolean literal is used as an operand.
    ///
    /// Defaults to `false`.
    pub no_boolean_literal_comparison: bool,

    /// Check for missing type hints on parameters, properties, and return types.
    ///
    /// When enabled, the analyzer will report warnings for function parameters, class properties,
    /// and function return types that lack explicit type declarations.
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

    /// Check for missing `#[Override]` attributes on overriding methods.
    ///
    /// When enabled, the analyzer reports methods that override a parent method without
    /// the `#[Override]` attribute (PHP 8.3+).
    ///
    /// Defaults to `true`.
    pub check_missing_override: bool,

    /// Find and report unused function/method parameters.
    ///
    /// When enabled, the analyzer reports parameters that are declared but never used
    /// within the function body.
    ///
    /// Defaults to `true`.
    pub find_unused_parameters: bool,

    /// Trust symbol existence checks to narrow types.
    ///
    /// When enabled, conditional checks like `method_exists()`, `property_exists()`,
    /// `function_exists()`, and `defined()` will narrow the type within the conditional block,
    /// suppressing errors for symbols that are verified to exist at runtime.
    ///
    /// When disabled, these checks are ignored and the analyzer requires explicit type hints,
    /// which is stricter but may produce more false positives for dynamic code.
    ///
    /// Defaults to `true`.
    pub trust_existence_checks: bool,

    /// Method names treated as class initializers (like `__construct`).
    ///
    /// Properties initialized in these methods count as "definitely initialized"
    /// just like in the constructor. This is useful for frameworks that use
    /// lifecycle methods like PHPUnit's `setUp()` or framework `boot()` methods.
    ///
    /// Example: `["setUp", "initialize", "boot"]`
    ///
    /// Defaults to empty (no additional initializers).
    pub class_initializers: Vec<String>,

    /// Enable property initialization checking (`missing-constructor`, `uninitialized-property`).
    ///
    /// When `false`, disables both `missing-constructor` and `uninitialized-property` issues
    /// entirely. This is useful for projects that prefer to rely on runtime errors for
    /// property initialization.
    ///
    /// Defaults to `false`.
    pub check_property_initialization: bool,

    /// Check for non-existent symbols in use statements.
    ///
    /// When enabled, the analyzer will report use statements that import symbols
    /// (classes, interfaces, traits, enums, functions, or constants) that do not exist
    /// in the codebase.
    ///
    /// Defaults to `false`.
    pub check_use_statements: bool,

    /// **Deprecated**: Use `check-missing-override` and `find-unused-parameters` instead.
    ///
    /// When set to `true`, enables both `check-missing-override` and `find-unused-parameters`.
    /// When set to `false`, disables both.
    ///
    /// This option is kept for backwards compatibility with existing configurations.
    #[serde(skip_serializing)]
    pub perform_heuristic_checks: Option<bool>,
}

impl AnalyzerConfiguration {
    pub fn to_settings(&self, php_version: PHPVersion, color_choice: ColorChoice, enable_diff: bool) -> Settings {
        // Backwards compatibility: if perform_heuristic_checks is set, use it for both options
        let check_missing_override = self.perform_heuristic_checks.unwrap_or(self.check_missing_override);
        let find_unused_parameters = self.perform_heuristic_checks.unwrap_or(self.find_unused_parameters);

        Settings {
            version: php_version,
            analyze_dead_code: self.analyze_dead_code,
            find_unused_definitions: self.find_unused_definitions,
            find_unused_expressions: self.find_unused_expressions,
            memoize_properties: self.memoize_properties,
            allow_possibly_undefined_array_keys: self.allow_possibly_undefined_array_keys,
            check_throws: self.check_throws,
            unchecked_exceptions: self.unchecked_exceptions.iter().map(|s| atom(s.as_str())).collect(),
            unchecked_exception_classes: self.unchecked_exception_classes.iter().map(|s| atom(s.as_str())).collect(),
            check_missing_override,
            find_unused_parameters,
            strict_list_index_checks: self.strict_list_index_checks,
            no_boolean_literal_comparison: self.no_boolean_literal_comparison,
            check_missing_type_hints: self.check_missing_type_hints,
            check_closure_missing_type_hints: self.check_closure_missing_type_hints,
            check_arrow_function_missing_type_hints: self.check_arrow_function_missing_type_hints,
            register_super_globals: self.register_super_globals,
            use_colors: match color_choice {
                ColorChoice::Always => true,
                ColorChoice::Never => false,
                ColorChoice::Auto => std::io::stdout().is_terminal(),
            },
            diff: enable_diff,
            trust_existence_checks: self.trust_existence_checks,
            class_initializers: self.class_initializers.iter().map(|s| ascii_lowercase_atom(s.as_str())).collect(),
            check_property_initialization: self.check_property_initialization,
            check_use_statements: self.check_use_statements,
        }
    }
}

impl Default for AnalyzerConfiguration {
    fn default() -> Self {
        let defaults = Settings::default();

        Self {
            excludes: vec![],
            ignore: vec![],
            baseline: None,
            baseline_variant: BaselineVariant::default(),
            find_unused_expressions: defaults.find_unused_expressions,
            find_unused_definitions: defaults.find_unused_definitions,
            analyze_dead_code: defaults.analyze_dead_code,
            memoize_properties: defaults.memoize_properties,
            allow_possibly_undefined_array_keys: defaults.allow_possibly_undefined_array_keys,
            check_throws: defaults.check_throws,
            unchecked_exceptions: vec![],
            unchecked_exception_classes: vec![],
            check_missing_override: defaults.check_missing_override,
            find_unused_parameters: defaults.find_unused_parameters,
            strict_list_index_checks: defaults.strict_list_index_checks,
            no_boolean_literal_comparison: defaults.no_boolean_literal_comparison,
            check_missing_type_hints: defaults.check_missing_type_hints,
            check_closure_missing_type_hints: defaults.check_closure_missing_type_hints,
            check_arrow_function_missing_type_hints: defaults.check_arrow_function_missing_type_hints,
            register_super_globals: defaults.register_super_globals,
            trust_existence_checks: defaults.trust_existence_checks,
            class_initializers: vec![],
            check_property_initialization: defaults.check_property_initialization,
            check_use_statements: defaults.check_use_statements,
            perform_heuristic_checks: None,
        }
    }
}
