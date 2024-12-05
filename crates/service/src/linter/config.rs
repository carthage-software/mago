use ahash::HashMap;
use serde::Deserialize;
use serde::Serialize;
use toml::value::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LinterLevel {
    Off,
    Help,
    Note,
    Warning,
    Error,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinterConfiguration {
    pub level: Option<LinterLevel>,
    pub default_plugins: Option<bool>,
    pub plugins: Vec<String>,
    pub rules: Vec<LinterRuleConfiguration>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct LinterRuleConfiguration {
    pub name: String,
    pub level: Option<LinterLevel>,
    #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
    pub options: HashMap<String, Value>,
}
