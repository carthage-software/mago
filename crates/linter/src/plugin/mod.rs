use std::fmt::Debug;

use crate::rule::Rule;

pub mod best_practices;
pub mod comment;
pub mod consistency;
pub mod naming;
pub mod phpunit;
pub mod redundancy;
pub mod safety;
pub mod strictness;
pub mod symfony;

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
