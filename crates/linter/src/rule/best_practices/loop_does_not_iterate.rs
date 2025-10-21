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
pub struct LoopDoesNotIterateRule {
    meta: &'static RuleMeta,
    cfg: LoopDoesNotIterateConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct LoopDoesNotIterateConfig {
    pub level: Level,
}

impl Default for LoopDoesNotIterateConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for LoopDoesNotIterateConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for LoopDoesNotIterateRule {
    type Config = LoopDoesNotIterateConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Loop Does Not Iterate",
            code: "loop-does-not-iterate",
            description: indoc! {r#"
                Detects loops (for, foreach, while, do-while) that unconditionally break or return
                before executing even a single iteration. Such loops are misleading or redundant
                since they give the impression of iteration but never actually do so.
            "#},
            good_example: indoc! {r#"
                <?php

                for ($i = 0; $i < 3; $i++) {
                    echo $i;
                    if ($some_condition) {
                        break; // This break is conditional.
                    }
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                for ($i = 0; $i < 3; $i++) {
                    break; // The loop never truly iterates, as this break is unconditional.
                }
            "#},
            category: Category::BestPractices,

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
        let terminator = match node {
            Node::For(for_loop) => match &for_loop.body {
                ForBody::Statement(stmt) => get_loop_terminator_from_statement(stmt),
                ForBody::ColonDelimited(block) => get_loop_terminator_from_statements(block.statements.as_slice()),
            },
            Node::Foreach(foreach) => match &foreach.body {
                ForeachBody::Statement(stmt) => get_loop_terminator_from_statement(stmt),
                ForeachBody::ColonDelimited(block) => get_loop_terminator_from_statements(block.statements.as_slice()),
            },
            Node::While(while_loop) => match &while_loop.body {
                WhileBody::Statement(stmt) => get_loop_terminator_from_statement(stmt),
                WhileBody::ColonDelimited(block) => get_loop_terminator_from_statements(block.statements.as_slice()),
            },
            Node::DoWhile(do_while) => get_loop_terminator_from_statement(do_while.statement),
            _ => None,
        };

        if let Some(terminator) = terminator {
            check_loop(node, terminator, ctx, &self.cfg, self.meta);
        }
    }
}

fn check_loop<'ast, 'arena>(
    r#loop: Node<'ast, 'arena>,
    terminator: LoopTerminator<'ast, 'arena>,
    ctx: &mut LintContext,
    cfg: &LoopDoesNotIterateConfig,
    meta: &'static RuleMeta,
) {
    let (terminator_span, terminator_name) = match terminator {
        LoopTerminator::Break(stmt) => (stmt.span(), "`break`"),
        LoopTerminator::Return(stmt) => (stmt.span(), "`return`"),
    };

    let issue = Issue::new(cfg.level, "Loop is unconditionally terminated and will not iterate.")
        .with_code(meta.code)
        .with_annotation(Annotation::primary(r#loop.span()).with_message("This loop will only execute once at most"))
        .with_annotation(
            Annotation::secondary(terminator_span)
                .with_message(format!("The loop is exited by this unconditional {terminator_name} statement")),
        )
        .with_note("Loops that do not iterate are often misleading and can indicate a logic error or redundant code.")
        .with_help("Consider refactoring the code. If the loop is intended to run only once, an `if` statement might be clearer.");

    ctx.collector.report(issue);
}

#[derive(Debug)]
enum LoopTerminator<'ast, 'arena> {
    Break(&'ast Break<'arena>),
    Return(&'ast Return<'arena>),
}

#[inline]
fn get_loop_terminator_from_statements<'ast, 'arena>(
    statements: &'ast [Statement<'arena>],
) -> Option<LoopTerminator<'ast, 'arena>> {
    for statement in statements.iter() {
        if might_skip_terminator(statement) {
            return None;
        }

        if let Some(terminator) = get_loop_terminator_from_statement(statement) {
            return Some(terminator);
        }
    }

    None
}

#[inline]
fn get_loop_terminator_from_statement<'ast, 'arena>(
    statement: &'ast Statement<'arena>,
) -> Option<LoopTerminator<'ast, 'arena>> {
    match statement {
        Statement::Block(block) => get_loop_terminator_from_statements(block.statements.as_slice()),
        Statement::Break(break_stmt) => match break_stmt.level {
            None | Some(Expression::Literal(Literal::Integer(LiteralInteger { value: Some(1), .. }))) => {
                Some(LoopTerminator::Break(break_stmt))
            }
            Some(_) => None,
        },
        Statement::Return(return_stmt) => Some(LoopTerminator::Return(return_stmt)),
        _ => None,
    }
}

#[inline]
fn might_skip_terminator<'ast, 'arena>(statement: &'ast Statement<'arena>) -> bool {
    match statement {
        Statement::Continue(_) | Statement::Goto(_) => true,
        Statement::Block(block) => block.statements.iter().any(might_skip_terminator),
        Statement::If(if_stmt) => match &if_stmt.body {
            IfBody::Statement(body) => {
                if might_skip_terminator(body.statement) {
                    return true;
                }

                if body.else_clause.as_ref().is_some_and(|clause| might_skip_terminator(clause.statement)) {
                    return true;
                }

                body.else_if_clauses.iter().any(|clause| might_skip_terminator(clause.statement))
            }
            IfBody::ColonDelimited(body) => {
                if body.statements.iter().any(might_skip_terminator) {
                    return true;
                }

                if body.else_clause.as_ref().is_some_and(|clause| clause.statements.iter().any(might_skip_terminator)) {
                    return true;
                }

                body.else_if_clauses.iter().any(|clause| clause.statements.iter().any(might_skip_terminator))
            }
        },
        Statement::While(while_stmt) => match &while_stmt.body {
            WhileBody::Statement(body) => might_skip_terminator(body),
            WhileBody::ColonDelimited(body) => body.statements.iter().any(might_skip_terminator),
        },
        Statement::DoWhile(do_while_stmt) => might_skip_terminator(do_while_stmt.statement),
        Statement::For(for_stmt) => match &for_stmt.body {
            ForBody::Statement(body) => might_skip_terminator(body),
            ForBody::ColonDelimited(body) => body.statements.iter().any(might_skip_terminator),
        },
        Statement::Foreach(foreach_stmt) => match &foreach_stmt.body {
            ForeachBody::Statement(body) => might_skip_terminator(body),
            ForeachBody::ColonDelimited(body) => body.statements.iter().any(might_skip_terminator),
        },
        Statement::Namespace(namespace) => namespace.statements().iter().any(might_skip_terminator),
        Statement::Declare(declare) => match &declare.body {
            DeclareBody::Statement(body) => might_skip_terminator(body),
            DeclareBody::ColonDelimited(body) => body.statements.iter().any(might_skip_terminator),
        },
        Statement::Try(try_stmt) => {
            if try_stmt.block.statements.iter().any(might_skip_terminator) {
                return true;
            }

            if try_stmt.catch_clauses.iter().any(|clause| clause.block.statements.iter().any(might_skip_terminator)) {
                return true;
            }

            try_stmt
                .finally_clause
                .as_ref()
                .is_some_and(|clause| clause.block.statements.iter().any(might_skip_terminator))
        }
        Statement::Switch(switch_stmt) => {
            switch_stmt.body.cases().iter().any(|case| case.statements().iter().any(might_skip_terminator))
        }
        _ => false,
    }
}
