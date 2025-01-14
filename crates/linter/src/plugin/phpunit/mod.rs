use crate::definition::PluginDefinition;
use crate::plugin::phpunit::rules::consistency::assertions_style::AssertionsStyleRule;
use crate::plugin::phpunit::rules::strictness::strict_assertions::StrictAssertionsRule;

use crate::plugin::Plugin;
use crate::rule::Rule;

pub mod rules;

#[derive(Debug)]
pub struct PHPUnitPlugin;

impl Plugin for PHPUnitPlugin {
    fn get_definition(&self) -> PluginDefinition {
        PluginDefinition {
            name: "PHPUnit",
            description: "Provides rules that enforce best practices for PHPUnit tests.",
            enabled_by_default: true,
        }
    }

    fn get_rules(&self) -> Vec<Box<dyn Rule>> {
        vec![Box::new(AssertionsStyleRule), Box::new(StrictAssertionsRule)]
    }
}
