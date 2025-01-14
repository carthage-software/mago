use ahash::HashMap;
use serde::Serialize;
use toml::Value;

use mago_casing;
use mago_reporting::Level;

/// Represents a single configuration option for a linting rule, including its name, type,
/// a short description of how it affects the rule, and its default value.
#[derive(PartialEq, Clone, Debug, Serialize)]
pub struct RuleOptionDefinition {
    /// A short identifier for the option, such as `"Threshold"` or `"Syntax"`.
    pub name: &'static str,
    /// A user-facing string that indicates what kind of value is expected (e.g. `"integer"`).
    pub r#type: &'static str,
    /// A brief explanation of the option’s purpose and impact.
    pub description: &'static str,
    /// The default value for this option.
    pub default: Value,
}

/// Represents a usage example (valid or invalid) for a linting rule, including a description,
/// a code snippet, an optional set of configuration overrides, and a flag indicating whether
/// the snippet is expected to pass (`valid`) or fail (`invalid`) the rule.
#[derive(PartialEq, Clone, Debug, Serialize)]
pub struct RuleUsageExample {
    /// Indicates whether this snippet should pass or fail.
    pub valid: bool,
    /// A brief explanation of why this snippet is considered valid or invalid.
    pub description: &'static str,
    /// The code snippet to demonstrate the rule in practice (e.g., PHP code).
    pub snippet: &'static str,
    /// A map of configuration overrides applied to the rule for this snippet.
    pub options: HashMap<&'static str, Value>,
}

/// Contains all the defining characteristics of a linting rule, including
/// its name, default severity level, description, recognized configuration options,
/// and code examples showing valid/invalid usage.
#[derive(PartialEq, Clone, Debug, Serialize)]
pub struct RuleDefinition {
    /// The short name of the rule, used in slugs and references.
    pub name: &'static str,
    /// The default severity level for this rule, if enabled.
    pub level: Option<Level>,
    /// A human-readable summary of the rule’s purpose and usage.
    pub description: &'static str,
    /// A list of configuration options recognized by this rule.
    pub options: Vec<RuleOptionDefinition>,
    /// A list of usage examples demonstrating valid or invalid code snippets.
    pub examples: Vec<RuleUsageExample>,
}

/// Holds high-level information about a plugin. Plugins generally bundle multiple rules together,
/// and can be selectively enabled or disabled. This struct describes the plugin's identity,
/// a short explanation of its purpose, and whether it is enabled by default.
#[derive(PartialEq, Clone, Debug, Serialize)]
pub struct PluginDefinition {
    /// The short name of the plugin, used in slugs and references.
    pub name: &'static str,
    /// A brief description of the plugin’s purpose or the rules it contains.
    pub description: &'static str,
    /// Indicates whether this plugin should be considered enabled if not otherwise specified.
    pub enabled_by_default: bool,
}

impl RuleUsageExample {
    /// Creates a new **valid** usage example with no special configuration options.
    ///
    /// # Parameters
    ///
    /// * `description` - Explains why this snippet is valid.
    /// * `snippet` - The actual code snippet demonstrating the rule in a passing scenario.
    ///
    /// # Returns
    ///
    /// A `RuleUsageExample` marked as valid, with an empty `options` map.
    pub fn valid(description: &'static str, snippet: &'static str) -> Self {
        Self { valid: true, description, snippet, options: HashMap::default() }
    }

    /// Creates a new **invalid** usage example with no special configuration options.
    ///
    /// # Parameters
    ///
    /// * `description` - Explains why this snippet is invalid.
    /// * `snippet` - The code snippet demonstrating the rule in a failing scenario.
    ///
    /// # Returns
    ///
    /// A `RuleUsageExample` marked as invalid, with an empty `options` map.
    pub fn invalid(description: &'static str, snippet: &'static str) -> Self {
        Self { valid: false, description, snippet, options: HashMap::default() }
    }

    /// Adds or updates a single `(key, value)` pair in this example’s `options` map,
    /// returning the updated `RuleUsageExample`.
    ///
    /// # Parameters
    ///
    /// * `key` - The configuration option key to set.
    /// * `value` - A `toml::Value` representing the new or updated configuration value.
    ///
    /// # Returns
    ///
    /// An updated `RuleUsageExample` with the new/modified option in its `options` map.
    pub fn with_option(mut self, key: &'static str, value: Value) -> Self {
        self.options.insert(key, value);
        self
    }
}

impl RuleDefinition {
    /// Constructs a `RuleDefinition` for a rule that is enabled by default at a specific severity.
    ///
    /// # Parameters
    ///
    /// * `name` - The short name of the rule (e.g., `"excessive-nesting"`).
    /// * `level` - The default severity level for this rule (e.g., `Level::Warning`).
    ///
    /// # Returns
    ///
    /// A `RuleDefinition` initialized with the given name, level, and no description, options, or examples.
    pub fn enabled(name: &'static str, level: Level) -> Self {
        Self { name, level: Some(level), description: "", options: Vec::new(), examples: Vec::new() }
    }

    /// Constructs a `RuleDefinition` for a rule that is disabled by default.
    ///
    /// # Parameters
    ///
    /// * `name` - The short name of the rule (e.g., `"some-rule"`).
    ///
    /// # Returns
    ///
    /// A `RuleDefinition` initialized with the given name and no severity level (`None`),
    /// plus empty description, options, and examples.
    pub fn disabled(name: &'static str) -> Self {
        Self { name, level: None, description: "", options: Vec::new(), examples: Vec::new() }
    }

    /// Sets or updates the description of this rule, returning the modified `RuleDefinition`.
    ///
    /// # Parameters
    ///
    /// * `description` - A human-readable summary explaining the rule’s purpose and usage.
    ///
    /// # Returns
    ///
    /// A modified `RuleDefinition` with the new description.
    pub fn with_description(mut self, description: &'static str) -> Self {
        self.description = description;
        self
    }

    /// Adds a single `RuleOptionDefinition` to this rule’s list of supported configuration options.
    ///
    /// # Parameters
    ///
    /// * `option` - The option definition to add (including name, type, description, default).
    ///
    /// # Returns
    ///
    /// A modified `RuleDefinition` with the new option appended.
    pub fn with_option(mut self, option: RuleOptionDefinition) -> Self {
        self.options.push(option);
        self
    }

    /// Appends one usage example to this rule’s list of examples, returning the modified `RuleDefinition`.
    ///
    /// # Parameters
    ///
    /// * `example` - A `RuleUsageExample` describing a valid or invalid scenario for this rule.
    ///
    /// # Returns
    ///
    /// A modified `RuleDefinition` containing the new example.
    pub fn with_example(mut self, example: RuleUsageExample) -> Self {
        self.examples.push(example);
        self
    }

    /// Generates a slug (kebab-case string) from the rule's `name`. For instance,
    /// `"Excessive Nesting"` becomes `"excessive-nesting"`.
    ///
    /// # Returns
    ///
    /// A `String` containing the kebab-cased version of the rule name, suitable for referencing
    /// in CLI commands, config files, or external documentation.
    pub fn get_slug(&self) -> String {
        mago_casing::to_kebab_case(self.name)
    }
}

impl PluginDefinition {
    /// Produces a slug (kebab-case string) from the plugin’s `name`.
    /// Useful for referencing the plugin in config files or CLI usage.
    ///
    /// # Returns
    ///
    /// A `String` containing the plugin's name in kebab-case.
    pub fn get_slug(&self) -> String {
        mago_casing::to_kebab_case(self.name)
    }
}
