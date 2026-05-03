use std::path::PathBuf;

use clap::ColorChoice;
use mago_algebra::DEFAULT_CONSENSUS_LIMIT;
use mago_algebra::DEFAULT_DISJUNCTION_COMPLEXITY;
use mago_algebra::DEFAULT_NEGATION_COMPLEXITY;
use mago_algebra::DEFAULT_SATURATION_COMPLEXITY;
use mago_analyzer::settings::DEFAULT_FORMULA_SIZE_THRESHOLD;
use mago_analyzer::settings::DEFAULT_LOOP_ASSIGNMENT_DEPTH_THRESHOLD;
use mago_analyzer::settings::Settings;
use mago_atom::ascii_lowercase_atom;
use mago_atom::atom;
use mago_codex::ttype::combiner::DEFAULT_ARRAY_COMBINATION_THRESHOLD;
use mago_codex::ttype::combiner::DEFAULT_INTEGER_COMBINATION_THRESHOLD;
use mago_codex::ttype::combiner::DEFAULT_STRING_COMBINATION_THRESHOLD;
use mago_php_version::PHPVersion;
use mago_reporting::IgnoreEntry;
use mago_reporting::Level;
use mago_reporting::baseline::BaselineVariant;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::utils::should_use_colors;

/// Configuration options for the static analyzer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct AnalyzerConfiguration {
    /// A list of patterns to exclude from analysis.
    pub excludes: Vec<String>,

    /// Ignore specific issues based on their code, optionally scoped to paths.
    pub ignore: Vec<IgnoreEntry>,

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

    /// Set the minimum issue severity that causes the command to fail.
    ///
    /// The command will exit with a non-zero status if any issues at or above
    /// this level are found. For example, setting this to `"warning"` means
    /// the command fails on warnings and errors, but not on notes or help suggestions.
    ///
    /// Options: `"note"`, `"help"`, `"warning"`, `"error"`
    ///
    /// Can be overridden by the `--minimum-fail-level` CLI flag.
    ///
    /// Defaults to `"error"`.
    pub minimum_fail_level: Level,

    /// Disable all default plugins (including stdlib).
    ///
    /// When set to `true`, no plugins will be loaded by default, and only plugins
    /// explicitly listed in `plugins` will be enabled.
    ///
    /// Defaults to `false`.
    pub disable_default_plugins: bool,

    /// List of plugins to enable (by name or alias).
    ///
    /// Plugins can be specified by their canonical name or any of their aliases:
    /// - `stdlib` (aliases: `standard`, `std`, `php-stdlib`)
    /// - `psl` (aliases: `php-standard-library`, `azjezz-psl`)
    /// - `flow-php` (aliases: `flow`, `flow-etl`)
    /// - `psr-container` (aliases: `psr-11`)
    ///
    /// Example: `plugins = ["stdlib", "psl"]`
    pub plugins: Vec<String>,

    /// Whether to find unused expressions.
    pub find_unused_expressions: bool,

    /// Whether to find unused definitions.
    pub find_unused_definitions: bool,

    /// Whether to warn when a function's declared return type contains a branch the body never
    /// actually returns (e.g. `: string|false` on a function that always returns a string).
    pub find_overly_wide_return_types: bool,

    /// Whether to analyze dead code.
    pub analyze_dead_code: bool,

    /// Whether to memoize properties.
    pub memoize_properties: bool,

    /// Allow accessing array keys that may not be defined without reporting an issue.
    ///
    /// **Deprecated:** prefer `strict_array_index_existence` for new configurations.
    /// Setting this option to `false` only emits a warning on possibly-undefined
    /// `array<K, V>` reads with a single literal key, and does not widen the result
    /// to `T|null` — so downstream `=== null` and `??` checks behave inconsistently.
    /// `strict_array_index_existence = true` warns more thoroughly (lists, shapes,
    /// and `array<K, V>`), and widens the type to `T|null` so the runtime semantics
    /// are reflected in the type system.
    ///
    /// This setting is retained for backwards compatibility and may be removed in
    /// a future release.
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

    /// Treat array/list indices that are not provably present as `T|null` and warn on access.
    ///
    /// When `true`, reading an array or list key whose presence is not guaranteed emits a
    /// `possibly-undefined-{int,string}-array-index` warning and the resulting type is
    /// widened to include `null`. This applies to `list<T>` (non-zero indices),
    /// optional entries of `array{...}` shapes, and `array<K, V>` lookups by arbitrary keys.
    /// It lets `=== null`, `??`, and `??=` reflect PHP's actual runtime semantics.
    ///
    /// Defaults to `false` to keep existing PHP-friendly behavior, where missing keys are
    /// merely tracked internally as possibly-undefined without emitting a warning.
    pub strict_array_index_existence: bool,

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

    /// Skip missing-type-hint checks for closures and arrow functions used
    /// as the right-hand side of the pipe operator (`|>`).
    ///
    /// When `true`, an inline pipe callable like
    /// `$x |> fn($p) => strtoupper($p)` will not warn about missing parameter
    /// or return types, even when the closure / arrow-function checks are
    /// otherwise enabled. The pipe operand's type is enough to derive the
    /// parameter type, so the hint is mostly noise.
    ///
    /// Defaults to `false`.
    pub allow_implicit_pipe_callable_types: bool,

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

    /// Check for incorrect casing when referencing classes, interfaces, traits, enums,
    /// and functions.
    ///
    /// Defaults to `false`.
    #[serde(default)]
    pub check_experimental: bool,

    /// Defaults to `false`.
    #[serde(default)]
    pub check_name_casing: bool,

    /// Enforce that concrete classes are declared `final`.
    ///
    /// When enabled, the analyzer reports a warning for any class that is not
    /// `final`, `abstract`, or annotated with `@api`, provided the class has no children.
    ///
    /// Defaults to `false`.
    pub enforce_class_finality: bool,

    /// Require `@api` or `@internal` annotations on abstract classes, interfaces, and traits.
    ///
    /// When enabled, the analyzer reports a warning for any abstract class, interface,
    /// or trait that is not annotated with either `@api` or `@internal`.
    ///
    /// Defaults to `false`.
    pub require_api_or_internal: bool,

    /// Whether to allow calls to impure functions inside conditions.
    ///
    /// When set to `false`, any call to a function not marked `@pure` or
    /// `@mutation-free` inside an `if`, `while`, `for`, ternary, or `match`
    /// condition is reported. This helps catch surprising evaluation-order
    /// bugs where a side effect in one part of a condition silently alters
    /// a variable used in another part.
    ///
    /// Defaults to `true` (impure calls in conditions are allowed).
    pub allow_side_effects_in_conditions: bool,

    /// **Deprecated**: Use `check-missing-override` and `find-unused-parameters` instead.
    ///
    /// When set to `true`, enables both `check-missing-override` and `find-unused-parameters`.
    /// When set to `false`, disables both.
    ///
    /// This option is kept for backwards compatibility with existing configurations.
    #[serde(skip_serializing)]
    pub perform_heuristic_checks: Option<bool>,

    /// Performance tuning settings.
    ///
    /// These thresholds control how deeply the analyzer explores complex logical formulas.
    /// Higher values allow more precise analysis but may significantly increase analysis time.
    /// Lower values improve speed but may reduce precision on complex conditional code.
    #[serde(default)]
    pub performance: PerformanceConfiguration,
}

/// Performance tuning settings for the analyzer.
///
/// These thresholds control the complexity limits for logical formula operations.
/// Adjusting these values allows trading off between analysis precision and speed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct PerformanceConfiguration {
    /// Maximum number of clauses to process during CNF saturation.
    ///
    /// Controls how many clauses the simplification algorithm will work with.
    /// If exceeded, saturation returns an empty result to avoid performance issues.
    ///
    /// Defaults to `8192`.
    pub saturation_complexity_threshold: u16,

    /// Maximum number of clauses per side in disjunction operations.
    ///
    /// Controls the complexity limit for OR operations between clause sets.
    /// If either side exceeds this, the disjunction returns an empty result.
    ///
    /// Defaults to `4096`.
    pub disjunction_complexity_threshold: u16,

    /// Maximum cumulative complexity during formula negation.
    ///
    /// Controls how complex the negation of a formula can become.
    /// If exceeded, negation gives up to avoid exponential blowup.
    ///
    /// Defaults to `4096`.
    pub negation_complexity_threshold: u16,

    /// Upper limit for consensus optimization during saturation.
    ///
    /// Controls when the consensus rule is applied during saturation.
    /// Only applies when clause count is between 3 and this limit.
    ///
    /// Defaults to `256`.
    pub consensus_limit_threshold: u16,

    /// Maximum logical formula size during conditional analysis.
    ///
    /// Limits the size of generated formulas to prevent exponential blowup
    /// in deeply nested conditionals.
    ///
    /// Defaults to `512`.
    pub formula_size_threshold: u16,

    /// Maximum number of literal strings to track before generalizing.
    ///
    /// When combining types with many different literal string values, tracking each
    /// literal individually causes O(n) memory and O(n²) comparison time.
    /// Once the threshold is exceeded, we generalize to the base string type.
    ///
    /// Defaults to `128`.
    #[serde(alias = "string_concat_combination_threshold", alias = "string-concat-combination-threshold")]
    pub string_combination_threshold: u16,

    /// Maximum number of literal integers to track before generalizing.
    ///
    /// When combining types with many different literal integer values, tracking each
    /// literal individually causes O(n) memory and O(n²) comparison time.
    /// Once the threshold is exceeded, we generalize to the base int type.
    ///
    /// Defaults to `128`.
    pub integer_combination_threshold: u16,

    /// Maximum number of array elements to track individually.
    ///
    /// When building array types through repeated push operations (`$arr[] = ...`),
    /// this limits how many individual elements are tracked before generalizing
    /// to a simpler array type. This prevents memory explosion on files with
    /// thousands of array pushes.
    ///
    /// Defaults to `32`.
    pub array_combination_threshold: u16,

    /// Maximum depth of the loop assignment dependency graph that the
    /// fixed-point analyzer will explore when re-analysing loop bodies.
    ///
    /// The analyzer uses fixed-point iteration to propagate widened types
    /// along loop-carried dependency chains. A chain of length `N` can
    /// require up to `N` extra passes for the type at the end of the chain
    /// to fully stabilise, and each pass re-analyses the entire loop body.
    /// On large, complex loops the per-pass cost dominates file analysis
    /// time.
    ///
    /// The default of `1` means each loop body is re-analysed at most once
    /// after the initial pass; enough to stabilise virtually all real-world
    /// code while keeping analysis cost bounded. Projects that require
    /// maximally precise narrowing of long loop-carried chains can raise
    /// this value (typically to `2` or `3`) at the cost of significantly
    /// slower analysis on complex files. Setting this to `0` disables
    /// fixed-point iteration entirely.
    ///
    /// Defaults to `1`.
    pub loop_assignment_depth_threshold: u8,
}

impl Default for PerformanceConfiguration {
    fn default() -> Self {
        Self {
            saturation_complexity_threshold: DEFAULT_SATURATION_COMPLEXITY,
            disjunction_complexity_threshold: DEFAULT_DISJUNCTION_COMPLEXITY,
            negation_complexity_threshold: DEFAULT_NEGATION_COMPLEXITY,
            consensus_limit_threshold: DEFAULT_CONSENSUS_LIMIT,
            formula_size_threshold: DEFAULT_FORMULA_SIZE_THRESHOLD,
            string_combination_threshold: DEFAULT_STRING_COMBINATION_THRESHOLD,
            integer_combination_threshold: DEFAULT_INTEGER_COMBINATION_THRESHOLD,
            array_combination_threshold: DEFAULT_ARRAY_COMBINATION_THRESHOLD,
            loop_assignment_depth_threshold: DEFAULT_LOOP_ASSIGNMENT_DEPTH_THRESHOLD,
        }
    }
}

impl AnalyzerConfiguration {
    pub fn to_settings(&self, php_version: PHPVersion, color_choice: ColorChoice, enable_diff: bool) -> Settings {
        // Backwards compatibility: if perform_heuristic_checks is set, use it for both options
        let check_missing_override = self.perform_heuristic_checks.unwrap_or(self.check_missing_override);
        let find_unused_parameters = self.perform_heuristic_checks.unwrap_or(self.find_unused_parameters);

        if !self.allow_possibly_undefined_array_keys {
            tracing::warn!(
                "`allow-possibly-undefined-array-keys = false` is deprecated and will be removed in a future release. \
                 Prefer `strict-array-index-existence = true`, which warns more thoroughly (lists, shapes, and \
                 `array<K, V>`) and widens the result type to `T|null` so `=== null` and `??` checks reflect runtime \
                 semantics."
            );
        }

        Settings {
            version: php_version,
            analyze_dead_code: self.analyze_dead_code,
            find_unused_definitions: self.find_unused_definitions,
            find_overly_wide_return_types: self.find_overly_wide_return_types,
            find_unused_expressions: self.find_unused_expressions,
            memoize_properties: self.memoize_properties,
            allow_possibly_undefined_array_keys: self.allow_possibly_undefined_array_keys,
            check_throws: self.check_throws,
            unchecked_exceptions: self.unchecked_exceptions.iter().map(|s| atom(s.as_str())).collect(),
            unchecked_exception_classes: self.unchecked_exception_classes.iter().map(|s| atom(s.as_str())).collect(),
            check_missing_override,
            find_unused_parameters,
            strict_list_index_checks: self.strict_list_index_checks,
            strict_array_index_existence: self.strict_array_index_existence,
            no_boolean_literal_comparison: self.no_boolean_literal_comparison,
            enforce_class_finality: self.enforce_class_finality,
            require_api_or_internal: self.require_api_or_internal,
            check_missing_type_hints: self.check_missing_type_hints,
            check_closure_missing_type_hints: self.check_closure_missing_type_hints,
            check_arrow_function_missing_type_hints: self.check_arrow_function_missing_type_hints,
            allow_implicit_pipe_callable_types: self.allow_implicit_pipe_callable_types,
            register_super_globals: self.register_super_globals,
            use_colors: should_use_colors(color_choice),
            diff: enable_diff,
            trust_existence_checks: self.trust_existence_checks,
            class_initializers: self.class_initializers.iter().map(|s| ascii_lowercase_atom(s.as_str())).collect(),
            check_property_initialization: self.check_property_initialization,
            check_use_statements: self.check_use_statements,
            check_experimental: self.check_experimental,
            check_name_casing: self.check_name_casing,
            allow_side_effects_in_conditions: self.allow_side_effects_in_conditions,
            saturation_complexity_threshold: self.performance.saturation_complexity_threshold,
            disjunction_complexity_threshold: self.performance.disjunction_complexity_threshold,
            negation_complexity_threshold: self.performance.negation_complexity_threshold,
            consensus_limit_threshold: self.performance.consensus_limit_threshold,
            formula_size_threshold: self.performance.formula_size_threshold,
            string_combination_threshold: self.performance.string_combination_threshold,
            integer_combination_threshold: self.performance.integer_combination_threshold,
            array_combination_threshold: self.performance.array_combination_threshold,
            loop_assignment_depth_threshold: self.performance.loop_assignment_depth_threshold,
        }
    }
}

impl Default for AnalyzerConfiguration {
    fn default() -> Self {
        let defaults = Settings::default();

        Self {
            disable_default_plugins: false,
            plugins: vec![],
            excludes: vec![],
            ignore: vec![],
            baseline: None,
            baseline_variant: BaselineVariant::default(),
            minimum_fail_level: Level::Error,
            find_unused_expressions: defaults.find_unused_expressions,
            find_unused_definitions: defaults.find_unused_definitions,
            find_overly_wide_return_types: defaults.find_overly_wide_return_types,
            analyze_dead_code: defaults.analyze_dead_code,
            memoize_properties: defaults.memoize_properties,
            allow_possibly_undefined_array_keys: defaults.allow_possibly_undefined_array_keys,
            check_throws: defaults.check_throws,
            unchecked_exceptions: vec![],
            unchecked_exception_classes: vec![],
            check_missing_override: defaults.check_missing_override,
            find_unused_parameters: defaults.find_unused_parameters,
            strict_list_index_checks: defaults.strict_list_index_checks,
            strict_array_index_existence: defaults.strict_array_index_existence,
            no_boolean_literal_comparison: defaults.no_boolean_literal_comparison,
            enforce_class_finality: defaults.enforce_class_finality,
            require_api_or_internal: defaults.require_api_or_internal,
            check_missing_type_hints: defaults.check_missing_type_hints,
            check_closure_missing_type_hints: defaults.check_closure_missing_type_hints,
            check_arrow_function_missing_type_hints: defaults.check_arrow_function_missing_type_hints,
            allow_implicit_pipe_callable_types: defaults.allow_implicit_pipe_callable_types,
            register_super_globals: defaults.register_super_globals,
            trust_existence_checks: defaults.trust_existence_checks,
            class_initializers: vec![],
            check_property_initialization: defaults.check_property_initialization,
            check_use_statements: defaults.check_use_statements,
            check_experimental: defaults.check_experimental,
            check_name_casing: defaults.check_name_casing,
            allow_side_effects_in_conditions: defaults.allow_side_effects_in_conditions,
            perform_heuristic_checks: None,
            performance: PerformanceConfiguration::default(),
        }
    }
}
