use indoc::indoc;
use mago_allocator::Arena;
use schemars::JsonSchema;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::cst::BinaryOperator;
use mago_syntax::cst::Block;
use mago_syntax::cst::Closure;
use mago_syntax::cst::Expression;
use mago_syntax::cst::Function;
use mago_syntax::cst::If;
use mago_syntax::cst::IfBody;
use mago_syntax::cst::Method;
use mago_syntax::cst::MethodBody;
use mago_syntax::cst::Node;
use mago_syntax::cst::NodeKind;
use mago_syntax::cst::Statement;
use mago_syntax::cst::UnaryPrefixOperator;
use mago_text_edit::TextEdit;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct PreferEarlyReturnRule {
    meta: &'static RuleMeta,
    cfg: PreferEarlyReturnConfig,
}

#[derive(Debug, Clone, Copy, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct PreferEarlyReturnConfig {
    pub level: Level,
    pub max_allowed_statements: usize,
}

impl Default for PreferEarlyReturnConfig {
    fn default() -> Self {
        Self { level: Level::Help, max_allowed_statements: 0 }
    }
}

impl Config for PreferEarlyReturnConfig {
    fn level(&self) -> Level {
        self.level
    }

    fn default_enabled() -> bool {
        false
    }
}

#[allow(clippy::unwrap_used)]
impl LintRule for PreferEarlyReturnRule {
    type Config = PreferEarlyReturnConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Prefer Early Return",
            code: "prefer-early-return",
            description: indoc! {"
                Suggests using early return pattern when a function body contains only a single if statement.

                This improves code readability by reducing nesting and making the control flow more explicit.
            "},
            good_example: indoc! {r"
                <?php

                function process($condition) {
                    if (!$condition) {
                        return;
                    }
                    doSomething();
                }
            "},
            bad_example: indoc! {r"
                <?php

                function process($condition) {
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
        const TARGETS: &[NodeKind] = &[NodeKind::Function, NodeKind::Method, NodeKind::Closure];
        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let (if_statement, function_span) = match node {
            Node::Function(Function { body, .. }) => {
                (extract_single_if_from_statements(body.statements.as_slice()), node.span())
            }
            Node::Method(Method { body: MethodBody::Concrete(body), .. }) => {
                (extract_single_if_from_statements(body.statements.as_slice()), node.span())
            }
            Node::Closure(Closure { body, .. }) => {
                (extract_single_if_from_statements(body.statements.as_slice()), node.span())
            }
            _ => return,
        };

        let Some(if_stmt) = if_statement else { return };

        let has_else = match &if_stmt.body {
            IfBody::Statement(body) => !body.else_if_clauses.is_empty() || body.else_clause.is_some(),
            IfBody::ColonDelimited(body) => !body.else_if_clauses.is_empty() || body.else_clause.is_some(),
        };

        if has_else {
            return;
        }

        let body_len = match &if_stmt.body {
            IfBody::Statement(body) => statement_len(body.statement),
            IfBody::ColonDelimited(body) => body.statements.len(),
        };

        if body_len <= self.cfg.max_allowed_statements {
            return;
        }

        // Skip if the body is an early exit statement (return, throw).
        // These are already simple single-statement bodies with no nesting to reduce,
        // and transforming them doesn't improve readability.
        let is_early_exit = match &if_stmt.body {
            IfBody::Statement(body) => is_early_exit_statement(body.statement),
            IfBody::ColonDelimited(body) => {
                body.statements.len() == 1 && is_early_exit_statement(&body.statements.nodes[0])
            }
        };

        if is_early_exit {
            return;
        }

        let issue = Issue::new(self.cfg.level(), "Consider using early return pattern to reduce nesting.")
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(if_stmt.span()).with_message("This if statement wraps the entire function body"),
            )
            .with_annotation(
                Annotation::secondary(function_span)
                    .with_message("The function can benefit from early return to improve readability"),
            )
            .with_help(
                "Invert the condition and use `return` to exit early, then place the main logic outside the if block.",
            )
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

            let source = ctx.source_file.contents.as_ref();
            let gap_has_comment =
                |from: u32, to: u32| source[from as usize..to as usize].iter().any(|byte| !byte.is_ascii_whitespace());

            match &if_stmt.body {
                IfBody::Statement(body) => {
                    if let Statement::Block(block) = body.statement {
                        if block.statements.is_empty() {
                            edits.push(TextEdit::replace(block.left_brace.join(block.right_brace), "{ return; }"));
                        } else {
                            let first_stmt_start = block.statements.nodes[0].start_offset();
                            if gap_has_comment(block.left_brace.end_offset(), first_stmt_start) {
                                edits.push(TextEdit::replace(block.left_brace, "{ return; }\n"));
                            } else {
                                let range_to_replace = block.left_brace.start_offset()..first_stmt_start;
                                edits.push(TextEdit::replace(range_to_replace, "{ return; }\n\n"));
                            }

                            let last_stmt_end = block.statements.nodes.last().unwrap().end_offset();
                            if gap_has_comment(last_stmt_end, block.right_brace.start_offset()) {
                                edits.push(TextEdit::delete(block.right_brace));
                            } else {
                                let range_to_delete = last_stmt_end..block.right_brace.end_offset();
                                edits.push(TextEdit::delete(range_to_delete));
                            }
                        }
                    } else {
                        let stmt_start = body.statement.start_offset();
                        if gap_has_comment(if_stmt.right_parenthesis.end_offset(), stmt_start) {
                            edits.push(TextEdit::insert(if_stmt.right_parenthesis.end_offset(), " { return; }\n"));
                        } else {
                            let range_to_replace = if_stmt.right_parenthesis.end_offset()..stmt_start;
                            edits.push(TextEdit::replace(range_to_replace, " { return; }\n\n"));
                        }
                    }
                }
                IfBody::ColonDelimited(body) => {
                    if body.statements.is_empty() {
                        let range = body.colon.start_offset()..body.terminator.end_offset();
                        edits.push(TextEdit::replace(range, "{ return; }"));
                    } else {
                        let first_stmt_start = body.statements.nodes[0].start_offset();
                        if gap_has_comment(body.colon.end_offset(), first_stmt_start) {
                            edits.push(TextEdit::replace(body.colon, "{ return; }\n"));
                        } else {
                            let range_to_replace = body.colon.start_offset()..first_stmt_start;
                            edits.push(TextEdit::replace(range_to_replace, "{ return; }\n\n"));
                        }

                        let last_stmt_end = body.statements.nodes.last().unwrap().end_offset();
                        if gap_has_comment(last_stmt_end, body.endif.span().start_offset()) {
                            edits.push(TextEdit::delete(body.endif.span().join(body.terminator.span())));
                        } else {
                            let endif_end = body.terminator.end_offset();
                            let range_to_delete = last_stmt_end..endif_end;
                            edits.push(TextEdit::delete(range_to_delete));
                        }
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
    let non_empty: Vec<_> = stmts.iter().filter(|s| statement_len(s) > 0).collect();

    if non_empty.len() != 1 {
        return None;
    }

    extract_single_if_from_statement(non_empty[0])
}

fn statement_len(stmt: &Statement) -> usize {
    match stmt {
        Statement::Noop(_) => 0,
        Statement::Block(Block { statements, .. }) => statements.len(),
        _ => 1,
    }
}

/// Checks if a statement is an "early exit" pattern that doesn't benefit from
/// the early return transformation (return, throw).
fn is_early_exit_statement(stmt: &Statement) -> bool {
    match stmt {
        Statement::Return(_) => true,
        Statement::Expression(expr) => matches!(expr.expression, Expression::Throw(_)),
        Statement::Block(block) => block.statements.len() == 1 && is_early_exit_statement(&block.statements.nodes[0]),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::PreferEarlyReturnRule;
    use crate::test_lint_fix;

    test_lint_fix! {
        name = fix_without_comments_keeps_compact_shape,
        rule = PreferEarlyReturnRule,
        code = indoc! {r"
            <?php

            function process($node) {
                if (is_array($node)) {
                    handle($node);
                }
            }
        "},
        fixed = indoc! {r"
            <?php

            function process($node) {
                if (!(is_array($node))) { return; }

            handle($node);
            }
        "}
    }

    test_lint_fix! {
        name = fix_preserves_leading_comment,
        rule = PreferEarlyReturnRule,
        code = indoc! {r"
            <?php

            function process($node) {
                if (is_array($node)) {
                    // keep only array nodes
                    handle($node);
                }
            }
        "},
        fixed = indoc! {r"
            <?php

            function process($node) {
                if (!(is_array($node))) { return; }

                    // keep only array nodes
                    handle($node);
            }
        "}
    }

    test_lint_fix! {
        name = fix_method_body,
        rule = PreferEarlyReturnRule,
        code = indoc! {r"
            <?php

            class Foo {
                public function bar($condition) {
                    if ($condition) {
                        doSomething();
                    }
                }
            }
        "},
        fixed = indoc! {r"
            <?php

            class Foo {
                public function bar($condition) {
                    if (!($condition)) { return; }

            doSomething();
                }
            }
        "}
    }
}
