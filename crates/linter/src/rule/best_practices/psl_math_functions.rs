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
pub struct PslMathFunctionsRule {
    meta: &'static RuleMeta,
    cfg: PslMathFunctionsConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct PslMathFunctionsConfig {
    pub level: Level,
}

impl Default for PslMathFunctionsConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for PslMathFunctionsConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for PslMathFunctionsRule {
    type Config = PslMathFunctionsConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Psl Math Functions",
            code: "psl-math-functions",
            description: indoc! {"
                This rule enforces the usage of Psl math functions over their PHP counterparts.
                Psl math functions are preferred because they are type-safe and provide more consistent behavior.
            "},
            good_example: indoc! {r#"
                <?php

                $abs = Psl\Math\abs($number);
            "#},
            bad_example: indoc! {r#"
                <?php

                $abs = abs($number);
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
        let Node::FunctionCall(function_call) = node else { return };
        let Expression::Identifier(identifier) = function_call.function else { return };

        let target_functions: Vec<&'static str> = MATH_FUNCTION_REPLACEMENTS.keys().copied().collect();

        if let Some(matched_name) = function_call_matches_any(ctx, function_call, &target_functions) {
            let replacements = &MATH_FUNCTION_REPLACEMENTS[matched_name];

            ctx.collector.report(
                Issue::new(
                    self.cfg.level(),
                    "Use the Psl math function instead of the PHP counterpart.",
                )
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(identifier.span())
                        .with_message("This is a PHP math function"),
                )
                .with_note("Psl math functions are preferred because they are type-safe and provide more consistent behavior.")
                .with_help(format!("Use `{}` instead.", format_replacements(replacements))),
            );
        }
    }
}

static MATH_FUNCTION_REPLACEMENTS: LazyLock<HashMap<&'static str, Vec<&'static str>>> = LazyLock::new(|| {
    HashMap::from_iter([
        ("abs", vec!["Psl\\Math\\abs"]),
        ("acos", vec!["Psl\\Math\\acos"]),
        ("acos", vec!["Psl\\Math\\acos"]),
        ("asin", vec!["Psl\\Math\\asin"]),
        ("atan", vec!["Psl\\Math\\atan2"]),
        ("base_convert", vec!["Psl\\Math\\base_convert"]),
        ("ceil", vec!["Psl\\Math\\ceil"]),
        ("cos", vec!["Psl\\Math\\cos"]),
        ("intdiv", vec!["Psl\\Math\\div"]),
        ("exp", vec!["Psl\\Math\\exp"]),
        ("floor", vec!["Psl\\Math\\floor"]),
        ("hexdec", vec!["Psl\\Math\\from_base"]),
        ("bindec", vec!["Psl\\Math\\from_base"]),
        ("decbin", vec!["Psl\\Math\\to_base"]),
        ("dechex", vec!["Psl\\Math\\to_base"]),
        ("decoct", vec!["Psl\\Math\\to_base"]),
        ("log", vec!["Psl\\Math\\log"]),
        ("max", vec!["Psl\\Math\\max", "Psl\\Math\\maxva", "Psl\\Math\\max_by"]),
        ("min", vec!["Psl\\Math\\min", "Psl\\Math\\minva", "Psl\\Math\\min_by"]),
        ("round", vec!["Psl\\Math\\round"]),
        ("sin", vec!["Psl\\Math\\sin"]),
        ("sqrt", vec!["Psl\\Math\\sqrt"]),
        ("tan", vec!["Psl\\Math\\tan"]),
    ])
});
