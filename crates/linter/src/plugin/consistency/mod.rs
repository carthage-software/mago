use crate::definition::PluginDefinition;
use crate::plugin::consistency::rules::array_syntax::ArraySyntaxRule;
use crate::plugin::consistency::rules::lowercase_hint::LowercaseHintRule;
use crate::plugin::consistency::rules::lowercase_keyword::LowercaseKeywordRule;
use crate::plugin::consistency::rules::no_function_aliases::NoFunctionAliasesRule;
use crate::plugin::consistency::rules::no_tag_pair_terminator::NoTagPairTerminatorRule;
use crate::plugin::consistency::rules::require_block_statement_body::RequireBlockStatementBodyRule;

use crate::plugin::Plugin;
use crate::rule::Rule;

pub mod rules;

#[derive(Debug)]
pub struct ConsistencyPlugin;

impl Plugin for ConsistencyPlugin {
    fn get_definition(&self) -> PluginDefinition {
        PluginDefinition {
            name: "Consistency",
            description: "Provides rules that enforce consistent coding standards.",
            enabled_by_default: true,
        }
    }

    fn get_rules(&self) -> Vec<Box<dyn Rule>> {
        vec![
            Box::new(ArraySyntaxRule),
            Box::new(LowercaseHintRule),
            Box::new(LowercaseKeywordRule),
            Box::new(NoFunctionAliasesRule),
            Box::new(NoTagPairTerminatorRule),
            Box::new(RequireBlockStatementBodyRule),
        ]
    }
}
