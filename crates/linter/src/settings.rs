use ahash::HashMap;
use serde::Deserialize;
use serde::Serialize;
use toml::value::Value;

use mago_reporting::Level;

/// `Settings` is a struct that holds all the configuration options for the linter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    /// Indicates whether all plugins that mark themselves as "default" are automatically enabled,
    /// in addition to those explicitly listed in [`plugins`].
    pub default_plugins: bool,

    /// A list of plugin slugs (e.g., `"analysis"`, `"migration"`) that are explicitly enabled.
    ///
    /// If `default_plugins` is `true`, those “default” plugins also become enabled,
    /// even if not listed here.
    pub plugins: Vec<String>,

    /// A map of `rule_slug -> RuleSettings`, where `rule_slug` might look like `"analysis/instantiation"`.
    ///
    /// This allows fine-grained control (e.g. enabling or disabling a particular rule, or overriding
    /// its default level) on a rule-by-rule basis.
    pub rules: HashMap<String, RuleSettings>,
}

/// Specifies how a single rule is configured in user settings, such as whether it’s enabled,
/// which severity level applies, and any custom options specific to that rule.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RuleSettings {
    /// If `false`, the rule is disabled entirely.
    pub enabled: bool,

    /// The severity level set by the user (e.g., `Level::Error` or `Level::Warning`).
    ///
    /// If `None`, the rule uses its default level.
    pub level: Option<Level>,

    /// A map of additional **rule-specific** configuration options (e.g., a threshold or a
    /// boolean toggle), stored as a [`toml::Value`]. If none are needed, this may be empty.
    pub options: HashMap<String, Value>,
}

impl Settings {
    pub fn new() -> Self {
        Self { default_plugins: true, plugins: Vec::new(), rules: HashMap::default() }
    }

    pub fn get_rule_settings(&self, rule_name: &str) -> Option<&RuleSettings> {
        self.rules.get(rule_name)
    }

    pub fn with_default_plugins(mut self, default_plugins: bool) -> Self {
        self.default_plugins = default_plugins;
        self
    }

    pub fn with_plugins(mut self, plugins: Vec<String>) -> Self {
        self.plugins = plugins;
        self
    }

    pub fn with_rules(mut self, rules: HashMap<String, RuleSettings>) -> Self {
        self.rules = rules;
        self
    }

    pub fn with_rule(mut self, rule: impl Into<String>, settings: RuleSettings) -> Self {
        self.rules.insert(rule.into(), settings);
        self
    }
}

impl RuleSettings {
    pub fn enabled() -> Self {
        Self { enabled: true, level: None, options: Default::default() }
    }

    pub fn disabled() -> Self {
        Self { enabled: false, level: None, options: Default::default() }
    }

    pub fn from_level(level: Option<Level>) -> Self {
        Self { enabled: true, level, options: Default::default() }
    }

    pub fn with_options(mut self, options: HashMap<String, Value>) -> Self {
        self.options = options;

        self
    }

    pub fn is_enabled(&self) -> bool {
        self.level.is_some()
    }

    pub fn get_option<'c>(&'c self, option_name: &str) -> Option<&'c Value> {
        self.options.get(option_name)
    }
}

impl std::default::Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}
