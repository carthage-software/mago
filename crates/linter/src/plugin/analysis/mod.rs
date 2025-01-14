use crate::definition::PluginDefinition;
use crate::plugin::analysis::rules::inheritance::InheritanceRule;
use crate::plugin::analysis::rules::instantiation::InstantiationRule;
use crate::plugin::analysis::rules::undefined_constant::UndefinedConstantRule;
use crate::plugin::analysis::rules::undefined_function::UndefinedFunctionRule;

use crate::plugin::Plugin;
use crate::rule::Rule;

pub mod rules;

#[derive(Debug)]
pub struct AnalysisPlugin;

impl Plugin for AnalysisPlugin {
    fn get_definition(&self) -> PluginDefinition {
        PluginDefinition {
            name: "Analysis",
            description: "Provides rules that analyze the codebase for potential runtime issues.",
            enabled_by_default: true,
        }
    }

    fn get_rules(&self) -> Vec<Box<dyn Rule>> {
        vec![
            Box::new(InheritanceRule),
            Box::new(InstantiationRule),
            Box::new(UndefinedConstantRule),
            Box::new(UndefinedFunctionRule),
        ]
    }
}
