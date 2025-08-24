use std::ops::Deref;
use std::rc::Rc;

use ahash::HashSet;

use mago_algebra::clause::Clause;
use mago_algebra::find_satisfying_assignments;
use mago_algebra::saturate_clauses;
use mago_codex::assertion::Assertion;
use mago_codex::ttype::TType;
use mago_codex::ttype::combine_optional_union_types;
use mago_codex::ttype::combine_union_types;
use mago_codex::ttype::get_mixed;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::context::scope::if_scope::IfScope;
use crate::context::utils::inherit_branch_context_properties;
use crate::error::AnalysisError;
use crate::formula::get_formula;
use crate::formula::negate_or_synthesize;
use crate::reconciler::assertion_reconciler;
use crate::reconciler::reconcile_keyed_types;
use crate::utils::conditional;
use crate::utils::expression::is_derived_access_path;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for Conditional<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        analyze_conditional(context, block_context, artifacts, self.condition, self.then, self.r#else, self.span())
    }
}

pub(super) fn analyze_conditional<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    condition: &'ast Expression<'arena>,
    then: Option<&'ast Expression<'arena>>,
    r#else: &'ast Expression<'arena>,
    span: Span,
) -> Result<(), AnalysisError> {
    let mut if_scope = IfScope::new();
    let (if_conditional_scope, applied_block_context) =
        conditional::analyze(context, block_context.clone(), artifacts, &mut if_scope, condition, false)?;

    *block_context = applied_block_context;

    let mut if_block_context = if_conditional_scope.if_body_context;
    let mut conditionally_referenced_variable_ids = if_conditional_scope.conditionally_referenced_variable_ids;

    let assertion_context = context.get_assertion_context_from_block(block_context);
    let mut if_clauses = get_formula(condition.span(), condition.span(), condition, assertion_context, artifacts).unwrap_or_else(|| {
        context.collector.report_with_code(
            IssueCode::ConditionIsTooComplex,
            Issue::warning("Condition is too complex for precise type analysis.")
                .with_annotation(
                    Annotation::primary(condition.span())
                        .with_message("This conditional expression is too complex for the analyzer to fully understand"),
                )
                .with_annotation(
                    Annotation::secondary(span)
                        .with_message("As a result, type inference for this conditional expression may be inaccurate"),
                )
                .with_note(
                    "The analyzer limits the number of logical paths it explores for a single condition to prevent performance issues with exponentially complex expressions."
                )
                .with_note(
                    "Because this limit was exceeded, type assertions from the condition will not be applied, which may lead to incorrect type information for variables used in the `then` or `else` branches."
                )
                .with_help(
                    "Consider refactoring this complex condition into a simpler `if/else` block or breaking it down into smaller, intermediate boolean variables.",
                ),
        );

        vec![]
    });

    let mut mixed_variables = HashSet::default();
    for (variable_id, variable_type) in block_context.locals.iter() {
        if variable_type.is_mixed() {
            mixed_variables.insert(variable_id.clone());
        }
    }

    for variable_id in &block_context.variables_possibly_in_scope {
        if !block_context.locals.contains_key(variable_id) {
            mixed_variables.insert(variable_id.clone());
        }
    }

    for clause in &mut if_clauses {
        let keys = clause.possibilities.keys().cloned().collect::<Vec<String>>();
        mixed_variables.retain(|i| !keys.contains(i));

        'outer: for key in keys {
            for mixed_var_id in &mixed_variables {
                if is_derived_access_path(&key, mixed_var_id) {
                    *clause = Clause::new(
                        Default::default(),
                        condition.span(),
                        condition.span(),
                        Some(true),
                        Some(true),
                        Some(false),
                    );

                    break 'outer;
                }
            }
        }
    }

    let entry_clauses = block_context.clauses.to_vec();

    if_clauses = saturate_clauses(&if_clauses);
    let mut conditional_context_clauses = if entry_clauses.is_empty() {
        if_clauses.clone().into_iter().map(Rc::new).collect::<Vec<_>>()
    } else {
        saturate_clauses(if_clauses.iter().chain(entry_clauses.iter().map(Rc::deref)))
            .into_iter()
            .map(Rc::new)
            .collect::<Vec<_>>()
    };

    if !if_block_context.reconciled_expression_clauses.is_empty() {
        conditional_context_clauses.retain(|clause| !if_block_context.reconciled_expression_clauses.contains(clause));

        if if_block_context.clauses.len() == 1
            && if_block_context.clauses[0].wedge
            && if_block_context.clauses[0].possibilities.is_empty()
        {
            if_block_context.clauses.clear();
            if_block_context.reconciled_expression_clauses.clear();
        }
    }

    if_scope.negated_clauses =
        negate_or_synthesize(if_clauses, condition, context.get_assertion_context_from_block(block_context), artifacts);

    if_scope.negated_types = find_satisfying_assignments(
        saturate_clauses(block_context.clauses.iter().map(Rc::deref).chain(if_scope.negated_clauses.iter())).as_slice(),
        None,
        &mut HashSet::default(),
    )
    .0;

    let (reconcilable_if_types, active_if_types) = find_satisfying_assignments(
        conditional_context_clauses.into_iter().map(|rc| (*rc).clone()).collect::<Vec<_>>().as_slice(),
        Some(condition.span()),
        &mut conditionally_referenced_variable_ids,
    );

    if !reconcilable_if_types.is_empty() {
        let mut changed_variable_ids = HashSet::default();

        reconcile_keyed_types(
            context,
            &reconcilable_if_types,
            active_if_types,
            &mut if_block_context,
            &mut changed_variable_ids,
            &conditionally_referenced_variable_ids,
            &condition.span(),
            false,
            false,
        );
    }

    let mut else_block_context = block_context.clone();

    if let Some(then) = then.as_ref() {
        then.analyze(context, &mut if_block_context, artifacts)?;

        block_context
            .conditionally_referenced_variable_ids
            .extend(if_block_context.conditionally_referenced_variable_ids.iter().cloned());
    }

    else_block_context.clauses =
        saturate_clauses(else_block_context.clauses.iter().map(Rc::deref).chain(if_scope.negated_clauses.iter()))
            .into_iter()
            .map(Rc::new)
            .collect::<Vec<_>>();

    if !if_scope.negated_types.is_empty() {
        let mut changed_variable_ids = HashSet::default();

        reconcile_keyed_types(
            context,
            &if_scope.negated_types,
            Default::default(), // todo: this is sort of a hack, we should probably pass the active types here
            &mut else_block_context,
            &mut changed_variable_ids,
            &conditionally_referenced_variable_ids,
            &condition.span(),
            true,
            false,
        );

        else_block_context.clauses = BlockContext::remove_reconciled_clauses(
            &else_block_context.clauses.iter().map(Rc::deref).cloned().collect(),
            &changed_variable_ids,
        )
        .0
        .into_iter()
        .map(Rc::new)
        .collect();
    }

    let was_inside_general_use = else_block_context.inside_general_use;
    else_block_context.inside_general_use = true;
    r#else.analyze(context, &mut else_block_context, artifacts)?;
    else_block_context.inside_general_use = was_inside_general_use;

    let if_assigned_variables = if_block_context.assigned_variable_ids.keys().cloned().collect::<HashSet<_>>();
    let else_assigned_variables = else_block_context.assigned_variable_ids.keys().cloned().collect::<HashSet<_>>();
    let assigned_variables =
        if_assigned_variables.intersection(&else_assigned_variables).cloned().collect::<HashSet<_>>();

    for assigned_variable in assigned_variables {
        let Some(if_type) = if_block_context.locals.get(&assigned_variable) else {
            continue;
        };

        let Some(else_type) = else_block_context.locals.get(&assigned_variable) else {
            continue;
        };

        block_context.locals.insert(
            assigned_variable,
            Rc::new(combine_union_types(if_type.as_ref(), else_type.as_ref(), context.codebase, false)),
        );
    }

    let if_redefined_variables = if_block_context
        .get_redefined_locals(&block_context.locals, false, &mut HashSet::default())
        .keys()
        .cloned()
        .collect::<HashSet<_>>();

    let else_redefined_variables = else_block_context
        .get_redefined_locals(&block_context.locals, false, &mut HashSet::default())
        .keys()
        .cloned()
        .collect::<HashSet<_>>();

    let redefined_variable_ids =
        if_redefined_variables.intersection(&else_redefined_variables).cloned().collect::<HashSet<_>>();

    for redefined_variable_id in redefined_variable_ids {
        let if_type = if_block_context.locals.get(&redefined_variable_id);
        let else_type = else_block_context.locals.get(&redefined_variable_id);

        let combined_type = combine_optional_union_types(
            if_type.map(|rc| rc.as_ref()),
            else_type.map(|rc| rc.as_ref()),
            context.codebase,
        );

        block_context.locals.insert(redefined_variable_id, Rc::new(combined_type));
    }

    inherit_branch_context_properties(context, block_context, &if_block_context);
    inherit_branch_context_properties(context, block_context, &else_block_context);

    for if_redefined_variable in if_redefined_variables {
        let Some(if_type) = if_block_context.locals.get(&if_redefined_variable) else {
            continue;
        };

        let Some(previous_type) = block_context.locals.get(&if_redefined_variable) else {
            continue;
        };

        let combined_type =
            combine_optional_union_types(Some(if_type.as_ref()), Some(previous_type.as_ref()), context.codebase);

        block_context.locals.insert(if_redefined_variable, Rc::new(combined_type));
    }

    for else_redefined_variable in else_redefined_variables {
        let Some(else_type) = else_block_context.locals.get(&else_redefined_variable) else {
            continue;
        };

        let Some(previous_type) = block_context.locals.get(&else_redefined_variable) else {
            continue;
        };

        let combined_type =
            combine_optional_union_types(Some(else_type.as_ref()), Some(previous_type.as_ref()), context.codebase);

        block_context.locals.insert(else_redefined_variable, Rc::new(combined_type));
    }

    block_context.variables_possibly_in_scope.extend(if_block_context.variables_possibly_in_scope);
    block_context.variables_possibly_in_scope.extend(else_block_context.variables_possibly_in_scope);

    block_context.conditionally_referenced_variable_ids.extend(if_block_context.conditionally_referenced_variable_ids);
    block_context
        .conditionally_referenced_variable_ids
        .extend(else_block_context.conditionally_referenced_variable_ids);

    let mut then_type = None;
    let condition_type = artifacts.get_rc_expression_type(&condition).cloned();
    if let Some(then_expression) = then.as_ref() {
        if let Some(rc_then_type) = artifacts.get_rc_expression_type(then_expression).cloned() {
            then_type = Some(rc_then_type);
        }
    } else if let Some(condition_type) = condition_type.as_ref() {
        let if_return_type_reconciled = assertion_reconciler::reconcile(
            context,
            &Assertion::Truthy,
            Some(condition_type.as_ref()),
            false,
            Some(&"".to_string()),
            block_context.inside_loop,
            Some(&condition.span()),
            false,
            false,
        );

        then_type = Some(Rc::new(if_return_type_reconciled));
    }

    let mut is_condition_truthy = false;
    let mut is_condition_falsy = false;
    if let Some(condition_type) = condition_type.as_ref() {
        if condition_type.is_always_truthy() {
            is_condition_truthy = true;

            let issue =
                if let Some(then) = then {
                    // `$A ? $B : $C` where `$A` is always truthy
                    Issue::help("Redundant ternary operator: condition is always truthy.")
                        .with_annotation(Annotation::primary(condition.span()).with_message(format!(
                            "This condition (type `{}`) is always truthy",
                            condition_type.get_id()
                        )))
                        .with_annotation(Annotation::secondary(then.span()).with_message(
                            "This `then` branch is always evaluated, making it the result of the expression",
                        ))
                        .with_annotation(
                            Annotation::secondary(r#else.span())
                                .with_message("This `else` branch will never be evaluated"),
                        )
                        .with_note(
                            "The ternary operator `? :` evaluates the `else` branch only when the condition is falsy.",
                        )
                        .with_help("Consider replacing the entire expression with just this `then` branch.")
                } else {
                    // `$A ?: $C` where `$A` is always truthy
                    Issue::help("Redundant Elvis operator: left-hand side is always truthy.")
                    .with_annotation(Annotation::primary(condition.span()).with_message(format!(
                        "This expression (type `{}`) is always truthy",
                        condition_type.get_id()
                    )))
                    .with_annotation(
                        Annotation::secondary(r#else.span())
                            .with_message("This right-hand side will never be evaluated"),
                    )
                    .with_note(
                        "The Elvis operator `?:` evaluates the right-hand side only if the left-hand side is falsy.",
                    )
                    .with_help("Consider removing the `?:` operator and the right-hand side expression.")
                };

            context.collector.report_with_code(IssueCode::RedundantCondition, issue);
        } else if condition_type.is_always_falsy() {
            is_condition_falsy = true;

            // https://en.wikipedia.org/wiki/Ternary_conditional_operator
            let issue =
                if let Some(then) = then {
                    // `$A ? $B : $C` where `$A` is always falsy
                    Issue::warning("Redundant ternary operator: condition is always falsy.")
                        .with_annotation(Annotation::primary(condition.span()).with_message(format!(
                            "This condition (type `{}`) is always falsy",
                            condition_type.get_id()
                        )))
                        .with_annotation(
                            Annotation::secondary(then.span())
                                .with_message("This `then` branch will never be evaluated"),
                        )
                        .with_annotation(Annotation::secondary(r#else.span()).with_message(
                            "This `else` branch is always evaluated, making it the result of the expression",
                        ))
                        .with_note(
                            "The ternary operator `? :` evaluates the `then` branch only when the condition is truthy.",
                        )
                        .with_help("Consider replacing the entire expression with just this `else` branch.")
                } else {
                    // `$A ?: $C` where `$A` is always falsy
                    Issue::warning("Redundant Elvis operator: left-hand side is always falsy.")
                    .with_annotation(Annotation::primary(condition.span()).with_message(format!(
                        "This expression (type `{}`) is always falsy",
                        condition_type.get_id()
                    )))
                    .with_annotation(Annotation::secondary(r#else.span()).with_message(
                        "This right-hand side is always evaluated, making it the result of the expression",
                    ))
                    .with_note(
                        "The Elvis operator `?:` evaluates the right-hand side only if the left-hand side is falsy.",
                    )
                    .with_help("Consider replacing the entire expression with just the right-hand side.")
                };

            context.collector.report_with_code(IssueCode::ImpossibleCondition, issue);
        }
    }

    artifacts.set_rc_expression_type(
        &span,
        if is_condition_truthy {
            if let Some(then_type) = then_type { then_type } else { Rc::new(get_mixed()) }
        } else if is_condition_falsy {
            if let Some(else_type) = artifacts.get_rc_expression_type(r#else).cloned() {
                else_type
            } else {
                Rc::new(get_mixed())
            }
        } else if let (Some(then_type), Some(else_type)) = (then_type, artifacts.get_rc_expression_type(r#else)) {
            Rc::new(combine_union_types(then_type.as_ref(), else_type.as_ref(), context.codebase, false))
        } else {
            Rc::new(get_mixed())
        },
    );

    Ok(())
}
