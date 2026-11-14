use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::config::twig_formatter::TwigFormatterConfiguration;
use crate::config::twig_linter::TwigLinterConfiguration;

/// Settings for the Twig tools. Omitted tools use their own defaults.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct TwigConfiguration {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formatter: Option<TwigFormatterConfiguration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linter: Option<TwigLinterConfiguration>,
}
