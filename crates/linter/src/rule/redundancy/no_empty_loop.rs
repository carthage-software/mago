use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_fixer::SafetyClassification;
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
pub struct NoEmptyLoopRule {
    meta: &'static RuleMeta,
    cfg: NoEmptyLoopConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoEmptyLoopConfig {
    pub level: Level,
}

impl Default for NoEmptyLoopConfig {
    fn default() -> Self {
        Self { level: Level::Note }
    }
}

impl Config for NoEmptyLoopConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoEmptyLoopRule {
    type Config = NoEmptyLoopConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Empty Loop",
            code: "no-empty-loop",
            description: indoc! {r#"
                Detects loops (`for`, `foreach`, `while`, `do-while`) that have an empty body. An empty
                loop body does not perform any actions and is likely a mistake or redundant code.
            "#},
            good_example: indoc! {r#"
                <?php

                foreach ($items as $item) {
                    process($item);
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                while (should_wait()) {
                    // Empty loop body
                }
            "#},
            category: Category::Redundancy,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::For, NodeKind::Foreach, NodeKind::While, NodeKind::DoWhile];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let is_empty = match node {
            Node::For(for_loop) => match &for_loop.body {
                ForBody::Statement(stmt) => is_statement_empty(stmt),
                ForBody::ColonDelimited(body) => are_statements_empty(body.statements.as_slice()),
            },
            Node::Foreach(foreach) => match &foreach.body {
                ForeachBody::Statement(stmt) => is_statement_empty(stmt),
                ForeachBody::ColonDelimited(body) => are_statements_empty(body.statements.as_slice()),
            },
            Node::While(while_loop) => match &while_loop.body {
                WhileBody::Statement(stmt) => is_statement_empty(stmt),
                WhileBody::ColonDelimited(body) => are_statements_empty(body.statements.as_slice()),
            },
            Node::DoWhile(do_while) => is_statement_empty(do_while.statement),
            _ => return,
        };

        if !is_empty {
            return;
        }

        let issue = Issue::new(self.cfg.level, "Loop body is empty.")
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(node.span()).with_message("This loop body is empty and performs no actions"),
            )
            .with_note("Empty loops are often a sign of a logic error or dead code.")
            .with_help("Consider removing this loop or adding meaningful logic to its body.");

        ctx.collector.propose(issue, |plan| {
            plan.delete(node.span().to_range(), SafetyClassification::PotentiallyUnsafe);
        });
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
