use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoRedundantMathRule {
    meta: &'static RuleMeta,
    cfg: NoRedundantMathConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoRedundantMathConfig {
    pub level: Level,
}

impl Default for NoRedundantMathConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for NoRedundantMathConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoRedundantMathRule {
    type Config = NoRedundantMathConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Redundant Math",
            code: "no-redundant-math",
            description: indoc! {"
                Detects redundant mathematical operations that can be simplified or removed.
                Includes operations like multiplying by 1/-1, adding 0, modulo 1/-1, etc.
            "},
            good_example: indoc! {r#"
                <?php

                $result = $value * 2;
                $sum = 1 + $total;
                $difference = $value - 1;
                $remainder = $x % 2;
            "#},
            bad_example: indoc! {r#"
                <?php

                $result = $value * 1;
                $sum = 0 + $total;
                $difference = $value - 0;
                $remainder = $x % 1;
                $negative = $value * -1;
            "#},
            category: Category::Redundancy,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Binary];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::Binary(binary) = node else {
            return;
        };

        let issue = match &binary.operator {
            BinaryOperator::Division(_) => match get_expression_value(binary.rhs) {
                Some(1) => {
                    let mut issue = Issue::new(
                        self.cfg.level(),
                        "Redundant division by `1`: dividing by 1 does not change the value.",
                    )
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(binary.operator.span()).with_message("`$x / 1` is equivalent to `$x`"),
                    )
                    .with_note("Division by 1 always returns the original value.")
                    .with_help("Remove the division by `1` operation.");

                    if !binary.rhs.is_literal() {
                        issue = issue.with_annotation(
                            Annotation::secondary(binary.rhs.span()).with_message("This expression evaluates to `1`"),
                        );
                    }

                    issue
                }
                Some(-1) => {
                    let mut issue = Issue::new(
                        self.cfg.level(),
                        "Redundant division by `-1`: dividing by -1 is equivalent to negation.",
                    )
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(binary.operator.span()).with_message("`$x / -1` is equivalent to `-$x`"),
                    )
                    .with_note("Dividing by -1 negates the value.")
                    .with_help("Replace the division by `-1` with unary negation.");

                    if !binary.rhs.is_literal() {
                        issue = issue.with_annotation(
                            Annotation::secondary(binary.rhs.span()).with_message("This expression evaluates to `-1`"),
                        );
                    }

                    issue
                }
                _ => {
                    return;
                }
            },
            BinaryOperator::Multiplication(_) => {
                match (get_expression_value(binary.lhs), get_expression_value(binary.rhs)) {
                    values @ (Some(1), _) | values @ (_, Some(1)) => {
                        let mut issue = Issue::new(
                            self.cfg.level(),
                            "Redundant multiplication by `1`: multiplying by 1 does not change the value.",
                        )
                        .with_code(self.meta.code)
                        .with_annotation(
                            Annotation::primary(binary.operator.span()).with_message("`$x * 1` is equivalent to `$x`"),
                        )
                        .with_note("Multiplying by 1 returns the original value.")
                        .with_help("Remove the multiplication by `1` operation.");

                        if matches!(values.0, Some(1)) && !binary.lhs.is_literal() {
                            issue = issue.with_annotation(
                                Annotation::secondary(binary.lhs.span())
                                    .with_message("This expression evaluates to `1`"),
                            );
                        } else if !binary.rhs.is_literal() {
                            issue = issue.with_annotation(
                                Annotation::secondary(binary.rhs.span())
                                    .with_message("This expression evaluates to `1`"),
                            );
                        }

                        issue
                    }
                    values @ (Some(-1), _) | values @ (_, Some(-1)) => {
                        let mut issue = Issue::new(
                            self.cfg.level(),
                            "Redundant multiplication by `-1`: multiplication by -1 is equivalent to negation.",
                        )
                        .with_code(self.meta.code)
                        .with_annotation(
                            Annotation::primary(binary.operator.span())
                                .with_message("`$x * -1` is equivalent to `-$x`"),
                        )
                        .with_note("Multiplying by -1 negates the value.")
                        .with_help("Replace the multiplication by `-1` with unary negation.");

                        if matches!(values.0, Some(-1)) && !binary.lhs.is_literal() {
                            issue = issue.with_annotation(
                                Annotation::secondary(binary.lhs.span())
                                    .with_message("This expression evaluates to `-1`"),
                            );
                        } else if !binary.rhs.is_literal() {
                            issue = issue.with_annotation(
                                Annotation::secondary(binary.rhs.span())
                                    .with_message("This expression evaluates to `-1`"),
                            );
                        }

                        issue
                    }
                    _ => {
                        return;
                    }
                }
            }
            BinaryOperator::Addition(_) => {
                let zero = if let Some(0) = get_expression_value(binary.lhs) {
                    &binary.lhs
                } else if let Some(0) = get_expression_value(binary.rhs) {
                    &binary.rhs
                } else {
                    return;
                };

                let mut issue =
                    Issue::new(self.cfg.level(), "Redundant addition of `0`: adding 0 does not alter the value.")
                        .with_code(self.meta.code)
                        .with_annotation(
                            Annotation::primary(binary.operator.span()).with_message("`$x + 0` is equivalent to `$x`"),
                        )
                        .with_note("Adding 0 has no effect.")
                        .with_help("Remove the `+ 0` operation.");

                if !zero.is_literal() {
                    issue = issue.with_annotation(
                        Annotation::secondary(zero.span()).with_message("This expression evaluates to `0`"),
                    );
                }

                issue
            }
            BinaryOperator::Subtraction(_) => {
                let zero = if let Some(0) = get_expression_value(binary.lhs) {
                    &binary.lhs
                } else if let Some(0) = get_expression_value(binary.rhs) {
                    &binary.rhs
                } else {
                    return;
                };

                let mut issue = Issue::new(
                    self.cfg.level(),
                    "Redundant subtraction of `0`: subtracting 0 does not change the value.",
                )
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(binary.operator.span()).with_message("`$x - 0` is equivalent to `$x`"),
                )
                .with_note("Subtracting 0 has no effect.")
                .with_help("Remove the `- 0` operation.");

                if !zero.is_literal() {
                    issue = issue.with_annotation(
                        Annotation::secondary(zero.span()).with_message("This expression evaluates to `0`"),
                    );
                }

                issue
            }
            BinaryOperator::Modulo(_) => match get_expression_value(binary.rhs) {
                Some(1) => {
                    let mut issue = Issue::new(self.cfg.level(), "Redundant modulo by `1`: the result is always `0`.")
                        .with_code(self.meta.code)
                        .with_annotation(
                            Annotation::primary(binary.operator.span()).with_message("`$x % 1` always equals `0`"),
                        )
                        .with_note("Modulo by 1 always returns 0.")
                        .with_help("Replace the modulo operation with `0`.");

                    if !binary.rhs.is_literal() {
                        issue = issue.with_annotation(
                            Annotation::secondary(binary.rhs.span()).with_message("This expression evaluates to `1`"),
                        );
                    }

                    issue
                }
                Some(-1) => {
                    let mut issue = Issue::new(self.cfg.level(), "Redundant modulo by `-1`: the result is always `0`.")
                        .with_code(self.meta.code)
                        .with_annotation(
                            Annotation::primary(binary.operator.span()).with_message("`$x % -1` always equals `0`"),
                        )
                        .with_note("Modulo by -1 always returns 0.")
                        .with_help("Replace the modulo operation with `0`.");

                    if !binary.rhs.is_literal() {
                        issue = issue.with_annotation(
                            Annotation::secondary(binary.rhs.span()).with_message("This expression evaluates to `-1`"),
                        );
                    }

                    issue
                }
                _ => {
                    return;
                }
            },
            BinaryOperator::BitwiseAnd(_) => {
                if !matches!(get_expression_value(binary.rhs), Some(-1)) {
                    return;
                }

                let mut issue = Issue::new(
                    self.cfg.level(),
                    "Redundant bitwise AND with `-1`: this operation does not change the value.",
                )
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(binary.operator.span()).with_message("`$x & -1` is equivalent to `$x`"),
                )
                .with_note("Bitwise AND with -1 leaves the value unchanged.")
                .with_help("Remove the bitwise AND with `-1`.");

                if !binary.rhs.is_literal() {
                    issue = issue.with_annotation(
                        Annotation::secondary(binary.rhs.span()).with_message("This expression evaluates to `-1`"),
                    );
                }

                issue
            }
            BinaryOperator::BitwiseOr(_) | BinaryOperator::BitwiseXor(_) => {
                if !matches!(get_expression_value(binary.rhs), Some(0)) {
                    return;
                }

                let (operator_name, help_msg) = match binary.operator {
                    BinaryOperator::BitwiseOr(_) => ("OR", "bitwise OR with 0"),
                    BinaryOperator::BitwiseXor(_) => ("XOR", "bitwise XOR with 0"),
                    _ => unreachable!(),
                };

                let mut issue = Issue::new(
                    self.cfg.level(),
                    format!("Redundant bitwise {} with `0`: this operation does not alter the value.", operator_name),
                )
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(binary.operator.span())
                        .with_message(format!("`$x {} 0` is equivalent to `$x`", operator_name)),
                )
                .with_note(format!("Bitwise {} with 0 leaves the value unchanged.", operator_name))
                .with_help(format!("Remove the {}.", help_msg));

                if !binary.rhs.is_literal() {
                    issue = issue.with_annotation(
                        Annotation::secondary(binary.rhs.span()).with_message("This expression evaluates to `0`"),
                    );
                }

                issue
            }
            BinaryOperator::LeftShift(_) | BinaryOperator::RightShift(_) => {
                if !matches!(get_expression_value(binary.rhs), Some(0)) {
                    return;
                }

                let operator = match binary.operator {
                    BinaryOperator::LeftShift(_) => "<<",
                    BinaryOperator::RightShift(_) => ">>",
                    _ => unreachable!(),
                };

                let mut issue =
                    Issue::new(self.cfg.level(), "Redundant shift operation: shifting by `0` bits is unnecessary.")
                        .with_code(self.meta.code)
                        .with_annotation(
                            Annotation::primary(binary.operator.span())
                                .with_message(format!("`$x {} 0` is equivalent to `$x`", operator)),
                        )
                        .with_note("Shifting by 0 bits does not change the value.")
                        .with_help("Remove the shift by `0` operation.");

                if !binary.rhs.is_literal() {
                    issue = issue.with_annotation(
                        Annotation::secondary(binary.rhs.span()).with_message("This expression evaluates to `0`"),
                    );
                }

                issue
            }
            _ => {
                return;
            }
        };

        ctx.collector.report(issue);
    }
}

/// A super simple expression evaluator that can handle basic arithmetic operations.
///
/// This function is used to evaluate the value of an expression, if possible.
#[inline]
fn get_expression_value<'ast, 'arena>(expression: &'ast Expression<'arena>) -> Option<isize> {
    match expression {
        Expression::Parenthesized(Parenthesized { expression, .. }) => get_expression_value(expression),
        Expression::Literal(Literal::Integer(LiteralInteger { value: Some(it), .. })) => Some(*it as isize),
        Expression::UnaryPrefix(UnaryPrefix { operator, operand }) => {
            let value = get_expression_value(operand)?;

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
            let lhs_value = get_expression_value(lhs)?;
            let rhs_value = get_expression_value(rhs)?;

            match operator {
                BinaryOperator::Addition(_) => Some(lhs_value + rhs_value),
                BinaryOperator::Subtraction(_) => Some(lhs_value - rhs_value),
                BinaryOperator::Multiplication(_) => Some(lhs_value * rhs_value),
                BinaryOperator::Division(_) => {
                    if rhs_value == 0 || lhs_value % rhs_value != 0 {
                        None
                    } else {
                        Some(lhs_value / rhs_value)
                    }
                }
                BinaryOperator::Modulo(_) => {
                    if rhs_value == 0 {
                        None
                    } else {
                        Some(lhs_value % rhs_value)
                    }
                }
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

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::NoRedundantMathRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = float_division_is_not_redundant,
        rule = NoRedundantMathRule,
        code = indoc! {r#"
            <?php

            function fraction(int $x): float {
                return (4 / 3) * $x;
            }
        "#}
    }

    test_lint_success! {
        name = complex_float_division,
        rule = NoRedundantMathRule,
        code = indoc! {r#"
            <?php

            $val = 10 / 4; // 2.5
            $val2 = 5 / 2; // 2.5
        "#}
    }

    test_lint_success! {
        name = valid_arithmetic,
        rule = NoRedundantMathRule,
        code = indoc! {r#"
            <?php

            $a = $x * 2;
            $b = $x + 1;
            $c = $x - 5;
            $d = $x / 2;
        "#}
    }

    test_lint_failure! {
        name = redundant_division_by_one,
        rule = NoRedundantMathRule,
        code = indoc! {r#"
            <?php

            $a = $x / 1;
        "#}
    }

    test_lint_failure! {
        name = redundant_multiplication_by_one,
        rule = NoRedundantMathRule,
        count = 2,
        code = indoc! {r#"
            <?php

            $a = $x * 1;
            $b = 1 * $x;
        "#}
    }

    test_lint_failure! {
        name = redundant_addition_of_zero,
        rule = NoRedundantMathRule,
        count = 2,
        code = indoc! {r#"
            <?php

            $a = $x + 0;
            $b = 0 + $x;
        "#}
    }

    test_lint_failure! {
        name = redundant_subtraction_of_zero,
        rule = NoRedundantMathRule,
        code = indoc! {r#"
            <?php

            $a = $x - 0;
        "#}
    }

    test_lint_failure! {
        name = redundant_modulo_one,
        rule = NoRedundantMathRule,
        code = indoc! {r#"
            <?php

            $a = $x % 1;
        "#}
    }

    test_lint_failure! {
        name = redundant_negation_operations,
        rule = NoRedundantMathRule,
        count = 3,
        code = indoc! {r#"
            <?php

            $a = $x * -1;
            $b = $x / -1;
            $c = $x % -1;
        "#}
    }
}
