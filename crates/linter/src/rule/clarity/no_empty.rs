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
pub struct NoEmptyRule {
    meta: &'static RuleMeta,
    cfg: NoEmptyConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoEmptyConfig {
    pub level: Level,
}

impl Default for NoEmptyConfig {
    fn default() -> Self {
        Self { level: Level::Error }
    }
}

impl Config for NoEmptyConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoEmptyRule {
    type Config = NoEmptyConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Empty",
            code: "no-empty",
            description: indoc! {"
                Detects the use of the `empty()` construct.

                The `empty()` language construct can lead to ambiguous and potentially buggy code due to
                loose and counterintuitive definition of emptiness. It fails to clearly convey
                developer's intent or expectation, making it preferable to use explicit checks.
            "},
            good_example: indoc! {r#"
                <?php

                if ($myArray === []) {
                    // ...
                }
            "#},
            bad_example: indoc! {r"
                <?php

                if (!empty($myArray)) {
                    // ...
                }
            "},
            category: Category::Clarity,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::EmptyConstruct];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::EmptyConstruct(construct) = node else {
            return;
        };

        ctx.collector.report(
            Issue::new(self.cfg.level(), "Use of the `empty` construct.")
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(construct.span())
                        .with_message("Ambiguous check due to `empty()` loose semantic"),
                )
                .with_note("`empty()` exhibits unexpected behavior on specific value.")
                .with_note("It is unclear what condition is being treated with `empty()`.")
                .with_help("Use strict comparison or specific predicate function to clearly convey your intent."),
        );
    }
}
