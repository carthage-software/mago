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
pub struct CombineConsecutiveIssetsRule {
    meta: &'static RuleMeta,
    cfg: CombineConsecutiveIssetsConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct CombineConsecutiveIssetsConfig {
    pub level: Level,
}

impl Default for CombineConsecutiveIssetsConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for CombineConsecutiveIssetsConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for CombineConsecutiveIssetsRule {
    type Config = CombineConsecutiveIssetsConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Combine Consecutive Issets",
            code: "combine-consecutive-issets",
            description: indoc! {r#"
                Suggests combining consecutive calls to `isset()` when they are joined by a logical AND.

                For example, `isset($a) && isset($b)` can be turned into `isset($a, $b)`, which is more concise
                and avoids repeated function calls. If one or both `isset()` calls are wrapped in parentheses,
                the rule will still warn, but it will not attempt an automated fix.
            "#},
            good_example: indoc! {r#"
                <?php

                if (isset($a, $b)) {
                    // ...
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                if (isset($a) && isset($b)) {
                    // ...
                }
            "#},
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

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::Binary(binary) = node else {
            return;
        };

        let BinaryOperator::And(_) = binary.operator else {
            return;
        };

        let Some((left_parenthesized, left_isset)) = get_isset_construct(binary.lhs, true) else {
            return;
        };
        let Some((right_parenthesized, right_isset)) = get_isset_construct(binary.rhs, false) else {
            return;
        };

        let issue = Issue::new(self.cfg.level, "Consecutive isset calls can be combined.")
            .with_code(self.meta.code)
            .with_annotation(Annotation::primary(left_isset.span()).with_message("first `isset` call"))
            .with_annotation(Annotation::primary(right_isset.span()).with_message("second `isset` call"))
            .with_annotation(
                Annotation::secondary(binary.span()).with_message("these calls are combined with a binary operator"),
            )
            .with_note("Using multiple `isset` calls joined with `&&` is redundant and less idiomatic.")
            .with_help("Combine the calls into a single `isset`, e.g. `isset($a, $b)`.");

        if left_parenthesized || right_parenthesized {
            ctx.collector.report(issue);

            return;
        }

        ctx.collector.propose(issue, |plan| {
            let to_replace = left_isset.right_parenthesis.join(binary.operator.span());
            let to_delete = right_isset.isset.span.join(right_isset.left_parenthesis);

            plan.replace(to_replace.to_range(), ",".to_string(), SafetyClassification::Safe);
            plan.delete(to_delete.to_range(), SafetyClassification::Safe);
        });
    }
}

fn get_isset_construct<'ast, 'arena>(
    mut expression: &'ast Expression<'arena>,
    select_binary_rhs: bool,
) -> Option<(bool, &'ast IssetConstruct<'arena>)> {
    let mut between_parentheses = false;

    while let Expression::Parenthesized(parenthesized) = expression {
        expression = parenthesized.expression;
        between_parentheses = true;
    }

    match expression {
        Expression::Construct(construct) => {
            if let Construct::Isset(isset) = construct {
                Some((between_parentheses, isset))
            } else {
                None
            }
        }
        Expression::Binary(binary) if select_binary_rhs => {
            if let BinaryOperator::And(_) = binary.operator {
                let (lhs_between_parentheses, lhs_isset) = get_isset_construct(binary.rhs, true)?;
                Some((between_parentheses || lhs_between_parentheses, lhs_isset))
            } else {
                None
            }
        }
        _ => None,
    }
}
