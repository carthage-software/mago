use std::cell::RefCell;
use std::rc::Rc;

use ahash::HashMap;
use ahash::HashSet;
use indexmap::IndexMap;

use mago_codex::ttype::TType;
use mago_codex::ttype::union::TUnion;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::context::scope::conditional_scope::IfConditionalScope;
use crate::context::scope::if_scope::IfScope;
use crate::error::AnalysisError;
use crate::reconciler::reconcile_keyed_types;

pub(crate) fn analyze<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    mut outer_context: BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    if_scope: &mut IfScope,
    condition: &Expression<'arena>,
    check_for_paradoxes: bool,
) -> Result<(IfConditionalScope<'ctx>, BlockContext<'ctx>), AnalysisError> {
    let mut entry_clauses = vec![];

    let old_outer_context = outer_context.clone();
    let mut has_outer_context_changes = false;

    if !if_scope.negated_clauses.is_empty() {
        entry_clauses.extend(if_scope.negated_clauses.iter().cloned());

        let mut changed_var_ids = HashSet::default();

        if !if_scope.negated_types.is_empty() {
            let mut tmp_context = outer_context.clone();

            reconcile_keyed_types(
                context,
                &if_scope.negated_types,
                IndexMap::new(),
                &mut tmp_context,
                &mut changed_var_ids,
                &HashSet::default(),
                &condition.span(),
                check_for_paradoxes,
                false,
            );

            if !changed_var_ids.is_empty() {
                outer_context = tmp_context;
                has_outer_context_changes = true;
            }
        }
    }

    let externally_applied_if_cond_expr = get_definitely_evaluated_expression_after_if(condition);
    let internally_applied_if_cond_expr = get_definitely_evaluated_expression_inside_if(condition);
    let mut externally_applied_context = if has_outer_context_changes { outer_context } else { old_outer_context };

    let pre_condition_locals = externally_applied_context.locals.clone();
    let pre_referenced_var_ids = std::mem::take(&mut externally_applied_context.conditionally_referenced_variable_ids);
    let pre_assigned_var_ids = std::mem::take(&mut externally_applied_context.assigned_variable_ids);

    let mut if_body_context = None;
    if externally_applied_if_cond_expr != internally_applied_if_cond_expr {
        if_body_context = Some(externally_applied_context.clone());
    }

    let was_inside_conditional = externally_applied_context.inside_conditional;

    externally_applied_context.inside_conditional = true;
    let tmp_if_body_context = std::mem::take(&mut externally_applied_context.if_body_context);
    externally_applied_if_cond_expr.analyze(context, &mut externally_applied_context, artifacts)?;
    externally_applied_context.if_body_context = tmp_if_body_context;

    let first_cond_assigned_var_ids = externally_applied_context.assigned_variable_ids.clone();
    let first_cond_referenced_var_ids = externally_applied_context.conditionally_referenced_variable_ids.clone();

    externally_applied_context.assigned_variable_ids.extend(pre_assigned_var_ids);
    externally_applied_context.conditionally_referenced_variable_ids.extend(pre_referenced_var_ids);
    externally_applied_context.inside_conditional = was_inside_conditional;

    let mut if_body_context = if_body_context.unwrap_or_else(|| externally_applied_context.clone());

    let tmp_if_body_context_nested = if_body_context.if_body_context;
    if_body_context.if_body_context = None;

    let mut if_conditional_context = if_body_context.clone();
    if_conditional_context.if_body_context = Some(Rc::new(RefCell::new(if_body_context)));

    let post_if_context = externally_applied_context.clone();
    let mut conditionally_referenced_variable_ids;
    let assigned_in_conditional_variable_ids;
    if internally_applied_if_cond_expr != condition || externally_applied_if_cond_expr != condition {
        if_conditional_context.assigned_variable_ids = HashMap::default();
        if_conditional_context.conditionally_referenced_variable_ids = HashSet::default();

        let was_inside_conditional = if_conditional_context.inside_conditional;
        if_conditional_context.inside_conditional = true;
        condition.analyze(context, &mut if_conditional_context, artifacts)?;
        if_conditional_context.inside_conditional = was_inside_conditional;

        if_conditional_context.conditionally_referenced_variable_ids.extend(first_cond_referenced_var_ids);
        if_conditional_context.assigned_variable_ids.extend(first_cond_assigned_var_ids);

        conditionally_referenced_variable_ids = if_conditional_context.conditionally_referenced_variable_ids.clone();
        assigned_in_conditional_variable_ids = if_conditional_context.assigned_variable_ids.clone();
    } else {
        conditionally_referenced_variable_ids = first_cond_referenced_var_ids.clone();
        assigned_in_conditional_variable_ids = first_cond_assigned_var_ids.clone();
    }

    let newish_var_ids = if_conditional_context
        .locals
        .into_keys()
        .filter(|k| {
            !pre_condition_locals.contains_key(k)
                && !conditionally_referenced_variable_ids.contains(k)
                && !assigned_in_conditional_variable_ids.contains_key(k)
        })
        .collect::<HashSet<_>>();

    if check_for_paradoxes && let Some(condition_type) = artifacts.get_rc_expression_type(condition) {
        handle_paradoxical_condition(context, condition, condition_type);
    }

    conditionally_referenced_variable_ids.retain(|k| !assigned_in_conditional_variable_ids.contains_key(k));
    conditionally_referenced_variable_ids.extend(newish_var_ids);

    let mut if_body_context = unsafe {
        // SAFETY: We know the Option is `Some` and the `Rc` has a strong count of 1.
        let rc = if_conditional_context.if_body_context.unwrap_unchecked();
        let ref_cell = Rc::try_unwrap(rc).unwrap_unchecked();
        ref_cell.into_inner()
    };

    if_body_context.if_body_context = tmp_if_body_context_nested;

    Ok((
        IfConditionalScope {
            if_body_context,
            post_if_context,
            conditionally_referenced_variable_ids,
            assigned_in_conditional_variable_ids,
            entry_clauses,
        },
        externally_applied_context,
    ))
}

fn get_definitely_evaluated_expression_after_if<'ast, 'arena>(
    condition: &'ast Expression<'arena>,
) -> &'ast Expression<'arena> {
    match &condition {
        Expression::Parenthesized(p) => {
            return get_definitely_evaluated_expression_after_if(p.expression);
        }
        Expression::Binary(binary) => {
            if let BinaryOperator::Or(_) | BinaryOperator::LowOr(_) = binary.operator {
                return get_definitely_evaluated_expression_after_if(binary.lhs);
            }

            return condition;
        }
        Expression::UnaryPrefix(unary) => {
            if let UnaryPrefixOperator::Not(_) = unary.operator {
                let inner_expression = get_definitely_evaluated_expression_inside_if(unary.operand);

                if inner_expression != unary.operand {
                    return inner_expression;
                }
            }
        }
        _ => {}
    }

    condition
}

fn get_definitely_evaluated_expression_inside_if<'ast, 'arena>(
    condition: &'ast Expression<'arena>,
) -> &'ast Expression<'arena> {
    match &condition {
        Expression::Parenthesized(p) => {
            return get_definitely_evaluated_expression_inside_if(p.expression);
        }
        Expression::Binary(binary) => {
            if let BinaryOperator::Or(_) | BinaryOperator::LowOr(_) = binary.operator {
                return get_definitely_evaluated_expression_inside_if(binary.lhs);
            }

            return condition;
        }
        Expression::UnaryPrefix(unary) => {
            if let UnaryPrefixOperator::Not(_) = unary.operator {
                let inner_expression = get_definitely_evaluated_expression_inside_if(unary.operand);

                if inner_expression != unary.operand {
                    return inner_expression;
                }
            }
        }
        _ => {}
    }

    condition
}

pub fn handle_paradoxical_condition<'ctx, 'ast, 'arena, T: HasSpan>(
    context: &mut Context<'ctx, 'arena>,
    expression: &'ast T,
    expression_type: &TUnion,
) {
    let type_id = expression_type.get_id();

    if expression_type.is_always_falsy() {
        context.collector.report_with_code(
            IssueCode::ImpossibleCondition,
            Issue::warning(format!(
                "This condition (type `{type_id}`) will always evaluate to false."
            ))
            .with_annotation(
                Annotation::primary(expression.span())
                    .with_message(format!("Expression of type `{type_id}` is always falsy")),
            )
            .with_note(
                "Because this condition is always false, the code block it controls will never be executed."
            )
            .with_help(
                "Check the logic of this expression. If the code block is intended to be unreachable, consider removing it. Otherwise, revise the condition.",
            ),
        );
    } else if expression_type.is_always_truthy() {
        context.collector.report_with_code(
            IssueCode::RedundantCondition,
            Issue::warning(format!(
                "This condition (type `{type_id}`) will always evaluate to true."
            ))
            .with_annotation(
                Annotation::primary(expression.span())
                    .with_message(format!("Expression of type `{type_id}` is always truthy")),
            )
            .with_note(
                "Because this condition is always true, the code block it controls will always execute if this part of the code is reached."
            ).with_note(
                "The explicit condition might be redundant."
            )
            .with_help(
                "Consider simplifying or removing the conditional check if the guarded code should always execute, or verify the expression's logic if a conditional check is truly needed.",
            ),
        );
    }
}
