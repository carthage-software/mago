use crate::definition::PluginDefinition;
use crate::plugin::Plugin;
use crate::plugin::laravel::rules::best_practices::middleware_in_routes::MiddlewareInRoutesRule;
use crate::plugin::laravel::rules::best_practices::view_array_parameter::ViewArrayParameterRule;
use crate::plugin::laravel::rules::safety::no_request_all::NoRequestAllRule;
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
        vec![Box::new(NoRequestAllRule), Box::new(MiddlewareInRoutesRule), Box::new(ViewArrayParameterRule)]
    }
}
