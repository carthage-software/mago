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
    fn get_name(&self) -> &'static str {
        "analysis"
    }

    fn is_enabled_by_default(&self) -> bool {
        true
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
