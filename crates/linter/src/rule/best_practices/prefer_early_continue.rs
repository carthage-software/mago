use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::BinaryOperator;
use mago_syntax::ast::Expression;
use mago_syntax::ast::ForBody;
use mago_syntax::ast::ForeachBody;
use mago_syntax::ast::If;
use mago_syntax::ast::IfBody;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_syntax::ast::Statement;
use mago_syntax::ast::UnaryPrefixOperator;
use mago_syntax::ast::WhileBody;
use mago_text_edit::TextEdit;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct PreferEarlyContinueRule {
    meta: &'static RuleMeta,
    cfg: PreferEarlyContinueConfig,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct PreferEarlyContinueConfig {
    pub level: Level,
}

impl Default for PreferEarlyContinueConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for PreferEarlyContinueConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for PreferEarlyContinueRule {
    type Config = PreferEarlyContinueConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Prefer Early Continue",
            code: "prefer-early-continue",
            description: indoc! {"
                Suggests using early continue pattern when a loop body contains only a single if statement.

                This improves code readability by reducing nesting and making the control flow more explicit.
            "},
            good_example: indoc! {r"
                <?php

                for ($i = 0; $i < 10; $i++) {
                    if (!$condition) {
                        continue;
                    }
                    doSomething();
                }
            "},
            bad_example: indoc! {r"
                <?php

                for ($i = 0; $i < 10; $i++) {
                    if ($condition) {
                        doSomething();
                    }
                }
            "},
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

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let (if_statement, loop_span) = match node {
            Node::For(for_loop) => {
                let body = match &for_loop.body {
                    ForBody::Statement(stmt) => extract_single_if_from_statement(stmt),
                    ForBody::ColonDelimited(block) => extract_single_if_from_statements(block.statements.as_slice()),
                };
                (body, for_loop.span())
            }
            Node::Foreach(foreach_loop) => {
                let body = match &foreach_loop.body {
                    ForeachBody::Statement(stmt) => extract_single_if_from_statement(stmt),
                    ForeachBody::ColonDelimited(block) => {
                        extract_single_if_from_statements(block.statements.as_slice())
                    }
                };
                (body, foreach_loop.span())
            }
            Node::While(while_loop) => {
                let body = match &while_loop.body {
                    WhileBody::Statement(stmt) => extract_single_if_from_statement(stmt),
                    WhileBody::ColonDelimited(block) => extract_single_if_from_statements(block.statements.as_slice()),
                };
                (body, while_loop.span())
            }
            Node::DoWhile(do_while) => (extract_single_if_from_statement(do_while.statement), do_while.span()),
            _ => return,
        };

        let Some(if_stmt) = if_statement else { return };

        let has_else = match &if_stmt.body {
            IfBody::Statement(body) => body.else_if_clauses.is_empty() && body.else_clause.is_none(),
            IfBody::ColonDelimited(body) => body.else_if_clauses.is_empty() && body.else_clause.is_none(),
        };

        if !has_else {
            return;
        }

        let has_body_content = match &if_stmt.body {
            IfBody::Statement(body) => !is_empty_statement(body.statement),
            IfBody::ColonDelimited(body) => !body.statements.is_empty(),
        };

        if !has_body_content {
            return;
        }

        let issue = Issue::new(self.cfg.level(), "Consider using early continue pattern to reduce nesting.")
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(if_stmt.span()).with_message("This if statement wraps the entire loop body"),
            )
            .with_annotation(
                Annotation::secondary(loop_span).with_message(
                    "The loop can benefit from early continue to improve readability",
                )
            )
            .with_help("Invert the condition and use `continue` to exit early, then place the main logic outside the if block.")
            .with_note("This pattern improves readability by reducing nesting levels.");

        ctx.collector.propose(issue, |edits| {
            let condition = &if_stmt.condition;
            match condition {
                Expression::UnaryPrefix(unary) if matches!(unary.operator, UnaryPrefixOperator::Not(_)) => {
                    // Already negated, remove the negation
                    edits.push(TextEdit::replace(
                        condition.span(),
                        &ctx.source_file.contents.as_ref()
                            [unary.operand.start_offset() as usize..unary.operand.end_offset() as usize],
                    ));
                }
                Expression::Binary(binary) => {
                    // Negate binary operators directly when possible
                    let negated_op = match binary.operator {
                        BinaryOperator::Equal(_) => Some("!="),
                        BinaryOperator::NotEqual(_) => Some("=="),
                        BinaryOperator::Identical(_) => Some("!=="),
                        BinaryOperator::NotIdentical(_) => Some("==="),
                        BinaryOperator::AngledNotEqual(_) => Some("=="),
                        BinaryOperator::LessThan(_) => Some(">="),
                        BinaryOperator::LessThanOrEqual(_) => Some(">"),
                        BinaryOperator::GreaterThan(_) => Some("<="),
                        BinaryOperator::GreaterThanOrEqual(_) => Some("<"),
                        _ => None,
                    };

                    if let Some(op) = negated_op {
                        // Replace the operator with its negation
                        edits.push(TextEdit::replace(binary.operator.span(), op));
                    } else {
                        // Can't negate the operator directly, wrap in !(...)
                        edits.push(TextEdit::insert(condition.start_offset(), "!("));
                        edits.push(TextEdit::insert(condition.end_offset(), ")"));
                    }
                }
                _ => {
                    // For other expressions, wrap in !(...)
                    edits.push(TextEdit::insert(condition.start_offset(), "!("));
                    edits.push(TextEdit::insert(condition.end_offset(), ")"));
                }
            }

            match &if_stmt.body {
                IfBody::Statement(body) => {
                    if let Statement::Block(block) = body.statement {
                        if block.statements.is_empty() {
                            edits.push(TextEdit::replace(block.left_brace.join(block.right_brace), "{ continue; }"));
                        } else {
                            let first_stmt_start = block.statements.nodes[0].start_offset();
                            let range_to_replace = block.left_brace.start_offset()..first_stmt_start;
                            edits.push(TextEdit::replace(range_to_replace, "{ continue; }\n\n"));

                            let last_stmt_end = block.statements.nodes.last().unwrap().end_offset();
                            let range_to_delete = last_stmt_end..block.right_brace.end_offset();
                            edits.push(TextEdit::delete(range_to_delete));
                        }
                    }
                }
                IfBody::ColonDelimited(body) => {
                    if body.statements.is_empty() {
                        let range = body.colon.start_offset()..body.terminator.end_offset();
                        edits.push(TextEdit::replace(range, "{ continue; }"));
                    } else {
                        let first_stmt_start = body.statements.nodes[0].start_offset();
                        let range_to_replace = body.colon.start_offset()..first_stmt_start;
                        edits.push(TextEdit::replace(range_to_replace, "{ continue; }\n\n"));

                        let last_stmt_end = body.statements.nodes.last().unwrap().end_offset();
                        let endif_end = body.terminator.end_offset();
                        let range_to_delete = last_stmt_end..endif_end;
                        edits.push(TextEdit::delete(range_to_delete));
                    }
                }
            }
        });
    }
}

fn extract_single_if_from_statement<'ast, 'arena>(stmt: &'ast Statement<'arena>) -> Option<&'ast If<'arena>> {
    match stmt {
        Statement::If(if_stmt) => Some(if_stmt),
        Statement::Block(block) => extract_single_if_from_statements(block.statements.as_slice()),
        _ => None,
    }
}

fn extract_single_if_from_statements<'ast, 'arena>(stmts: &'ast [Statement<'arena>]) -> Option<&'ast If<'arena>> {
    let non_empty: Vec<_> = stmts.iter().filter(|s| !is_empty_statement(s)).collect();

    if non_empty.len() != 1 {
        return None;
    }

    extract_single_if_from_statement(non_empty[0])
}

fn is_empty_statement(stmt: &Statement) -> bool {
    matches!(stmt, Statement::Noop(_))
}
