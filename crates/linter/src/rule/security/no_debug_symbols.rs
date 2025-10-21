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
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::call::function_call_matches;
use crate::rule::utils::call::function_call_matches_any;
use crate::rule::utils::consts::DEBUG_FUNCTIONS;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoDebugSymbolsRule {
    meta: &'static RuleMeta,
    cfg: NoDebugSymbolsConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoDebugSymbolsConfig {
    pub level: Level,
}

impl Default for NoDebugSymbolsConfig {
    fn default() -> Self {
        Self { level: Level::Note }
    }
}

impl Config for NoDebugSymbolsConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoDebugSymbolsRule {
    type Config = NoDebugSymbolsConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Debug Symbols",
            code: "no-debug-symbols",
            description: indoc! {r#"
                Flags calls to debug functions like `var_dump`, `print_r`, `dd`, etc.

                These functions are useful for debugging, but they should not be committed to
                version control as they can expose sensitive information and are generally not
                intended for production environments.
            "#},
            good_example: indoc! {r#"
                <?php

                // Production-safe code
                error_log('Processing user request.');
            "#},
            bad_example: indoc! {r#"
                <?php

                function process_request(array $data) {
                    var_dump($data); // Debug call that should be removed
                    // ...
                }
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
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::FunctionCall(function_call) = node else {
            return;
        };

        // Allow `var_export` and `print_r` if they have the second argument
        if function_call_matches_any(ctx, function_call, &["var_export", "print_r"]).is_some()
            && function_call.argument_list.arguments.len() > 1
        {
            return;
        }

        for debug_function in DEBUG_FUNCTIONS.iter() {
            if !function_call_matches(ctx, function_call, debug_function) {
                continue;
            }

            let issue = Issue::new(self.cfg.level, "Do not commit debug functions.")
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(function_call.span())
                        .with_message(format!("Function `{debug_function}` should be removed before committing")),
                )
                .with_note("Debug functions can expose sensitive application data if left in production code.")
                .with_help("Remove this function call.");

            ctx.collector.report(issue);

            break;
        }
    }
}
