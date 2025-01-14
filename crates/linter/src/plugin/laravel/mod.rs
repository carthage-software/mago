use crate::definition::PluginDefinition;
use crate::plugin::laravel::rules::safety::no_request_all::NoRequestAllRule;

use crate::plugin::Plugin;
use crate::rule::Rule;

pub mod rules;

#[derive(Debug)]
pub struct LaravelPlugin;

impl Plugin for LaravelPlugin {
    fn get_definition(&self) -> PluginDefinition {
        PluginDefinition {
            name: "Laravel",
            description: "Provides rules that enforce best practices for Laravel applications.",
            enabled_by_default: false,
        }
    }

    fn get_rules(&self) -> Vec<Box<dyn Rule>> {
        vec![Box::new(NoRequestAllRule)]
    }
}
