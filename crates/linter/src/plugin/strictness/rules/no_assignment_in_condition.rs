use indoc::indoc;

use mago_ast::ast::*;
use mago_ast_utils::assignment::get_assignment_from_expression;
use mago_reporting::*;
use mago_span::*;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
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
}

impl NoAssignmentInConditionRule {
    fn report<'ast>(&self, condition: &'ast Expression, assignment: &'ast Assignment, context: &mut LintContext) {
        let mut issue = Issue::new(context.level(), "Avoid assignments in conditions.")
            .with_annotation(Annotation::primary(assignment.span()).with_message("This is an assignment."))
            .with_annotation(Annotation::secondary(condition.span()).with_message("This is the condition."))
            .with_note("Assigning a value within a condition can lead to unexpected behavior and make the code harder to read and understand.");

        if matches!(&assignment.operator, AssignmentOperator::Assign(_)) {
            issue = issue.with_note("It's easy to confuse assignment (`=`) with comparison (`==`) in this context. ensure you're using the correct operator.");
        }

        context.report(issue);
    }
}

impl<'a> Walker<LintContext<'a>> for NoAssignmentInConditionRule {
    fn walk_in_if<'ast>(&self, r#if: &'ast If, context: &mut LintContext<'a>) {
        if let Some(assignment) = get_assignment_from_expression(&r#if.condition) {
            self.report(&r#if.condition, assignment, context);
        }
    }

    fn walk_in_if_statement_body_else_if_clause<'ast>(
        &self,
        if_statement_body_else_if_clause: &'ast IfStatementBodyElseIfClause,
        context: &mut LintContext<'a>,
    ) {
        if let Some(assignment) = get_assignment_from_expression(&if_statement_body_else_if_clause.condition) {
            self.report(&if_statement_body_else_if_clause.condition, assignment, context);
        }
    }

    fn walk_in_if_colon_delimited_body_else_if_clause<'ast>(
        &self,
        if_colon_delimited_body_else_if_clause: &'ast IfColonDelimitedBodyElseIfClause,
        context: &mut LintContext<'a>,
    ) {
        if let Some(assignment) = get_assignment_from_expression(&if_colon_delimited_body_else_if_clause.condition) {
            self.report(&if_colon_delimited_body_else_if_clause.condition, assignment, context);
        }
    }

    fn walk_in_while<'ast>(&self, r#while: &'ast While, context: &mut LintContext<'a>) {
        if let Some(assignment) = get_assignment_from_expression(&r#while.condition) {
            self.report(&r#while.condition, assignment, context);
        }
    }

    fn walk_in_do_while<'ast>(&self, do_while: &'ast DoWhile, context: &mut LintContext<'a>) {
        if let Some(assignment) = get_assignment_from_expression(&do_while.condition) {
            self.report(&do_while.condition, assignment, context);
        }
    }
}
