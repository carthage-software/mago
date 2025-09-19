use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use mago_linter::integration::Integration;
use mago_linter::settings::RulesSettings;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct LinterConfiguration {
    /// A list of patterns to exclude from linting.
    pub excludes: Vec<String>,

    /// Integrations to enable during linting.
    pub integrations: Vec<Integration>,

    /// Settings for various linting rules.
    pub rules: RulesSettings,

    /// Path to a baseline file to ignore listed issues.
    pub baseline: Option<PathBuf>,
}
