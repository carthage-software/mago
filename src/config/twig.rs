use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::config::twig_formatter::TwigFormatterConfiguration;

/// Settings for the standalone Twig tools.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct TwigConfiguration {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formatter: Option<TwigFormatterConfiguration>,
}
