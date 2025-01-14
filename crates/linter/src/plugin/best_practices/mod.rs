use crate::definition::PluginDefinition;
use crate::plugin::best_practices::rules::combine_consecutive_issets::CombineConsecutiveIssetsRule;
use crate::plugin::best_practices::rules::disallowed_functions::DisallowedFunctionsRule;
use crate::plugin::best_practices::rules::excessive_nesting::ExcessiveNesting;
use crate::plugin::best_practices::rules::loop_does_not_iterate::LoopDoesNotIterateRule;
use crate::plugin::best_practices::rules::no_debug_symbols::NoDebugSymbolsRule;
use crate::plugin::best_practices::rules::no_empty_loop::NoEmptyLoopRule;
use crate::plugin::best_practices::rules::no_goto::NoGotoRule;
use crate::plugin::best_practices::rules::no_multi_assignments::NoMultiAssignmentsRule;
use crate::plugin::best_practices::rules::no_unused_parameter::NoUnusedParameterRule;
use crate::plugin::best_practices::rules::use_while_instead_of_for::UseWhileInsteadOfForRule;

use crate::plugin::Plugin;
use crate::rule::Rule;

pub mod rules;

#[derive(Debug)]
pub struct BestPracticesPlugin;

impl Plugin for BestPracticesPlugin {
    fn get_definition(&self) -> PluginDefinition {
        PluginDefinition {
            name: "Best Practices",
            description: "Provides rules that enforce best practices and idiomatic PHP.",
            enabled_by_default: true,
        }
    }

    fn get_rules(&self) -> Vec<Box<dyn Rule>> {
        vec![
            Box::new(CombineConsecutiveIssetsRule),
            Box::new(DisallowedFunctionsRule),
            Box::new(NoUnusedParameterRule),
            Box::new(ExcessiveNesting),
            Box::new(LoopDoesNotIterateRule),
            Box::new(NoGotoRule),
            Box::new(NoDebugSymbolsRule),
            Box::new(NoMultiAssignmentsRule),
            Box::new(NoEmptyLoopRule),
            Box::new(UseWhileInsteadOfForRule),
        ]
    }
}
