use std::sync::LazyLock;

use ahash::HashMap;
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
use crate::integration::Integration;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::call::function_call_matches_any;
use crate::rule::utils::format_replacements;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct PslSleepFunctionsRule {
    meta: &'static RuleMeta,
    cfg: PslSleepFunctionsConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct PslSleepFunctionsConfig {
    pub level: Level,
}

impl Default for PslSleepFunctionsConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for PslSleepFunctionsConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for PslSleepFunctionsRule {
    type Config = PslSleepFunctionsConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Psl Sleep Functions",
            code: "psl-sleep-functions",
            description: indoc! {"
                This rule enforces the usage of Psl sleep functions over their PHP counterparts.

                Psl sleep functions are preferred because they are type-safe, provide more consistent behavior,
                and allow other tasks within the event loop to continue executing while the current Fiber pauses.
            "},
            good_example: indoc! {r#"
                <?php

                use Psl\Async;
                use Psl\DateTime;

                Async\sleep(DateTime\Duration::seconds(1));
            "#},
            bad_example: indoc! {r#"
                <?php

                sleep(1);
            "#},
            category: Category::BestPractices,

            requirements: RuleRequirements::Integration(Integration::Psl),
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

        let target_functions: Vec<&'static str> = SLEEP_FUNCTION_REPLACEMENTS.keys().copied().collect();
        let Some(matched_name) = function_call_matches_any(ctx, function_call, &target_functions) else {
            return;
        };

        let replacements = &SLEEP_FUNCTION_REPLACEMENTS[matched_name];

        ctx.collector.report(
            Issue::new(
                self.cfg.level(),
                "Use the Psl sleep function instead of the PHP counterpart.",
            )
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(function_call.function.span())
                    .with_message("This is a PHP sleep function"),
            )
            .with_note("Psl sleep functions are preferred because they are type-safe, provide more consistent behavior, and allow other tasks within the event loop to continue executing while the current Fiber pauses.")
            .with_help(format!("Use {} instead.", format_replacements(replacements))),
        );
    }
}

static SLEEP_FUNCTION_REPLACEMENTS: LazyLock<HashMap<&'static str, Vec<&'static str>>> = LazyLock::new(|| {
    HashMap::from_iter([
        ("sleep", vec!["Psl\\Async\\sleep"]),
        ("usleep", vec!["Psl\\Async\\sleep"]),
        ("time_sleep_until", vec!["Psl\\Async\\sleep"]),
        ("time_nanosleep", vec!["Psl\\Async\\sleep"]),
    ])
});
