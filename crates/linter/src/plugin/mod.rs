use std::fmt::Debug;

use crate::rule::Rule;

pub mod analysis;
pub mod best_practices;
pub mod comment;
pub mod consistency;
pub mod deprecation;
pub mod laravel;
pub mod migration;
pub mod naming;
pub mod phpunit;
pub mod redundancy;
pub mod safety;
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
        $do($crate::plugin::migration::MigrationPlugin);
        $do($crate::plugin::naming::NamingPlugin);
        $do($crate::plugin::phpunit::PHPUnitPlugin);
        $do($crate::plugin::redundancy::RedundancyPlugin);
        $do($crate::plugin::analysis::AnalysisPlugin);
        $do($crate::plugin::safety::SafetyPlugin);
        $do($crate::plugin::strictness::StrictnessPlugin);
        $do($crate::plugin::symfony::SymfonyPlugin);
    };
}

/// Represents a linter plugin.
///
/// A plugin is a collection of rules that are applied to the codebase.
pub trait Plugin: Send + Sync + Debug {
    /// Returns the name of the plugin.
    ///
    /// # Returns
    ///
    /// A static string slice representing the name of the plugin.
    ///
    /// This name is used in configurations to enable or disable the entire plugin.
    fn get_name(&self) -> &'static str;

    /// Return wheather this plugin is enabled by default.
    fn is_enabled_by_default(&self) -> bool {
        false
    }

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
    fn get_name(&self) -> &'static str {
        self.as_ref().get_name()
    }

    fn is_enabled_by_default(&self) -> bool {
        self.as_ref().is_enabled_by_default()
    }

    fn get_rules(&self) -> Vec<Box<dyn Rule>> {
        self.as_ref().get_rules()
    }
}
