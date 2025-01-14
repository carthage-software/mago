use crate::definition::PluginDefinition;
use crate::plugin::symfony::rules::quality::interface_should_be_used::InterfaceShouldBeUsed;

use crate::plugin::Plugin;
use crate::rule::Rule;

pub mod rules;

#[derive(Debug)]
pub struct SymfonyPlugin;

impl Plugin for SymfonyPlugin {
    fn get_definition(&self) -> PluginDefinition {
        PluginDefinition {
            name: "Symfony",
            description: "Provides rules that enforce best practices for Symfony applications.",
            enabled_by_default: false,
        }
    }

    fn get_rules(&self) -> Vec<Box<dyn Rule>> {
        vec![Box::new(InterfaceShouldBeUsed)]
    }
}
