use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct BlockStatementRule {
    meta: &'static RuleMeta,
    cfg: BlockStatementConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct BlockStatementConfig {
    pub level: Level,
}

impl Default for BlockStatementConfig {
    fn default() -> Self {
        Self { level: Level::Note }
    }
}

impl Config for BlockStatementConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for BlockStatementRule {
    type Config = BlockStatementConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Block Statement",
            code: "block-statement",
            description: indoc! {"
                Enforces that `if`, `else`, `for`, `foreach`, `while`, `do-while` statements always use a block
                statement body (`{ ... }`) even if they contain only a single statement.

                This improves readability and prevents potential errors when adding new statements.
            "},
            good_example: indoc! {r#"
                <?php

                if (true) {
                    echo "Hello";
                }

                for ($i = 0; $i < 10; $i++) {
                    echo $i;
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                if (true)
                    echo "Hello";

                for ($i = 0; $i < 10; $i++)
                    echo $i;
            "#},
            category: Category::Consistency,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] =
            &[NodeKind::If, NodeKind::For, NodeKind::Foreach, NodeKind::While, NodeKind::DoWhile];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let mut report = |construct_name: &str, construct_span: Span, body_span: Span| {
            let issue = Issue::new(
                self.cfg.level,
                format!("`{construct_name}` statement should use a block body."),
            )
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(body_span).with_message("This statement is not wrapped in a block"),
            )
            .with_annotation(
                Annotation::secondary(construct_span)
                    .with_message(format!("`{construct_name}` statement is defined here")),
            )
            .with_note(
                "Always using block statements `{...}` improves readability and prevents bugs when adding more lines of code.",
            )
            .with_help(format!("Add curly braces `{{ .. }}` around the body of the `{construct_name}` statement."));

            ctx.collector.report(issue);
        };

        match node {
            Node::If(if_stmt) => {
                let IfBody::Statement(body) = &if_stmt.body else {
                    return;
                };

                if !matches!(body.statement, Statement::Block(_)) {
                    report("if", if_stmt.r#if.span(), body.statement.span());
                }

                for else_if_clause in body.else_if_clauses.iter() {
                    if !matches!(else_if_clause.statement, Statement::Block(_)) {
                        report("else if", else_if_clause.elseif.span(), else_if_clause.statement.span());
                    }
                }

                if let Some(else_clause) = &body.else_clause
                    && !matches!(else_clause.statement, Statement::Block(_))
                {
                    report("else", else_clause.r#else.span(), else_clause.statement.span());
                }
            }
            Node::For(r#for) => {
                let ForBody::Statement(statement) = &r#for.body else {
                    return;
                };

                if !matches!(statement, Statement::Block(_)) {
                    report("for", r#for.r#for.span(), statement.span());
                }
            }
            Node::Foreach(r#foreach) => {
                let ForeachBody::Statement(statement) = &r#foreach.body else {
                    return;
                };

                if !matches!(statement, Statement::Block(_)) {
                    report("foreach", r#foreach.r#foreach.span(), statement.span());
                }
            }
            Node::While(r#while) => {
                let WhileBody::Statement(statement) = &r#while.body else {
                    return;
                };

                if !matches!(statement, Statement::Block(_)) {
                    report("while", r#while.r#while.span(), statement.span());
                }
            }
            Node::DoWhile(do_while) => {
                if !matches!(do_while.statement, Statement::Block(_)) {
                    report("do-while", do_while.r#do.span(), do_while.statement.span());
                }
            }
            _ => {}
        }
    }
}
