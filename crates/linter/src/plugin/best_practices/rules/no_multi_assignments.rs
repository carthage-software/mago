use indoc::indoc;

use mago_reporting::*;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::directive::LintDirective;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct NoMultiAssignmentsRule;

impl Rule for NoMultiAssignmentsRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("No Multi Assignments", Level::Warning).with_description(indoc! {"
            Flags any instances of multiple assignments in a single statement. This can lead to confusion
            and unexpected behavior, and is generally considered poor practice.
        "})
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::Assignment(assignment) = node else { return LintDirective::default() };

        let Expression::Assignment(other_assignment) = assignment.rhs.as_ref() else {
            return LintDirective::default();
        };

        let code = context.lookup(&context.source.content);
        let a = &code[assignment.lhs.span().to_range()];
        let b = &code[other_assignment.lhs.span().to_range()];
        let c = &code[other_assignment.rhs.span().to_range()];

        context.report(
            Issue::new(context.level(), "Avoid using multiple assignments in a single statement.")
                .with_annotation(
                    Annotation::primary(assignment.span())
                        .with_message("Consider splitting this statement into multiple assignments."),
                )
                .with_note(
                    "Multiple assignments in a single statement can be confusing and lead to unexpected behavior.",
                )
                .with_help(format!("Did you mean `{a} = ({b} == {c})` instead? Ensure the intended logic is clear.")),
        );

        LintDirective::default()
    }
}
