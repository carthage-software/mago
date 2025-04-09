use indoc::indoc;

use mago_fixer::SafetyClassification;
use mago_reporting::*;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct RedundantContinueRule;

impl Rule for RedundantContinueRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Redundant Continue", Level::Help)
            .with_description(indoc! {"
                Detects redundant `continue` statements in loops.
            "})
            .with_example(RuleUsageExample::invalid(
                "A redundant `continue` statement in a loop",
                indoc! {r#"
                    <?php

                    while (true) {
                        echo "Hello, world!";

                        continue; // Redundant `continue` statement
                    }
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let r#continue = match node {
            Node::Foreach(foreach) => match &foreach.body {
                ForeachBody::Statement(stmt) => get_continue_from_statement(stmt),
                ForeachBody::ColonDelimited(body) => get_continue_from_last_statement(body.statements.as_slice()),
            },
            Node::For(r#for) => match &r#for.body {
                ForBody::Statement(stmt) => get_continue_from_statement(stmt),
                ForBody::ColonDelimited(body) => get_continue_from_last_statement(body.statements.as_slice()),
            },
            Node::While(r#while) => match &r#while.body {
                WhileBody::Statement(stmt) => get_continue_from_statement(stmt),
                WhileBody::ColonDelimited(body) => get_continue_from_last_statement(body.statements.as_slice()),
            },
            Node::DoWhile(do_while) => get_continue_from_statement(&do_while.statement),
            _ => None,
        };

        let Some(r#continue) = r#continue else {
            return LintDirective::default();
        };

        let issue = Issue::new(context.level(), "Redundant continue statement in loop body.")
            .with_annotations([
                Annotation::primary(r#continue.span()).with_message(
                    "This `continue` statement is redundant because it is the last statement in the loop body.",
                ),
                Annotation::secondary(node.span()),
            ])
            .with_help("Remove this `continue` statement, as it does not affect the loop's behavior.");

        context.propose(issue, |plan| {
            plan.delete(r#continue.span().to_range(), SafetyClassification::Safe);
        });

        LintDirective::default()
    }
}

#[inline]
fn get_continue_from_last_statement(statements: &[Statement]) -> Option<&Continue> {
    let last = statements.last()?;

    get_continue_from_statement(last)
}

#[inline]
fn get_continue_from_statement(statement: &Statement) -> Option<&Continue> {
    match statement {
        Statement::Block(block) => get_continue_from_statement(block.statements.last()?),
        Statement::Continue(cont) => match cont.level {
            None | Some(Expression::Literal(Literal::Integer(LiteralInteger { value: 1, .. }))) => Some(cont),
            Some(_) => None,
        },
        _ => None,
    }
}
