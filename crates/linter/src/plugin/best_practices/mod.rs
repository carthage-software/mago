use crate::definition::PluginDefinition;
use crate::plugin::best_practices::rules::combine_consecutive_issets::CombineConsecutiveIssetsRule;
use crate::plugin::best_practices::rules::disallowed_functions::DisallowedFunctionsRule;
use crate::plugin::best_practices::rules::dont_catch_error::DontCatchErrorRule;
use crate::plugin::best_practices::rules::excessive_nesting::ExcessiveNesting;
use crate::plugin::best_practices::rules::literal_named_argument::LiteralNamedArgumentRule;
use crate::plugin::best_practices::rules::loop_does_not_iterate::LoopDoesNotIterateRule;
use crate::plugin::best_practices::rules::no_boolean_flag_parameter::NoBooleanFlagParameterRule;
use crate::plugin::best_practices::rules::no_boolean_literal_comparison::NoBooleanLiteralComparisonRule;
use crate::plugin::best_practices::rules::no_debug_symbols::NoDebugSymbolsRule;
use crate::plugin::best_practices::rules::no_else_clause::NoElseClauseRule;
use crate::plugin::best_practices::rules::no_empty_catch_clause::NoEmptyCatchClauseRule;
use crate::plugin::best_practices::rules::no_empty_loop::NoEmptyLoopRule;
use crate::plugin::best_practices::rules::no_goto::NoGotoRule;
use crate::plugin::best_practices::rules::no_hash_emoji::NoHashEmojiRule;
use crate::plugin::best_practices::rules::no_multi_assignments::NoMultiAssignmentsRule;
use crate::plugin::best_practices::rules::no_sprintf_concatenation::NoSprintfConcatenationRule;
use crate::plugin::best_practices::rules::no_unused_parameter::NoUnusedParameterRule;
use crate::plugin::best_practices::rules::override_attribute::OverrideAttributeRule;
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
            Box::new(DontCatchErrorRule),
            Box::new(NoSprintfConcatenationRule),
            Box::new(NoUnusedParameterRule),
            Box::new(ExcessiveNesting),
            Box::new(LiteralNamedArgumentRule),
            Box::new(LoopDoesNotIterateRule),
            Box::new(NoBooleanFlagParameterRule),
            Box::new(NoBooleanLiteralComparisonRule),
            Box::new(NoGotoRule),
            Box::new(NoHashEmojiRule),
            Box::new(NoDebugSymbolsRule),
            Box::new(NoElseClauseRule),
            Box::new(NoEmptyCatchClauseRule),
            Box::new(NoMultiAssignmentsRule),
            Box::new(NoEmptyLoopRule),
            Box::new(OverrideAttributeRule),
            Box::new(UseWhileInsteadOfForRule),
        ]
    }
}
