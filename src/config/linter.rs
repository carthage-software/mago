use std::path::PathBuf;

use mago_reporting::OnlyEntry;
use mago_reporting::OnlyInEntry;
use mago_reporting::baseline::BaselineVariant;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_linter::integration::Integration;
use mago_linter::integration::IntegrationSet;
use mago_linter::rule::filter_rules_settings;
use mago_linter::settings::RulesSettings;
use mago_php_version::PHPVersion;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
#[derive(Default)]
pub struct LinterConfiguration {
    /// A list of patterns to exclude from linting.
    pub excludes: Vec<String>,

    /// Run only specific rule codes, optionally scoped to paths.
    ///
    /// Same formats as analyzer/linter `ignore`: plain string (e.g. `"strict-types"`),
    /// or `{ code = "strict-types", in = "src/" }` / `in = ["src/", "tests/"]`.
    /// When non-empty, only these rules run and only matching issues are reported.
    #[serde(default)]
    pub only: Vec<OnlyEntry>,

    /// In the given paths, run only the specified rule; other paths are not restricted.
    ///
    /// Each entry is `{ code = "...", in = "path/" }` or `in = ["a/", "b/"]`. Files outside any entry's paths
    /// are unaffected.
    #[serde(default, rename = "only-in")]
    pub only_in: Vec<OnlyInEntry>,

    /// Integrations to enable during linting.
    pub integrations: Vec<Integration>,

    /// Settings for various linting rules.
    pub rules: RulesSettings,

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
}

impl LinterConfiguration {
    /// Returns a filtered version of the configuration suitable for display.
    ///
    /// This method excludes rules that don't match the configured integrations,
    /// so that only applicable rules are shown in the output.
    #[must_use]
    pub fn to_filtered_value(&self, php_version: PHPVersion) -> serde_json::Value {
        let integrations = IntegrationSet::from_slice(&self.integrations);
        let filtered_rules = filter_rules_settings(&self.rules, php_version, integrations);

        serde_json::json!({
            "excludes": self.excludes,
            "only": self.only,
            "only-in": self.only_in,
            "integrations": self.integrations,
            "rules": filtered_rules,
            "baseline": self.baseline,
            "baseline-variant": self.baseline_variant,
        })
    }
}
