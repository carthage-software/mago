use indoc::indoc;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct RedundantMathematicalOperationRule;

impl Rule for RedundantMathematicalOperationRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Redundant Mathematical Operation", Level::Help)
            .with_description(indoc! {"
                Detects redundant mathematical operations that can be simplified or removed.
                Includes operations like multiplying by 1/-1, adding 0, modulo 1/-1, etc.
            "})
            .with_example(RuleUsageExample::invalid(
                "Redundant multiplication by 1",
                indoc! {r#"
                    <?php
                    $result = $value * 1;  // Can be simplified to $value
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Redundant addition of 0",
                indoc! {r#"
                    <?php
                    $sum = 0 + $total;  // Can be simplified to $total
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Redundant subtraction of 0",
                indoc! {r#"
                    <?php
                    $difference = $value - 0;  // Can be simplified to $value
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Redundant modulo 1",
                indoc! {r#"
                    <?php
                    $remainder = $x % 1;  // Always 0 for integers
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Multiplication by -1",
                indoc! {r#"
                    <?php
                    $negative = $value * -1;  // Can be simplified to -$value
                "#},
            ))
            .with_example(RuleUsageExample::valid(
                "Valid mathematical operations",
                indoc! {r#"
                    <?php
                    $a = 5 * 2;     // Valid multiplication
                    $b = 10 + 5;    // Valid addition
                    $c = 7 % 3;     // Valid modulo
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::Binary(binary) = node else {
            return LintDirective::default();
        };

        let issue = match &binary.operator {
            BinaryOperator::Division(_) => match get_expression_value(&binary.rhs) {
                Some(1) => {
                    let mut issue = Issue::new(
                        context.level(),
                        "Redundant division by `1`: dividing by 1 does not change the value.",
                    )
                    .with_annotation(
                        Annotation::primary(binary.operator.span()).with_message("`$x / 1` is equivalent to `$x`."),
                    )
                    .with_note("Division by 1 always returns the original value.")
                    .with_help("Remove the division by `1` operation.");

                    if !binary.rhs.is_literal() {
                        issue = issue.with_annotation(
                            Annotation::secondary(binary.rhs.span()).with_message("This expression evaluates to `1`."),
                        );
                    }

                    issue
                }
                Some(-1) => {
                    let mut issue = Issue::new(
                        context.level(),
                        "Redundant division by `-1`: dividing by -1 is equivalent to negation.",
                    )
                    .with_annotation(
                        Annotation::primary(binary.operator.span()).with_message("`$x / -1` is equivalent to `-$x`."),
                    )
                    .with_note("Dividing by -1 negates the value.")
                    .with_help("Replace the division by `-1` with unary negation.");

                    if !binary.rhs.is_literal() {
                        issue = issue.with_annotation(
                            Annotation::secondary(binary.rhs.span()).with_message("This expression evaluates to `-1`."),
                        );
                    }

                    issue
                }
                _ => {
                    return LintDirective::default();
                }
            },
            BinaryOperator::Multiplication(_) => {
                match (get_expression_value(&binary.lhs), get_expression_value(&binary.rhs)) {
                    values @ (Some(1), _) | values @ (_, Some(1)) => {
                        let mut issue = Issue::new(
                            context.level(),
                            "Redundant multiplication by `1`: multiplying by 1 does not change the value.",
                        )
                        .with_annotation(
                            Annotation::primary(binary.operator.span()).with_message("`$x * 1` is equivalent to `$x`."),
                        )
                        .with_note("Multiplying by 1 returns the original value.")
                        .with_help("Remove the multiplication by `1` operation.");

                        if matches!(values.0, Some(1)) && !binary.lhs.is_literal() {
                            issue = issue.with_annotation(
                                Annotation::secondary(binary.lhs.span())
                                    .with_message("This expression evaluates to `1`."),
                            );
                        } else if !binary.rhs.is_literal() {
                            issue = issue.with_annotation(
                                Annotation::secondary(binary.rhs.span())
                                    .with_message("This expression evaluates to `1`."),
                            );
                        }

                        issue
                    }
                    values @ (Some(-1), _) | values @ (_, Some(-1)) => {
                        let mut issue = Issue::new(
                            context.level(),
                            "Redundant multiplication by `-1`: multiplication by -1 is equivalent to negation.",
                        )
                        .with_annotation(
                            Annotation::primary(binary.operator.span())
                                .with_message("`$x * -1` is equivalent to `-$x`."),
                        )
                        .with_note("Multiplying by -1 negates the value.")
                        .with_help("Replace the multiplication by `-1` with unary negation.");

                        if matches!(values.0, Some(-1)) && !binary.lhs.is_literal() {
                            issue = issue.with_annotation(
                                Annotation::secondary(binary.lhs.span())
                                    .with_message("This expression evaluates to `-1`."),
                            );
                        } else if !binary.rhs.is_literal() {
                            issue = issue.with_annotation(
                                Annotation::secondary(binary.rhs.span())
                                    .with_message("This expression evaluates to `-1`."),
                            );
                        }

                        issue
                    }
                    _ => {
                        return LintDirective::default();
                    }
                }
            }
            BinaryOperator::Addition(_) => {
                let zero = if let Some(0) = get_expression_value(&binary.lhs) {
                    &binary.lhs
                } else if let Some(0) = get_expression_value(&binary.rhs) {
                    &binary.rhs
                } else {
                    return LintDirective::default();
                };

                let mut issue =
                    Issue::new(context.level(), "Redundant addition of `0`: adding 0 does not alter the value.")
                        .with_annotation(
                            Annotation::primary(binary.operator.span()).with_message("`$x + 0` is equivalent to `$x`."),
                        )
                        .with_note("Adding 0 has no effect.")
                        .with_help("Remove the `+ 0` operation.");

                if !zero.is_literal() {
                    issue = issue.with_annotation(
                        Annotation::secondary(zero.span()).with_message("This expression evaluates to `0`."),
                    );
                }

                issue
            }
            BinaryOperator::Subtraction(_) => {
                let zero = if let Some(0) = get_expression_value(&binary.lhs) {
                    &binary.lhs
                } else if let Some(0) = get_expression_value(&binary.rhs) {
                    &binary.rhs
                } else {
                    return LintDirective::default();
                };

                let mut issue = Issue::new(
                    context.level(),
                    "Redundant subtraction of `0`: subtracting 0 does not change the value.",
                )
                .with_annotation(
                    Annotation::primary(binary.operator.span()).with_message("`$x - 0` is equivalent to `$x`."),
                )
                .with_note("Subtracting 0 has no effect.")
                .with_help("Remove the `- 0` operation.");

                if !zero.is_literal() {
                    issue = issue.with_annotation(
                        Annotation::secondary(zero.span()).with_message("This expression evaluates to `0`."),
                    );
                }

                issue
            }
            BinaryOperator::Modulo(_) => match get_expression_value(&binary.rhs) {
                Some(1) => {
                    let mut issue = Issue::new(context.level(), "Redundant modulo by `1`: the result is always `0`.")
                        .with_annotation(
                            Annotation::primary(binary.operator.span()).with_message("`$x % 1` always equals `0`."),
                        )
                        .with_note("Modulo by 1 always returns 0.")
                        .with_help("Replace the modulo operation with `0`.");

                    if !binary.rhs.is_literal() {
                        issue = issue.with_annotation(
                            Annotation::secondary(binary.rhs.span()).with_message("This expression evaluates to `1`."),
                        );
                    }

                    issue
                }
                Some(-1) => {
                    let mut issue = Issue::new(context.level(), "Redundant modulo by `-1`: the result is always `0`.")
                        .with_annotation(
                            Annotation::primary(binary.operator.span()).with_message("`$x % -1` always equals `0`."),
                        )
                        .with_note("Modulo by -1 always returns 0.")
                        .with_help("Replace the modulo operation with `0`.");

                    if !binary.rhs.is_literal() {
                        issue = issue.with_annotation(
                            Annotation::secondary(binary.rhs.span()).with_message("This expression evaluates to `-1`."),
                        );
                    }

                    issue
                }
                _ => {
                    return LintDirective::default();
                }
            },
            BinaryOperator::BitwiseAnd(_) => {
                if !matches!(get_expression_value(&binary.rhs), Some(-1)) {
                    return LintDirective::default();
                }

                let mut issue = Issue::new(
                    context.level(),
                    "Redundant bitwise AND with `-1`: this operation does not change the value.",
                )
                .with_annotation(
                    Annotation::primary(binary.operator.span()).with_message("`$x & -1` is equivalent to `$x`."),
                )
                .with_note("Bitwise AND with -1 leaves the value unchanged.")
                .with_help("Remove the bitwise AND with `-1`.");

                if !binary.rhs.is_literal() {
                    issue = issue.with_annotation(
                        Annotation::secondary(binary.rhs.span()).with_message("This expression evaluates to `-1`."),
                    );
                }

                issue
            }
            BinaryOperator::BitwiseOr(_) | BinaryOperator::BitwiseXor(_) => {
                if !matches!(get_expression_value(&binary.rhs), Some(0)) {
                    return LintDirective::default();
                }

                let (operator_name, help_msg) = match binary.operator {
                    BinaryOperator::BitwiseOr(_) => ("OR", "bitwise OR with 0"),
                    BinaryOperator::BitwiseXor(_) => ("XOR", "bitwise XOR with 0"),
                    _ => unreachable!(),
                };

                let mut issue = Issue::new(
                    context.level(),
                    format!("Redundant bitwise {} with `0`: this operation does not alter the value.", operator_name),
                )
                .with_annotation(
                    Annotation::primary(binary.operator.span())
                        .with_message(format!("`$x {} 0` is equivalent to `$x`.", operator_name)),
                )
                .with_note(format!("Bitwise {} with 0 leaves the value unchanged.", operator_name))
                .with_help(format!("Remove the {}", help_msg));

                if !binary.rhs.is_literal() {
                    issue = issue.with_annotation(
                        Annotation::secondary(binary.rhs.span()).with_message("This expression evaluates to `0`."),
                    );
                }

                issue
            }
            BinaryOperator::LeftShift(_) | BinaryOperator::RightShift(_) => {
                if !matches!(get_expression_value(&binary.rhs), Some(0)) {
                    return LintDirective::default();
                }

                let operator = match binary.operator {
                    BinaryOperator::LeftShift(_) => "<<",
                    BinaryOperator::RightShift(_) => ">>",
                    _ => unreachable!(),
                };

                let mut issue = Issue::new(
                    context.level(),
                    "Redundant shift operation: shifting by `0` bits is unnecessary.".to_string(),
                )
                .with_annotation(
                    Annotation::primary(binary.operator.span())
                        .with_message(format!("`$x {} 0` is equivalent to `$x`.", operator)),
                )
                .with_note("Shifting by 0 bits does not change the value.")
                .with_help("Remove the shift by `0` operation.".to_string());

                if !binary.rhs.is_literal() {
                    issue = issue.with_annotation(
                        Annotation::secondary(binary.rhs.span()).with_message("This expression evaluates to `0`."),
                    );
                }

                issue
            }
            _ => {
                return LintDirective::default();
            }
        };

        context.report(issue);

        LintDirective::Continue
    }
}

/// A super simple expression evaluator that can handle basic arithmetic operations.
///
/// This function is used to evaluate the value of an expression, if possible.
#[inline]
const fn get_expression_value(expression: &Expression) -> Option<isize> {
    match expression {
        Expression::Parenthesized(Parenthesized { expression, .. }) => get_expression_value(expression),
        Expression::Literal(Literal::Integer(LiteralInteger { value, .. })) => Some(*value as isize),
        Expression::UnaryPrefix(UnaryPrefix { operator, operand }) => {
            let value = match get_expression_value(operand) {
                Some(it) => it,
                None => return None,
            };

            match operator {
                UnaryPrefixOperator::Negation(_) => Some(-value),
                UnaryPrefixOperator::BitwiseNot(_) => Some(!value),
                UnaryPrefixOperator::Reference(_) => Some(value),
                UnaryPrefixOperator::ErrorControl(_) => Some(value),
                UnaryPrefixOperator::IntCast(_, _) => Some(value),
                _ => None,
            }
        }
        Expression::Binary(Binary { lhs, operator, rhs }) => {
            let lhs_value = match get_expression_value(lhs) {
                Some(it) => it,
                None => return None,
            };
            let rhs_value = match get_expression_value(rhs) {
                Some(it) => it,
                None => return None,
            };

            match operator {
                BinaryOperator::Addition(_) => Some(lhs_value + rhs_value),
                BinaryOperator::Subtraction(_) => Some(lhs_value - rhs_value),
                BinaryOperator::Multiplication(_) => Some(lhs_value * rhs_value),
                BinaryOperator::Division(_) => Some(lhs_value / rhs_value),
                BinaryOperator::Modulo(_) => Some(lhs_value % rhs_value),
                BinaryOperator::Exponentiation(_) => Some(lhs_value.pow(rhs_value as u32)),
                BinaryOperator::BitwiseAnd(_) => Some(lhs_value & rhs_value),
                BinaryOperator::BitwiseOr(_) => Some(lhs_value | rhs_value),
                BinaryOperator::BitwiseXor(_) => Some(lhs_value ^ rhs_value),
                BinaryOperator::LeftShift(_) => Some(lhs_value << rhs_value),
                BinaryOperator::RightShift(_) => Some(lhs_value >> rhs_value),
                _ => None,
            }
        }
        _ => None,
    }
}
