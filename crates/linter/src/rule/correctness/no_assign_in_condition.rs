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
pub struct NoAssignInConditionRule {
    meta: &'static RuleMeta,
    cfg: NoAssignInConditionConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoAssignInConditionConfig {
    pub level: Level,
}

impl Default for NoAssignInConditionConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoAssignInConditionConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoAssignInConditionRule {
    type Config = NoAssignInConditionConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Assign In Condition",
            code: "no-assign-in-condition",
            description: indoc! {"
                Detects assignments in conditions which can lead to unexpected behavior and make the code harder
                to read and understand.
            "},
            good_example: indoc! {r#"
                <?php

                $x = 1;
                if ($x == 1) {
                    // ...
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                if ($x = 1) {
                    // ...
                }
            "#},
            category: Category::Correctness,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[
            NodeKind::If,
            NodeKind::While,
            NodeKind::DoWhile,
            NodeKind::IfStatementBodyElseIfClause,
            NodeKind::IfColonDelimitedBodyElseIfClause,
        ];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let (condition, assignment) = match node {
            Node::If(r#if) => (&r#if.condition, get_assignment_from_expression(r#if.condition)),
            Node::While(r#while) => (&r#while.condition, get_assignment_from_expression(r#while.condition)),
            Node::DoWhile(do_while) => (&do_while.condition, get_assignment_from_expression(do_while.condition)),
            Node::IfStatementBodyElseIfClause(if_statement_body_else_if_clause) => (
                &if_statement_body_else_if_clause.condition,
                get_assignment_from_expression(if_statement_body_else_if_clause.condition),
            ),
            Node::IfColonDelimitedBodyElseIfClause(if_colon_delimited_body_else_if_clause) => (
                &if_colon_delimited_body_else_if_clause.condition,
                get_assignment_from_expression(if_colon_delimited_body_else_if_clause.condition),
            ),
            _ => return,
        };

        let Some(assignment) = assignment else {
            return;
        };

        let mut issue = Issue::new(self.cfg.level(), "Avoid assignments in conditions.")
            .with_code(self.meta.code)
            .with_annotation(Annotation::primary(assignment.span()).with_message("This is an assignment"))
            .with_annotation(Annotation::secondary(condition.span()).with_message("This is the condition"))
            .with_note("Assigning a value within a condition can lead to unexpected behavior and make the code harder to read and understand.");

        if matches!(&assignment.operator, AssignmentOperator::Assign(_)) {
            issue = issue.with_note("It's easy to confuse assignment (`=`) with comparison (`==`) in this context. Ensure you're using the correct operator.");
        }

        ctx.collector.report(issue);
    }
}

#[inline]
fn get_assignment_from_expression<'ast, 'arena>(
    expression: &'ast Expression<'arena>,
) -> Option<&'ast Assignment<'arena>> {
    match expression {
        Expression::Parenthesized(parenthesized) => get_assignment_from_expression(parenthesized.expression),
        Expression::Assignment(assignment) => Some(assignment),
        _ => None,
    }
}
