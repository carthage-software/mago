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
pub struct NoRedundantContinueRule {
    meta: &'static RuleMeta,
    cfg: NoRedundantContinueConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoRedundantContinueConfig {
    pub level: Level,
}

impl Default for NoRedundantContinueConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for NoRedundantContinueConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoRedundantContinueRule {
    type Config = NoRedundantContinueConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Redundant Continue",
            code: "no-redundant-continue",
            description: indoc! {"
                Detects redundant `continue` statements in loops.
            "},
            good_example: indoc! {r#"
                <?php

                while (true) {
                    echo "Hello, world!";
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                while (true) {
                    echo "Hello, world!";
                    continue; // Redundant `continue` statement
                }
            "#},
            category: Category::Redundancy,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Foreach, NodeKind::For, NodeKind::While, NodeKind::DoWhile];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let r#continue = match node {
            Node::Foreach(foreach) => match &foreach.body {
                ForeachBody::Statement(stmt) => get_continue_from_statement(stmt),
                ForeachBody::ColonDelimited(body) => get_continue_from_last_statement(body.statements.as_slice()),
            },
            Node::For(r#for) => match &r#for.body {
                ForBody::Statement(stmt) => get_continue_from_statement(stmt),
                ForBody::ColonDelimited(body) => get_continue_from_last_statement(body.statements.as_slice()),
            },
            Node::While(r#while) => match &r#while.body {
                WhileBody::Statement(stmt) => get_continue_from_statement(stmt),
                WhileBody::ColonDelimited(body) => get_continue_from_last_statement(body.statements.as_slice()),
            },
            Node::DoWhile(do_while) => get_continue_from_statement(do_while.statement),
            _ => None,
        };

        let Some(r#continue) = r#continue else {
            return;
        };

        let issue = Issue::new(self.cfg.level(), "Redundant continue statement in loop body.")
            .with_code(self.meta.code)
            .with_annotation(Annotation::primary(r#continue.span()).with_message(
                "This `continue` statement is redundant because it is the last statement in the loop body",
            ))
            .with_annotation(Annotation::secondary(node.span()))
            .with_help("Remove this `continue` statement, as it does not affect the loop's behavior.");

        ctx.collector.report(issue);
    }
}

#[inline]
fn get_continue_from_last_statement<'ast, 'arena>(
    statements: &'ast [Statement<'arena>],
) -> Option<&'ast Continue<'arena>> {
    let last = statements.last()?;

    get_continue_from_statement(last)
}

#[inline]
fn get_continue_from_statement<'ast, 'arena>(statement: &'ast Statement<'arena>) -> Option<&'ast Continue<'arena>> {
    match statement {
        Statement::Block(block) => get_continue_from_statement(block.statements.last()?),
        Statement::Continue(cont) => match cont.level {
            None | Some(Expression::Literal(Literal::Integer(LiteralInteger { value: Some(1), .. }))) => Some(cont),
            Some(_) => None,
        },
        _ => None,
    }
}
