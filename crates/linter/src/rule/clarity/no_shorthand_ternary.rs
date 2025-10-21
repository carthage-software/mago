use indoc::indoc;
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
pub struct NoShorthandTernaryRule {
    meta: &'static RuleMeta,
    cfg: NoShorthandTernaryConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoShorthandTernaryConfig {
    pub level: Level,
}

impl Default for NoShorthandTernaryConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoShorthandTernaryConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoShorthandTernaryRule {
    type Config = NoShorthandTernaryConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Shorthand Ternary",
            code: "no-shorthand-ternary",
            description: indoc! {"
                Detects the use of the shorthand ternary and elvis operators.

                Both shorthand ternary operator (`$a ? : $b`) and elvis operator (`$a ?: $b`) relies on loose comparison.
                "},
            good_example: indoc! {"
                <?php

                $value = $foo ?? $default;
                $value = $foo ? $foo : $default;
            "},
            bad_example: indoc! {"
                <?php
                $value = $foo ?: $default;
                $value = $foo ? : $default;
            "},
            category: Category::Clarity,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Conditional];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let issue = match node {
            Node::Conditional(Conditional { then: None, .. }) => {
                Issue::new(self.cfg.level(), "Use of the shorthand ternary operator.")
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(node.span()).with_message("Ambiguous check due to `? :` loose comparison"),
                    )
            }
            _ => return,
        };

        ctx.collector.report(
            issue.with_help("Use null coalesce operator (`??`) or ternary operator with explicit strict comparison."),
        );
    }
}
