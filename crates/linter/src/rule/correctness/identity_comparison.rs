use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_syntax::ast::*;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct IdentityComparisonRule {
    meta: &'static RuleMeta,
    cfg: IdentityComparisonConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct IdentityComparisonConfig {
    pub level: Level,
}

impl Default for IdentityComparisonConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for IdentityComparisonConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for IdentityComparisonRule {
    type Config = IdentityComparisonConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Identity Comparison",
            code: "identity-comparison",
            description: indoc! {"
                Detects equality and inequality comparisons that should use identity comparison operators.
            "},
            good_example: indoc! {r#"
                <?php

                if ($a === $b) {
                    echo '$a is same as $b';
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                if ($a == $b) {
                    echo '$a is same as $b';
                }
            "#},
            category: Category::Correctness,

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

        match &binary.operator {
            BinaryOperator::Equal(span) => {
                let issue = Issue::new(
                    self.cfg.level(),
                    "Use identity comparison `===` instead of equality comparison `==`."
                )
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(*span).with_message("Equality operator is used here"))
                .with_note("Identity comparison `===` checks for both value and type equality, while equality comparison `==` performs type coercion, which can lead to unexpected results.")
                .with_help("Use `===` to ensure both value and type are equal.");

                ctx.collector.report(issue);
            }
            BinaryOperator::NotEqual(span) => {
                let issue = Issue::new(
                    self.cfg.level(),
                    "Use identity inequality `!==` instead of inequality comparison `!=`."
                )
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(*span).with_message("Inequality operator is used here"))
                .with_note("Identity inequality `!==` checks for both value and type inequality, while inequality comparison `!=` performs type coercion, which can lead to unexpected results.")
                .with_help("Use `!==` to ensure both value and type are different.");

                ctx.collector.report(issue);
            }
            BinaryOperator::AngledNotEqual(span) => {
                let issue = Issue::new(
                    self.cfg.level(),
                    "Use identity inequality `!==` instead of angled inequality comparison `<>`."
                )
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(*span).with_message("Angled inequality operator is used here"))
                .with_note("Identity inequality `!==` checks for both value and type inequality, while angled inequality comparison `<>` performs type coercion, which can lead to unexpected results.")
                .with_help("Use `!==` to ensure both value and type are different.");

                ctx.collector.report(issue);
            }
            _ => {}
        }
    }
}
