use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_fixer::SafetyClassification;
use mago_php_version::PHPVersion;
use mago_php_version::PHPVersionRange;
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
use crate::rule::utils::call::function_call_matches;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

const STR_CONTAINS: &str = "str_contains";
const STRPOS: &str = "strpos";

#[derive(Debug, Clone)]
pub struct StrContainsRule {
    meta: &'static RuleMeta,
    cfg: StrContainsConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct StrContainsConfig {
    pub level: Level,
}

impl Default for StrContainsConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for StrContainsConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for StrContainsRule {
    type Config = StrContainsConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Str Contains",
            code: "str-contains",
            description: indoc! {"
                Detects `strpos($a, $b) !== false` and `strpos($a, $b) === false` comparisons and suggests
                replacing them with `str_contains($a, $b)` or `!str_contains($a, $b)` for improved readability
                and intent clarity.
            "},
            good_example: indoc! {r#"
                <?php

                $a = 'hello world';
                $b = 'world';

                if (str_contains($a, $b)) {
                    echo 'Found';
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                $a = 'hello world';
                $b = 'world';

                if (strpos($a, $b) !== false) {
                    echo 'Found';
                }
            "#},
            category: Category::Clarity,
            requirements: RuleRequirements::PHPVersion(PHPVersionRange::from(PHPVersion::PHP80)),
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
        let Node::Binary(binary) = node else { return };

        // Check if this is a comparison with false
        // Only strict equality is safe to replace:
        // !== false → str_contains
        // === false → !str_contains
        //
        // Loose equality (==, !=, <>) is UNSAFE because strpos can return 0:
        // strpos('hello', 'hello') returns 0
        // 0 == false is true, but 0 === false is false
        let is_strict_equality =
            matches!(binary.operator, BinaryOperator::Identical(_) | BinaryOperator::NotIdentical(_));
        let needs_negation = matches!(binary.operator, BinaryOperator::Identical(_));

        if !matches!(
            binary.operator,
            BinaryOperator::NotIdentical(_)
                | BinaryOperator::NotEqual(_)
                | BinaryOperator::AngledNotEqual(_)
                | BinaryOperator::Identical(_)
                | BinaryOperator::Equal(_)
        ) {
            return;
        }

        let (left, call) = match (binary.lhs, binary.rhs) {
            (
                Expression::Call(Call::Function(call @ FunctionCall { argument_list: arguments, .. })),
                Expression::Literal(Literal::False(_)),
            ) if arguments.arguments.len() == 2 => (true, call),
            (
                Expression::Literal(Literal::False(_)),
                Expression::Call(Call::Function(call @ FunctionCall { argument_list: arguments, .. })),
            ) if arguments.arguments.len() == 2 => (false, call),
            _ => {
                return;
            }
        };

        if !function_call_matches(ctx, call, STRPOS) {
            return;
        }

        // For non-strict equality, report issue but don't provide a fix
        if !is_strict_equality {
            let issue = Issue::new(
                self.cfg.level,
                "Using non-strict equality with `strpos` can lead to unexpected behavior.",
            )
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(binary.span())
                    .with_message("Non-strict comparison with `strpos` is potentially unsafe"),
            )
            .with_note("When the needle is found at position 0, `strpos` returns 0, which is loosely equal to `false`.")
            .with_note("This means `strpos($haystack, $needle) == false` is true when the needle is at position 0.")
            .with_note("Use strict equality (=== or !==) to avoid this issue.")
            .with_help("Replace `==` with `===` or `!=` with `!==`, then you can safely use `str_contains`.");

            ctx.collector.report(issue);
            return;
        }

        let (message, help_text) = if needs_negation {
            (
                "Consider replacing `strpos` with `!str_contains` for improved readability and intent clarity.",
                "`strpos($a, $b) === false` can be simplified to `!str_contains($a, $b)`.",
            )
        } else {
            (
                "Consider replacing `strpos` with `str_contains` for improved readability and intent clarity.",
                "`strpos($a, $b) !== false` can be simplified to `str_contains($a, $b)`.",
            )
        };

        let issue = Issue::new(self.cfg.level, message)
            .with_code(self.meta.code)
            .with_annotation(Annotation::primary(binary.span()).with_message("This comparison can be simplified."))
            .with_help(help_text)
            .with_note("Using `str_contains` makes the code easier to understand and more expressive.");

        ctx.collector.propose(issue, |plan| {
            let function_span = call.function.span();

            if needs_negation {
                // For === false, we need to add ! before str_contains
                // Replace entire binary expression with !str_contains(...)
                if left {
                    // strpos(...) === false
                    plan.replace(function_span.to_range(), format!("!{}", STR_CONTAINS), SafetyClassification::Safe);
                    plan.delete(binary.operator.span().join(binary.rhs.span()).to_range(), SafetyClassification::Safe);
                } else {
                    // false === strpos(...)
                    plan.delete(binary.lhs.span().join(binary.operator.span()).to_range(), SafetyClassification::Safe);
                    plan.replace(function_span.to_range(), format!("!{}", STR_CONTAINS), SafetyClassification::Safe);
                }
            } else {
                // For !== false, just replace with str_contains
                plan.replace(function_span.to_range(), STR_CONTAINS.to_string(), SafetyClassification::Safe);

                // Remove comparison part
                if left {
                    plan.delete(binary.operator.span().join(binary.rhs.span()).to_range(), SafetyClassification::Safe);
                } else {
                    plan.delete(binary.lhs.span().join(binary.operator.span()).to_range(), SafetyClassification::Safe);
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::StrContainsRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = str_contains_is_preferred,
        rule = StrContainsRule,
        code = indoc! {r#"
            <?php

            $haystack = 'hello world';
            $needle = 'world';

            if (str_contains($haystack, $needle)) {
                echo 'Found';
            }

            if (!str_contains($haystack, 'foo')) {
                echo 'Not found';
            }
        "#}
    }

    test_lint_failure! {
        name = strpos_not_identical_false_should_use_str_contains,
        rule = StrContainsRule,
        code = indoc! {r#"
            <?php

            if (strpos('hello', 'world') !== false) {
                echo 'Found';
            }
        "#}
    }

    test_lint_failure! {
        name = strpos_identical_false_should_use_negated_str_contains,
        rule = StrContainsRule,
        code = indoc! {r#"
            <?php

            if (strpos('hello', 'world') === false) {
                echo 'Not found';
            }
        "#}
    }

    test_lint_failure! {
        name = strpos_loose_equal_false_warns_about_unsafe_comparison,
        rule = StrContainsRule,
        code = indoc! {r#"
            <?php

            if (strpos('hello', 'world') == false) {
                echo 'Not found';
            }
        "#}
    }

    test_lint_failure! {
        name = strpos_not_equal_false_warns_about_unsafe_comparison,
        rule = StrContainsRule,
        code = indoc! {r#"
            <?php

            if (strpos('hello', 'world') != false) {
                echo 'Found';
            }
        "#}
    }

    test_lint_failure! {
        name = strpos_angled_not_equal_false_warns_about_unsafe_comparison,
        rule = StrContainsRule,
        code = indoc! {r#"
            <?php

            if (strpos('hello', 'world') <> false) {
                echo 'Found';
            }
        "#}
    }

    test_lint_failure! {
        name = false_on_left_side_also_detected,
        rule = StrContainsRule,
        code = indoc! {r#"
            <?php

            if (false !== strpos('hello', 'world')) {
                echo 'Found';
            }
        "#}
    }

    test_lint_failure! {
        name = multiple_strpos_comparisons_strict_and_loose,
        rule = StrContainsRule,
        count = 4,
        code = indoc! {r#"
            <?php

            // Strict - can be safely replaced
            if (strpos($a, $b) !== false) {
                echo 'Found';
            }

            // Strict - can be safely replaced
            if (strpos($c, $d) === false) {
                echo 'Not found';
            }

            // Loose - unsafe, only warning
            if (strpos($e, $f) == false) {
                echo 'Maybe not found';
            }

            // Loose - unsafe, only warning
            if (strpos($g, $h) != false) {
                echo 'Maybe found';
            }
        "#}
    }
}
