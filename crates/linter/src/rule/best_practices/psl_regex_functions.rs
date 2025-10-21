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
pub struct PslRegexFunctionsRule {
    meta: &'static RuleMeta,
    cfg: PslRegexFunctionsConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct PslRegexFunctionsConfig {
    pub level: Level,
}

impl Default for PslRegexFunctionsConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for PslRegexFunctionsConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for PslRegexFunctionsRule {
    type Config = PslRegexFunctionsConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Psl Regex Functions",
            code: "psl-regex-functions",
            description: indoc! {"
                This rule enforces the usage of Psl regex functions over their PHP counterparts.

                Psl regex functions are preferred because they are type-safe and provide more consistent behavior.
            "},
            good_example: indoc! {r#"
                <?php

                $result = Psl\Regex\matches('Hello, World!', '/\w+/');
            "#},
            bad_example: indoc! {r#"
                <?php

                $result = preg_match('/\w+/', 'Hello, World!');
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

        let target_functions: Vec<&'static str> = REGEX_FUNCTION_REPLACEMENTS.keys().copied().collect();
        let Some(matched_name) = function_call_matches_any(ctx, function_call, &target_functions) else {
            return;
        };

        let replacements = &REGEX_FUNCTION_REPLACEMENTS[matched_name];

        ctx.collector.report(
            Issue::new(
                self.cfg.level(),
                "Use the Psl regex function instead of the PHP counterpart.",
            )
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(function_call.function.span())
                    .with_message("This is a PHP regex function"),
            )
            .with_note("Psl regex functions are preferred because they are type-safe and provide more consistent behavior.")
            .with_help(format!("Use {} instead.", format_replacements(replacements))),
        );
    }
}

static REGEX_FUNCTION_REPLACEMENTS: LazyLock<HashMap<&'static str, Vec<&'static str>>> = LazyLock::new(|| {
    HashMap::from_iter([
        ("preg_match", vec!["Psl\\Regex\\matches"]),
        ("preg_match_all", vec!["Psl\\Regex\\matches"]),
        ("preg_replace", vec!["Psl\\Regex\\replace"]),
        ("preg_replace_callback", vec!["Psl\\Regex\\replace_with"]),
        ("preg_replace_callback_array", vec!["Psl\\Regex\\replace_with"]),
        ("preg_split", vec!["Psl\\Regex\\split"]),
        ("preg_grep", vec!["Psl\\Regex\\every_match"]),
        ("preg_filter", vec!["Psl\\Regex\\every_match"]),
    ])
});
