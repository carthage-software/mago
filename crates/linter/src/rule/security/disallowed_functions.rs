use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_atom::AtomSet;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::call::function_call_matches;
use crate::rule::utils::consts::EXTENSION_FUNCTIONS;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct DisallowedFunctionsRule {
    meta: &'static RuleMeta,
    cfg: DisallowedFunctionsConfig,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct DisallowedFunctionsConfig {
    pub level: Level,
    pub functions: AtomSet,
    pub extensions: AtomSet,
}

impl Default for DisallowedFunctionsConfig {
    fn default() -> Self {
        Self { level: Level::Warning, functions: AtomSet::default(), extensions: AtomSet::default() }
    }
}

impl Config for DisallowedFunctionsConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for DisallowedFunctionsRule {
    type Config = DisallowedFunctionsConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Disallowed Functions",
            code: "disallowed-functions",
            description: indoc! {r#"
                Flags calls to functions that are disallowed via rule configuration.

                You can specify which functions or extensions should be disallowed through the
                `functions` or `extensions` options. This helps enforce coding standards,
                security restrictions, or the usage of preferred alternatives.
            "#},
            good_example: indoc! {r#"
                <?php

                function allowed_function(): void {
                    // ...
                }

                allowed_function(); // Not flagged
            "#},
            bad_example: indoc! {r#"
                <?php

                curl_init(); // Error: part of a disallowed extension
            "#},
            category: Category::Security,

            requirements: RuleRequirements::None,
        };
        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::FunctionCall];
        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config.clone() }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::FunctionCall(function_call) = node else {
            return;
        };

        // Check explicit disallowed functions
        for function_name in self.cfg.functions.iter() {
            if !function_call_matches(ctx, function_call, function_name) {
                continue;
            }

            let issue = Issue::new(self.cfg.level, format!("Function `{function_name}` is disallowed."))
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(function_call.span())
                        .with_message(format!("call to disallowed function `{function_name}`")),
                )
                .with_note(format!(
                    "The function `{function_name}` is explicitly disallowed by your project configuration."
                ))
                .with_help(
                    "Use an alternative function or update your configuration if this restriction is no longer needed.",
                );

            ctx.collector.report(issue);

            return;
        }

        // Check disallowed extensions
        for (extension, functions) in EXTENSION_FUNCTIONS.iter() {
            for function_name in functions.iter() {
                if !function_call_matches(ctx, function_call, function_name) {
                    continue;
                }

                if self.cfg.extensions.iter().any(|e| e.eq_ignore_ascii_case(extension)) {
                    let issue = Issue::new(
                        self.cfg.level,
                        format!("Function `{function_name}` from the `{extension}` extension is disallowed."),
                    )
                    .with_code(self.meta.code)
                    .with_annotation(Annotation::primary(function_call.span()).with_message(format!(
                        "call to `{function_name}`, which belongs to the disallowed `{extension}` extension"
                    )))
                    .with_note(format!(
                        "All functions from the `{extension}` extension are disallowed by your project configuration."
                    ))
                    .with_help("Avoid using this extension or update your configuration if exceptions are acceptable.");

                    ctx.collector.report(issue);

                    return;
                }
            }
        }
    }
}
