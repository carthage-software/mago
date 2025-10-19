use std::io::IsTerminal;
use std::path::PathBuf;

use clap::ColorChoice;
use serde::Deserialize;
use serde::Serialize;

use mago_analyzer::settings::Settings;
use mago_php_version::PHPVersion;

/// Configuration options for the static analyzer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct AnalyzerConfiguration {
    /// A list of patterns to exclude from analysis.
    pub excludes: Vec<String>,

    /// Ignore specific issues based on their code.
    pub ignore: Vec<String>,

    /// Path to a baseline file to ignore listed issues.
    pub baseline: Option<PathBuf>,

    /// Report all issues related to the use of `mixed` types.
    pub mixed_issues: bool,

    /// Report all issues related to possibly `false` values.
    pub falsable_issues: bool,

    /// Report all issues related to possibly `null` values.
    pub nullable_issues: bool,

    /// Report all issues related to redundant code.
    pub redundancy_issues: bool,

    /// Report all issues related to by-reference variables.
    pub reference_issues: bool,

    /// Report all issues related to unreachable code.
    pub unreachable_issues: bool,

    /// Report all issues related to using deprecated code.
    pub deprecation_issues: bool,

    /// Report all issues related to logically impossible conditions.
    pub impossibility_issues: bool,

    /// Report all issues related to ambiguous code constructs.
    pub ambiguity_issues: bool,

    /// Report all issues related to the existence of symbols (e.g., classes, functions, constants).
    pub existence_issues: bool,

    /// Report all issues related to generic template types and their usage.
    pub template_issues: bool,

    /// Report all issues related to function arguments.
    pub argument_issues: bool,

    /// Report all issues related to operands in expressions.
    pub operand_issues: bool,

    /// Report all issues related to properties and their usage.
    pub property_issues: bool,

    /// Report all issues related to the use of generators.
    pub generator_issues: bool,

    /// Report all issues related to array operations and usage.
    pub array_issues: bool,

    /// Report issues related to the return type of functions and methods.
    pub return_issues: bool,

    /// Report issues related to methods and their usage.
    pub method_issues: bool,

    /// Report issues related to iterators and their usage.
    pub iterator_issues: bool,

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

    /// Whether to perform heuristic checks.
    pub perform_heuristic_checks: bool,
}

impl AnalyzerConfiguration {
    pub fn to_settings(&self, php_version: PHPVersion, color_choice: ColorChoice) -> Settings {
        Settings {
            version: php_version,
            mixed_issues: self.mixed_issues,
            falsable_issues: self.falsable_issues,
            nullable_issues: self.nullable_issues,
            redundancy_issues: self.redundancy_issues,
            reference_issues: self.reference_issues,
            unreachable_issues: self.unreachable_issues,
            deprecation_issues: self.deprecation_issues,
            impossibility_issues: self.impossibility_issues,
            ambiguity_issues: self.ambiguity_issues,
            existence_issues: self.existence_issues,
            template_issues: self.template_issues,
            argument_issues: self.argument_issues,
            operand_issues: self.operand_issues,
            property_issues: self.property_issues,
            generator_issues: self.generator_issues,
            array_issues: self.array_issues,
            return_issues: self.return_issues,
            method_issues: self.method_issues,
            iterator_issues: self.iterator_issues,
            analyze_dead_code: self.analyze_dead_code,
            find_unused_definitions: self.find_unused_definitions,
            find_unused_expressions: self.find_unused_expressions,
            memoize_properties: self.memoize_properties,
            allow_possibly_undefined_array_keys: self.allow_possibly_undefined_array_keys,
            check_throws: self.check_throws,
            perform_heuristic_checks: self.perform_heuristic_checks,
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
            diff: false,
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
            mixed_issues: defaults.mixed_issues,
            falsable_issues: defaults.falsable_issues,
            nullable_issues: defaults.nullable_issues,
            redundancy_issues: defaults.redundancy_issues,
            reference_issues: defaults.reference_issues,
            unreachable_issues: defaults.unreachable_issues,
            deprecation_issues: defaults.deprecation_issues,
            impossibility_issues: defaults.impossibility_issues,
            ambiguity_issues: defaults.ambiguity_issues,
            existence_issues: defaults.existence_issues,
            template_issues: defaults.template_issues,
            argument_issues: defaults.argument_issues,
            operand_issues: defaults.operand_issues,
            property_issues: defaults.property_issues,
            generator_issues: defaults.generator_issues,
            array_issues: defaults.array_issues,
            return_issues: defaults.return_issues,
            method_issues: defaults.method_issues,
            iterator_issues: defaults.iterator_issues,
            find_unused_expressions: defaults.find_unused_expressions,
            find_unused_definitions: defaults.find_unused_definitions,
            analyze_dead_code: defaults.analyze_dead_code,
            memoize_properties: defaults.memoize_properties,
            allow_possibly_undefined_array_keys: defaults.allow_possibly_undefined_array_keys,
            check_throws: defaults.check_throws,
            perform_heuristic_checks: defaults.perform_heuristic_checks,
            strict_list_index_checks: defaults.strict_list_index_checks,
            no_boolean_literal_comparison: defaults.no_boolean_literal_comparison,
            check_missing_type_hints: defaults.check_missing_type_hints,
            check_closure_missing_type_hints: defaults.check_closure_missing_type_hints,
            check_arrow_function_missing_type_hints: defaults.check_arrow_function_missing_type_hints,
            register_super_globals: defaults.register_super_globals,
        }
    }
}
