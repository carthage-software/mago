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
pub struct PreferWhileLoopRule {
    meta: &'static RuleMeta,
    cfg: PreferWhileLoopConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct PreferWhileLoopConfig {
    pub level: Level,
}

impl Default for PreferWhileLoopConfig {
    fn default() -> Self {
        Self { level: Level::Note }
    }
}

impl Config for PreferWhileLoopConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for PreferWhileLoopRule {
    type Config = PreferWhileLoopConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Prefer While Loop",
            code: "prefer-while-loop",
            description: indoc! {"
                Suggests using a `while` loop instead of a `for` loop when the `for` loop does not have any
                initializations or increments. This can make the code more readable and concise.
            "},
            good_example: indoc! {r#"
                <?php

                while ($i < 10) {
                    echo $i;

                    $i++;
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                for (; $i < 10;) {
                    echo $i;

                    $i++;
                }
            "#},
            category: Category::BestPractices,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::For];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::For(r#for) = node else {
            return;
        };

        if !r#for.initializations.is_empty() || !r#for.increments.is_empty() {
            return;
        }

        let issue = Issue::new(
            self.cfg.level(),
            "Use `while` loop instead of `for` loop.",
        )
        .with_code(self.meta.code)
        .with_annotation(Annotation::primary(r#for.span()).with_message("This `for` loop can be simplified to a `while` loop"))
        .with_note("This `for` loop can be simplified to a `while` loop since it doesn't have initializations or increments.")
        .with_help("Use a `while` loop instead of a `for` loop.");

        ctx.collector.report(issue);
    }
}
