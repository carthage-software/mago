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
pub struct PslArrayFunctionsRule {
    meta: &'static RuleMeta,
    cfg: PslArrayFunctionsConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct PslArrayFunctionsConfig {
    pub level: Level,
}

impl Default for PslArrayFunctionsConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for PslArrayFunctionsConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for PslArrayFunctionsRule {
    type Config = PslArrayFunctionsConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Psl Array Functions",
            code: "psl-array-functions",
            description: indoc! {"
                This rule enforces the usage of Psl array functions over their PHP counterparts.
                Psl array functions are preferred because they are type-safe and provide more consistent behavior.
            "},
            good_example: indoc! {"
                <?php

                $filtered = Psl\\Vec\\filter($xs, fn($x) => $x > 2);
            "},
            bad_example: indoc! {"
                <?php

                $filtered = array_filter($xs, fn($x) => $x > 2);
            "},
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

        let target_functions: Vec<&'static str> = ARRAY_FUNCTION_REPLACEMENTS.keys().copied().collect();

        if let Some(matched_name) = function_call_matches_any(ctx, function_call, &target_functions) {
            let replacements = &ARRAY_FUNCTION_REPLACEMENTS[matched_name];

            ctx.collector.report(
                Issue::new(
                    self.cfg.level(),
                    "Use the Psl array function instead of the PHP counterpart.",
                )
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(function_call.function.span())
                        .with_message("This is a native PHP array function"),
                )
                .with_note("Psl array functions are preferred because they are type-safe and provide more consistent behavior.")
                .with_help(format!("Use {} instead.", format_replacements(replacements))),
            );
        }
    }
}

static ARRAY_FUNCTION_REPLACEMENTS: LazyLock<HashMap<&'static str, Vec<&'static str>>> = LazyLock::new(|| {
    HashMap::from_iter([
        ("array_all", vec!["Psl\\Iter\\all"]),
        ("array_any", vec!["Psl\\Iter\\any"]),
        ("array_chunk", vec!["Psl\\Vec\\chunk", "Psl\\Vec\\chunk_with_keys"]),
        ("array_combine", vec!["Psl\\Dict\\associate"]),
        ("array_diff", vec!["Psl\\Dict\\diff"]),
        ("array_diff_key", vec!["Psl\\Dict\\diff_by_key"]),
        ("array_fill", vec!["Psl\\Vec\\fill"]),
        (
            "array_filter",
            vec![
                "Psl\\Vec\\filter",
                "Psl\\Vec\\filter_keys",
                "Psl\\Vec\\filter_with_keys",
                "Psl\\Vec\\filter_nulls",
                "Psl\\Dict\\filter",
                "Psl\\Dict\\filter_keys",
                "Psl\\Dict\\filter_with_keys",
                "Psl\\Dict\\filter_nulls",
            ],
        ),
        ("array_flip", vec!["Psl\\Dict\\flip"]),
        ("array_find", vec!["Psl\\Iter\\search", "Psl\\Iter\\search_with_keys", "Psl\\Iter\\search_keys"]),
        ("array_intersect", vec!["Psl\\Dict\\intersect"]),
        ("array_intersect_key", vec!["Psl\\Dict\\intersect_by_key"]),
        ("array_key_exists", vec!["Psl\\Iter\\contains_key"]),
        ("array_key_first", vec!["Psl\\Iter\\first_key"]),
        ("array_key_last", vec!["Psl\\Iter\\last_key"]),
        ("array_keys", vec!["Psl\\Vec\\keys"]),
        (
            "array_map",
            vec![
                "Psl\\Vec\\map",
                "Psl\\Vec\\map_with_key",
                "Psl\\Dict\\map",
                "Psl\\Dict\\map_keys",
                "Psl\\Dict\\map_with_key",
            ],
        ),
        ("array_merge", vec!["Psl\\Vec\\concat", "Psl\\Dict\\merge"]),
        ("array_rand", vec!["Psl\\Iter\\random"]),
        ("array_reduce", vec!["Psl\\Iter\\reduce", "Psl\\Iter\\reduce_keys", "Psl\\Iter\\reduce_with_keys"]),
        ("array_reverse", vec!["Psl\\Vec\\reverse"]),
        (
            "array_slice",
            vec![
                "Psl\\Vec\\slice",
                "Psl\\Vec\\take",
                "Psl\\Vec\\take_while",
                "Psl\\Vec\\drop",
                "Psl\\Vec\\drop_while",
                "Psl\\Dict\\slice",
                "Psl\\Dict\\take",
                "Psl\\Dict\\take_while",
                "Psl\\Dict\\drop",
                "Psl\\Dict\\drop_while",
            ],
        ),
        ("array_sum", vec!["Psl\\Math\\sum", "Psl\\Math\\sum_floats"]),
        (
            "array_unique",
            vec![
                "Psl\\Vec\\unique",
                "Psl\\Vec\\unique_by",
                "Psl\\Vec\\unique_scalar",
                "Psl\\Dict\\unique",
                "Psl\\Dict\\unique_by",
                "Psl\\Dict\\unique_scalar",
            ],
        ),
        ("array_walk", vec!["Psl\\Iter\\apply"]),
        ("uasort", vec!["Psl\\Dict\\sort", "Psl\\Dict\\sort_by", "Psl\\Vec\\sort_by"]),
        ("asort", vec!["Psl\\Dict\\sort", "Psl\\Dict\\sort_by", "Psl\\Vec\\sort_by"]),
        ("uksort", vec!["Psl\\Dict\\sort_by_key"]),
        ("ksort", vec!["Psl\\Dict\\sort_by_key"]),
        ("usort", vec!["Psl\\Vec\\sort", "Psl\\Vec\\sort_by"]),
        ("sort", vec!["Psl\\Vec\\sort", "Psl\\Vec\\sort_by"]),
        ("array_values", vec!["Psl\\Vec\\values"]),
        ("sizeof", vec!["Psl\\Iter\\count"]),
        ("count", vec!["Psl\\Iter\\count"]),
        ("in_array", vec!["Psl\\Iter\\contains"]),
        ("shuffle", vec!["Psl\\Iter\\shuffle"]),
    ])
});
