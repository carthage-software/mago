use indoc::indoc;
use schemars::JsonSchema;

use mago_allocator::Arena;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::cst::MatchArm;
use mago_syntax::cst::Node;
use mago_syntax::cst::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::const_value::ConstantValue;
use crate::rule::utils::const_value::get_constant_value;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoDuplicateMatchArmRule {
    meta: &'static RuleMeta,
    cfg: NoDuplicateMatchArmConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct NoDuplicateMatchArmConfig {
    pub level: Level,
}

impl Default for NoDuplicateMatchArmConfig {
    fn default() -> Self {
        Self { level: Level::Note }
    }
}

impl Config for NoDuplicateMatchArmConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoDuplicateMatchArmRule {
    type Config = NoDuplicateMatchArmConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Duplicate Match Arm",
            code: "no-duplicate-match-arm",
            description: indoc! {"
                Detects `match` arms that produce the same result as an earlier arm.

                Arms with an identical result can be merged into a single arm by
                combining their conditions, which removes the duplication. The result
                is compared by its constant value, so `1 + 2` and `-(-3)` are treated
                as the same result.
            "},
            good_example: indoc! {r#"
                <?php

                $result = match ($value) {
                    1 => 'low',
                    2, 3 => 'high',
                };
            "#},
            bad_example: indoc! {r#"
                <?php

                $result = match ($value) {
                    1 => 'low',
                    2 => 'high',
                    3 => 'high',
                };
            "#},
            category: Category::Redundancy,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Match];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let Node::Match(match_expression) = node else {
            return;
        };

        let mut seen: Vec<(ConstantValue<'arena>, Span)> = Vec::new();
        for arm in match_expression.arms.iter() {
            let MatchArm::Expression(arm) = arm else {
                continue;
            };

            let Some(value) = get_constant_value(ctx.resolved_names, arm.expression) else {
                continue;
            };

            let body_span = arm.expression.span();
            if let Some((_, first_span)) = seen.iter().find(|(seen_value, _)| *seen_value == value) {
                let issue = Issue::new(self.cfg.level(), "This `match` arm has the same result as an earlier arm.")
                    .with_code(self.meta.code)
                    .with_annotation(Annotation::primary(body_span).with_message("This result is a duplicate"))
                    .with_annotation(
                        Annotation::secondary(*first_span).with_message("The same result first appears here"),
                    )
                    .with_help("Merge these arms by combining their conditions into a single arm.");

                ctx.collector.report(issue);
            } else {
                seen.push((value, body_span));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use indoc::indoc;

    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_failure! {
        name = duplicate_literal_results_are_reported,
        rule = NoDuplicateMatchArmRule,
        count = 2,
        code = indoc! {r#"
            <?php

            $result = match ($value) {
                1 => 'high',
                2 => 'high', // duplicate of arm 1
                3 => 'high', // duplicate of arm 1
            };
        "#},
    }

    test_lint_failure! {
        name = computed_results_are_compared_by_value,
        rule = NoDuplicateMatchArmRule,
        count = 1,
        code = indoc! {r#"
            <?php

            $result = match ($value) {
                1 => 1 + 2,  // 3
                2 => -(-3),  // 3 - duplicate of arm 1
            };
        "#},
    }

    test_lint_failure! {
        name = boolean_negation_is_folded,
        rule = NoDuplicateMatchArmRule,
        count = 1,
        code = indoc! {r#"
            <?php

            $result = match ($value) {
                1 => !23,   // false
                2 => false, // false - duplicate of arm 1
            };
        "#},
    }

    test_lint_failure! {
        name = bitwise_and_comparison_are_folded,
        rule = NoDuplicateMatchArmRule,
        count = 3,
        code = indoc! {r#"
            <?php

            $result = match ($value) {
                1 => 1 << 2,    // 4
                2 => 2 ** 2,    // 4    - duplicate of arm 1
                3 => 2 + 2,     // 4    - duplicate of arm 1
                4 => 2 > 1,     // true
                5 => true,      // true - duplicate of arm 4
            };
        "#},
    }

    test_lint_success! {
        name = division_by_zero_is_not_folded,
        rule = NoDuplicateMatchArmRule,
        code = indoc! {r#"
            <?php

            $result = match ($value) {
                1 => 1 / 0,
                2 => 1 / 0,
            };
        "#},
    }

    test_lint_success! {
        name = distinct_results_are_allowed,
        rule = NoDuplicateMatchArmRule,
        code = indoc! {r#"
            <?php

            $result = match ($value) {
                1 => 'low',
                2, 3 => 'high',
            };
        "#},
    }

    test_lint_success! {
        name = non_constant_results_are_ignored,
        rule = NoDuplicateMatchArmRule,
        code = indoc! {r#"
            <?php

            $result = match ($value) {
                1 => foo(),
                2 => foo(),
            };
        "#},
    }
}
