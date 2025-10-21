use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_php_version::PHPVersion;
use mago_php_version::PHPVersionRange;
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
pub struct StrictBehaviorRule {
    meta: &'static RuleMeta,
    cfg: StrictBehaviorConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct StrictBehaviorConfig {
    pub level: Level,
    pub allow_loose_behavior: bool,
}

impl Default for StrictBehaviorConfig {
    fn default() -> Self {
        Self { level: Level::Warning, allow_loose_behavior: false }
    }
}

impl Config for StrictBehaviorConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for StrictBehaviorRule {
    type Config = StrictBehaviorConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Strict Behavior",
            code: "strict-behavior",
            description: indoc! {"
                Detects functions relying on loose comparison unless the `$strict` parameter is specified.
                The use of loose comparison for these functions may lead to hard-to-debug, unexpected behaviors.
            "},
            good_example: indoc! {r#"
                <?php

                in_array(1, ['foo', 'bar', 'baz'], strict: true);
            "#},
            bad_example: indoc! {r#"
                <?php

                in_array(1, ['foo', 'bar', 'baz']);
            "#},
            category: Category::Correctness,
            requirements: RuleRequirements::PHPVersion(PHPVersionRange::from(PHPVersion::PHP70)),
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
        let Node::FunctionCall(call) = node else {
            return;
        };

        let (function_name, expected_position) = if function_call_matches(ctx, call, "base64_decode") {
            ("base64_decode", 1)
        } else if function_call_matches(ctx, call, "in_array") {
            ("in_array", 2)
        } else if function_call_matches(ctx, call, "array_search") {
            ("array_search", 2)
        } else if call.argument_list.arguments.len() > 1 && function_call_matches(ctx, call, "array_keys") {
            ("array_keys", 2)
        } else {
            return;
        };

        let mut found = false;
        let mut correct = false;
        for (position, argument) in call.argument_list.arguments.iter().enumerate() {
            match argument {
                Argument::Positional(argument) if position == expected_position => {
                    found = true;

                    if matches!(argument.value, Expression::Literal(Literal::True(_)))
                        || (self.cfg.allow_loose_behavior
                            && matches!(argument.value, Expression::Literal(Literal::False(_))))
                    {
                        correct = true;
                        break;
                    }
                }
                Argument::Named(argument) => {
                    if argument.name.value != "strict" {
                        continue;
                    }

                    found = true;
                    if matches!(argument.value, Expression::Literal(Literal::True(_)))
                        || (self.cfg.allow_loose_behavior
                            && matches!(argument.value, Expression::Literal(Literal::False(_))))
                    {
                        correct = true;
                        break;
                    }
                }
                _ => {
                    continue;
                }
            }
        }

        if found && (correct || self.cfg.allow_loose_behavior) {
            return;
        }

        let mut issue =
            Issue::new(self.cfg.level(), format!("Call to `{function_name}` must enforce strict comparison."))
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(call.function.span()).with_message(format!(
                    "Function `{function_name}` relies on loose comparison which can lead to unexpected behavior",
                )))
                .with_help(format!("Call the function `{function_name}` with the `$strict` parameter set to `true`.",));

        if self.cfg.allow_loose_behavior {
            issue = issue.with_note(
                "The `allow_loose_behavior` option is enabled; you may set the `$strict` parameter to `false`."
                    .to_string(),
            );
        }

        ctx.collector.report(issue);
    }
}
