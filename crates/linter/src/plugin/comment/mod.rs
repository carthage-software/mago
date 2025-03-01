use crate::definition::PluginDefinition;
use crate::plugin::comment::rules::docblock_syntax::DocblockSyntaxRule;
use crate::plugin::comment::rules::no_empty_comments::NoEmptyCommentsRule;
use crate::plugin::comment::rules::no_shell_style::NoShellStyleRule;
use crate::plugin::comment::rules::no_trailing_whitespace::NoTrailingWhitespaceRule;
use crate::plugin::comment::rules::no_untagged_fixme::NoUntaggedFixmeRule;
use crate::plugin::comment::rules::no_untagged_todo::NoUntaggedTodoRule;
use crate::plugin::comment::rules::use_expect_instead_of_ignore::UseExpectInsteadOfIgnoreRule;

use crate::plugin::Plugin;
use crate::rule::Rule;

pub mod rules;

#[derive(Debug)]
pub struct CommentPlugin;

impl Plugin for CommentPlugin {
    fn get_definition(&self) -> PluginDefinition {
        PluginDefinition {
            name: "Comment",
            description: "Provides rules that enforce best practices for comments.",
            enabled_by_default: true,
        }
    }

    fn get_rules(&self) -> Vec<Box<dyn Rule>> {
        vec![
            Box::new(NoUntaggedTodoRule),
            Box::new(NoEmptyCommentsRule),
            Box::new(NoUntaggedFixmeRule),
            Box::new(NoShellStyleRule),
            Box::new(NoTrailingWhitespaceRule),
            Box::new(DocblockSyntaxRule),
            Box::new(UseExpectInsteadOfIgnoreRule),
        ]
    }
}
