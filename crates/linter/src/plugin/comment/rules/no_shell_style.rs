use indoc::indoc;

use mago_fixer::SafetyClassification;
use mago_reporting::*;
use mago_span::*;
use mago_syntax::ast::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::directive::LintDirective;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct NoShellStyleRule;

impl Rule for NoShellStyleRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("No Shell Style", Level::Warning).with_description(indoc! {"
            Detects shell-style comments ('#') in PHP code. Double slash comments ('//') are preferred
            in PHP, as they are more consistent with the language's syntax and are easier to read.
        "})
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::Program(program) = node else { return LintDirective::Abort };

        for trivia in program.trivia.iter() {
            if let TriviaKind::HashComment = trivia.kind {
                let comment_span = trivia.span();
                let comment_pos = comment_span.start;

                let issue = Issue::new(context.level(), "Shell-style comments ('#') are not allowed.")
                    .with_annotation(Annotation::primary(comment_span).with_message("This is a shell-style comment."))
                    .with_help("Consider using double slash comments ('//') instead.");

                context.propose(issue, |plan| {
                    plan.replace(comment_pos.range_for(1), "//", SafetyClassification::Safe);
                });
            }
        }

        LintDirective::Abort
    }
}
