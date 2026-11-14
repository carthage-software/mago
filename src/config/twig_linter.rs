use std::path::PathBuf;

use mago_database::GlobSettings;
use mago_reporting::Level;
use mago_reporting::baseline::BaselineVariant;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_twig_linter::settings::RulesSettings;
use mago_twig_linter::settings::Settings;
use mago_twig_linter::version::TwigVersion;

/// Settings used only by `mago twig:lint`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct TwigLinterConfiguration {
    /// Whether `mago twig:lint` runs at all.
    pub enabled: bool,

    /// The oldest Twig 3 version the project supports, such as `"3.12"`.
    ///
    /// Rules that need a newer version stay off, even with `--pedantic` or `--only`.
    pub version: TwigVersion,

    /// A list of patterns to exclude from linting.
    pub excludes: Vec<String>,

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

    /// Set the minimum issue severity that causes the command to fail.
    ///
    /// Options: `"note"`, `"help"`, `"warning"`, `"error"`
    ///
    /// Can be overridden by the `--minimum-fail-level` CLI flag.
    ///
    /// Defaults to `"error"`.
    pub minimum_fail_level: Level,
}

impl Default for TwigLinterConfiguration {
    fn default() -> Self {
        Self {
            enabled: true,
            version: TwigVersion::default(),
            excludes: vec![],
            rules: RulesSettings::default(),
            baseline: None,
            baseline_variant: BaselineVariant::default(),
            minimum_fail_level: Level::Error,
        }
    }
}

impl TwigLinterConfiguration {
    /// The linter settings for this configuration.
    #[must_use]
    pub fn settings(&self, glob: GlobSettings) -> Settings {
        Settings { version: self.version, rules: self.rules.clone(), glob }
    }
}
