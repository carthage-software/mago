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
use crate::rule::utils::call::function_call_matches;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct AssertDescriptionRule {
    meta: &'static RuleMeta,
    cfg: AssertDescriptionConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct AssertDescriptionConfig {
    pub level: Level,
}

impl Default for AssertDescriptionConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for AssertDescriptionConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for AssertDescriptionRule {
    type Config = AssertDescriptionConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Assert Description",
            code: "assert-description",
            description: indoc! {"
                Detects assert functions that do not have a description.

                Assert functions should have a description to make it easier to understand the purpose of the assertion.
            "},
            good_example: indoc! {r###"
                <?php

                assert($user->isActivated(), 'User MUST be activated at this point.');
            "###},
            bad_example: indoc! {r###"
                <?php

                assert($user->isActivated());
            "###},
            category: Category::Correctness,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::FunctionCall];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::FunctionCall(function_call) = node else {
            return;
        };

        if !function_call_matches(ctx, function_call, "assert") {
            return;
        }

        if function_call.argument_list.arguments.get(1).is_none() {
            ctx.collector.report(
                Issue::new(
                    self.cfg.level(),
                    "Missing description in assert function.",
                )
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(function_call.span()).with_message("`assert` function is called here"))
                .with_note("Assert functions should have a description to make it easier to understand the purpose of the assertion.")
                .with_help("Add a description to the assert function to clarify its purpose."),
            );
        }
    }
}
