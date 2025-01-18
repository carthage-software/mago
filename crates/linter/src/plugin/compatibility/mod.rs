use crate::definition::PluginDefinition;
use crate::plugin::compatibility::rules::php74::arrow_functions_feature::ArrowFunctionsFeatureRule;
use crate::plugin::compatibility::rules::php74::null_coalesce_assignment_feature::NullCoalesceAssignmentFeatureRule;
use crate::plugin::compatibility::rules::php80::named_arguments_feature::NamedArgumentsFeatureRule;
use crate::plugin::compatibility::rules::php80::promoted_properties_feature::PromotedPropertiesFeatureRule;
use crate::plugin::compatibility::rules::php80::union_type_hint_feature::UnionTypeHintFeatureRule;
use crate::plugin::compatibility::rules::php81::closure_creation_feature::ClosureCreationFeatureRule;
use crate::plugin::compatibility::rules::php84::asymmetric_visibility_feature::AsymmetricVisibilityFeatureRule;

use crate::plugin::Plugin;
use crate::rule::Rule;

pub mod rules;

#[derive(Debug)]
pub struct CompatibilityPlugin;

impl Plugin for CompatibilityPlugin {
    fn get_definition(&self) -> PluginDefinition {
        PluginDefinition {
            name: "Compatibility",
            description: "Provides rules that detect incompatibilities with specific PHP versions.",
            enabled_by_default: true,
        }
    }

    fn get_rules(&self) -> Vec<Box<dyn Rule>> {
        vec![
            // PHP 7.4
            Box::new(ArrowFunctionsFeatureRule),
            Box::new(NullCoalesceAssignmentFeatureRule),
            // PHP 8.0
            Box::new(NamedArgumentsFeatureRule),
            Box::new(PromotedPropertiesFeatureRule),
            Box::new(UnionTypeHintFeatureRule),
            // PHP 8.1
            Box::new(ClosureCreationFeatureRule),
            // PHP 8.4
            Box::new(AsymmetricVisibilityFeatureRule),
        ]
    }
}
