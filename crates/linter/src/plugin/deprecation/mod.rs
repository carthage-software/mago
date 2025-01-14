use crate::definition::PluginDefinition;
use crate::plugin::deprecation::rules::php80::optional_parameter_before_required::OptionalParameterBeforeRequiredRule;
use crate::plugin::deprecation::rules::php82::return_by_reference_from_void_function::ReturnByReferenceFromVoidFunctionRule;
use crate::plugin::deprecation::rules::php84::implicitly_nullable_parameter::ImplicitlyNullableParameterRule;
use crate::plugin::deprecation::rules::php84::underscore_classname::UnderscoreClassNameRule;

use crate::plugin::Plugin;
use crate::rule::Rule;

pub mod rules;

#[derive(Debug)]
pub struct DeprecationPlugin;

impl Plugin for DeprecationPlugin {
    fn get_definition(&self) -> PluginDefinition {
        PluginDefinition {
            name: "Deprecation",
            description: "Provides rules that detect deprecated features in PHP code.",
            enabled_by_default: true,
        }
    }

    fn get_rules(&self) -> Vec<Box<dyn Rule>> {
        vec![
            // PHP 8.0
            Box::new(OptionalParameterBeforeRequiredRule),
            // PHP 8.2
            Box::new(ReturnByReferenceFromVoidFunctionRule),
            // PHP 8.4
            Box::new(ImplicitlyNullableParameterRule),
            Box::new(UnderscoreClassNameRule),
        ]
    }
}
