use indoc::indoc;

use mago_reporting::*;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct NoBooleanLiteralComparisonRule;

impl Rule for NoBooleanLiteralComparisonRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::disabled("No Boolean Literal Comparison")
            .with_description(indoc! {"
                Disallows comparisons where a boolean literal is used as an operand.

                Comparing with a boolean literal (true or false) using operators such as ===, ==, !==, !=, or <>
                can lead to unintended behavior. Review the logic and remove the literal if it is not required.
            "})
            .with_example(RuleUsageExample::valid(
                "Comparison without a boolean literal",
                indoc! {r#"
                    <?php

                    if ($x === $y) {
                        // ...
                    }
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Comparison with a boolean literal",
                indoc! {r#"
                    <?php

                    if (true === $x) {
                        // ...
                    }
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::Binary(binary) = node else {
            return LintDirective::default();
        };

        match binary.operator {
            BinaryOperator::Equal(_)
            | BinaryOperator::Identical(_)
            | BinaryOperator::NotEqual(_)
            | BinaryOperator::NotIdentical(_)
            | BinaryOperator::AngledNotEqual(_) => {
                if let Some(value) = get_boolean_literal(&binary.lhs) {
                    let literal = if value { "true" } else { "false" };

                    let issue =
                        Issue::new(context.level(), format!("Avoid comparing the boolean literal `{}`.", literal))
                            .with_annotation(
                                Annotation::primary(binary.lhs.span())
                                    .with_message(format!("Boolean literal `{}` is used here.", literal)),
                            )
                            .with_annotation(
                                Annotation::secondary(binary.span())
                                    .with_message("Revise the logic to avoid using a boolean literal."),
                            )
                            .with_note("Comparisons with a boolean literal can be error-prone.")
                            .with_help("Review and adjust the logic to remove the boolean literal comparison.");

                    context.report(issue);
                }

                if let Some(value) = get_boolean_literal(&binary.rhs) {
                    let literal = if value { "true" } else { "false" };

                    let issue =
                        Issue::new(context.level(), format!("Avoid comparing the boolean literal `{}`.", literal))
                            .with_annotation(
                                Annotation::primary(binary.rhs.span())
                                    .with_message(format!("Boolean literal `{}` is used here.", literal)),
                            )
                            .with_annotation(
                                Annotation::secondary(binary.span())
                                    .with_message("Revise the logic to avoid using a boolean literal."),
                            )
                            .with_note("Comparisons with a boolean literal can be error-prone.")
                            .with_help("Review and adjust the logic to remove the boolean literal comparison.");

                    context.report(issue);
                }
            }
            _ => {}
        }

        LintDirective::default()
    }
}

/// Attempts to extract a boolean literal from an expression.
///
/// Returns Some(true) if the expression is the literal `true`,
/// Some(false) if the expression is the literal `false`, or None otherwise.
const fn get_boolean_literal(expr: &Expression) -> Option<bool> {
    match expr {
        Expression::Literal(Literal::True(_)) => Some(true),
        Expression::Literal(Literal::False(_)) => Some(false),
        Expression::Parenthesized(Parenthesized { expression, .. }) => get_boolean_literal(expression),
        _ => None,
    }
}
