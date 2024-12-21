use crate::plugin::runtime::rules::undefined_function::UndefinedFunctionRule;

use crate::plugin::Plugin;
use crate::rule::Rule;

pub mod rules;

#[derive(Debug)]
pub struct RuntimePlugin;

impl Plugin for RuntimePlugin {
    fn get_name(&self) -> &'static str {
        "runtime"
    }

    fn is_enabled_by_default(&self) -> bool {
        true
    }

    fn get_rules(&self) -> Vec<Box<dyn Rule>> {
        vec![Box::new(UndefinedFunctionRule)]
    }
}
