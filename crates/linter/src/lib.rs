use std::sync::Arc;
use std::sync::RwLock;

use mago_interner::ThreadedInterner;
use mago_reflection::CodebaseReflection;
use mago_reporting::IssueCollection;
use mago_semantics::Semantics;

use crate::context::Context;
use crate::plugin::Plugin;
use crate::rule::ConfiguredRule;
use crate::rule::Rule;
use crate::settings::RuleSettings;
use crate::settings::Settings;

pub mod consts;
pub mod context;
pub mod plugin;
pub mod rule;
pub mod settings;

#[derive(Debug, Clone)]
pub struct Linter {
    settings: Settings,
    interner: ThreadedInterner,
    codebase: Arc<CodebaseReflection>,
    rules: Arc<RwLock<Vec<ConfiguredRule>>>,
}

impl Linter {
    /// Creates a new linter.
    ///
    /// This method will create a new linter with the given settings and interner.
    ///
    /// # Parameters
    ///
    /// - `settings`: The settings to use for the linter.
    /// - `interner`: The interner to use for the linter, usually the same one used by the parser, and the semantics.
    /// - `codebase`: The codebase reflection to use for the linter.
    ///
    /// # Returns
    ///
    /// A new linter.
    pub fn new(settings: Settings, interner: ThreadedInterner, codebase: CodebaseReflection) -> Self {
        Self { settings, interner, codebase: Arc::new(codebase), rules: Arc::new(RwLock::new(Vec::new())) }
    }

    /// Creates a new linter with all plugins enabled.
    ///
    /// This method will create a new linter with all plugins enabled. This is useful for
    /// when you want to lint a source with all available rules.
    ///
    /// # Parameters
    ///
    /// - `settings`: The settings to use for the linter.
    /// - `interner`: The interner to use for the linter, usually the same one used by the parser, and the semantics.
    /// - `codebase`: The codebase reflection to use for the linter.
    ///
    /// # Returns
    ///
    /// A new linter with all plugins enabled.
    pub fn with_all_plugins(settings: Settings, interner: ThreadedInterner, codebase: CodebaseReflection) -> Self {
        let mut linter = Self::new(settings, interner, codebase);

        crate::foreach_plugin!(|plugin| linter.add_plugin(plugin));

        linter
    }

    /// Adds a plugin to the linter.
    ///
    /// This method will add a plugin to the linter. The plugin will be enabled if it is enabled in the settings.
    /// If the plugin is not enabled in the settings, it will only be enabled if it is a default plugin.
    ///
    /// # Parameters
    ///
    /// - `plugin`: The plugin to add to the linter.
    pub fn add_plugin(&mut self, plugin: impl Plugin) {
        let name = plugin.get_name();

        tracing::debug!("Adding plugin `{name}`...");

        let enabled = self.settings.plugins.iter().any(|p| p.eq(name));
        if !enabled {
            if self.settings.default_plugins && plugin.is_enabled_by_default() {
                tracing::debug!("Enabling default plugin `{name}`.");
            } else {
                tracing::debug!(
                    "Plugin `{name}` is not enabled in the configuration and is not a default plugin. Skipping."
                );

                return;
            }
        } else {
            tracing::debug!("Enabling plugin `{name}`.");
        }

        for rule in plugin.get_rules() {
            self.add_rule(name, rule);
        }
    }

    /// Adds a rule to the linter.
    ///
    /// This method will add a rule to the linter. The rule will be enabled if it is enabled in the settings.
    ///
    /// # Parameters
    ///
    /// - `plugin`: The name of the plugin that the rule belongs to.
    /// - `rule`: The rule to add to the linter.
    pub fn add_rule(&mut self, plugin: impl Into<String>, rule: Box<dyn Rule>) {
        let plugin = plugin.into();
        let rule_name = rule.get_name();
        let full_name = format!("{}/{}", plugin, rule_name);

        tracing::debug!("Adding rule `{full_name}`...");

        let settings = self.settings.get_rule_settings(full_name.as_str()).cloned().unwrap_or_else(|| {
            tracing::debug!("No configuration found for rule `{full_name}`, using default.");

            RuleSettings::from_level(rule.get_default_level())
        });

        if !settings.enabled {
            tracing::debug!("Rule `{full_name}` is configured to be off. Skipping.");

            return;
        }

        let level = match settings.level {
            Some(level) => level,
            None => match rule.get_default_level() {
                Some(level) => level,
                None => {
                    tracing::debug!("Rule `{full_name}` does not have a default level. Skipping.");

                    return;
                }
            },
        };

        tracing::debug!("Enabling rule `{full_name}` with level `{level:?}`.");

        self.rules.write().expect("Unable to add rule: poisoned lock").push(ConfiguredRule {
            level,
            settings,
            plugin,
            rule,
        });
    }

    /// Lints the given semantics.
    ///
    /// This method will lint the given semantics and return a collection of issues.
    ///
    /// # Parameters
    ///
    /// - `semantics`: The semantics to lint.
    ///
    /// # Returns
    ///
    /// A collection of issues.
    pub fn lint(&self, semantics: &Semantics) -> IssueCollection {
        let source_name = self.interner.lookup(&semantics.source.identifier.value());

        tracing::debug!("Linting source `{}`...", source_name);

        let mut context = Context::new(&self.interner, &self.codebase, semantics);

        let configured_rules = self.rules.read().expect("Unable to read rules: poisoned lock");

        tracing::debug!("Linting source `{}` with {} rules...", source_name, configured_rules.len());

        for configured_rule in configured_rules.iter() {
            tracing::trace!("Running rule `{}`...", configured_rule.rule.get_name());

            let mut lint_context = context.for_rule(configured_rule);

            configured_rule.rule.as_ref().lint(&semantics.program, &mut lint_context);
        }

        context.take_issue_collection()
    }
}
