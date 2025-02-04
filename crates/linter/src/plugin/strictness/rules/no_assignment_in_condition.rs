use indoc::indoc;

use mago_ast::*;
use mago_ast_utils::assignment::get_assignment_from_expression;
use mago_reporting::*;
use mago_span::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct NoAssignmentInConditionRule;

impl Rule for NoAssignmentInConditionRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("No Assignment In Condition", Level::Warning)
            .with_description(indoc! {"
                Detects assignments in conditions which can lead to unexpected behavior and make the code harder
                to read and understand.
            "})
            .with_example(RuleUsageExample::invalid(
                "An assignment in a condition",
                indoc! {r#"
                    <?php

                    if ($x = 1) {
                        // ...
                    }
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let (condition, assignment) = match node {
            Node::If(r#if) => (&r#if.condition, get_assignment_from_expression(&r#if.condition)),
            Node::While(r#while) => (&r#while.condition, get_assignment_from_expression(&r#while.condition)),
            Node::DoWhile(do_while) => (&do_while.condition, get_assignment_from_expression(&do_while.condition)),
            Node::IfStatementBodyElseIfClause(if_statement_body_else_if_clause) => (
                &if_statement_body_else_if_clause.condition,
                get_assignment_from_expression(&if_statement_body_else_if_clause.condition),
            ),
            Node::IfColonDelimitedBodyElseIfClause(if_colon_delimited_body_else_if_clause) => (
                &if_colon_delimited_body_else_if_clause.condition,
                get_assignment_from_expression(&if_colon_delimited_body_else_if_clause.condition),
            ),
            _ => return LintDirective::default(),
        };

        let Some(assignment) = assignment else {
            return LintDirective::default();
        };

        let mut issue = Issue::new(context.level(), "Avoid assignments in conditions.")
            .with_annotation(Annotation::primary(assignment.span()).with_message("This is an assignment."))
            .with_annotation(Annotation::secondary(condition.span()).with_message("This is the condition."))
            .with_note("Assigning a value within a condition can lead to unexpected behavior and make the code harder to read and understand.");

        if matches!(&assignment.operator, AssignmentOperator::Assign(_)) {
            issue = issue.with_note("It's easy to confuse assignment (`=`) with comparison (`==`) in this context. ensure you're using the correct operator.");
        }

        context.report(issue);

        LintDirective::default()
    }
}
