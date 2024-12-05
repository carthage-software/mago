use fennec_ast::*;
use fennec_fixer::SafetyClassification;
use fennec_reporting::*;
use fennec_span::HasSpan;
use fennec_walker::Walker;

use crate::context::LintContext;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct RedundantParenthesesRule;

impl RedundantParenthesesRule {
    fn report<'ast>(&self, parenthesized: &'ast Parenthesized, context: &mut LintContext<'_>) {
        let issue = Issue::new(context.level(), "redundant parentheses")
            .with_annotations([
                Annotation::primary(parenthesized.left_parenthesis),
                Annotation::primary(parenthesized.right_parenthesis),
                Annotation::secondary(parenthesized.expression.span())
                    .with_message("expression does not need to be parenthesized"),
            ])
            .with_help("remove the redundant inner parentheses");

        context.report_with_fix(issue, |plan| {
            plan.delete(parenthesized.left_parenthesis.to_range(), SafetyClassification::Safe);
            plan.delete(parenthesized.right_parenthesis.to_range(), SafetyClassification::Safe);
        });
    }
}

impl Rule for RedundantParenthesesRule {
    fn get_name(&self) -> &'static str {
        "redundant-parentheses"
    }

    fn get_default_level(&self) -> Option<Level> {
        Some(Level::Help)
    }
}

impl<'a> Walker<LintContext<'a>> for RedundantParenthesesRule {
    fn walk_in_parenthesized<'ast>(&self, parenthesized: &'ast Parenthesized, context: &mut LintContext<'a>) {
        if let Expression::Parenthesized(inner) = parenthesized.expression.as_ref() {
            self.report(inner, context);
        }
    }

    fn walk_in_assignment<'ast>(&self, assignment: &'ast Assignment, context: &mut LintContext<'a>) {
        if let Expression::Parenthesized(rhs) = assignment.rhs.as_ref() {
            self.report(rhs, context);
        }
    }

    fn walk_in_positional_argument<'ast>(
        &self,
        positional_argument: &'ast PositionalArgument,
        context: &mut LintContext<'a>,
    ) {
        if let Expression::Parenthesized(value) = &positional_argument.value {
            self.report(value, context);
        }
    }

    fn walk_in_named_argument<'ast>(&self, named_argument: &'ast NamedArgument, context: &mut LintContext<'a>) {
        if let Expression::Parenthesized(value) = &named_argument.value {
            self.report(value, context);
        }
    }

    fn walk_in_if<'ast>(&self, r#if: &'ast If, context: &mut LintContext<'a>) {
        if let Expression::Parenthesized(condition) = &r#if.condition {
            self.report(condition, context);
        }
    }

    fn walk_in_if_statement_body_else_if_clause<'ast>(
        &self,
        if_statement_body_else_if_clause: &'ast IfStatementBodyElseIfClause,
        context: &mut LintContext<'a>,
    ) {
        if let Expression::Parenthesized(condition) = &if_statement_body_else_if_clause.condition {
            self.report(condition, context);
        }
    }

    fn walk_in_if_colon_delimited_body_else_if_clause<'ast>(
        &self,
        if_colon_delimited_body_else_if_clause: &'ast IfColonDelimitedBodyElseIfClause,
        context: &mut LintContext<'a>,
    ) {
        if let Expression::Parenthesized(condition) = &if_colon_delimited_body_else_if_clause.condition {
            self.report(condition, context);
        }
    }
}
