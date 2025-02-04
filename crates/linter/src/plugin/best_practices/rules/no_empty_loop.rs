use indoc::indoc;

use mago_ast::*;
use mago_fixer::SafetyClassification;
use mago_reporting::*;
use mago_span::HasSpan;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::directive::LintDirective;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct NoEmptyLoopRule;

impl Rule for NoEmptyLoopRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("No Empty Loop", Level::Note).with_description(indoc! {"
            Detects loops (for, foreach, while, do-while) that have an empty body. An empty loop body
            does not perform any actions and is likely a mistake or a sign of redundant code.
        "})
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let empty_loop = match node {
            Node::Foreach(foreach) => match &foreach.body {
                ForeachBody::Statement(stmt) => is_statement_empty(stmt),
                ForeachBody::ColonDelimited(body) => are_statements_empty(body.statements.as_slice()),
            },
            Node::For(for_loop) => match &for_loop.body {
                ForBody::Statement(stmt) => is_statement_empty(stmt),
                ForBody::ColonDelimited(body) => are_statements_empty(body.statements.as_slice()),
            },
            Node::While(while_loop) => match &while_loop.body {
                WhileBody::Statement(stmt) => is_statement_empty(stmt),
                WhileBody::ColonDelimited(body) => are_statements_empty(body.statements.as_slice()),
            },
            Node::DoWhile(do_while) => is_statement_empty(&do_while.statement),
            _ => {
                return LintDirective::default();
            }
        };

        if !empty_loop {
            return LintDirective::default();
        }

        let loop_span = node.span();

        let issue = Issue::new(context.level(), "Loop body is empty")
            .with_annotation(
                Annotation::primary(loop_span)
                    .with_message("This loop body is empty and does not perform any actions."),
            )
            .with_help("Consider removing this loop or adding meaningful logic to its body.");

        context.report_with_fix(issue, |plan| {
            plan.delete(loop_span.to_range(), SafetyClassification::PotentiallyUnsafe);
        });

        LintDirective::default()
    }
}

#[inline]
fn is_statement_empty(statement: &Statement) -> bool {
    match statement {
        Statement::Block(block) => are_statements_empty(block.statements.as_slice()),
        Statement::Noop(_) => true,
        _ => false,
    }
}

#[inline]
fn are_statements_empty(statements: &[Statement]) -> bool {
    statements.is_empty() || statements.iter().all(is_statement_empty)
}
