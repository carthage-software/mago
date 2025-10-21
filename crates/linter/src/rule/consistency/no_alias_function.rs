use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::*;
use mago_syntax::ast::*;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::call::function_call_matches;
use crate::rule::utils::consts::ALIAS_TO_FUNCTION;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoAliasFunctionRule {
    meta: &'static RuleMeta,
    cfg: NoAliasFunctionConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoAliasFunctionConfig {
    pub level: Level,
}

impl Default for NoAliasFunctionConfig {
    fn default() -> Self {
        Self { level: Level::Note }
    }
}

impl Config for NoAliasFunctionConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoAliasFunctionRule {
    type Config = NoAliasFunctionConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Alias Function",
            code: "no-alias-function",
            description: indoc! {"
                Detects usage of function aliases (e.g., `diskfreespace` instead of `disk_free_space`)
                and suggests calling the canonical (original) function name instead.
                This is primarily for consistency and clarity.
            "},
            good_example: indoc! {r###"
                <?php

                // 'disk_free_space' is the proper name instead of 'diskfreespace'
                $freeSpace = disk_free_space("/");
            "###},
            bad_example: indoc! {r###"
                <?php

                // 'diskfreespace' is an alias for 'disk_free_space'
                $freeSpace = diskfreespace("/");
            "###},
            category: Category::Consistency,

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

        for (function_name, original_name) in ALIAS_TO_FUNCTION.iter() {
            if !function_call_matches(ctx, function_call, function_name) {
                continue;
            }

            let issue = Issue::new(self.cfg.level(), format!("Function alias `{function_name}` should not be used."))
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(function_call.function.span())
                        .with_message(format!("This function is an alias of `{original_name}`")),
                )
                .with_note(format!("The function `{function_name}` is an alias of `{original_name}`."))
                .with_help(format!("Consider using the function `{original_name}` instead."));

            ctx.collector.report(issue);

            break;
        }
    }
}
