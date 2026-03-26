use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::Binary;
use mago_syntax::ast::BinaryOperator;
use mago_syntax::ast::Expression;
use mago_syntax::ast::MethodCall;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_syntax::ast::UnaryPrefix;
use mago_syntax::ast::UnaryPrefixOperator;
use mago_text_edit::TextEdit;

use crate::category::Category;
use crate::context::LintContext;
use crate::integration::Integration;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::call::method_name_equals;
use crate::rule::utils::pest::find_expect_call;
use crate::rule::utils::pest::get_expect_argument;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct UseSimplerExpectationRule {
    meta: &'static RuleMeta,
    cfg: UseSimplerExpectationConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct UseSimplerExpectationConfig {
    pub level: Level,
}

impl Default for UseSimplerExpectationConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for UseSimplerExpectationConfig {
    fn level(&self) -> Level {
        self.level
    }
}

/// Represents the type of issue detected by this rule.
#[derive(PartialEq)]
enum IssueKind {
    /// `expect(!$x)->toBeTrue()` -> `expect($x)->toBeFalse()`
    NegatedToBeTrue,
    /// `expect(!$x)->toBeFalse()` -> `expect($x)->toBeTrue()`
    NegatedToBeFalse,
    /// `expect($a > $b)->toBeTrue()` -> `expect($a)->toBeGreaterThan($b)`
    GreaterThan,
    /// `expect($a >= $b)->toBeTrue()` -> `expect($a)->toBeGreaterThanOrEqual($b)`
    GreaterThanOrEqual,
    /// `expect($a < $b)->toBeTrue()` -> `expect($a)->toBeLessThan($b)`
    LessThan,
    /// `expect($a <= $b)->toBeTrue()` -> `expect($a)->toBeLessThanOrEqual($b)`
    LessThanOrEqual,
    /// `expect($a === $b)->toBeTrue()` -> `expect($a)->toBe($b)`
    Identical,
    /// `expect($a !== $b)->toBeTrue()` -> `expect($a)->not->toBe($b)`
    NotIdentical,
    /// `expect($x instanceof Y)->toBeTrue()` -> `expect($x)->toBeInstanceOf(Y::class)`
    Instanceof,
    /// `expect($x >= min && $x <= max)->toBeTrue()` -> `expect($x)->toBeBetween(min, max)`
    Between,
}

impl IssueKind {
    fn message(&self) -> &'static str {
        match self {
            Self::NegatedToBeTrue => {
                "Use `expect($condition)->toBeFalse()` instead of `expect(!$condition)->toBeTrue()`."
            }
            Self::NegatedToBeFalse => {
                "Use `expect($condition)->toBeTrue()` instead of `expect(!$condition)->toBeFalse()`."
            }
            Self::GreaterThan => "Use `toBeGreaterThan()` instead of comparing and asserting `toBeTrue()`.",
            Self::GreaterThanOrEqual => {
                "Use `toBeGreaterThanOrEqual()` instead of comparing and asserting `toBeTrue()`."
            }
            Self::LessThan => "Use `toBeLessThan()` instead of comparing and asserting `toBeTrue()`.",
            Self::LessThanOrEqual => "Use `toBeLessThanOrEqual()` instead of comparing and asserting `toBeTrue()`.",
            Self::Identical => "Use `toBe()` instead of comparing with `===` and asserting `toBeTrue()`.",
            Self::NotIdentical => "Use `not->toBe()` instead of comparing with `!==` and asserting `toBeTrue()`.",
            Self::Instanceof => "Use `toBeInstanceOf()` instead of `instanceof` with `toBeTrue()`.",
            Self::Between => "Use `toBeBetween()` for range checks.",
        }
    }

    fn annotation(&self) -> &'static str {
        match self {
            Self::NegatedToBeTrue => "This expectation can be simplified.",
            Self::NegatedToBeFalse => "This expectation can be simplified.",
            Self::GreaterThan => "This can be simplified to `toBeGreaterThan()`.",
            Self::GreaterThanOrEqual => "This can be simplified to `toBeGreaterThanOrEqual()`.",
            Self::LessThan => "This can be simplified to `toBeLessThan()`.",
            Self::LessThanOrEqual => "This can be simplified to `toBeLessThanOrEqual()`.",
            Self::Identical => "This can be simplified.",
            Self::NotIdentical => "This can be simplified.",
            Self::Instanceof => "This can be simplified to `toBeInstanceOf()`.",
            Self::Between => "This can be simplified to `toBeBetween()`.",
        }
    }

    fn help(&self) -> &'static str {
        match self {
            Self::NegatedToBeTrue => "Remove the negation and use `toBeFalse()` instead.",
            Self::NegatedToBeFalse => "Remove the negation and use `toBeTrue()` instead.",
            Self::GreaterThan => "Replace `expect($a > $b)->toBeTrue()` with `expect($a)->toBeGreaterThan($b)`.",
            Self::GreaterThanOrEqual => {
                "Replace `expect($a >= $b)->toBeTrue()` with `expect($a)->toBeGreaterThanOrEqual($b)`."
            }
            Self::LessThan => "Replace `expect($a < $b)->toBeTrue()` with `expect($a)->toBeLessThan($b)`.",
            Self::LessThanOrEqual => {
                "Replace `expect($a <= $b)->toBeTrue()` with `expect($a)->toBeLessThanOrEqual($b)`."
            }
            Self::Identical => "Replace `expect($a === $b)->toBeTrue()` with `expect($a)->toBe($b)`.",
            Self::NotIdentical => "Replace `expect($a !== $b)->toBeTrue()` with `expect($a)->not->toBe($b)`.",
            Self::Instanceof => {
                "Replace `expect($obj instanceof Class)->toBeTrue()` with `expect($obj)->toBeInstanceOf(Class::class)`."
            }
            Self::Between => {
                "Replace `expect($x >= min && $x <= max)->toBeTrue()` with `expect($x)->toBeBetween(min, max)`."
            }
        }
    }
}

/// Check if an expression is a lower bound check (>= or >)
fn is_lower_bound_check(expr: &Expression<'_>) -> bool {
    let Expression::Binary(binary) = expr else {
        return false;
    };

    matches!(binary.operator, BinaryOperator::GreaterThanOrEqual(_) | BinaryOperator::GreaterThan(_))
}

/// Check if an expression is an upper bound check (<= or <)
fn is_upper_bound_check(expr: &Expression<'_>) -> bool {
    let Expression::Binary(binary) = expr else {
        return false;
    };

    matches!(binary.operator, BinaryOperator::LessThanOrEqual(_) | BinaryOperator::LessThan(_))
}

/// Check if expression is a range check pattern like ($x >= min && $x <= max)
fn is_range_check(expr: &Expression<'_>) -> bool {
    let Expression::Binary(binary) = expr else {
        return false;
    };

    // Check for logical AND
    if !matches!(binary.operator, BinaryOperator::And(_) | BinaryOperator::LowAnd(_)) {
        return false;
    }

    // Check if we have lower bound on one side and upper bound on the other
    (is_lower_bound_check(binary.lhs) && is_upper_bound_check(binary.rhs))
        || (is_upper_bound_check(binary.lhs) && is_lower_bound_check(binary.rhs))
}

impl LintRule for UseSimplerExpectationRule {
    type Config = UseSimplerExpectationConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Use Simpler Expectation",
            code: "use-simpler-expectation",
            description: indoc! {"
                Simplify expect() expressions in Pest tests by using dedicated matchers.

                This rule detects patterns where the expect() argument contains an expression that can be simplified:
                - `expect(!$x)->toBeTrue()` -> `expect($x)->toBeFalse()`
                - `expect(!$x)->toBeFalse()` -> `expect($x)->toBeTrue()`
                - `expect($a > $b)->toBeTrue()` -> `expect($a)->toBeGreaterThan($b)`
                - `expect($a >= $b)->toBeTrue()` -> `expect($a)->toBeGreaterThanOrEqual($b)`
                - `expect($a < $b)->toBeTrue()` -> `expect($a)->toBeLessThan($b)`
                - `expect($a <= $b)->toBeTrue()` -> `expect($a)->toBeLessThanOrEqual($b)`
                - `expect($a === $b)->toBeTrue()` -> `expect($a)->toBe($b)`
                - `expect($a !== $b)->toBeTrue()` -> `expect($a)->not->toBe($b)`
                - `expect($x instanceof Y)->toBeTrue()` -> `expect($x)->toBeInstanceOf(Y::class)`
                - `expect($x >= min && $x <= max)->toBeTrue()` -> `expect($x)->toBeBetween(min, max)`

                Using dedicated matchers provides clearer intent and better error messages.
            "},
            good_example: indoc! {r#"
                <?php

                test('simplified expectations', function () {
                    expect($condition)->toBeFalse();
                    expect($a)->toBeGreaterThan($b);
                    expect($a)->toBe($b);
                    expect($obj)->toBeInstanceOf(ClassName::class);
                    expect($x)->toBeBetween(1, 10);
                });
            "#},
            bad_example: indoc! {r#"
                <?php

                test('complex expectations', function () {
                    expect(!$condition)->toBeTrue();
                    expect($a > $b)->toBeTrue();
                    expect($a === $b)->toBeTrue();
                    expect($obj instanceof ClassName)->toBeTrue();
                    expect($x >= 1 && $x <= 10)->toBeTrue();
                });
            "#},
            category: Category::Clarity,
            requirements: RuleRequirements::Integration(Integration::Pest),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::MethodCall];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::MethodCall(method_call) = node else {
            return;
        };

        let is_to_be_true = method_name_equals(method_call, "toBeTrue");
        let is_to_be_false = method_name_equals(method_call, "toBeFalse");

        if !is_to_be_true && !is_to_be_false {
            return;
        }

        let Some(expect_call) = find_expect_call(method_call) else {
            return;
        };

        let Some(expect_arg) = get_expect_argument(ctx, expect_call) else {
            return;
        };

        if let Expression::UnaryPrefix(unary) = expect_arg
            && matches!(unary.operator, UnaryPrefixOperator::Not(_))
        {
            let kind = if is_to_be_true { IssueKind::NegatedToBeTrue } else { IssueKind::NegatedToBeFalse };
            self.report_negation(ctx, method_call, unary, kind);
            return;
        }

        if !is_to_be_true {
            return;
        }

        let Expression::Binary(binary) = expect_arg else {
            return;
        };

        if is_range_check(expect_arg) {
            self.report_no_fix(ctx, method_call.span(), IssueKind::Between);
            return;
        }

        let issue_kind = match &binary.operator {
            BinaryOperator::GreaterThan(_) => IssueKind::GreaterThan,
            BinaryOperator::GreaterThanOrEqual(_) => IssueKind::GreaterThanOrEqual,
            BinaryOperator::LessThan(_) => IssueKind::LessThan,
            BinaryOperator::LessThanOrEqual(_) => IssueKind::LessThanOrEqual,
            BinaryOperator::Identical(_) => IssueKind::Identical,
            BinaryOperator::NotIdentical(_) => IssueKind::NotIdentical,
            BinaryOperator::Instanceof(_) => IssueKind::Instanceof,
            _ => return,
        };

        self.report_binary(ctx, method_call, binary, issue_kind);
    }
}

impl UseSimplerExpectationRule {
    /// Report an issue without a fix (for complex cases like range checks)
    fn report_no_fix(&self, ctx: &mut LintContext<'_, '_>, span: mago_span::Span, kind: IssueKind) {
        let issue = Issue::new(self.cfg.level(), kind.message())
            .with_code(self.meta.code)
            .with_annotation(Annotation::primary(span).with_message(kind.annotation()))
            .with_help(kind.help());

        ctx.collector.report(issue);
    }

    /// Report and fix a negation pattern: expect(!$x)->toBeTrue() or expect(!$x)->toBeFalse()
    fn report_negation(
        &self,
        ctx: &mut LintContext<'_, '_>,
        method_call: &MethodCall<'_>,
        unary: &UnaryPrefix<'_>,
        kind: IssueKind,
    ) {
        let issue = Issue::new(self.cfg.level(), kind.message())
            .with_code(self.meta.code)
            .with_annotation(Annotation::primary(method_call.span()).with_message(kind.annotation()))
            .with_help(kind.help());

        let replacement_method = match kind {
            IssueKind::NegatedToBeTrue => "toBeFalse()",
            IssueKind::NegatedToBeFalse => "toBeTrue()",
            _ => {
                ctx.collector.report(issue);
                return;
            }
        };

        ctx.collector.propose(issue, |edits| {
            edits.push(TextEdit::delete(unary.operator.span()));
            edits.push(TextEdit::replace(
                method_call.method.span().join(method_call.argument_list.span()),
                replacement_method,
            ));
        });
    }

    /// Report and fix a binary expression pattern
    fn report_binary(
        &self,
        ctx: &mut LintContext<'_, '_>,
        method_call: &MethodCall<'_>,
        binary: &Binary<'_>,
        kind: IssueKind,
    ) {
        let issue = Issue::new(self.cfg.level(), kind.message())
            .with_code(self.meta.code)
            .with_annotation(Annotation::primary(method_call.span()).with_message(kind.annotation()))
            .with_help(kind.help());

        let (operator_replacement, method_replacement) = match kind {
            IssueKind::GreaterThan => (")->toBeGreaterThan(", ""),
            IssueKind::GreaterThanOrEqual => (")->toBeGreaterThanOrEqual(", ""),
            IssueKind::LessThan => (")->toBeLessThan(", ""),
            IssueKind::LessThanOrEqual => (")->toBeLessThanOrEqual(", ""),
            IssueKind::Identical => (")->toBe(", ""),
            IssueKind::NotIdentical => (")->not->toBe(", ""),
            IssueKind::Instanceof => (")->toBeInstanceOf(", "::class"),
            _ => {
                ctx.collector.report(issue);
                return;
            }
        };

        let rhs_span = binary.rhs.span();
        let operator_span = binary.operator.span();
        let delete_span = Span::new(rhs_span.file_id, rhs_span.end, method_call.argument_list.left_parenthesis.end);

        ctx.collector.propose(issue, |edits| {
            edits.push(TextEdit::replace(operator_span, operator_replacement));

            if kind == IssueKind::Instanceof {
                edits.push(TextEdit::replace(delete_span, method_replacement));
            } else {
                edits.push(TextEdit::delete(delete_span));
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    // Success cases
    test_lint_success! {
        name = direct_expectations,
        rule = UseSimplerExpectationRule,
        code = indoc! {r#"
            <?php

            expect($condition)->toBeTrue();
            expect($condition)->toBeFalse();
        "#}
    }

    test_lint_success! {
        name = dedicated_matchers,
        rule = UseSimplerExpectationRule,
        code = indoc! {r#"
            <?php

            expect($a)->toBeGreaterThan($b);
            expect($a)->toBeGreaterThanOrEqual($b);
            expect($a)->toBeLessThan($b);
            expect($a)->toBeLessThanOrEqual($b);
            expect($a)->toBe($b);
            expect($a)->not->toBe($b);
            expect($obj)->toBeInstanceOf(ClassName::class);
            expect($value)->toBeBetween(1, 10);
        "#}
    }

    test_lint_success! {
        name = non_pattern_expressions,
        rule = UseSimplerExpectationRule,
        code = indoc! {r#"
            <?php

            expect($a && $b)->toBeTrue();
            expect($a || $b)->toBeFalse();
            expect($a == $b)->toBeTrue();
            expect($a != $b)->toBeTrue();
        "#}
    }

    // Negation patterns
    test_lint_failure! {
        name = negated_to_be_true,
        rule = UseSimplerExpectationRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect(!$condition)->toBeTrue();
        "#}
    }

    test_lint_failure! {
        name = negated_to_be_false,
        rule = UseSimplerExpectationRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect(!$condition)->toBeFalse();
        "#}
    }

    // Comparison patterns
    test_lint_failure! {
        name = greater_than_to_be_true,
        rule = UseSimplerExpectationRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect($a > $b)->toBeTrue();
        "#}
    }

    test_lint_failure! {
        name = greater_than_or_equal_to_be_true,
        rule = UseSimplerExpectationRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect($a >= $b)->toBeTrue();
        "#}
    }

    test_lint_failure! {
        name = less_than_to_be_true,
        rule = UseSimplerExpectationRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect($a < $b)->toBeTrue();
        "#}
    }

    test_lint_failure! {
        name = less_than_or_equal_to_be_true,
        rule = UseSimplerExpectationRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect($a <= $b)->toBeTrue();
        "#}
    }

    // Equality patterns
    test_lint_failure! {
        name = identical_to_be_true,
        rule = UseSimplerExpectationRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect($a === $b)->toBeTrue();
        "#}
    }

    test_lint_failure! {
        name = not_identical_to_be_true,
        rule = UseSimplerExpectationRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect($a !== $b)->toBeTrue();
        "#}
    }

    // Instanceof pattern
    test_lint_failure! {
        name = instanceof_to_be_true,
        rule = UseSimplerExpectationRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect($obj instanceof ClassName)->toBeTrue();
        "#}
    }

    test_lint_failure! {
        name = instanceof_fqn_to_be_true,
        rule = UseSimplerExpectationRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect($obj instanceof \Namespace\ClassName)->toBeTrue();
        "#}
    }

    // Between pattern
    test_lint_failure! {
        name = range_check_gte_lte,
        rule = UseSimplerExpectationRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect($x >= 1 && $x <= 10)->toBeTrue();
        "#}
    }

    test_lint_failure! {
        name = range_check_gt_lt,
        rule = UseSimplerExpectationRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect($x > 0 && $x < 100)->toBeTrue();
        "#}
    }

    test_lint_failure! {
        name = range_check_reversed_order,
        rule = UseSimplerExpectationRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect($x <= 10 && $x >= 1)->toBeTrue();
        "#}
    }

    // Case insensitivity
    test_lint_failure! {
        name = case_insensitive_to_be_true,
        rule = UseSimplerExpectationRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect(!$condition)->TOBETRUE();
        "#}
    }

    test_lint_failure! {
        name = case_insensitive_to_be_false,
        rule = UseSimplerExpectationRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect(!$condition)->ToBeFalse();
        "#}
    }

    // Multiple violations
    test_lint_failure! {
        name = multiple_violations,
        rule = UseSimplerExpectationRule,
        count = 5,
        code = indoc! {r#"
            <?php

            expect(!$a)->toBeTrue();
            expect($a > $b)->toBeTrue();
            expect($a === $b)->toBeTrue();
            expect($obj instanceof Foo)->toBeTrue();
            expect($x >= 1 && $x <= 10)->toBeTrue();
        "#}
    }
}
