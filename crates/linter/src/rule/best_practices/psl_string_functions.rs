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
pub struct PslStringFunctionsRule {
    meta: &'static RuleMeta,
    cfg: PslStringFunctionsConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct PslStringFunctionsConfig {
    pub level: Level,
}

impl Default for PslStringFunctionsConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for PslStringFunctionsConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for PslStringFunctionsRule {
    type Config = PslStringFunctionsConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Psl String Functions",
            code: "psl-string-functions",
            description: indoc! {"                This rule enforces the usage of Psl string functions over their PHP counterparts.

                Psl string functions are preferred because they are type-safe and provide more consistent behavior.
            "},
            good_example: indoc! {r###"
                <?php

                $capitalized = Psl\Str\capitalize($string);
            "###},
            bad_example: indoc! {r###"
                <?php

                $capitalized = ucfirst($string);
            "###},
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

        let target_functions: Vec<&'static str> = STRING_FUNCTION_REPLACEMENTS.keys().copied().collect();
        let Some(matched_name) = function_call_matches_any(ctx, function_call, &target_functions) else {
            return;
        };

        let replacements = &STRING_FUNCTION_REPLACEMENTS[matched_name];

        ctx.collector.report(
            Issue::new(
                self.cfg.level(),
                "Use the Psl string function instead of the PHP counterpart.",
            )
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(function_call.function.span())
                    .with_message("This is a PHP string function"),
            )
            .with_note("Psl string functions are preferred because they are type-safe and provide more consistent behavior.")
            .with_help(format!("Use {} instead.", format_replacements(replacements))),
        );
    }
}

static STRING_FUNCTION_REPLACEMENTS: LazyLock<HashMap<&'static str, Vec<&'static str>>> = LazyLock::new(|| {
    HashMap::from_iter([
        ("ucfirst", vec!["Psl\\Str\\Byte\\capitalize", "Psl\\Str\\capitalize"]),
        ("ucwords", vec!["Psl\\Str\\Byte\\capitalize_words"]),
        ("chr", vec!["Psl\\Str\\Byte\\chr"]),
        ("strncmp", vec!["Psl\\Str\\Byte\\compare"]),
        ("strcmp", vec!["Psl\\Str\\Byte\\compare"]),
        ("strncasecmp", vec!["Psl\\Str\\Byte\\compare_ci"]),
        ("strcasecmp", vec!["Psl\\Str\\Byte\\compare_ci"]),
        (
            "str_contains",
            vec![
                "Psl\\Str\\contains",
                "Psl\\Str\\contains_ci",
                "Psl\\Str\\Byte\\contains",
                "Psl\\Str\\Byte\\contains_ci",
            ],
        ),
        (
            "str_starts_with",
            vec![
                "Psl\\Str\\starts_with",
                "Psl\\Str\\starts_with_ci",
                "Psl\\Str\\Byte\\starts_with",
                "Psl\\Str\\Byte\\starts_with_ci",
            ],
        ),
        (
            "str_ends_with",
            vec![
                "Psl\\Str\\ends_with",
                "Psl\\Str\\ends_with_ci",
                "Psl\\Str\\Byte\\ends_with",
                "Psl\\Str\\Byte\\ends_with_ci",
            ],
        ),
        ("strlen", vec!["Psl\\Str\\Byte\\length"]),
        ("strtolower", vec!["Psl\\Str\\Byte\\lowercase"]),
        ("str_ord", vec!["Psl\\Str\\Byte\\ord"]),
        (
            "str_pad",
            vec!["Psl\\Str\\pad_right", "Psl\\Str\\pad_left", "Psl\\Str\\Byte\\pad_right", "Psl\\Str\\Byte\\pad_left"],
        ),
        ("str_replace", vec!["Psl\\Str\\replace", "Psl\\Str\\Byte\\replace", "Psl\\Str\\Byte\\replace_every"]),
        (
            "str_ireplace",
            vec!["Psl\\Str\\replace_ci", "Psl\\Str\\Byte\\replace_ci", "Psl\\Str\\Byte\\replace_every_ci"],
        ),
        ("strrev", vec!["Psl\\Str\\Byte\\reverse"]),
        ("str_rot13", vec!["Psl\\Str\\Byte\\rot13"]),
        ("strpos", vec!["Psl\\Str\\Byte\\search"]),
        ("stripos", vec!["Psl\\Str\\Byte\\search_ci"]),
        ("strrpos", vec!["Psl\\Str\\Byte\\search_last"]),
        ("strripos", vec!["Psl\\Str\\Byte\\search_last_ci"]),
        ("str_shuffle", vec!["Psl\\Str\\Byte\\shuffle"]),
        (
            "substr",
            vec![
                "Psl\\Str\\Byte\\slice",
                "Psl\\Str\\Byte\\range",
                "Psl\\Str\\Byte\\strip_prefix",
                "Psl\\Str\\Byte\\strip_suffix",
            ],
        ),
        ("substr_replace", vec!["Psl\\Str\\Byte\\splice"]),
        ("explode", vec!["Psl\\Str\\Byte\\split"]),
        ("str_split", vec!["Psl\\Str\\Byte\\chunk", "Psl\\Str\\Byte\\split"]),
        ("trim", vec!["Psl\\Str\\trim", "Psl\\Str\\Byte\\trim"]),
        ("ltrim", vec!["Psl\\Str\\trim_left", "Psl\\Str\\Byte\\trim_left"]),
        ("rtrim", vec!["Psl\\Str\\trim_right", "Psl\\Str\\Byte\\trim_right"]),
        ("strtoupper", vec!["Psl\\Str\\Byte\\uppercase"]),
        ("wordwrap", vec!["Psl\\Str\\Byte\\wrap", "Psl\\Str\\wrap"]),
        ("str_word_count", vec!["Psl\\Str\\Byte\\words"]),
        ("mb_strwidth", vec!["Psl\\Str\\width"]),
        ("mb_strtoupper", vec!["Psl\\Str\\uppercase"]),
        ("mb_strtolower", vec!["Psl\\Str\\lowercase"]),
        ("mb_convert_case", vec!["Psl\\Str\\uppercase", "Psl\\Str\\lowercase", "Psl\\Str\\capitalize_words"]),
        ("mb_chr", vec!["Psl\\Str\\chr"]),
        ("mb_str_split", vec!["Psl\\Str\\split"]),
        ("mb_convert_encoding", vec!["Psl\\Str\\convert_encoding"]),
        ("mb_detect_encoding", vec!["Psl\\Str\\detect_encoding", "Psl\\Str\\is_utf8"]),
        ("sprintf", vec!["Psl\\Str\\format"]),
        ("vsprintf", vec!["Psl\\Str\\format"]),
        ("number_format", vec!["Psl\\Str\\format_number"]),
        ("implode", vec!["Psl\\Str\\join"]),
        ("join", vec!["Psl\\Str\\join"]),
        ("mb_strlen", vec!["Psl\\Str\\length"]),
        ("levenshtein", vec!["Psl\\Str\\levenshtein"]),
        ("mb_strtolower", vec!["Psl\\Str\\lowercase"]),
        ("metaphone", vec!["Psl\\Str\\metaphone"]),
        ("mb_ord", vec!["Psl\\Str\\ord"]),
        ("mb_ord", vec!["Psl\\Str\\ord"]),
        ("str_repeat", vec!["Psl\\Str\\repeat"]),
        ("str_repeat", vec!["Psl\\Str\\repeat"]),
        ("mb_substr", vec!["Psl\\Str\\slice", "Psl\\Str\\range", "Psl\\Str\\strip_prefix", "Psl\\Str\\strip_suffix"]),
        ("mb_strimwidth", vec!["Psl\\Str\\truncate"]),
        (
            "grapheme_substr",
            vec![
                "Psl\\Str\\Grapheme\\slice",
                "Psl\\Str\\Grapheme\\range",
                "Psl\\Str\\Grapheme\\strip_prefix",
                "Psl\\Str\\Grapheme\\strip_suffix",
            ],
        ),
        ("grapheme_strripos", vec!["Psl\\Str\\Grapheme\\search_last_ci"]),
        ("grapheme_strrpos", vec!["Psl\\Str\\Grapheme\\search_last"]),
        ("grapheme_stripos", vec!["Psl\\Str\\Grapheme\\search_ci"]),
        ("grapheme_strpos", vec!["Psl\\Str\\Grapheme\\search"]),
        ("grapheme_strlen", vec!["Psl\\Str\\Grapheme\\length"]),
    ])
});
