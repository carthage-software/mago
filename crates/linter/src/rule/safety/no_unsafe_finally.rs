use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::*;
use mago_span::HasSpan;
use mago_syntax::ast::*;
use mago_syntax::utils::control_flow::ControlFlow;
use mago_syntax::utils::control_flow::find_control_flows_in_block;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoUnsafeFinallyRule {
    meta: &'static RuleMeta,
    cfg: NoUnsafeFinallyConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoUnsafeFinallyConfig {
    pub level: Level,
}

impl Default for NoUnsafeFinallyConfig {
    fn default() -> Self {
        Self { level: Level::Error }
    }
}

impl Config for NoUnsafeFinallyConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoUnsafeFinallyRule {
    type Config = NoUnsafeFinallyConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Unsafe Finally",
            code: "no-unsafe-finally",
            description: indoc! {"
                Detects control flow statements in `finally` blocks.

                Control flow statements in `finally` blocks override control flows from `try` and `catch` blocks,
                leading to unexpected behavior.
            "},
            good_example: indoc! {r#"
                <?php

                function example(): int {
                    try {
                        return get_value();
                    } finally {
                        // no control flow statements
                    }
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                function example(): int {
                    try {
                        return get_value();
                    } finally {
                        return 42; // Unsafe control flow statement in finally block
                    }
                }
            "#},
            category: Category::Safety,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Try];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::Try(r#try) = node else {
            return;
        };

        let Some(finally) = r#try.finally_clause.as_ref() else {
            return;
        };

        for control_flow in find_control_flows_in_block(&finally.block) {
            let kind = match control_flow {
                ControlFlow::Return(_) => "return",
                ControlFlow::Throw(_) => "throw",
                ControlFlow::Continue(_) => "continue",
                ControlFlow::Break(_) => "break",
            };

            let issue = Issue::new(self.cfg.level(), "Unsafe control flow in finally block.")
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(control_flow.span())
                        .with_message(format!("Control flow statement `{}` in `finally` block.", kind)),
                )
                .with_annotation(
                    Annotation::secondary(r#try.span())
                        .with_message("This `finally` block is associated with this `try` block."),
                )
                .with_note(
                    "Control flow statements in `finally` blocks override control flows from `try` and `catch` blocks, leading to unexpected behavior.",
                )
                .with_help("Avoid using control flow statements in `finally` blocks.");

            ctx.collector.report(issue);
        }
    }
}
