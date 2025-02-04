use indoc::indoc;

use mago_ast::*;
use mago_fixer::FixPlan;
use mago_fixer::SafetyClassification;
use mago_reporting::*;
use mago_source::*;
use mago_span::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::directive::LintDirective;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct NoTrailingWhitespaceRule;

impl Rule for NoTrailingWhitespaceRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("No Trailing Whitespace", Level::Note).with_description(indoc! {"
            Detects trailing whitespace at the end of comments. Trailing whitespace can cause unnecessary
            diffs and formatting issues, so it is recommended to remove it.
        "})
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::Program(program) = node else { return LintDirective::Abort };

        let mut issues = vec![];
        for trivia in program.trivia.iter() {
            if !trivia.kind.is_comment() {
                continue;
            }

            let comment_span = trivia.span();
            let value = context.lookup(&trivia.value);
            let lines = value.lines().collect::<Vec<_>>();

            let mut offset = 0;
            for line in lines.iter() {
                let trimmed = line.trim_end();
                let trimmed_length = trimmed.len();
                let trailing_whitespace_length = line.len() - trimmed_length;
                if trailing_whitespace_length > 0 {
                    let whitespace_start = offset + trimmed_length;

                    let whitespace_span = Span::new(
                        comment_span.start.forward(whitespace_start),
                        comment_span.start.forward(whitespace_start + trailing_whitespace_length),
                    );

                    issues.push(
                        Issue::new(context.level(), "Trailing whitespace detected in comment.")
                            .with_annotations([
                                Annotation::primary(whitespace_span).with_message("Trailing whitespace detected."),
                                Annotation::secondary(comment_span).with_message("Comment with trailing whitespace."),
                            ])
                            .with_note("Trailing whitespaces can cause unnecessary diffs and formatting issues.")
                            .with_help("Remove the extra whitespace.")
                            .with_suggestion(whitespace_span.source(), {
                                let mut plan = FixPlan::new();

                                plan.delete(whitespace_span.to_range(), SafetyClassification::Safe);
                                plan
                            }),
                    );
                }

                offset += line.len() + 1;
            }
        }

        for issue in issues {
            context.report(issue);
        }

        LintDirective::Abort
    }
}
