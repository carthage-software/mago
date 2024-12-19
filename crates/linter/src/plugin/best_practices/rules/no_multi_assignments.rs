use mago_ast::*;
use mago_reporting::*;
use mago_span::HasSpan;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct NoMultiAssignmentsRule;

impl Rule for NoMultiAssignmentsRule {
    fn get_name(&self) -> &'static str {
        "no-multi-assignments"
    }

    fn get_default_level(&self) -> Option<Level> {
        Some(Level::Warning)
    }
}

impl<'a> Walker<LintContext<'a>> for NoMultiAssignmentsRule {
    fn walk_in_assignment(&self, assignment: &Assignment, context: &mut LintContext<'a>) {
        let Expression::AssignmentOperation(other_assignment) = assignment.rhs.as_ref() else {
            return;
        };

        let code = context.lookup(&context.semantics.source.content);
        let a = &code[assignment.lhs.span().to_range()];
        let b = &code[other_assignment.lhs.span().to_range()];
        let c = &code[other_assignment.rhs.span().to_range()];

        let issue = Issue::new(context.level(), "Avoid using multiple assignments in a single statement.")
            .with_annotation(
                Annotation::primary(assignment.span())
                    .with_message("Consider splitting this statement into multiple assignments."),
            )
            .with_note("Multiple assignments in a single statement can be confusing and lead to unexpected behavior.")
            .with_help(format!("Did you mean `{a} = ({b} === {c})` instead? Ensure the intended logic is clear."));

        context.report(issue);
    }
}
