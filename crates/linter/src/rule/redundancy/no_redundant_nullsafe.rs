use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_fixer::SafetyClassification;
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
pub struct NoRedundantNullsafeRule {
    meta: &'static RuleMeta,
    cfg: NoRedundantNullsafeConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoRedundantNullsafeConfig {
    pub level: Level,
}

impl Default for NoRedundantNullsafeConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for NoRedundantNullsafeConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoRedundantNullsafeRule {
    type Config = NoRedundantNullsafeConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Redundant Nullsafe",
            code: "no-redundant-nullsafe",
            description: indoc! {"
                Flags the use of the nullsafe operator (`?->`) in contexts where its null-checking behavior is redundant.

                This occurs in two common situations:
                1. When an expression using `?->` is immediately followed by the null coalescing operator (`??`).
                2. When an expression using `?->` is checked with `isset()`.

                In both scenarios, the surrounding language construct (`??` or `isset()`) already handles `null` values safely,
                making the `?->` operator superfluous and the code unnecessarily verbose.
            "},
            good_example: indoc! {r#"
                <?php

                $name = $user->name ?? 'Guest';

                if (isset($user->profile)) {
                    // Do something with $user->profile
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                $name = $user?->name ?? 'Guest';

                if (isset($user?->profile)) {
                    // Do something with $user->profile
                }
            "#},
            category: Category::Redundancy,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Binary, NodeKind::IssetConstruct];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        match node {
            Node::Binary(Binary { lhs, operator: BinaryOperator::NullCoalesce(op), .. }) => {
                let Expression::Access(Access::NullSafeProperty(null_safe)) = lhs else {
                    return;
                };

                let issue = Issue::new(self.cfg.level(), "The nullsafe operator (`?->`) is redundant when used with `??`.")
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(null_safe.question_mark_arrow)
                            .with_message("This nullsafe operator is redundant..."),
                    )
                    .with_annotation(
                        Annotation::secondary(*op)
                            .with_message("...because this null coalescing operator already provides the fallback.")
                    )
                    .with_note("The `??` operator already handles cases where the left-hand side is `null`, making the `?->` operator unnecessary.")
                    .with_help("Replace `?->` with the standard `->` operator for clarity.");

                ctx.collector.propose(issue, |plan| {
                    plan.replace(null_safe.question_mark_arrow.to_range(), "->", SafetyClassification::Safe);
                });
            }
            Node::IssetConstruct(construct) => {
                for value in construct.values.iter() {
                    let Expression::Access(Access::NullSafeProperty(null_safe)) = value else {
                        continue;
                    };

                    let issue = Issue::new(self.cfg.level(), "The nullsafe operator (`?->`) is redundant in `isset` checks.")
                        .with_code(self.meta.code)
                        .with_annotation(
                            Annotation::primary(null_safe.question_mark_arrow.span())
                                .with_message("This nullsafe operator is redundant...")
                        )
                        .with_annotation(
                            Annotation::secondary(construct.isset.span())
                                .with_message("...because `isset` already checks for the existence of the property.")
                        )
                        .with_note("Using the nullsafe operator in `isset` checks is unnecessary because `isset` inherently checks for the existence of the property, making the `?->` operator superfluous.")
                        .with_help("Replace `?->` with the standard `->` operator for clarity.");

                    ctx.collector.propose(issue, |plan| {
                        plan.replace(null_safe.question_mark_arrow.to_range(), "->", SafetyClassification::Safe);
                    });
                }
            }
            _ => {}
        }
    }
}
