use crate::definition::PluginDefinition;
use crate::plugin::maintainability::rules::cyclomatic_complexity::CyclomaticComplexityRule;
use crate::plugin::maintainability::rules::excessive_parameter_list::ExcessiveParameterListRule;
use crate::plugin::maintainability::rules::halstead::HalsteadRule;
use crate::plugin::maintainability::rules::kan_defect::KanDefectRule;
use crate::plugin::maintainability::rules::too_many_enum_cases::TooManyEnumCasesRule;
use crate::plugin::maintainability::rules::too_many_methods::TooManyMethodsRule;
use crate::plugin::maintainability::rules::too_many_properties::TooManyPropertiesRule;

use crate::plugin::Plugin;
use crate::rule::Rule;

pub mod rules;

#[derive(Debug)]
pub struct MaintainabilityPlugin;

impl Plugin for MaintainabilityPlugin {
    fn get_definition(&self) -> PluginDefinition {
        PluginDefinition {
            name: "Maintainability",
            description: "Provides rules to ensure the maintainability of the codebase.",
            enabled_by_default: true,
        }
    }

    fn get_rules(&self) -> Vec<Box<dyn Rule>> {
        vec![
            Box::new(CyclomaticComplexityRule),
            Box::new(ExcessiveParameterListRule),
            Box::new(HalsteadRule),
            Box::new(KanDefectRule),
            Box::new(TooManyEnumCasesRule),
            Box::new(TooManyMethodsRule),
            Box::new(TooManyPropertiesRule),
        ]
    }
}
