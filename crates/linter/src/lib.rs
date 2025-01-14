use std::sync::Arc;
use std::sync::RwLock;
use std::sync::RwLockReadGuard;

use mago_interner::ThreadedInterner;
use mago_reflection::CodebaseReflection;
use mago_reporting::IssueCollection;
use mago_reporting::Level;
use mago_semantics::Semantics;

use crate::context::Context;
use crate::plugin::Plugin;
use crate::rule::ConfiguredRule;
use crate::rule::Rule;
use crate::settings::RuleSettings;
use crate::settings::Settings;

pub mod consts;
pub mod context;
pub mod definition;
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
        let plugin_definition = plugin.get_definition();
        let plugin_slug = plugin_definition.get_slug();

        tracing::debug!("Adding plugin `{plugin_slug}`...");

        let enabled = self.settings.plugins.iter().any(|p| p.eq(&plugin_slug));
        if !enabled {
            if self.settings.default_plugins && plugin_definition.enabled_by_default {
                tracing::debug!("Enabling default plugin `{plugin_slug}`.");
            } else {
                tracing::debug!(
                    "Plugin `{plugin_slug}` is not enabled in the configuration and is not a default plugin. Skipping."
                );

                return;
            }
        } else {
            tracing::debug!("Enabling plugin `{plugin_slug}`.");
        }

        for rule in plugin.get_rules() {
            self.add_rule(&plugin_slug, rule);
        }
    }

    /// Adds a rule to the linter.
    ///
    /// This method will add a rule to the linter. The rule will be enabled if it is enabled in the settings.
    ///
    /// # Parameters
    ///
    /// - `plugin_slug`: The slug of the plugin that the rule belongs to.
    /// - `rule`: The rule to add to the linter.
    pub fn add_rule(&mut self, plugin_slug: impl Into<String>, rule: Box<dyn Rule>) {
        let rule_definition = rule.get_definition();

        let plugin_slug = plugin_slug.into();
        let slug = format!("{}/{}", plugin_slug, rule_definition.get_slug());

        tracing::debug!("Adding rule `{slug}`...");

        let settings = self.settings.get_rule_settings(slug.as_str()).cloned().unwrap_or_else(|| {
            tracing::debug!("No configuration found for rule `{slug}`, using default.");

            RuleSettings::from_level(rule_definition.level)
        });

        if !settings.enabled {
            tracing::debug!("Rule `{slug}` is configured to be off. Skipping.");

            return;
        }

        let level = match settings.level {
            Some(level) => level,
            None => match rule_definition.level {
                Some(level) => level,
                None => {
                    tracing::debug!("Rule `{slug}` does not have a default level. Skipping.");

                    return;
                }
            },
        };

        tracing::debug!("Enabling rule `{slug}` with level `{level:?}`.");

        self.rules.write().expect("Unable to add rule: poisoned lock").push(ConfiguredRule {
            slug,
            level,
            settings,
            rule,
        });
    }

    /// Returns a read lock for the vector of [`ConfiguredRule`] instances maintained by the linter.
    ///
    /// This method provides direct, read-only access to all currently configured rules.
    /// You can iterate over them or inspect their fields (e.g., `slug`, `level`, etc.).
    ///
    /// # Panics
    ///
    /// If the underlying `RwLock` is poisoned (e.g. another thread panicked while holding
    /// the lock), this method will panic with `"Unable to get rule: poisoned lock"`.
    pub fn get_configured_rules(&self) -> RwLockReadGuard<'_, Vec<ConfiguredRule>> {
        self.rules.read().expect("Unable to get rule: poisoned lock")
    }

    /// Retrieves the **level** of a rule by its fully qualified slug.
    ///
    /// This method looks up a configured rule by its slug (e.g., `"plugin-slug/rule-slug"`)
    /// and returns the rule’s current level (e.g., `Level::Warning`).
    ///
    /// # Parameters
    ///
    /// - `slug`: The fully qualified slug of the rule (e.g. `"best-practices/excessive-nesting"`).
    ///
    /// # Returns
    ///
    /// An [`Option<Level>`]. Returns `Some(Level)` if the slug is found among the currently
    /// configured rules, or `None` if no matching rule is found.
    pub fn get_rule_level(&self, slug: &str) -> Option<Level> {
        let configured_rules = self.rules.read().expect("Unable to get rule: poisoned lock");

        configured_rules.iter().find(|r| r.slug == slug).map(|r| r.level)
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
        if configured_rules.is_empty() {
            tracing::warn!("No rules configured. Skipping linting.");

            return IssueCollection::new();
        }

        tracing::debug!("Linting source `{}` with {} rules...", source_name, configured_rules.len());

        for configured_rule in configured_rules.iter() {
            tracing::trace!("Running rule `{}`...", configured_rule.rule.get_definition().name);

            let mut lint_context = context.for_rule(configured_rule);

            configured_rule.rule.as_ref().lint(&semantics.program, &mut lint_context);
        }

        context.take_issue_collection()
    }
}
