use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_formatter::settings::*;

/// Configuration options for formatting source code.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct FormatterConfiguration {
    /// A list of patterns to exclude from formatting.
    ///
    /// Defaults to `[]`.
    pub excludes: Vec<String>,

    #[serde(flatten)]
    pub settings: FormatSettings,
}
