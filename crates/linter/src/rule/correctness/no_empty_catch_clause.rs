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
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoEmptyCatchClauseRule {
    meta: &'static RuleMeta,
    cfg: NoEmptyCatchClauseConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoEmptyCatchClauseConfig {
    pub level: Level,
}

impl Default for NoEmptyCatchClauseConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoEmptyCatchClauseConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoEmptyCatchClauseRule {
    type Config = NoEmptyCatchClauseConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Empty Catch Clause",
            code: "no-empty-catch-clause",
            description: indoc! {r#"
                Warns when a `catch` clause is empty.

                An empty `catch` clause suppresses exceptions without handling or logging them,
                potentially hiding errors that should be addressed. This practice, known as
                "exception swallowing," can make debugging significantly more difficult.
            "#},
            good_example: indoc! {r#"
                <?php

                try {
                    // some code that might throw an exception
                } catch(Exception $e) {
                    // Handle the error, log it, or re-throw it.
                    error_log($e->getMessage());
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                try {
                    // some code
                } catch(Exception $e) {
                    // This block is empty and swallows the exception.
                }
            "#},
            category: Category::Correctness,

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
        let Node::Try(try_stmt) = node else {
            return;
        };

        for catch_clause in try_stmt.catch_clauses.iter() {
            if !are_statements_empty(catch_clause.block.statements.as_slice()) {
                continue;
            }

            let issue = Issue::new(self.cfg.level, "Do not use empty `catch` blocks.")
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(catch_clause.span())
                        .with_message("This `catch` block is empty and will swallow exceptions"),
                )
                .with_note("Swallowing exceptions silently can hide critical errors and make debugging extremely difficult.")
                .with_help(
                    "Either log the exception, re-throw it, or handle it. If ignoring it is intentional, add a `@mago-expect` comment explaining why.",
                );

            ctx.collector.report(issue);
        }
    }
}

#[inline]
fn is_statement_empty(statement: &Statement) -> bool {
    match statement {
        Statement::Block(block) => are_statements_empty(block.statements.as_slice()),
        Statement::Noop(_) => true,
        _ => false,
    }
}

#[inline]
fn are_statements_empty(statements: &[Statement]) -> bool {
    statements.is_empty() || statements.iter().all(is_statement_empty)
}
