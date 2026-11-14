use mago_twig_formatter::settings::FormatSettings;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

/// Settings used only by `mago twig:fmt`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct TwigFormatterConfiguration {
    pub enabled: bool,
    pub excludes: Vec<String>,
    #[serde(flatten)]
    pub settings: FormatSettings,
}

impl Default for TwigFormatterConfiguration {
    fn default() -> Self {
        Self { enabled: true, excludes: Vec::new(), settings: FormatSettings::default() }
    }
}
