use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Literal;
use mago_syntax::ast::MethodCall;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_text_edit::TextEdit;

use crate::category::Category;
use crate::context::LintContext;
use crate::integration::Integration;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::call::method_name_equals;
use crate::rule::utils::call::method_name_matches_any;
use crate::rule::utils::pest::find_expect_call;
use crate::rule::utils::pest::find_not_to_method_span;
use crate::rule::utils::pest::get_expect_argument;
use crate::rule::utils::pest::get_first_argument;
use crate::rule::utils::pest::has_not_modifier;
use crate::rule::utils::pest::is_empty_array;
use crate::rule::utils::pest::is_empty_string;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct UseSpecificExpectationsRule {
    meta: &'static RuleMeta,
    cfg: UseSpecificExpectationsConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct UseSpecificExpectationsConfig {
    pub level: Level,
}

impl Default for UseSpecificExpectationsConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for UseSpecificExpectationsConfig {
    fn level(&self) -> Level {
        self.level
    }
}

/// Represents the type of issue detected by this rule.
enum IssueKind {
    /// `toBe(true)` or `toEqual(true)` -> `toBeTrue()`
    ToBeTrue,
    /// `toBe(false)` or `toEqual(false)` -> `toBeFalse()`
    ToBeFalse,
    /// `toBe(null)` or `toEqual(null)` -> `toBeNull()`
    ToBeNull,
    /// `toBe([])` or `toBe('')` -> `toBeEmpty()`
    ToBeEmpty,
    /// `not->toBeFalse()` -> `toBeTrue()`
    NotToBeFalse,
    /// `not->toBeTrue()` -> `toBeFalse()`
    NotToBeTrue,
}

impl IssueKind {
    fn message(&self) -> &'static str {
        match self {
            Self::ToBeTrue => "Use `toBeTrue()` instead of `toBe(true)` for clearer boolean expectations.",
            Self::ToBeFalse => "Use `toBeFalse()` instead of `toBe(false)` for clearer boolean expectations.",
            Self::ToBeNull => "Use `toBeNull()` instead of `toBe(null)` for clearer null expectations.",
            Self::ToBeEmpty => "Use `toBeEmpty()` instead of comparing with empty values for clarity.",
            Self::NotToBeFalse => "Use `toBeTrue()` instead of `not->toBeFalse()` for clarity.",
            Self::NotToBeTrue => "Use `toBeFalse()` instead of `not->toBeTrue()` for clarity.",
        }
    }

    fn annotation(&self) -> &'static str {
        match self {
            Self::ToBeTrue => "This can be simplified to `toBeTrue()`.",
            Self::ToBeFalse => "This can be simplified to `toBeFalse()`.",
            Self::ToBeNull => "This can be simplified to `toBeNull()`.",
            Self::ToBeEmpty => "This can be simplified to `toBeEmpty()`.",
            Self::NotToBeFalse => "This can be simplified to `toBeTrue()`.",
            Self::NotToBeTrue => "This can be simplified to `toBeFalse()`.",
        }
    }

    fn help(&self) -> &'static str {
        match self {
            Self::ToBeTrue => "Replace with `toBeTrue()`.",
            Self::ToBeFalse => "Replace with `toBeFalse()`.",
            Self::ToBeNull => "Replace with `toBeNull()`.",
            Self::ToBeEmpty => "Replace with `toBeEmpty()`.",
            Self::NotToBeFalse => "Replace `->not->toBeFalse()` with `->toBeTrue()`.",
            Self::NotToBeTrue => "Replace `->not->toBeTrue()` with `->toBeFalse()`.",
        }
    }

    /// Returns the replacement method call text.
    fn replacement(&self) -> &'static str {
        match self {
            Self::ToBeTrue => "toBeTrue()",
            Self::ToBeFalse => "toBeFalse()",
            Self::ToBeNull => "toBeNull()",
            Self::ToBeEmpty => "toBeEmpty()",
            Self::NotToBeFalse => "->toBeTrue()",
            Self::NotToBeTrue => "->toBeFalse()",
        }
    }

    /// Returns whether this is a double-negative pattern (not->toBeX).
    fn is_double_negative(&self) -> bool {
        matches!(self, Self::NotToBeFalse | Self::NotToBeTrue)
    }
}

impl LintRule for UseSpecificExpectationsRule {
    type Config = UseSpecificExpectationsConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Use Specific Expectations",
            code: "use-specific-expectations",
            description: indoc! {"
                Use dedicated matchers instead of generic comparisons in Pest tests.

                This rule suggests more specific matchers for common patterns:
                - `toBe(true)` / `toEqual(true)` -> `toBeTrue()`
                - `toBe(false)` / `toEqual(false)` -> `toBeFalse()`
                - `toBe(null)` / `toEqual(null)` -> `toBeNull()`
                - `toBe([])` / `toBe('')` -> `toBeEmpty()`
                - `not->toBeFalse()` -> `toBeTrue()`
                - `not->toBeTrue()` -> `toBeFalse()`

                Using dedicated matchers provides clearer intent and better error messages.
            "},
            good_example: indoc! {r#"
                <?php

                test('specific matchers', function () {
                    expect($value)->toBeTrue();
                    expect($value)->toBeFalse();
                    expect($value)->toBeNull();
                    expect($array)->toBeEmpty();
                });
            "#},
            bad_example: indoc! {r#"
                <?php

                test('generic comparisons', function () {
                    expect($value)->toBe(true);
                    expect($value)->toBe(false);
                    expect($value)->toBe(null);
                    expect($array)->toBe([]);
                    expect($value)->not->toBeFalse();
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

        if has_not_modifier(method_call) {
            if method_name_equals(method_call, "toBeFalse") {
                if let Some(expect_call) = find_expect_call(method_call)
                    && get_expect_argument(ctx, expect_call).is_some()
                {
                    self.report(ctx, method_call, IssueKind::NotToBeFalse);
                    return;
                }
            } else if method_name_equals(method_call, "toBeTrue")
                && let Some(expect_call) = find_expect_call(method_call)
                && get_expect_argument(ctx, expect_call).is_some()
            {
                self.report(ctx, method_call, IssueKind::NotToBeTrue);
                return;
            }
        }

        if method_name_matches_any(method_call, &["toBe", "toEqual"]).is_none() {
            return;
        };

        let Some(expect_call) = find_expect_call(method_call) else {
            return;
        };

        if get_expect_argument(ctx, expect_call).is_none() {
            return;
        }

        let Some(first_arg) = get_first_argument(method_call) else {
            return;
        };

        let arg_value = first_arg.value();

        let issue_kind = match arg_value {
            Expression::Literal(Literal::True(_)) => IssueKind::ToBeTrue,
            Expression::Literal(Literal::False(_)) => IssueKind::ToBeFalse,
            Expression::Literal(Literal::Null(_)) => IssueKind::ToBeNull,
            _ if is_empty_array(arg_value) || is_empty_string(arg_value) => IssueKind::ToBeEmpty,
            _ => return,
        };

        self.report(ctx, method_call, issue_kind);
    }
}

impl UseSpecificExpectationsRule {
    fn report(&self, ctx: &mut LintContext<'_, '_>, method_call: &MethodCall<'_>, kind: IssueKind) {
        let issue = Issue::new(self.cfg.level(), kind.message())
            .with_code(self.meta.code)
            .with_annotation(Annotation::primary(method_call.span()).with_message(kind.annotation()))
            .with_help(kind.help());

        let fix_span = if kind.is_double_negative() {
            find_not_to_method_span(method_call)
        } else {
            Some(method_call.method.span().join(method_call.argument_list.span()))
        };

        if let Some(span) = fix_span {
            ctx.collector.propose(issue, |edits| {
                edits.push(TextEdit::replace(span, kind.replacement()));
            });
        } else {
            ctx.collector.report(issue);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    // Success cases
    test_lint_success! {
        name = correct_to_be_true,
        rule = UseSpecificExpectationsRule,
        code = indoc! {r#"
            <?php

            expect($value)->toBeTrue();
            expect($flag)->toBeFalse();
        "#}
    }

    test_lint_success! {
        name = correct_to_be_null,
        rule = UseSpecificExpectationsRule,
        code = indoc! {r#"
            <?php

            expect($value)->toBeNull();
        "#}
    }

    test_lint_success! {
        name = correct_to_be_empty,
        rule = UseSpecificExpectationsRule,
        code = indoc! {r#"
            <?php

            expect($array)->toBeEmpty();
            expect($string)->toBeEmpty();
        "#}
    }

    test_lint_success! {
        name = non_literal_values,
        rule = UseSpecificExpectationsRule,
        code = indoc! {r#"
            <?php

            expect($value)->toBe(42);
            expect($text)->toBe('hello');
            expect($result)->toEqual($expected);
            expect($array)->toBe([1, 2, 3]);
        "#}
    }

    // Failure cases - toBe/toEqual with literals
    test_lint_failure! {
        name = to_be_true_literal,
        rule = UseSpecificExpectationsRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect($value)->toBe(true);
        "#}
    }

    test_lint_failure! {
        name = to_be_false_literal,
        rule = UseSpecificExpectationsRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect($flag)->toBe(false);
        "#}
    }

    test_lint_failure! {
        name = to_equal_true_false,
        rule = UseSpecificExpectationsRule,
        count = 2,
        code = indoc! {r#"
            <?php

            expect($value)->toEqual(true);
            expect($flag)->toEqual(false);
        "#}
    }

    test_lint_failure! {
        name = to_be_null_literal,
        rule = UseSpecificExpectationsRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect($value)->toBe(null);
        "#}
    }

    test_lint_failure! {
        name = to_equal_null,
        rule = UseSpecificExpectationsRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect($value)->toEqual(null);
        "#}
    }

    test_lint_failure! {
        name = to_be_empty_array,
        rule = UseSpecificExpectationsRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect($array)->toBe([]);
        "#}
    }

    test_lint_failure! {
        name = to_be_empty_string,
        rule = UseSpecificExpectationsRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect($string)->toBe('');
        "#}
    }

    test_lint_failure! {
        name = to_equal_empty_array,
        rule = UseSpecificExpectationsRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect($array)->toEqual([]);
        "#}
    }

    // Failure cases - double negatives
    test_lint_failure! {
        name = not_to_be_false,
        rule = UseSpecificExpectationsRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect($value)->not->toBeFalse();
        "#}
    }

    test_lint_failure! {
        name = not_to_be_true,
        rule = UseSpecificExpectationsRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect($value)->not->toBeTrue();
        "#}
    }

    // Case insensitivity tests
    test_lint_failure! {
        name = case_insensitive_to_be,
        rule = UseSpecificExpectationsRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect($value)->TOBE(true);
        "#}
    }

    test_lint_failure! {
        name = case_insensitive_to_equal,
        rule = UseSpecificExpectationsRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect($value)->ToEqual(null);
        "#}
    }

    test_lint_failure! {
        name = case_insensitive_not_to_be_false,
        rule = UseSpecificExpectationsRule,
        count = 1,
        code = indoc! {r#"
            <?php

            expect($value)->not->TOBEFALSE();
        "#}
    }

    // Multiple violations
    test_lint_failure! {
        name = multiple_violations,
        rule = UseSpecificExpectationsRule,
        count = 6,
        code = indoc! {r#"
            <?php

            expect($a)->toBe(true);
            expect($b)->toBe(false);
            expect($c)->toBe(null);
            expect($d)->toBe([]);
            expect($e)->not->toBeFalse();
            expect($f)->not->toBeTrue();
        "#}
    }
}
