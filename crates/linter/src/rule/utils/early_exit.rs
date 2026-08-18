use mago_allocator::Arena;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::cst::BinaryOperator;
use mago_syntax::cst::Block;
use mago_syntax::cst::Expression;
use mago_syntax::cst::If;
use mago_syntax::cst::IfBody;
use mago_syntax::cst::Statement;
use mago_syntax::cst::UnaryPrefixOperator;
use mago_text_edit::TextEdit;

use crate::context::LintContext;

/// The per-rule wording and behavior for an early-exit transformation
/// (`prefer-early-return` and `prefer-early-continue`).
pub struct EarlyExitPattern {
    pub keyword: &'static str,
    pub code: &'static str,
    pub title: &'static str,
    pub primary_message: &'static str,
    pub secondary_message: &'static str,
    pub help: &'static str,
    pub is_early_exit_statement: fn(&Statement) -> bool,
}

pub fn check_early_exit<A>(
    ctx: &mut LintContext<'_, '_, A>,
    level: Level,
    if_stmt: &If,
    subject_span: Span,
    max_allowed_statements: usize,
    pattern: &EarlyExitPattern,
) where
    A: Arena,
{
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

    if body_len <= max_allowed_statements {
        return;
    }

    // Skip if the body is an early exit statement (return, throw, ...).
    // These are already simple single-statement bodies with no nesting to reduce,
    // and transforming them doesn't improve readability.
    let is_early_exit = match &if_stmt.body {
        IfBody::Statement(body) => (pattern.is_early_exit_statement)(body.statement),
        IfBody::ColonDelimited(body) => {
            body.statements.len() == 1 && (pattern.is_early_exit_statement)(&body.statements.nodes[0])
        }
    };

    if is_early_exit {
        return;
    }

    let issue = Issue::new(level, pattern.title)
        .with_code(pattern.code)
        .with_annotation(Annotation::primary(if_stmt.span()).with_message(pattern.primary_message))
        .with_annotation(Annotation::secondary(subject_span).with_message(pattern.secondary_message))
        .with_help(pattern.help)
        .with_note("This pattern improves readability by reducing nesting levels.");

    let keyword = pattern.keyword;
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

        let exit_block = format!("{{ {keyword}; }}");
        match &if_stmt.body {
            IfBody::Statement(body) => {
                if let Statement::Block(block) = body.statement {
                    let stmts = block.statements.as_slice();
                    if let [first, ..] = stmts {
                        let first_stmt_start = first.start_offset();
                        if gap_has_comment(block.left_brace.end_offset(), first_stmt_start) {
                            edits.push(TextEdit::replace(block.left_brace, format!("{exit_block}\n")));
                        } else {
                            let range_to_replace = block.left_brace.start_offset()..first_stmt_start;
                            edits.push(TextEdit::replace(range_to_replace, format!("{exit_block}\n\n")));
                        }

                        let last = match stmts {
                            [.., last] => last,
                            _ => first,
                        };

                        let last_stmt_end = last.end_offset();
                        if gap_has_comment(last_stmt_end, block.right_brace.start_offset()) {
                            edits.push(TextEdit::delete(block.right_brace));
                        } else {
                            let range_to_delete = last_stmt_end..block.right_brace.end_offset();
                            edits.push(TextEdit::delete(range_to_delete));
                        }
                    } else {
                        edits.push(TextEdit::replace(block.left_brace.join(block.right_brace), exit_block));
                    }
                } else {
                    let stmt_start = body.statement.start_offset();
                    if gap_has_comment(if_stmt.right_parenthesis.end_offset(), stmt_start) {
                        edits
                            .push(TextEdit::insert(if_stmt.right_parenthesis.end_offset(), format!(" {exit_block}\n")));
                    } else {
                        let range_to_replace = if_stmt.right_parenthesis.end_offset()..stmt_start;
                        edits.push(TextEdit::replace(range_to_replace, format!(" {exit_block}\n\n")));
                    }
                }
            }
            IfBody::ColonDelimited(body) => {
                let stmts = body.statements.as_slice();
                if let [first, ..] = stmts {
                    let first_stmt_start = first.start_offset();
                    if gap_has_comment(body.colon.end_offset(), first_stmt_start) {
                        edits.push(TextEdit::replace(body.colon, format!("{exit_block}\n")));
                    } else {
                        let range_to_replace = body.colon.start_offset()..first_stmt_start;
                        edits.push(TextEdit::replace(range_to_replace, format!("{exit_block}\n\n")));
                    }

                    let last = match stmts {
                        [.., last] => last,
                        _ => first,
                    };

                    let last_stmt_end = last.end_offset();
                    if gap_has_comment(last_stmt_end, body.endif.span().start_offset()) {
                        edits.push(TextEdit::delete(body.endif.span().join(body.terminator.span())));
                    } else {
                        let endif_end = body.terminator.end_offset();
                        let range_to_delete = last_stmt_end..endif_end;
                        edits.push(TextEdit::delete(range_to_delete));
                    }
                } else {
                    let range = body.colon.start_offset()..body.terminator.end_offset();
                    edits.push(TextEdit::replace(range, exit_block));
                }
            }
        }
    });
}

pub fn extract_single_if_from_statement<'ast, 'arena>(stmt: &'ast Statement<'arena>) -> Option<&'ast If<'arena>> {
    match stmt {
        Statement::If(if_stmt) => Some(if_stmt),
        Statement::Block(block) => extract_single_if_from_statements(block.statements.as_slice()),
        _ => None,
    }
}

pub fn extract_single_if_from_statements<'ast, 'arena>(stmts: &'ast [Statement<'arena>]) -> Option<&'ast If<'arena>> {
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
