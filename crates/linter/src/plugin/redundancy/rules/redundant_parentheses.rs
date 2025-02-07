use indoc::indoc;

use mago_ast::*;
use mago_fixer::SafetyClassification;
use mago_reporting::*;
use mago_span::HasSpan;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
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

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let parenthesized = match node {
            Node::Parenthesized(parenthesized) => {
                if let Expression::Parenthesized(inner) = parenthesized.expression.as_ref() {
                    inner
                } else {
                    return LintDirective::default();
                }
            }
            Node::ExpressionStatement(expression_statement) => match expression_statement.expression.as_ref() {
                Expression::Parenthesized(parenthesized) => parenthesized,
                Expression::Assignment(assignment) => {
                    if let Expression::Parenthesized(rhs) = assignment.rhs.as_ref() {
                        if rhs.expression.is_binary() {
                            return LintDirective::default(); // Allow parentheses around binary expressions on the right-hand side of an assignment.
                        }

                        rhs
                    } else {
                        return LintDirective::default();
                    }
                }
                _ => return LintDirective::default(),
            },
            Node::PositionalArgument(positional_argument) => {
                if positional_argument.ellipsis.is_some() {
                    return LintDirective::default();
                }

                if let Expression::Parenthesized(value) = &positional_argument.value {
                    value
                } else {
                    return LintDirective::default();
                }
            }
            Node::NamedArgument(named_argument) => {
                if named_argument.ellipsis.is_some() {
                    return LintDirective::default();
                }

                if let Expression::Parenthesized(value) = &named_argument.value {
                    value
                } else {
                    return LintDirective::default();
                }
            }
            Node::If(r#if) => {
                if let Expression::Parenthesized(condition) = r#if.condition.as_ref() {
                    condition
                } else {
                    return LintDirective::default();
                }
            }
            Node::IfStatementBodyElseIfClause(if_statement_body_else_if_clause) => {
                if let Expression::Parenthesized(condition) = if_statement_body_else_if_clause.condition.as_ref() {
                    condition
                } else {
                    return LintDirective::default();
                }
            }
            Node::IfColonDelimitedBodyElseIfClause(if_colon_delimited_body_else_if_clause) => {
                if let Expression::Parenthesized(condition) = if_colon_delimited_body_else_if_clause.condition.as_ref()
                {
                    condition
                } else {
                    return LintDirective::default();
                }
            }
            Node::FunctionLikeParameterDefaultValue(function_like_parameter_default_value) => {
                if let Expression::Parenthesized(value) = &function_like_parameter_default_value.value {
                    value
                } else {
                    return LintDirective::default();
                }
            }
            Node::EnumCaseBackedItem(enum_case_backed_item) => {
                if let Expression::Parenthesized(value) = &enum_case_backed_item.value {
                    value
                } else {
                    return LintDirective::default();
                }
            }
            Node::PropertyConcreteItem(property_concrete_item) => {
                if let Expression::Parenthesized(value) = &property_concrete_item.value {
                    value
                } else {
                    return LintDirective::default();
                }
            }
            Node::ConstantItem(constant_item) => {
                if let Expression::Parenthesized(value) = &constant_item.value {
                    value
                } else {
                    return LintDirective::default();
                }
            }
            Node::ClassLikeConstantItem(class_like_constant_item) => {
                if let Expression::Parenthesized(value) = &class_like_constant_item.value {
                    value
                } else {
                    return LintDirective::default();
                }
            }
            _ => return LintDirective::default(),
        };

        let issue = Issue::new(context.level(), "Redundant parentheses around expression.")
            .with_annotation(
                Annotation::primary(parenthesized.expression.span())
                    .with_message("expression does not need to be parenthesized."),
            )
            .with_help("Remove the redundant inner parentheses.");

        context.propose(issue, |plan| {
            plan.delete(parenthesized.left_parenthesis.to_range(), SafetyClassification::Safe);
            plan.delete(parenthesized.right_parenthesis.to_range(), SafetyClassification::Safe);
        });

        LintDirective::default()
    }
}
