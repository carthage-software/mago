use std::fmt::Debug;

use crate::definition::PluginDefinition;
use crate::rule::Rule;

pub mod best_practices;
pub mod comment;
pub mod consistency;
pub mod deprecation;
pub mod laravel;
pub mod maintainability;
pub mod migration;
pub mod naming;
pub mod phpunit;
pub mod psl;
pub mod redundancy;
pub mod safety;
pub mod security;
pub mod strictness;
pub mod symfony;

#[macro_export]
macro_rules! foreach_plugin {
    ($do:expr) => {
        $do($crate::plugin::best_practices::BestPracticesPlugin);
        $do($crate::plugin::comment::CommentPlugin);
        $do($crate::plugin::consistency::ConsistencyPlugin);
        $do($crate::plugin::deprecation::DeprecationPlugin);
        $do($crate::plugin::laravel::LaravelPlugin);
        $do($crate::plugin::maintainability::MaintainabilityPlugin);
        $do($crate::plugin::migration::MigrationPlugin);
        $do($crate::plugin::naming::NamingPlugin);
        $do($crate::plugin::phpunit::PHPUnitPlugin);
        $do($crate::plugin::psl::PslPlugin);
        $do($crate::plugin::redundancy::RedundancyPlugin);
        $do($crate::plugin::safety::SafetyPlugin);
        $do($crate::plugin::security::SecurityPlugin);
        $do($crate::plugin::strictness::StrictnessPlugin);
        $do($crate::plugin::symfony::SymfonyPlugin);
    };
}

/// Represents a linter plugin.
///
/// A plugin is a collection of rules that are applied to the codebase.
pub trait Plugin: Send + Sync + Debug {
    /// Retrieves the definition of this plugin.
    ///
    /// # Returns
    ///
    /// A [`PluginDefinition`] object representing the plugin.
    fn get_definition(&self) -> PluginDefinition;

    /// Retrieves the list of rules associated with this plugin.
    ///
    /// # Returns
    ///
    /// A vector of boxed [`Rule`] trait objects representing the rules to be applied.
    fn get_rules(&self) -> Vec<Box<dyn Rule>>;
}

impl<T> Plugin for Box<T>
where
    T: Plugin,
{
    fn get_definition(&self) -> PluginDefinition {
        self.as_ref().get_definition()
    }

    fn get_rules(&self) -> Vec<Box<dyn Rule>> {
        self.as_ref().get_rules()
    }
}
