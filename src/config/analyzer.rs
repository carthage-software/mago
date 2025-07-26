use config::ConfigBuilder;
use config::Value;
use config::ValueKind;
use config::builder::BuilderState;
use serde::Deserialize;
use serde::Serialize;

use crate::config::ConfigurationEntry;
use crate::error::Error;

/// Configuration options for formatting source code.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnalyzeConfiguration {
    /// A list of patterns to exclude from formatting.
    ///
    /// Defaults to `[]`.
    pub excludes: Vec<String>,

    /// Whether to find unused expressions.
    ///
    /// Defaults to `false`.
    pub find_unused_expressions: bool,

    /// Whether to find unused definitions.
    ///
    /// Defaults to `false`.
    pub find_unused_definitions: bool,

    /// Whether to find unused variables.
    ///
    /// Defaults to `false`.
    pub find_unused_variables: bool,

    /// Whether to analyze dead code.
    ///
    /// Defaults to `false`.
    pub analyze_dead_code: bool,

    /// Whether to analyze effects.
    ///
    /// Defaults to `false`.
    pub analyze_effects: bool,

    /// Whether to memoize properties.
    ///
    /// Defaults to `true`.
    pub memoize_properties: bool,

    /// Whether to allow the use of `include` construct.
    ///
    /// Defaults to `true`.
    pub allow_include: bool,
}

impl ConfigurationEntry for AnalyzeConfiguration {
    fn configure<St: BuilderState>(self, builder: ConfigBuilder<St>) -> Result<ConfigBuilder<St>, Error> {
        builder
            .set_default("analyze.excludes", Value::new(None, ValueKind::Array(vec![])))?
            .set_default("analyze.find_unused_definitions", Value::new(None, ValueKind::Boolean(false)))?
            .set_default("analyze.find_unused_expressions", Value::new(None, ValueKind::Boolean(false)))?
            .set_default("analyze.find_unused_variables", Value::new(None, ValueKind::Boolean(false)))?
            .set_default("analyze.analyze_dead_code", Value::new(None, ValueKind::Boolean(false)))?
            .set_default("analyze.analyze_effects", Value::new(None, ValueKind::Boolean(false)))?
            .set_default("analyze.memoize_properties", Value::new(None, ValueKind::Boolean(true)))?
            .set_default("analyze.allow_include", Value::new(None, ValueKind::Boolean(true)))
            .map_err(Error::from)
    }
}

impl Default for AnalyzeConfiguration {
    fn default() -> Self {
        Self {
            excludes: vec![],
            find_unused_expressions: false,
            find_unused_definitions: false,
            find_unused_variables: false,
            analyze_dead_code: false,
            analyze_effects: false,
            memoize_properties: true,
            allow_include: true,
        }
    }
}
