use std::rc::Rc;

use ahash::HashMap;
use ahash::HashSet;

use mago_algebra::find_satisfying_assignments;
use mago_algebra::saturate_clauses;
use mago_codex::ttype::combine_union_types;
use mago_codex::ttype::get_bool;
use mago_codex::ttype::get_false;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_true;
use mago_codex::ttype::union::TUnion;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::artifacts::get_expression_range;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::context::scope::if_scope::IfScope;
use crate::error::AnalysisError;
use crate::formula::get_formula;
use crate::formula::negate_or_synthesize;
use crate::reconciler;
use crate::utils::conditional;

#[inline]
pub fn analyze_logical_and_operation<'ctx, 'arena>(
    binary: &Binary<'arena>,
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
) -> Result<(), AnalysisError> {
    let mut left_block_context = block_context.clone();
    let pre_referenced_var_ids = left_block_context.conditionally_referenced_variable_ids.clone();
    let pre_assigned_var_ids = left_block_context.assigned_variable_ids.clone();
    left_block_context.conditionally_referenced_variable_ids.clear();
    left_block_context.assigned_variable_ids.clear();
    left_block_context.reconciled_expression_clauses = Vec::new();

    let left_was_inside_general_use = left_block_context.inside_general_use;
    left_block_context.inside_general_use = true;
    binary.lhs.analyze(context, &mut left_block_context, artifacts)?;
    left_block_context.inside_general_use = left_was_inside_general_use;

    let lhs_type = match artifacts.get_rc_expression_type(&binary.lhs).cloned() {
        Some(lhs_type) => {
            check_logical_operand(context, binary.lhs, &lhs_type, "Left", "&&")?;

            lhs_type
        }
        None => Rc::new(get_mixed()),
    };

    let left_clauses = get_formula(
        binary.lhs.span(),
        binary.lhs.span(),
        binary.lhs,
        context.get_assertion_context_from_block(block_context),
        artifacts,
    )
    .unwrap_or_default();

    for (var_id, var_type) in &left_block_context.locals {
        if left_block_context.assigned_variable_ids.contains_key(var_id) {
            block_context.locals.insert(var_id.clone(), var_type.clone());
        }
    }

    let mut left_referenced_var_ids = left_block_context.conditionally_referenced_variable_ids.clone();
    let mut context_clauses = left_block_context.clauses.iter().map(|v| &**v).collect::<Vec<_>>();
    block_context.conditionally_referenced_variable_ids.extend(pre_referenced_var_ids);
    block_context.assigned_variable_ids.extend(pre_assigned_var_ids);
    context_clauses.extend(left_clauses.iter());
    if !left_block_context.reconciled_expression_clauses.is_empty() {
        let left_reconciled_clauses_hashed =
            left_block_context.reconciled_expression_clauses.iter().map(|v| &**v).collect::<HashSet<_>>();

        context_clauses.retain(|c| !left_reconciled_clauses_hashed.contains(c));
        if context_clauses.len() == 1 {
            let first = &context_clauses[0];
            if first.wedge && first.possibilities.is_empty() {
                context_clauses = Vec::new();
            }
        }
    }

    let simplified_clauses = saturate_clauses(context_clauses);
    let (left_assertions, active_left_assertions) = find_satisfying_assignments(
        simplified_clauses.as_slice(),
        Some(binary.lhs.span()),
        &mut left_referenced_var_ids,
    );

    let mut changed_var_ids = HashSet::default();
    let mut right_block_context;
    if !left_assertions.is_empty() {
        right_block_context = block_context.clone();

        reconciler::reconcile_keyed_types(
            context,
            &left_assertions,
            active_left_assertions,
            &mut right_block_context,
            &mut changed_var_ids,
            &left_referenced_var_ids,
            &binary.rhs.span(),
            !binary.operator.span().is_zero(),
            !block_context.inside_negation,
        );
    } else {
        right_block_context = left_block_context.clone()
    }

    let partitioned_clauses = BlockContext::remove_reconciled_clause_refs(
        &{
            let mut c = left_block_context.clauses.clone();
            c.extend(left_clauses.into_iter().map(Rc::new));
            c
        },
        &changed_var_ids,
    );
    right_block_context.clauses = partitioned_clauses.0;

    let result_type: TUnion;
    if lhs_type.is_always_falsy() {
        report_redundant_logical_operation(context, binary, "always falsy", "not evaluated", "`false`");

        result_type = get_false();
        let mut dead_rhs_context = right_block_context.clone();
        dead_rhs_context.has_returned = true;
        binary.rhs.analyze(context, &mut dead_rhs_context, artifacts)?;
    } else {
        binary.rhs.analyze(context, &mut right_block_context, artifacts)?;
        let rhs_type = match artifacts.get_rc_expression_type(&binary.rhs).cloned() {
            Some(rhs_type) => {
                check_logical_operand(context, binary.rhs, &rhs_type, "Right", "&&")?;
                rhs_type
            }
            None => Rc::new(get_mixed()),
        };

        let left_is_truthy = lhs_type.is_always_truthy();
        if left_is_truthy {
            report_redundant_logical_operation(
                context,
                binary,
                "always truthy",
                "evaluated",
                "the boolean value of the right-hand side",
            );
        }

        if rhs_type.is_always_falsy() {
            report_redundant_logical_operation(context, binary, "evaluated", "always falsy", "`false`");

            result_type = get_false();
        } else if rhs_type.is_always_truthy() {
            report_redundant_logical_operation(
                context,
                binary,
                "evaluated",
                "always truthy",
                "the boolean value of the left-hand side",
            );

            if left_is_truthy {
                result_type = get_true();
            } else {
                result_type = get_bool();
            }
        } else {
            result_type = get_bool();
        }
    }

    artifacts.set_expression_type(binary, result_type);

    block_context.conditionally_referenced_variable_ids = left_block_context.conditionally_referenced_variable_ids;
    block_context
        .conditionally_referenced_variable_ids
        .extend(right_block_context.conditionally_referenced_variable_ids);

    if block_context.inside_conditional {
        block_context.assigned_variable_ids = left_block_context.assigned_variable_ids;
        block_context.assigned_variable_ids.extend(right_block_context.assigned_variable_ids);
    }

    if let Some(if_body_context) = &block_context.if_body_context {
        let mut if_body_context_inner = if_body_context.borrow_mut();

        if !block_context.inside_negation {
            block_context.locals = right_block_context.locals;

            if_body_context_inner.locals.extend(block_context.locals.clone());
            if_body_context_inner
                .conditionally_referenced_variable_ids
                .extend(block_context.conditionally_referenced_variable_ids.clone());
            if_body_context_inner.assigned_variable_ids.extend(block_context.assigned_variable_ids.clone());
            if_body_context_inner.reconciled_expression_clauses.extend(partitioned_clauses.1);
        } else {
            block_context.locals = left_block_context.locals;
        }
    } else {
        block_context.locals = left_block_context.locals;
    }

    Ok(())
}

#[inline]
pub fn analyze_logical_or_operation<'ctx, 'arena>(
    binary: &Binary<'arena>,
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
) -> Result<(), AnalysisError> {
    let mut left_block_context;
    let mut left_referenced_var_ids;
    let left_assigned_var_ids;

    if !is_logical_or_operation(binary.lhs, 3) {
        let mut if_scope = IfScope::default();

        let (if_conditional_scope, applied_block_context) =
            conditional::analyze(context, block_context.clone(), artifacts, &mut if_scope, binary.lhs, false)?;
        *block_context = applied_block_context;

        left_block_context = if_conditional_scope.if_body_context;
        left_referenced_var_ids = if_conditional_scope.conditionally_referenced_variable_ids;
    } else {
        let pre_referenced_var_ids = block_context.conditionally_referenced_variable_ids.clone();
        block_context.conditionally_referenced_variable_ids = HashSet::default();

        let pre_assigned_var_ids = block_context.assigned_variable_ids.clone();

        left_block_context = block_context.clone();
        left_block_context.assigned_variable_ids = HashMap::default();

        let tmp_if_body_block_context = left_block_context.if_body_context;
        left_block_context.if_body_context = None;

        binary.lhs.analyze(context, &mut left_block_context, artifacts)?;

        left_block_context.if_body_context = tmp_if_body_block_context;

        for var_id in &left_block_context.parent_conflicting_clause_variables {
            block_context.remove_variable_from_conflicting_clauses(context, var_id, None);
        }

        let cloned_vars = block_context.locals.clone();
        for (var_id, left_type) in &left_block_context.locals {
            if let Some(context_type) = cloned_vars.get(var_id) {
                block_context.locals.insert(
                    var_id.clone(),
                    Rc::new(combine_union_types(context_type, left_type, context.codebase, false)),
                );
            } else if left_block_context.assigned_variable_ids.contains_key(var_id) {
                block_context.locals.insert(var_id.clone(), left_type.clone());
            }
        }

        left_referenced_var_ids = left_block_context.conditionally_referenced_variable_ids.clone();
        left_block_context.conditionally_referenced_variable_ids.extend(pre_referenced_var_ids);

        left_assigned_var_ids = left_block_context.assigned_variable_ids.clone();
        left_block_context.assigned_variable_ids.extend(pre_assigned_var_ids);

        left_referenced_var_ids.retain(|id| !left_assigned_var_ids.contains_key(id));
    }

    let lhs_type = match artifacts.get_rc_expression_type(&binary.lhs).cloned() {
        Some(lhs_type) => {
            check_logical_operand(context, binary.lhs, &lhs_type, "Left", "||")?;

            lhs_type
        }
        None => Rc::new(get_mixed()),
    };

    let left_clauses = get_formula(
        binary.lhs.span(),
        binary.lhs.span(),
        binary.lhs,
        context.get_assertion_context_from_block(block_context),
        artifacts,
    )
    .unwrap_or_default();

    let mut negated_left_clauses = negate_or_synthesize(
        left_clauses,
        binary.lhs,
        context.get_assertion_context_from_block(block_context),
        artifacts,
    );

    if !left_block_context.reconciled_expression_clauses.is_empty() {
        let left_reconciled_clauses_hashed =
            left_block_context.reconciled_expression_clauses.iter().map(|v| &**v).collect::<HashSet<_>>();

        negated_left_clauses.retain(|c| !left_reconciled_clauses_hashed.contains(c));

        if negated_left_clauses.len() == 1 {
            let first = &negated_left_clauses[0];
            if first.wedge && first.possibilities.is_empty() {
                negated_left_clauses = Vec::new();
            }
        }
    }

    let clauses_for_right_analysis =
        saturate_clauses(block_context.clauses.iter().map(|v| &**v).chain(negated_left_clauses.iter()));

    let (negated_type_assertions, active_negated_type_assertions) = find_satisfying_assignments(
        clauses_for_right_analysis.as_slice(),
        Some(binary.lhs.span()),
        &mut left_referenced_var_ids,
    );

    let mut changed_var_ids = HashSet::default();
    let mut right_block_context = block_context.clone();

    let result_type: TUnion;

    if lhs_type.is_always_truthy() {
        report_redundant_logical_operation(context, binary, "always true", "not evaluated", "`true`");
        result_type = get_true();
        right_block_context.has_returned = true;
        binary.rhs.analyze(context, &mut right_block_context, artifacts)?;
    } else {
        if !negated_type_assertions.is_empty() {
            reconciler::reconcile_keyed_types(
                context,
                &negated_type_assertions,
                active_negated_type_assertions,
                &mut right_block_context,
                &mut changed_var_ids,
                &left_referenced_var_ids,
                &binary.lhs.span(),
                true,
                !block_context.inside_negation,
            );
        }

        right_block_context.clauses = clauses_for_right_analysis.iter().map(|v| Rc::new(v.clone())).collect();

        if !changed_var_ids.is_empty() {
            let partiioned_clauses =
                BlockContext::remove_reconciled_clause_refs(&right_block_context.clauses, &changed_var_ids);
            right_block_context.clauses = partiioned_clauses.0;
            right_block_context.reconciled_expression_clauses.extend(partiioned_clauses.1);

            let partiioned_clauses =
                BlockContext::remove_reconciled_clause_refs(&block_context.clauses, &changed_var_ids);
            block_context.clauses = partiioned_clauses.0;
            block_context.reconciled_expression_clauses.extend(partiioned_clauses.1);
        }

        let pre_referenced_var_ids = right_block_context.conditionally_referenced_variable_ids.clone();
        right_block_context.conditionally_referenced_variable_ids = HashSet::default();

        let pre_assigned_var_ids = right_block_context.assigned_variable_ids.clone();
        right_block_context.assigned_variable_ids = HashMap::default();

        let tmp_if_body_context = right_block_context.if_body_context;
        right_block_context.if_body_context = None;

        binary.rhs.analyze(context, &mut right_block_context, artifacts)?;

        right_block_context.if_body_context = tmp_if_body_context;

        let rhs_type = match artifacts.get_rc_expression_type(&binary.rhs).cloned() {
            Some(rhs_type) => {
                check_logical_operand(context, binary.rhs, &rhs_type, "Right", "||")?;

                rhs_type
            }
            None => Rc::new(get_mixed()),
        };

        if lhs_type.is_always_falsy() {
            if rhs_type.is_always_falsy() {
                report_redundant_logical_operation(context, binary, "always falsy", "always falsy", "`false`");
                result_type = get_false();
            } else if rhs_type.is_always_truthy() {
                report_redundant_logical_operation(context, binary, "always falsy", "always truthy", "`true`");
                result_type = get_true();
            } else {
                report_redundant_logical_operation(
                    context,
                    binary,
                    "always false",
                    "evaluated",
                    "the boolean value of the right-hand side",
                );

                result_type = get_bool();
            }
        } else if rhs_type.is_always_falsy() {
            report_redundant_logical_operation(
                context,
                binary,
                "evaluated",
                "always falsy",
                "the boolean value of the left-hand side",
            );

            result_type = get_bool();
        } else if rhs_type.is_always_truthy() {
            report_redundant_logical_operation(context, binary, "evaluated", "always truthy", "`true`");

            result_type = get_true();
        } else {
            result_type = get_bool();
        }

        let mut right_referenced_var_ids = right_block_context.conditionally_referenced_variable_ids.clone();
        right_block_context.conditionally_referenced_variable_ids.extend(pre_referenced_var_ids);

        let right_assigned_var_ids = right_block_context.assigned_variable_ids.clone();
        right_block_context.assigned_variable_ids.extend(pre_assigned_var_ids);

        let right_clauses = get_formula(
            binary.rhs.span(),
            binary.rhs.span(),
            binary.rhs,
            context.get_assertion_context_from_block(block_context),
            artifacts,
        )
        .unwrap_or_default();

        let mut clauses_for_right_analysis = BlockContext::remove_reconciled_clauses(
            &clauses_for_right_analysis,
            &right_assigned_var_ids.into_keys().collect::<HashSet<_>>(),
        )
        .0;

        clauses_for_right_analysis.extend(right_clauses);

        let combined_right_clauses = saturate_clauses(clauses_for_right_analysis.iter());

        let (right_type_assertions, active_right_type_assertions) = find_satisfying_assignments(
            combined_right_clauses.as_slice(),
            Some(binary.rhs.span()),
            &mut right_referenced_var_ids,
        );

        if !right_type_assertions.is_empty() {
            let mut right_changed_var_ids = HashSet::default();

            reconciler::reconcile_keyed_types(
                context,
                &right_type_assertions,
                active_right_type_assertions,
                &mut right_block_context.clone(),
                &mut right_changed_var_ids,
                &right_referenced_var_ids,
                &binary.rhs.span(),
                !binary.operator.span().is_zero(),
                block_context.inside_negation,
            );
        }

        block_context
            .conditionally_referenced_variable_ids
            .extend(right_block_context.conditionally_referenced_variable_ids);
        block_context.assigned_variable_ids.extend(right_block_context.assigned_variable_ids);
    }

    if let Some(if_body_context) = &block_context.if_body_context {
        let mut if_body_context_inner = if_body_context.borrow_mut();
        let left_vars = left_block_context.locals.clone();
        let if_vars = if_body_context_inner.locals.clone();
        for (var_id, right_type) in right_block_context.locals.clone() {
            if let Some(if_type) = if_vars.get(&var_id) {
                if_body_context_inner
                    .locals
                    .insert(var_id, Rc::new(combine_union_types(&right_type, if_type, context.codebase, false)));
            } else if let Some(left_type) = left_vars.get(&var_id) {
                if_body_context_inner
                    .locals
                    .insert(var_id, Rc::new(combine_union_types(&right_type, left_type, context.codebase, false)));
            }
        }

        if_body_context_inner
            .conditionally_referenced_variable_ids
            .extend(block_context.conditionally_referenced_variable_ids.clone());
        if_body_context_inner.assigned_variable_ids.extend(block_context.assigned_variable_ids.clone());
    }

    artifacts.set_expression_type(binary, result_type);

    Ok(())
}

/// Analyzes the logical XOR operator (`xor`).
///
/// The `xor` operator evaluates both operands and returns `true` if exactly one of them is truthy,
/// and `false` otherwise. The result type is always `bool`.
/// This function analyzes both operands, checks for problematic types in a boolean context,
/// determines if the result can be statically known, and sets up data flow.
pub fn analyze_logical_xor_operation<'ctx, 'arena>(
    binary: &Binary<'arena>,
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
) -> Result<(), AnalysisError> {
    binary.lhs.analyze(context, block_context, artifacts)?;
    binary.rhs.analyze(context, block_context, artifacts)?;

    let fallback_type = Rc::new(get_mixed());
    let lhs_type = artifacts.get_rc_expression_type(&binary.lhs).unwrap_or(&fallback_type);
    let rhs_type = artifacts.get_rc_expression_type(&binary.rhs).unwrap_or(&fallback_type);

    check_logical_operand(context, binary.lhs, lhs_type, "Left", "xor")?;
    check_logical_operand(context, binary.rhs, rhs_type, "Right", "xor")?;

    let result_type = if lhs_type.is_always_truthy() && rhs_type.is_always_truthy() {
        if !block_context.inside_loop_expressions {
            report_redundant_logical_operation(context, binary, "always true", "always true", "`false`");
        }

        get_false()
    } else if lhs_type.is_always_truthy() && rhs_type.is_always_falsy() {
        if !block_context.inside_loop_expressions {
            report_redundant_logical_operation(context, binary, "always true", "always false", "`true`");
        }

        get_true()
    } else if lhs_type.is_always_falsy() && rhs_type.is_always_truthy() {
        if !block_context.inside_loop_expressions {
            report_redundant_logical_operation(context, binary, "always false", "always true", "`true`");
        }

        get_true()
    } else if lhs_type.is_always_falsy() && rhs_type.is_always_falsy() {
        if !block_context.inside_loop_expressions {
            report_redundant_logical_operation(context, binary, "always false", "always false", "`false`");
        }

        get_false()
    } else {
        get_bool()
    };

    artifacts.expression_types.insert(get_expression_range(binary), Rc::new(result_type));

    Ok(())
}

/// Checks a single operand of a logical operation (like AND, OR, XOR) for problematic types.
/// Reports errors for `mixed` and warnings for types that PHP coerces to boolean
/// (e.g., `null`, `array`, `resource`, `object`).
fn check_logical_operand<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    operand: &'ast Expression<'arena>,
    operand_type: &TUnion,
    side: &'static str,
    operator_name: &'static str,
) -> Result<bool, AnalysisError> {
    let mut critical_error_found = false;

    if operand_type.is_mixed() {
        context.collector.report_with_code(
            IssueCode::MixedOperand,
            Issue::error(format!("{side} operand in `{operator_name}` operation has `mixed` type."))
                .with_annotation(Annotation::primary(operand.span()).with_message("This has type `mixed`"))
                .with_note(format!(
                    "Using `mixed` in a boolean context like `{operator_name}` is unsafe as its truthiness is unknown."
                ))
                .with_help("Ensure this operand has a known type or explicitly cast to `bool`."),
        );

        critical_error_found = true;
    } else if operand_type.is_null() {
        context.collector.report_with_code(
            IssueCode::NullOperand,
            Issue::warning(format!(
                "{side} operand in `{operator_name}` operation is `null`, which coerces to `false`."
            ))
            .with_annotation(Annotation::primary(operand.span()).with_message("This is `null` (coerces to `false`)"))
            .with_help("Explicitly check for `null` or cast to `bool` if this coercion is not intended."),
        );
    } else if operand_type.is_array() {
        context.collector.report_with_code(
            IssueCode::InvalidOperand,
            Issue::warning(format!("{side} operand in `{operator_name}` operation is an `array`."))
                .with_annotation(Annotation::primary(operand.span()).with_message("This is an `array`"))
                .with_note(
                    "Arrays coerce to `false` if empty, `true` if non-empty. This implicit conversion can be unclear.",
                )
                .with_help("Consider using `empty()` or `count()` for explicit checks, or cast to `bool`."),
        );
    } else if operand_type.is_objecty() {
        context.collector.report_with_code(
            IssueCode::InvalidOperand,
            Issue::warning(format!("{side} operand in `{operator_name}` operation is an `object`."))
                .with_annotation(Annotation::primary(operand.span()).with_message("This is an `object`"))
                .with_note(
                    "Objects generally coerce to `true` in boolean contexts. Ensure this is the intended behavior.",
                )
                .with_help("If specific truthiness is required, implement a method on the object or cast explicitly."),
        );
    } else if operand_type.is_resource() {
        context.collector.report_with_code(
            IssueCode::InvalidOperand,
            Issue::warning(format!("{side} operand in `{operator_name}` operation is a `resource`."))
                .with_annotation(Annotation::primary(operand.span()).with_message("This is a `resource`"))
                .with_note("Resources generally coerce to `true`. This implicit conversion can be unclear.")
                .with_help("Explicitly check the state of the resource or cast to `bool` if necessary."),
        );
    }

    Ok(critical_error_found)
}

/// Helper to report redundant logical operation issues.
fn report_redundant_logical_operation<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    binary: &'ast Binary<'arena>,
    lhs_description: &str,
    rhs_description: &str,
    result_value_str: &str,
) {
    let operator_span = binary.operator.span();
    if operator_span.is_zero() {
        // Do not report issues for synthetic nodes.
        return;
    }

    context.collector.report_with_code(
        IssueCode::RedundantLogicalOperation,
        Issue::help(format!(
            "Redundant `{}` operation: left operand is {} and right operand is {}.",
            binary.operator.as_str(),
            lhs_description,
            rhs_description
        ))
        .with_annotation(
            Annotation::primary(binary.lhs.span()).with_message(format!("Left operand is {lhs_description}")),
        )
        .with_annotation(
            Annotation::secondary(binary.rhs.span()).with_message(format!("Right operand is {rhs_description}")),
        )
        .with_note(format!(
            "The `{}` operator will always return {} in this case.",
            binary.operator.as_str(),
            result_value_str
        ))
        .with_help(format!(
            "Consider simplifying or removing this logical expression as it always evaluates to {result_value_str}."
        )),
    );
}

#[inline]
const fn is_logical_or_operation<'ast, 'arena>(expression: &'ast Expression<'arena>, max_nesting: usize) -> bool {
    if max_nesting == 0 {
        return true;
    }

    match expression {
        Expression::Parenthesized(p) => is_logical_or_operation(p.expression, max_nesting),
        Expression::Binary(b) => match b.operator {
            BinaryOperator::Or(_) | BinaryOperator::LowOr(_) => is_logical_or_operation(b.lhs, max_nesting - 1),
            _ => false,
        },
        _ => false,
    }
}
