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
pub struct PslRandomnessFunctionsRule {
    meta: &'static RuleMeta,
    cfg: PslRandomnessFunctionsConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct PslRandomnessFunctionsConfig {
    pub level: Level,
}

impl Default for PslRandomnessFunctionsConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for PslRandomnessFunctionsConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for PslRandomnessFunctionsRule {
    type Config = PslRandomnessFunctionsConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Psl Randomness Functions",
            code: "psl-randomness-functions",
            description: indoc! {"
                This rule enforces the usage of Psl randomness functions over their PHP counterparts.

                Psl randomness functions are preferred because they are type-safe and provide more consistent behavior.
            "},
            good_example: indoc! {r#"
                <?php

                $randomInt = Psl\SecureRandom\int(0, 10);
            "#},
            bad_example: indoc! {r#"
                <?php

                $randomInt = random_int(0, 10);
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

        let target_functions: Vec<&'static str> = RANDOM_FUNCTION_REPLACEMENTS.keys().copied().collect();
        let Some(matched_name) = function_call_matches_any(ctx, function_call, &target_functions) else {
            return;
        };

        let replacements = &RANDOM_FUNCTION_REPLACEMENTS[matched_name];

        ctx.collector.report(
            Issue::new(
                self.cfg.level(),
                "Use the Psl randomness function instead of the PHP counterpart.",
            )
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(function_call.function.span())
                    .with_message("This is a PHP randomness function"),
            )
            .with_note("Psl randomness functions are preferred because they are type-safe and provide more consistent behavior.")
            .with_help(format!("Use {} instead.", format_replacements(replacements))),
        );
    }
}

static RANDOM_FUNCTION_REPLACEMENTS: LazyLock<HashMap<&'static str, Vec<&'static str>>> = LazyLock::new(|| {
    HashMap::from_iter([
        ("rand", vec!["Psl\\PseudoRandom\\int", "Psl\\PseudoRandom\\float"]),
        ("mt_rand", vec!["Psl\\PseudoRandom\\int", "Psl\\PseudoRandom\\float"]),
        ("random_int", vec!["Psl\\SecureRandom\\int", "Psl\\SecureRandom\\float"]),
        ("random_bytes", vec!["Psl\\SecureRandom\\bytes", "Psl\\SecureRandom\\string"]),
    ])
});
