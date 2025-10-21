//! # Linter Rule Utilities
//!
//! This module provides helper functions for interacting with the linter's rule set,
//! such as retrieving metadata for all available rules.

use mago_linter::registry::RuleRegistry;
use mago_linter::rule::AnyRule;
use mago_linter::rule_meta::RuleMeta;
use mago_linter::settings::Settings;

/// Retrieves metadata for all available linter rules based on the provided settings.
pub fn get_available_rules(settings: &Settings) -> Vec<&'static RuleMeta> {
    let registry = RuleRegistry::build(settings, None, true);

    registry.rules().iter().map(AnyRule::meta).collect()
}
