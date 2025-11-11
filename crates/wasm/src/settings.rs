//! # WASM Settings
//!
//! This module defines the configuration structures used to control the behavior of
//! Mago's tools (linter, formatter, analyzer) when called from a WebAssembly environment.
//! These structs are designed to be deserialized from JavaScript objects.

use serde::Deserialize;

use mago_analyzer::settings::Settings as AnalyzerSettings;
use mago_formatter::settings::FormatSettings;
use mago_guard::settings::Settings as GuardSettings;
use mago_linter::integration::Integration;
use mago_linter::integration::IntegrationSet;
use mago_linter::settings::RulesSettings;
use mago_linter::settings::Settings as LinterSettings;
use mago_orchestrator::OrchestratorConfiguration;
use mago_php_version::PHPVersion;

/// The root settings object for the Mago WASM API.
#[derive(Debug, Deserialize, Default)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct WasmSettings {
    pub php_version: PHPVersion,
    pub linter: WasmLinterSettings,
    pub analyzer: WasmAnalyzerSettings,
    pub formatter: FormatSettings,
}

/// WASM-specific settings for the linter.
#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct WasmLinterSettings {
    pub integrations: Vec<Integration>,
    pub rules: RulesSettings,
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct WasmAnalyzerSettings {
    pub ignore: Vec<String>,
    pub find_unused_expressions: bool,
    pub find_unused_definitions: bool,
    pub analyze_dead_code: bool,
    pub memoize_properties: bool,
    pub allow_possibly_undefined_array_keys: bool,
    pub check_throws: bool,
    pub perform_heuristic_checks: bool,
    pub strict_list_index_checks: bool,
    pub no_boolean_literal_comparison: bool,
    pub check_missing_type_hints: bool,
    pub check_closure_missing_type_hints: bool,
    pub check_arrow_function_missing_type_hints: bool,
    pub register_super_globals: bool,
    pub use_colors: bool,
}

impl WasmLinterSettings {
    /// Converts WASM linter settings into the core linter settings struct.
    pub fn to_linter_settings(&self, php_version: PHPVersion) -> LinterSettings {
        LinterSettings {
            php_version,
            integrations: IntegrationSet::from_slice(&self.integrations),
            rules: self.rules.clone(),
        }
    }
}

impl WasmAnalyzerSettings {
    pub fn to_analyzer_settings(&self, php_version: PHPVersion) -> AnalyzerSettings {
        AnalyzerSettings {
            version: php_version,
            analyze_dead_code: self.analyze_dead_code,
            find_unused_definitions: self.find_unused_definitions,
            find_unused_expressions: self.find_unused_expressions,
            memoize_properties: self.memoize_properties,
            allow_possibly_undefined_array_keys: self.allow_possibly_undefined_array_keys,
            check_throws: self.check_throws,
            strict_list_index_checks: self.strict_list_index_checks,
            perform_heuristic_checks: self.perform_heuristic_checks,
            no_boolean_literal_comparison: self.no_boolean_literal_comparison,
            check_missing_type_hints: self.check_missing_type_hints,
            check_closure_missing_type_hints: self.check_closure_missing_type_hints,
            check_arrow_function_missing_type_hints: self.check_arrow_function_missing_type_hints,
            register_super_globals: self.register_super_globals,
            use_colors: self.use_colors,
            diff: false, // Not applicable in wasm context
        }
    }
}

impl WasmSettings {
    /// Converts WASM settings into an `OrchestratorConfiguration`.
    pub fn to_orchestrator_config(&self) -> OrchestratorConfiguration<'_> {
        OrchestratorConfiguration {
            php_version: self.php_version,
            analyzer_settings: self.analyzer.to_analyzer_settings(self.php_version),
            linter_settings: self.linter.to_linter_settings(self.php_version),
            guard_settings: GuardSettings::default(),
            formatter_settings: self.formatter,
            // WASM context does not support progress bars or path configurations
            paths: vec![],
            includes: vec![],
            excludes: vec![],
            extensions: vec![],
            use_progress_bars: false,
            use_colors: false,
        }
    }
}
