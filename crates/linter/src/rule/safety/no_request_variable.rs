use indoc::indoc;
use mago_allocator::Arena;
use schemars::JsonSchema;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

const REQUEST_VARIABLE: &[u8] = b"$_REQUEST";

#[derive(Debug, Clone)]
pub struct NoRequestVariableRule {
    meta: &'static RuleMeta,
    cfg: NoRequestVariableConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct NoRequestVariableConfig {
    pub level: Level,
}

impl Default for NoRequestVariableConfig {
    fn default() -> Self {
        Self { level: Level::Error }
    }
}

impl Config for NoRequestVariableConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoRequestVariableRule {
    type Config = NoRequestVariableConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Request Variable",
            code: "no-request-variable",
            description: indoc! {"
                Detects the use of the `$_REQUEST` variable, which is considered unsafe.

                Use `$_GET`, `$_POST`, or `$_COOKIE` instead for better clarity.
            "},
            good_example: indoc! {r"
                <?php

                $identifier = $_GET['id'];
            "},
            bad_example: indoc! {r"
                <?php

                $identifier = $_REQUEST['id'];
            "},
            category: Category::Safety,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::DirectVariable];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let Node::DirectVariable(direct_variable) = node else {
            return;
        };

        if !REQUEST_VARIABLE.eq(direct_variable.name) {
            return;
        }

        ctx.collector.report(
            Issue::new(self.cfg.level(), "Unsafe use of `$_REQUEST` variable.")
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(direct_variable.span).with_message("The `$_REQUEST` variable is used here"),
                )
                .with_help("Use `$_GET`, `$_POST`, or `$_COOKIE` instead for better clarity."),
        );
    }
}
