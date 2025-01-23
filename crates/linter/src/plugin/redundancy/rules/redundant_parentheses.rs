use indoc::indoc;

use mago_ast::*;
use mago_fixer::SafetyClassification;
use mago_reporting::*;
use mago_span::HasSpan;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct RedundantParenthesesRule;

impl Rule for RedundantParenthesesRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Redundant Parentheses", Level::Help)
            .with_description(indoc! {"
                Detects redundant parentheses around expressions.
            "})
            .with_example(RuleUsageExample::invalid(
                "A redundant parentheses around an expression",
                indoc! {r#"
                    <?php

                    $foo = (42);
                "#},
            ))
    }
}

impl RedundantParenthesesRule {
    fn report(&self, parenthesized: &Parenthesized, context: &mut LintContext<'_>) {
        let issue = Issue::new(context.level(), "Redundant parentheses around expression.")
            .with_annotation(
                Annotation::primary(parenthesized.expression.span())
                    .with_message("expression does not need to be parenthesized."),
            )
            .with_help("Remove the redundant inner parentheses.");

        context.report_with_fix(issue, |plan| {
            plan.delete(parenthesized.left_parenthesis.to_range(), SafetyClassification::Safe);
            plan.delete(parenthesized.right_parenthesis.to_range(), SafetyClassification::Safe);
        });
    }
}

impl<'a> Walker<LintContext<'a>> for RedundantParenthesesRule {
    fn walk_in_parenthesized<'ast>(&self, parenthesized: &'ast Parenthesized, context: &mut LintContext<'a>) {
        if let Expression::Parenthesized(inner) = parenthesized.expression.as_ref() {
            self.report(inner, context);
        }
    }

    fn walk_in_statement_expression(&self, statement_expression: &ExpressionStatement, context: &mut LintContext<'a>) {
        if let Expression::Parenthesized(parenthesized) = statement_expression.expression.as_ref() {
            self.report(parenthesized, context);

            return;
        }

        if let Expression::AssignmentOperation(assignment) = statement_expression.expression.as_ref() {
            if let Expression::Parenthesized(rhs) = assignment.rhs.as_ref() {
                self.report(rhs, context);
            }
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
        if let Expression::Parenthesized(condition) = r#if.condition.as_ref() {
            self.report(condition, context);
        }
    }

    fn walk_in_if_statement_body_else_if_clause<'ast>(
        &self,
        if_statement_body_else_if_clause: &'ast IfStatementBodyElseIfClause,
        context: &mut LintContext<'a>,
    ) {
        if let Expression::Parenthesized(condition) = if_statement_body_else_if_clause.condition.as_ref() {
            self.report(condition, context);
        }
    }

    fn walk_in_if_colon_delimited_body_else_if_clause<'ast>(
        &self,
        if_colon_delimited_body_else_if_clause: &'ast IfColonDelimitedBodyElseIfClause,
        context: &mut LintContext<'a>,
    ) {
        if let Expression::Parenthesized(condition) = if_colon_delimited_body_else_if_clause.condition.as_ref() {
            self.report(condition, context);
        }
    }

    fn walk_in_function_like_parameter_default_value(
        &self,
        function_like_parameter_default_value: &FunctionLikeParameterDefaultValue,
        context: &mut LintContext<'a>,
    ) {
        if let Expression::Parenthesized(value) = &function_like_parameter_default_value.value {
            self.report(value, context);
        }
    }

    fn walk_in_enum_case_backed_item(&self, enum_case_backed_item: &EnumCaseBackedItem, context: &mut LintContext<'a>) {
        if let Expression::Parenthesized(value) = &enum_case_backed_item.value {
            self.report(value, context);
        }
    }

    fn walk_in_property_concrete_item(
        &self,
        property_concrete_item: &PropertyConcreteItem,
        context: &mut LintContext<'a>,
    ) {
        if let Expression::Parenthesized(value) = &property_concrete_item.value {
            self.report(value, context);
        }
    }

    fn walk_in_constant_item(&self, constant_item: &ConstantItem, context: &mut LintContext<'a>) {
        if let Expression::Parenthesized(value) = &constant_item.value {
            self.report(value, context);
        }
    }

    fn walk_in_class_like_constant_item(
        &self,
        class_like_constant_item: &ClassLikeConstantItem,
        context: &mut LintContext<'a>,
    ) {
        if let Expression::Parenthesized(value) = &class_like_constant_item.value {
            self.report(value, context);
        }
    }
}
