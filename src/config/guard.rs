use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use mago_guard::settings::Settings;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct GuardConfiguration {
    /// A list of patterns to exclude from guard checking.
    pub excludes: Vec<String>,

    /// Path to a baseline file to ignore listed issues.
    pub baseline: Option<PathBuf>,

    /// Guard settings including rules, layers, and layering.
    #[serde(flatten)]
    pub settings: Settings,
}
