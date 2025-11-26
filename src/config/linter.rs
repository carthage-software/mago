use std::path::PathBuf;

use mago_reporting::baseline::BaselineVariant;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_linter::integration::Integration;
use mago_linter::settings::RulesSettings;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
#[derive(Default)]
pub struct LinterConfiguration {
    /// A list of patterns to exclude from linting.
    pub excludes: Vec<String>,

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
