use crate::definition::PluginDefinition;
use crate::plugin::safety::rules::no_error_control_operator::NoErrorControlOperatorRule;
use crate::plugin::safety::rules::no_eval::NoEvalRule;
use crate::plugin::safety::rules::no_ffi::NoFFIRule;
use crate::plugin::safety::rules::no_global::NoGlobalRule;
use crate::plugin::safety::rules::no_request_variable::NoRequestVariableRule;
use crate::plugin::safety::rules::no_shell_execute_string::NoShellExecuteStringRule;
use crate::plugin::safety::rules::no_unsafe_finally::NoUnsafeFinallyRule;

use crate::plugin::Plugin;
use crate::rule::Rule;

pub mod rules;

#[derive(Debug)]
pub struct SafetyPlugin;

impl Plugin for SafetyPlugin {
    fn get_definition(&self) -> PluginDefinition {
        PluginDefinition {
            name: "Safety",
            description: "Provides rules that enforce best practices for safe code.",
            enabled_by_default: true,
        }
    }

    fn get_rules(&self) -> Vec<Box<dyn Rule>> {
        vec![
            Box::new(NoFFIRule),
            Box::new(NoGlobalRule),
            Box::new(NoRequestVariableRule),
            Box::new(NoShellExecuteStringRule),
            Box::new(NoEvalRule),
            Box::new(NoErrorControlOperatorRule),
            Box::new(NoUnsafeFinallyRule),
        ]
    }
}
