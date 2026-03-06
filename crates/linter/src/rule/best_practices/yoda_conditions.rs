use indoc::indoc;
use mago_text_edit::TextEdit;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::BinaryOperator;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct YodaConditionsRule {
    meta: &'static RuleMeta,
    cfg: YodaConditionsConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum YodaConditionsStyle {
    /// Enforce Yoda style: constant on the left, variable on the right.
    Yoda,
    /// Enforce non-Yoda style: variable on the left, constant on the right.
    NonYoda,
}

impl Default for YodaConditionsStyle {
    fn default() -> Self {
        Self::Yoda
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct YodaConditionsConfig {
    pub level: Level,
    pub style: YodaConditionsStyle,
}

impl Default for YodaConditionsConfig {
    fn default() -> Self {
        Self { level: Level::Help, style: YodaConditionsStyle::Yoda }
    }
}

impl Config for YodaConditionsConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for YodaConditionsRule {
    type Config = YodaConditionsConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Yoda Conditions",
            code: "yoda-conditions",
            description: indoc! {"
                This rule enforces a consistent condition style for comparisons. In \"yoda\" mode (default),
                the constant should always be on the left side. In \"non-yoda\" mode, the variable should
                always be on the left side.
            "},
            good_example: indoc! {r"
                <?php

                if ( true === $is_active ) { /* ... */ }
                if ( 5 === $count ) { /* ... */ }
            "},
            bad_example: indoc! {r"
                <?php

                if ( $is_active === true ) { /* ... */ }
            "},
            category: Category::BestPractices,
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

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::Binary(binary) = node else {
            return;
        };

        let is_equality = matches!(
            binary.operator,
            BinaryOperator::Equal(_)
                | BinaryOperator::NotEqual(_)
                | BinaryOperator::Identical(_)
                | BinaryOperator::NotIdentical(_)
                | BinaryOperator::AngledNotEqual(_)
        );

        let is_comparison = matches!(
            binary.operator,
            BinaryOperator::LessThan(_)
                | BinaryOperator::LessThanOrEqual(_)
                | BinaryOperator::GreaterThan(_)
                | BinaryOperator::GreaterThanOrEqual(_)
        );

        if !is_equality && !is_comparison {
            return;
        }

        let left_is_variable = is_writable_variable(binary.lhs);
        let right_is_constant = is_constant_like(binary.rhs);
        let left_is_constant = is_constant_like(binary.lhs);
        let right_is_variable = is_writable_variable(binary.rhs);

        match self.cfg.style {
            YodaConditionsStyle::Yoda => {
                // Only check equality for Yoda (original behavior)
                if !is_equality {
                    return;
                }

                // If variable is on the left and constant is on the right, suggest Yoda condition
                if left_is_variable && right_is_constant {
                    let issue =
                        Issue::new(self.cfg.level(), "Use Yoda condition style for safer comparisons")
                            .with_code(self.meta.code)
                            .with_annotation(
                                Annotation::primary(binary.operator.span())
                                    .with_message("Variable should be on the right side"),
                            )
                            .with_note("Yoda conditions help prevent accidental assignment bugs where `=` is used instead of `==`")
                            .with_help("Move constant/literal to left: `5 === $count`");

                    let source_code = ctx.source_file.contents.as_ref();
                    ctx.collector.propose(issue, |edits| {
                        build_swap_edits(source_code, binary.lhs, binary.rhs, &binary.operator, edits);
                    });
                }
            }
            YodaConditionsStyle::NonYoda => {
                // Check both equality and comparison operators for NonYoda
                if left_is_constant && right_is_variable {
                    let issue =
                        Issue::new(self.cfg.level(), "Use non-Yoda condition style for readability")
                            .with_code(self.meta.code)
                            .with_annotation(
                                Annotation::primary(binary.operator.span())
                                    .with_message("Variable should be on the left side"),
                            )
                            .with_note("Non-Yoda conditions read more naturally: `$count === 5`")
                            .with_help("Move variable to left side of comparison");

                    let source_code = ctx.source_file.contents.as_ref();
                    ctx.collector.propose(issue, |edits| {
                        build_swap_edits(source_code, binary.lhs, binary.rhs, &binary.operator, edits);
                    });
                }
            }
        }
    }
}

fn build_swap_edits(
    source_code: &str,
    lhs: &Expression<'_>,
    rhs: &Expression<'_>,
    operator: &BinaryOperator,
    edits: &mut Vec<TextEdit>,
) {
    let right_side_span = rhs.span();
    let right_side_start = right_side_span.start_offset() as usize;
    let right_side_end = right_side_span.end_offset() as usize;
    let right_side = &source_code[right_side_start..right_side_end];

    let left_side_span = lhs.span();
    let left_side_start = left_side_span.start_offset() as usize;
    let left_side_end = left_side_span.end_offset() as usize;
    let left_side = &source_code[left_side_start..left_side_end];

    edits.push(TextEdit::replace(right_side_span, left_side));
    edits.push(TextEdit::replace(left_side_span, right_side));

    // For comparison operators, also flip the operator
    if let Some(flipped) = flip_comparison_operator(operator) {
        let op_span = operator.span();
        edits.push(TextEdit::replace(op_span, flipped));
    }
}

/// Returns the flipped operator string for comparison operators, or None for equality operators.
fn flip_comparison_operator(op: &BinaryOperator) -> Option<&'static str> {
    match op {
        BinaryOperator::LessThan(_) => Some(">"),
        BinaryOperator::LessThanOrEqual(_) => Some(">="),
        BinaryOperator::GreaterThan(_) => Some("<"),
        BinaryOperator::GreaterThanOrEqual(_) => Some("<="),
        _ => None,
    }
}

/// Check if an expression is "constant-like" (literal, named constant, magic constant, or array literal).
/// Does NOT include function calls — `strlen($x)` is not constant-like for yoda purposes,
/// matching PHP-CS-Fixer's behavior.
const fn is_constant_like(expr: &Expression) -> bool {
    matches!(
        expr,
        Expression::Literal(_)
            | Expression::ConstantAccess(_)
            | Expression::MagicConstant(_)
            | Expression::Array(_)
            | Expression::LegacyArray(_)
    )
}

const fn is_writable_variable(expr: &Expression) -> bool {
    matches!(expr, Expression::Variable(_) | Expression::Access(_) | Expression::ArrayAccess(_))
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::YodaConditionsRule;
    use super::YodaConditionsStyle;
    use crate::settings::Settings;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    // === Yoda style tests (default, original behavior) ===

    test_lint_success! {
        name = yoda_style_constant_on_left_is_ok,
        rule = YodaConditionsRule,
        settings = |settings: &mut Settings| {
            settings.rules.yoda_conditions.config.style = YodaConditionsStyle::Yoda;
        },
        code = indoc! {r"
            <?php

            if (true === $is_active) { }
            if (5 === $count) { }
        "}
    }

    test_lint_failure! {
        name = yoda_style_variable_on_left_is_bad,
        rule = YodaConditionsRule,
        settings = |settings: &mut Settings| {
            settings.rules.yoda_conditions.config.style = YodaConditionsStyle::Yoda;
        },
        code = indoc! {r"
            <?php

            if ($is_active === true) { }
        "}
    }

    // === Non-Yoda style tests ===

    test_lint_success! {
        name = non_yoda_variable_on_left_equality_is_ok,
        rule = YodaConditionsRule,
        settings = |settings: &mut Settings| {
            settings.rules.yoda_conditions.config.style = YodaConditionsStyle::NonYoda;
        },
        code = indoc! {r"
            <?php

            if ($is_active === true) { }
            if ($count == 5) { }
            if ($x !== null) { }
        "}
    }

    test_lint_failure! {
        name = non_yoda_constant_on_left_equality_is_bad,
        rule = YodaConditionsRule,
        settings = |settings: &mut Settings| {
            settings.rules.yoda_conditions.config.style = YodaConditionsStyle::NonYoda;
        },
        code = indoc! {r"
            <?php

            if (true === $is_active) { }
        "}
    }

    test_lint_success! {
        name = non_yoda_variable_on_left_comparison_is_ok,
        rule = YodaConditionsRule,
        settings = |settings: &mut Settings| {
            settings.rules.yoda_conditions.config.style = YodaConditionsStyle::NonYoda;
        },
        code = indoc! {r"
            <?php

            if ($count > 5) { }
            if ($x <= 10) { }
        "}
    }

    test_lint_failure! {
        name = non_yoda_constant_on_left_less_than_is_bad,
        rule = YodaConditionsRule,
        settings = |settings: &mut Settings| {
            settings.rules.yoda_conditions.config.style = YodaConditionsStyle::NonYoda;
        },
        code = indoc! {r"
            <?php

            if (5 < $count) { }
        "}
    }

    test_lint_failure! {
        name = non_yoda_constant_on_left_greater_than_is_bad,
        rule = YodaConditionsRule,
        settings = |settings: &mut Settings| {
            settings.rules.yoda_conditions.config.style = YodaConditionsStyle::NonYoda;
        },
        code = indoc! {r"
            <?php

            if (10 >= $count) { }
        "}
    }

    test_lint_success! {
        name = non_yoda_two_variables_is_ok,
        rule = YodaConditionsRule,
        settings = |settings: &mut Settings| {
            settings.rules.yoda_conditions.config.style = YodaConditionsStyle::NonYoda;
        },
        code = indoc! {r"
            <?php

            if ($a === $b) { }
        "}
    }

    test_lint_success! {
        name = non_yoda_two_constants_is_ok,
        rule = YodaConditionsRule,
        settings = |settings: &mut Settings| {
            settings.rules.yoda_conditions.config.style = YodaConditionsStyle::NonYoda;
        },
        code = indoc! {r"
            <?php

            if (1 === 2) { }
        "}
    }

    test_lint_success! {
        name = yoda_style_ignores_comparison_operators,
        rule = YodaConditionsRule,
        settings = |settings: &mut Settings| {
            settings.rules.yoda_conditions.config.style = YodaConditionsStyle::Yoda;
        },
        code = indoc! {r"
            <?php

            if ($count > 5) { }
            if (5 < $count) { }
        "}
    }
}
