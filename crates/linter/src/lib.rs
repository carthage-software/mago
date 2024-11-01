use std::sync::Arc;
use std::sync::RwLock;

use fennec_interner::ThreadedInterner;
use fennec_reporting::IssueCollection;
use fennec_semantics::Semantics;

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
    pub settings: Settings,

    interner: ThreadedInterner,
    rules: Arc<RwLock<Vec<ConfiguredRule>>>,
}

impl Linter {
    pub fn new(settings: Settings, interner: ThreadedInterner) -> Self {
        Self { settings, interner, rules: Arc::new(RwLock::new(Vec::new())) }
    }

    pub fn add_plugin(&mut self, plugin: impl Plugin) {
        let name = plugin.get_name();

        tracing::info!("Adding plugin `{name}`...");

        let enabled = self.settings.plugins.iter().any(|p| p.eq(name));
        if !enabled {
            if self.settings.default_plugins && plugin.is_enabled_by_default() {
                tracing::info!("Enabling default plugin `{name}`.");
            } else {
                tracing::debug!(
                    "Plugin `{name}` is not enabled in the configuration and is not a default plugin. Skipping."
                );

                return;
            }
        } else {
            tracing::info!("Enabling plugin `{name}`.");
        }

        for rule in plugin.get_rules() {
            self.add_rule(name, rule);
        }
    }

    pub fn add_rule(&mut self, plugin: impl Into<String>, rule: Box<dyn Rule>) {
        let plugin = plugin.into();
        let rule_name = rule.get_name();
        let full_name = format!("{}/{}", plugin, rule_name);

        tracing::info!("Adding rule `{full_name}`...");

        let settings = self.settings.get_rule_settings(full_name.as_str()).map(|c| c.clone()).unwrap_or_else(|| {
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
                    tracing::warn!("Rule `{full_name}` does not have a default level. Skipping.");

                    return;
                }
            },
        };

        tracing::info!("Enabling rule `{full_name}` with level `{level:?}`.");

        self.rules.write().expect("Unable to add rule: poisoned lock").push(ConfiguredRule {
            level,
            settings,
            plugin: plugin.into(),
            rule,
        });
    }

    pub fn lint<'a>(&self, semantics: &'a Semantics) -> IssueCollection {
        let source_name = self.interner.lookup(semantics.source.identifier.value());

        tracing::debug!("Linting source `{}`...", source_name);

        let mut context = Context::new(&self.interner, &semantics);

        if !self.settings.external && semantics.source.identifier.is_external() {
            tracing::debug!("Skipping linting of external source `{}`.", source_name);

            return context.take_issue_collection();
        }

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
