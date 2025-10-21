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
pub struct PslDatetimeRule {
    meta: &'static RuleMeta,
    cfg: PslDatetimeConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct PslDatetimeConfig {
    pub level: Level,
}

impl Default for PslDatetimeConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for PslDatetimeConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for PslDatetimeRule {
    type Config = PslDatetimeConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Psl DateTime",
            code: "psl-datetime",
            description: indoc! {"
                This rule enforces the usage of Psl DateTime classes and functions over their PHP counterparts.

                Psl DateTime classes and functions are preferred because they are type-safe and provide more consistent behavior.
            "},
            good_example: indoc! {r#"
                <?php

                $dateTime = new Psl\DateTime\DateTime();
            "#},
            bad_example: indoc! {r#"
                <?php

                $dateTime = new DateTime();
            "#},
            category: Category::BestPractices,

            requirements: RuleRequirements::Integration(Integration::Psl),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::FunctionCall, NodeKind::Instantiation, NodeKind::StaticMethodCall];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let identifier = match node {
            Node::FunctionCall(function_call) => {
                let Expression::Identifier(identifier) = function_call.function else {
                    return;
                };

                let target_functions: Vec<&'static str> = DATETIME_FUNCTION_REPLACEMENTS.keys().copied().collect();

                if let Some(matched_name) = function_call_matches_any(ctx, function_call, &target_functions) {
                    let replacements = &DATETIME_FUNCTION_REPLACEMENTS[matched_name];

                    ctx.collector.report(
                        Issue::new(
                            self.cfg.level(),
                            "Use the Psl DateTime function instead of the PHP counterpart.",
                        )
                        .with_code(self.meta.code)
                        .with_annotation(
                            Annotation::primary(identifier.span())
                                .with_message("This is a PHP DateTime function"),
                        )
                        .with_note("Psl DateTime classes and functions are preferred because they are type-safe and provide more consistent behavior.")
                        .with_help(format!("Use {} instead.", format_replacements(replacements))),
                    );
                }

                return;
            }
            Node::Instantiation(instantiation) => {
                let Expression::Identifier(identifier) = instantiation.class else {
                    return;
                };

                identifier
            }
            Node::StaticMethodCall(static_method_call) => {
                let Expression::Identifier(identifier) = static_method_call.class else {
                    return;
                };

                identifier
            }
            _ => return,
        };

        let class_name = ctx.lookup_name(identifier).to_lowercase();
        if let Some(replacements) = DATETIME_CLASS_REPLACEMENTS.get(class_name.as_str()) {
            ctx.collector.report(
                Issue::new(
                    self.cfg.level(),
                    "Use the Psl DateTime class instead of the PHP counterpart.",
                )
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(identifier.span())
                        .with_message("This is a PHP DateTime class"),
                )
                .with_note("Psl DateTime classes and functions are preferred because they are type-safe and provide more consistent behavior.")
                .with_help(format!("Use {} instead.", format_replacements(replacements))),
            );
        }
    }
}

static DATETIME_FUNCTION_REPLACEMENTS: LazyLock<HashMap<&'static str, Vec<&'static str>>> = LazyLock::new(|| {
    HashMap::from_iter([
        ("time", vec!["Psl\\DateTime\\Timestamp::now", "Psl\\DateTime\\DateTime::now"]),
        ("microtime", vec!["Psl\\DateTime\\Timestamp::now", "Psl\\DateTime\\DateTime::now"]),
        ("hrtime", vec!["Psl\\DateTime\\Timestamp::monotonic"]),
        (
            "strtotime",
            vec![
                "Psl\\DateTime\\Timestamp::parse",
                "Psl\\DateTime\\Timestamp::fromString",
                "Psl\\DateTime\\DateTime::parse",
                "Psl\\DateTime\\DateTime::fromString",
            ],
        ),
        ("date_default_timezone_get", vec!["Psl\\DateTime\\Timezone::default"]),
    ])
});

static DATETIME_CLASS_REPLACEMENTS: LazyLock<HashMap<&'static str, Vec<&'static str>>> = LazyLock::new(|| {
    HashMap::from_iter([
        ("datetime", vec!["Psl\\DateTime\\DateTime"]),
        ("datetimeimmutable", vec!["Psl\\DateTime\\DateTime"]),
        ("datetimezone", vec!["Psl\\DateTime\\Timezone"]),
        ("dateinterval", vec!["Psl\\DateTime\\Duration"]),
        ("intldateformatter", vec!["Psl\\DateTime\\DateTime"]),
        ("intltimezone", vec!["Psl\\DateTime\\Timezone"]),
        ("intltimezone", vec!["Psl\\DateTime\\Timezone"]),
    ])
});
