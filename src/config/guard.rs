use std::path::PathBuf;

use mago_reporting::Level;
use mago_reporting::baseline::BaselineVariant;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_guard::settings::Settings;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct GuardConfiguration {
    /// A list of patterns to exclude from guard checking.
    pub excludes: Vec<String>,

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
    /// Options: `"note"`, `"help"`, `"warning"`, `"error"`
    ///
    /// Can be overridden by the `--minimum-fail-level` CLI flag.
    ///
    /// Defaults to `"error"`.
    pub minimum_fail_level: Level,

    /// Guard settings including rules, layers, and layering.
    #[serde(flatten)]
    pub settings: Settings,
}

impl Default for GuardConfiguration {
    fn default() -> Self {
        Self {
            excludes: vec![],
            baseline: None,
            baseline_variant: BaselineVariant::default(),
            minimum_fail_level: Level::Error,
            settings: Settings::default(),
        }
    }
}
