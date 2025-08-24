use std::collections::BTreeMap;
use std::rc::Rc;

use ahash::HashSet;
use indexmap::IndexMap;

use mago_algebra::clause::Clause;
use mago_algebra::find_satisfying_assignments;
use mago_algebra::negate_formula;
use mago_algebra::saturate_clauses;
use mago_atom::atom;
use mago_codex::get_enum;
use mago_codex::ttype;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::bool::TBool;
use mago_codex::ttype::combine_union_types;
use mago_codex::ttype::combiner::combine;
use mago_codex::ttype::union::TUnion;
use mago_codex::ttype::*;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::analyze_statements;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::context::block::BreakContext;
use crate::context::scope::control_action::ControlAction;
use crate::context::scope::loop_scope::LoopScope;
use crate::context::utils::inherit_branch_context_properties;
use crate::error::AnalysisError;
use crate::formula::get_formula;
use crate::reconciler::reconcile_keyed_types;
use crate::statement::r#loop::assignment_map_visitor::get_assignment_map;
use crate::statement::r#loop::cleaner::clean_nodes;

mod assignment_map_visitor;
mod cleaner;

pub mod r#break;
pub mod r#continue;
pub mod r#do;
pub mod r#for;
pub mod foreach;
pub mod r#while;

fn analyze_for_or_while_loop<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    initializations: &'ast [Expression<'arena>],
    conditions: &'ast [Expression<'arena>],
    increments: &'ast [Expression<'arena>],
    statements: &'ast [Statement<'arena>],
    span: Span,
    infinite_loop: bool,
) -> Result<(), AnalysisError> {
    let pre_assigned_var_ids = block_context.assigned_variable_ids.clone();
    block_context.assigned_variable_ids.clear();
    for initialization_expression in initializations {
        initialization_expression.analyze(context, block_context, artifacts)?;
    }

    block_context.assigned_variable_ids.extend(pre_assigned_var_ids);

    let mut loop_block_context = block_context.clone();
    loop_block_context.inside_loop = true;
    loop_block_context.break_types.push(BreakContext::Loop);
    let previous_loop_bounds = loop_block_context.loop_bounds;
    loop_block_context.loop_bounds = span.to_offset_tuple();

    let mut loop_scope = LoopScope::new(span, block_context.locals.clone(), None);
    loop_scope.variables_possibly_in_scope =
        if infinite_loop { block_context.variables_possibly_in_scope.clone() } else { Default::default() };

    let (inner_loop_block_context, loop_scope) = analyze(
        context,
        statements,
        conditions.iter().collect(),
        increments.iter().collect(),
        loop_scope,
        &mut loop_block_context,
        block_context,
        artifacts,
        false,
        infinite_loop,
    )?;

    loop_block_context.loop_bounds = previous_loop_bounds;

    let always_enters_loop = infinite_loop || loop_scope.truthy_pre_conditions;

    inherit_loop_block_context(
        context,
        block_context,
        loop_block_context,
        inner_loop_block_context,
        loop_scope,
        always_enters_loop,
        infinite_loop,
    );

    Ok(())
}

fn inherit_loop_block_context<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    loop_block_context: BlockContext<'ctx>,
    inner_loop_block_context: BlockContext<'ctx>,
    loop_scope: LoopScope,
    always_enters_loop: bool,
    known_infinite_loop: bool,
) {
    let has_break = loop_scope.final_actions.contains(&ControlAction::Break);
    let has_continue = loop_scope.final_actions.contains(&ControlAction::Continue);
    let has_break_or_continue = has_break || has_continue;
    let can_leave_loop = !known_infinite_loop || has_break;

    inherit_branch_context_properties(context, block_context, &inner_loop_block_context);

    if !can_leave_loop {
        block_context.control_actions.insert(ControlAction::End);
        block_context.has_returned = true;
    } else {
        for (variable, variable_type) in inner_loop_block_context.locals {
            if !always_enters_loop {
                block_context.variables_possibly_in_scope.insert(variable);
                continue;
            }

            if !has_break_or_continue {
                block_context.locals.insert(variable, variable_type);
                continue;
            }

            if let Some(possible_type) = loop_scope.possibly_defined_loop_parent_variables.get(&variable) {
                block_context.locals.insert(
                    variable,
                    Rc::new(ttype::combine_union_types(&variable_type, possible_type, context.codebase, false)),
                );
            }
        }
    }

    if can_leave_loop {
        block_context.variables_possibly_in_scope.extend(loop_block_context.variables_possibly_in_scope);
        block_context.possibly_assigned_variable_ids.extend(loop_block_context.possibly_assigned_variable_ids);
    } else {
        block_context.variables_possibly_in_scope = loop_scope.variables_possibly_in_scope;
    }
}

fn analyze<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    statements: &'ast [Statement<'arena>],
    pre_conditions: Vec<&'ast Expression<'arena>>,
    post_expressions: Vec<&'ast Expression<'arena>>,
    mut loop_scope: LoopScope,
    loop_context: &mut BlockContext<'ctx>,
    loop_parent_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    is_do: bool,
    always_enters_loop: bool,
) -> Result<(BlockContext<'ctx>, LoopScope), AnalysisError> {
    let (assignment_map, first_variable_id) = get_assignment_map(&pre_conditions, &post_expressions, statements);
    let assignment_depth = if let Some(first_variable_id) = first_variable_id {
        get_assignment_map_depth(&first_variable_id, &mut assignment_map.clone())
    } else {
        0
    };

    let mut always_assigned_before_loop_body_variables = HashSet::default();

    let mut pre_condition_clauses = Vec::new();

    let codebase = context.codebase;

    if !pre_conditions.is_empty() {
        let assertion_context = context.get_assertion_context_from_block(loop_context);

        let mut complex_conditions = vec![];
        for pre_condition in &pre_conditions {
            let condition_span = pre_condition.span();
            let clauses = get_formula(condition_span, condition_span, pre_condition, assertion_context, artifacts)
                .unwrap_or_else(|| {
                    complex_conditions.push(condition_span);

                    vec![]
                });

            pre_condition_clauses.push(clauses)
        }

        let statements_span = match (statements.first(), statements.last()) {
            (Some(first), Some(last)) => Some(first.span().join(last.span())),
            _ => None,
        };

        if let Some(statements_span) = statements_span {
            for complex_condition in complex_conditions {
                context.collector.report_with_code(
                    IssueCode::ConditionIsTooComplex,
                    Issue::warning("Loop condition is too complex for precise type analysis.")
                        .with_annotation(
                            Annotation::primary(complex_condition)
                                .with_message("This loop condition is too complex for the analyzer to fully understand"),
                        )
                        .with_annotation(
                            Annotation::secondary(statements_span)
                                .with_message("Type inference within the loop statement(s) may be inaccurate as a result"),
                        )
                        .with_note(
                            "The analyzer limits the number of logical paths it explores for a single condition to prevent performance issues."
                        )
                        .with_note(
                            "Because this limit was exceeded, type assertions from the condition may not be applied correctly, which can affect variable types on subsequent loop iterations."
                        )
                        .with_help(
                            "Consider refactoring this complex condition into a simpler expression or breaking it down into intermediate boolean variables before the loop.",
                        ),
                );
            }
        }
    } else {
        always_assigned_before_loop_body_variables =
            BlockContext::get_new_or_updated_locals(loop_parent_context, loop_context);
    }

    let final_actions = ControlAction::from_statements(statements.iter().collect(), vec![], Some(artifacts), true);
    let does_always_break = final_actions.len() == 1 && final_actions.contains(&ControlAction::Break);

    let mut continue_context;
    let mut inner_do_context = None;

    let mut pre_conditions_applied = false;

    if assignment_depth == 0 || does_always_break {
        continue_context = loop_context.clone();

        for (condition_offset, pre_condition) in pre_conditions.iter().enumerate() {
            let Some(clauses) = pre_condition_clauses.get(condition_offset) else {
                continue;
            };

            apply_pre_condition_to_loop_context(
                context,
                pre_condition,
                clauses,
                &mut continue_context,
                loop_parent_context,
                artifacts,
                is_do,
                !pre_conditions_applied,
            )?;
        }

        pre_conditions_applied = true;

        artifacts.set_loop_scope(loop_scope.clone());
        analyze_statements(statements, context, &mut continue_context, artifacts)?;
        loop_scope = unsafe {
            // SAFETY: we know the loop scope will remain in the context.
            artifacts.take_loop_scope_unchecked()
        };

        update_loop_scope_contexts(&mut loop_scope, loop_context, &mut continue_context, loop_parent_context, context);

        loop_context.inside_loop_expressions = true;
        for post_expression in post_expressions {
            post_expression.analyze(context, loop_context, artifacts)?;
        }
        loop_context.inside_loop_expressions = true;
    } else {
        let original_parent_context = loop_parent_context.clone();

        let mut pre_loop_context = loop_context.clone();

        let (result, mut recorded_issues) = context.record(|context| {
            if !is_do {
                for (condition_offset, pre_condition) in pre_conditions.iter().enumerate() {
                    apply_pre_condition_to_loop_context(
                        context,
                        pre_condition,
                        unsafe {
                            // SAFETY: we know the pre_condition_clauses will contain
                            // the clauses for the pre_condition at condition_offset.
                            pre_condition_clauses.get_unchecked(condition_offset)
                        },
                        loop_context,
                        loop_parent_context,
                        artifacts,
                        is_do,
                        !pre_conditions_applied,
                    )?;
                }

                pre_conditions_applied = true;
            }

            let mut continue_context = loop_context.clone();

            loop_scope = {
                artifacts.set_loop_scope(loop_scope);
                analyze_statements(statements, context, &mut continue_context, artifacts)?;

                unsafe {
                    // SAFETY: we know the loop scope will remain in the context.
                    artifacts.take_loop_scope_unchecked()
                }
            };

            update_loop_scope_contexts(
                &mut loop_scope,
                loop_context,
                &mut continue_context,
                &original_parent_context,
                context,
            );

            if is_do {
                inner_do_context = Some(continue_context.clone());

                for (condition_offset, pre_condition) in pre_conditions.iter().enumerate() {
                    always_assigned_before_loop_body_variables.extend(apply_pre_condition_to_loop_context(
                        context,
                        pre_condition,
                        unsafe {
                            // SAFETY: we know the pre_condition_clauses will contain
                            // the clauses for the pre_condition at condition_offset.
                            pre_condition_clauses.get_unchecked(condition_offset)
                        },
                        &mut continue_context,
                        loop_parent_context,
                        artifacts,
                        is_do,
                        !pre_conditions_applied,
                    )?);
                }

                pre_conditions_applied = true;
            }

            continue_context.inside_loop_expressions = true;
            for post_expression in &post_expressions {
                post_expression.analyze(context, &mut continue_context, artifacts)?;
            }

            continue_context.inside_loop_expressions = false;

            Ok((loop_scope, continue_context))
        });

        (loop_scope, continue_context) = result?;

        let mut i = 0;
        while i < assignment_depth {
            let mut variables_to_remove = Vec::new();

            loop_scope.iteration_count += 1;

            let mut has_changes = false;

            // reset the $continue_context to what it was before we started the analysis,
            // but union the types with what's in the loop scope

            if pre_loop_context.locals.iter().any(|(variable_id, _)| !continue_context.locals.contains_key(variable_id))
            {
                has_changes = true;
            }

            let mut different_from_pre_loop_types = HashSet::default();

            for (variable_id, continue_context_type) in continue_context.locals.clone() {
                if always_assigned_before_loop_body_variables.contains(&variable_id) {
                    // set the variables to whatever the while/foreach loop expects them to be
                    if let Some(pre_loop_context_type) = pre_loop_context.locals.get(&variable_id) {
                        if continue_context_type != *pre_loop_context_type {
                            different_from_pre_loop_types.insert(variable_id.clone());
                            has_changes = true;
                        }
                    } else {
                        has_changes = true;
                    }
                } else if let Some(parent_context_type) = original_parent_context.locals.get(&variable_id) {
                    if continue_context_type != *parent_context_type {
                        has_changes = true;

                        // widen the foreach context type with the initial context type
                        continue_context.locals.insert(
                            variable_id.clone(),
                            Rc::new(combine_union_types(
                                &continue_context_type,
                                parent_context_type,
                                context.codebase,
                                false,
                            )),
                        );

                        pre_loop_context.remove_variable_from_conflicting_clauses(context, &variable_id, None);

                        loop_parent_context.possibly_assigned_variable_ids.insert(variable_id.clone());
                    }

                    if let Some(loop_context_type) = loop_context.locals.get(&variable_id) {
                        if continue_context_type != *loop_context_type {
                            has_changes = true;
                        }

                        // widen the foreach context type with the initial context type
                        continue_context.locals.insert(
                            variable_id.clone(),
                            Rc::new(combine_union_types(&continue_context_type, loop_context_type, codebase, false)),
                        );

                        // if there's a change, invalidate related clauses
                        pre_loop_context.remove_variable_from_conflicting_clauses(context, &variable_id, None);
                    }
                } else {
                    if !recorded_issues.is_empty() {
                        has_changes = true;
                    }

                    if !is_do {
                        variables_to_remove.push(variable_id.clone());
                    }
                }
            }

            continue_context.has_returned = false;

            // if there are no changes to the types, no need to re-examine
            if !has_changes {
                continue_context.inside_loop_expressions = true;
                for post_expression in &post_expressions {
                    post_expression.analyze(context, &mut continue_context, artifacts)?;
                }
                continue_context.inside_loop_expressions = false;

                break;
            }

            for variable_id in variables_to_remove {
                continue_context.locals.remove(&variable_id);
            }

            continue_context.clauses.clone_from(&pre_loop_context.clauses);
            continue_context.by_reference_constraints.clone_from(&pre_loop_context.by_reference_constraints);

            let (result, new_recorded_issues) = context.record(|context| -> Result<LoopScope, AnalysisError> {
                if !is_do {
                    for (condition_offset, pre_condition) in pre_conditions.iter().enumerate() {
                        apply_pre_condition_to_loop_context(
                            context,
                            pre_condition,
                            unsafe {
                                // SAFETY: we know the pre_condition_clauses will contain
                                // the clauses for the pre_condition at condition_offset.
                                pre_condition_clauses.get_unchecked(condition_offset)
                            },
                            &mut continue_context,
                            loop_parent_context,
                            artifacts,
                            is_do,
                            !pre_conditions_applied,
                        )?;
                    }

                    pre_conditions_applied = true;
                }

                for variable_id in &always_assigned_before_loop_body_variables {
                    let pre_loop_context_type = pre_loop_context.locals.get(variable_id);

                    if if different_from_pre_loop_types.contains(variable_id) {
                        true
                    } else if continue_context.locals.contains_key(variable_id) {
                        pre_loop_context_type.is_none()
                    } else {
                        true
                    } {
                        if let Some(pre_loop_context_type) = pre_loop_context_type {
                            continue_context.locals.insert(variable_id.clone(), pre_loop_context_type.clone());
                        } else {
                            continue_context.locals.remove(variable_id);
                        }
                    }
                }

                continue_context.clauses.clone_from(&pre_loop_context.clauses);

                clean_nodes(statements, artifacts);

                let mut loop_scope = {
                    artifacts.set_loop_scope(loop_scope);
                    analyze_statements(statements, context, &mut continue_context, artifacts)?;

                    unsafe {
                        // SAFETY: we know the loop scope will remain in the context.
                        artifacts.take_loop_scope_unchecked()
                    }
                };

                update_loop_scope_contexts(
                    &mut loop_scope,
                    loop_context,
                    &mut continue_context,
                    &original_parent_context,
                    context,
                );

                if is_do {
                    inner_do_context = Some(continue_context.clone());

                    for (condition_offset, pre_condition) in pre_conditions.iter().enumerate() {
                        apply_pre_condition_to_loop_context(
                            context,
                            pre_condition,
                            unsafe {
                                // SAFETY: we know the pre_condition_clauses will contain
                                // the clauses for the pre_condition at condition_offset.
                                pre_condition_clauses.get_unchecked(condition_offset)
                            },
                            &mut continue_context,
                            loop_parent_context,
                            artifacts,
                            is_do,
                            !pre_conditions_applied,
                        )?;
                    }

                    pre_conditions_applied = true;
                }

                continue_context.inside_loop_expressions = true;
                for post_expression in &post_expressions {
                    post_expression.analyze(context, &mut continue_context, artifacts)?;
                }
                continue_context.inside_loop_expressions = false;

                Ok(loop_scope)
            });

            loop_scope = result?;
            recorded_issues = new_recorded_issues;

            i += 1;
        }

        if !recorded_issues.is_empty() {
            for issue in recorded_issues {
                context.collector.report(issue);
            }
        }
    }

    debug_assert!(pre_conditions_applied, "Pre-conditions should have been applied at least once.");

    let does_sometimes_break = loop_scope.final_actions.contains(&ControlAction::Break);
    let does_always_break = does_sometimes_break && loop_scope.final_actions.len() == 1;

    if does_sometimes_break {
        if let Some(mut inner_do_context_inner) = inner_do_context {
            for (variable_id, possibly_redefined_variable_type) in &loop_scope.possibly_redefined_loop_parent_variables
            {
                if let Some(do_context_type) = inner_do_context_inner.locals.get_mut(variable_id) {
                    *do_context_type = if do_context_type == possibly_redefined_variable_type {
                        possibly_redefined_variable_type.clone()
                    } else {
                        Rc::new(combine_union_types(
                            possibly_redefined_variable_type,
                            do_context_type,
                            codebase,
                            always_enters_loop,
                        ))
                    };
                }

                loop_parent_context.possibly_assigned_variable_ids.insert(variable_id.clone());
            }

            inner_do_context = Some(inner_do_context_inner);
        } else {
            for (variable_id, variable_type) in &loop_scope.possibly_redefined_loop_parent_variables {
                if let Some(loop_parent_context_type) = loop_parent_context.locals.get_mut(variable_id) {
                    *loop_parent_context_type = Rc::new(combine_union_types(
                        variable_type,
                        loop_parent_context_type,
                        codebase,
                        always_enters_loop,
                    ));
                }

                loop_parent_context.possibly_assigned_variable_ids.insert(variable_id.clone());
            }
        }
    }

    for (variable_id, variable_type) in &loop_parent_context.locals.clone() {
        if let Some(loop_context_type) = loop_context.locals.get(variable_id) {
            if loop_context_type != variable_type {
                loop_parent_context.locals.insert(
                    variable_id.clone(),
                    Rc::new(combine_union_types(variable_type, loop_context_type, codebase, always_enters_loop)),
                );

                loop_parent_context.remove_variable_from_conflicting_clauses(context, variable_id, None);
            } else if let Some(loop_parent_context_type) = loop_parent_context.locals.get_mut(variable_id)
                && loop_parent_context_type != loop_context_type
            {
                *loop_parent_context_type = Rc::new((**loop_context_type).clone());
            }
        }
    }

    if !does_always_break {
        for (variable_id, variable_type) in loop_parent_context.locals.clone() {
            if let Some(continue_context_type) = continue_context.locals.get_mut(&variable_id) {
                if continue_context_type.is_mixed() {
                    *continue_context_type = Rc::new((**continue_context_type).clone());

                    loop_parent_context.locals.insert(variable_id.clone(), continue_context_type.clone());
                    loop_parent_context.remove_variable_from_conflicting_clauses(context, &variable_id, None);
                } else if continue_context_type != &variable_type {
                    loop_parent_context.locals.insert(
                        variable_id.clone(),
                        Rc::new(combine_union_types(
                            &variable_type,
                            continue_context_type,
                            codebase,
                            always_enters_loop,
                        )),
                    );
                    loop_parent_context.remove_variable_from_conflicting_clauses(context, &variable_id, None);
                } else if let Some(loop_parent_context_type) = loop_parent_context.locals.get_mut(&variable_id) {
                    *loop_parent_context_type = Rc::new((**continue_context_type).clone());
                }
            } else {
                loop_parent_context.locals.remove(&variable_id);
            }
        }
    }

    if !pre_conditions.is_empty() && !pre_condition_clauses.is_empty() && !does_sometimes_break {
        // if the loop contains an assertion and there are no break statements, we can negate that assertion
        // and apply it to the current context

        let negated_pre_condition_clauses =
            negate_formula(pre_condition_clauses.into_iter().flatten().collect()).unwrap_or_default();

        let (negated_pre_condition_types, _) =
            find_satisfying_assignments(negated_pre_condition_clauses.iter().as_slice(), None, &mut HashSet::default());

        if !negated_pre_condition_types.is_empty() {
            let mut changed_variable_ids = HashSet::default();

            reconcile_keyed_types(
                context,
                &negated_pre_condition_types,
                IndexMap::new(),
                &mut continue_context,
                &mut changed_variable_ids,
                &HashSet::default(),
                &unsafe {
                    // SAFETY: we know that pre_conditions is not empty, so we can safely
                    // get the span of the first pre_condition.
                    pre_conditions.get_unchecked(0).span()
                },
                true,
                false,
            );

            for variable_id in changed_variable_ids {
                if let Some(reconciled_type) = continue_context.locals.get(&variable_id) {
                    if loop_parent_context.locals.contains_key(&variable_id) {
                        loop_parent_context.locals.insert(variable_id.clone(), reconciled_type.clone());
                    }

                    loop_parent_context.remove_variable_from_conflicting_clauses(context, &variable_id, None);
                }
            }
        }
    }

    if always_enters_loop {
        let does_sometimes_continue = loop_scope.final_actions.contains(&ControlAction::Continue);

        for (variable_id, variable_type) in &continue_context.locals {
            // if there are break statements in the loop it's not certain
            // that the loop has finished executing, so the assertions at the end
            // the loop in the while conditional may not hold
            if does_sometimes_break || does_sometimes_continue {
                if let Some(possibly_defined_type) = loop_scope.possibly_defined_loop_parent_variables.get(variable_id)
                {
                    loop_parent_context.locals.insert(
                        variable_id.clone(),
                        Rc::new(combine_union_types(variable_type, possibly_defined_type, codebase, true)),
                    );
                }
            } else {
                loop_parent_context.locals.insert(variable_id.clone(), variable_type.clone());
            }
        }
    }

    if let Some(inner_do_context) = inner_do_context {
        continue_context = inner_do_context;
    }

    // Track references set in the loop to make sure they aren't reused later
    loop_parent_context.update_references_possibly_from_confusing_scope(&continue_context);

    Ok((continue_context, loop_scope))
}

fn get_assignment_map_depth(
    first_variable_id: &String,
    assignment_map: &mut BTreeMap<String, HashSet<String>>,
) -> usize {
    let Some(assignment_variable_ids) = assignment_map.remove(first_variable_id) else {
        return 0;
    };

    let mut max_depth = 0;
    for assignment_variable_id in assignment_variable_ids {
        let mut depth = 1;

        if assignment_map.contains_key(&assignment_variable_id) {
            depth += get_assignment_map_depth(&assignment_variable_id, assignment_map);
        }

        if depth > max_depth {
            max_depth = depth
        }
    }

    max_depth
}

fn apply_pre_condition_to_loop_context<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    pre_condition: &Expression<'arena>,
    pre_condition_clauses: &[Clause],
    loop_context: &mut BlockContext<'ctx>,
    loop_parent_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    is_do: bool,
    first_application: bool,
) -> Result<HashSet<String>, AnalysisError> {
    let pre_condition_span = pre_condition.span();
    let pre_referenced_variable_ids = std::mem::take(&mut loop_context.conditionally_referenced_variable_ids);

    loop_context.inside_conditional = true;
    loop_context.inside_loop_expressions = true;

    pre_condition.analyze(context, loop_context, artifacts)?;

    loop_context.inside_loop_expressions = false;
    loop_context.inside_conditional = false;

    if first_application {
        let is_truthy = if let Some(condition_type) = artifacts.get_expression_type(pre_condition) {
            condition_type.is_always_truthy()
        } else {
            false
        };

        if let Some(loop_scope) = artifacts.get_loop_scope_mut() {
            loop_scope.truthy_pre_conditions = loop_scope.truthy_pre_conditions && is_truthy;
        }
    }

    let mut new_referenced_variable_ids = loop_context.conditionally_referenced_variable_ids.clone();
    loop_context.conditionally_referenced_variable_ids.extend(pre_referenced_variable_ids);

    let always_assigned_before_loop_body_variables =
        BlockContext::get_new_or_updated_locals(loop_context, loop_parent_context);

    loop_context.clauses = saturate_clauses({
        let mut clauses = loop_parent_context.clauses.iter().map(|v| &**v).collect::<Vec<_>>();
        clauses.extend(pre_condition_clauses.iter());
        clauses
    })
    .into_iter()
    .map(|v| Rc::new(v.clone()))
    .collect();

    let (reconcilable_while_types, active_while_types) = find_satisfying_assignments(
        loop_context.clauses.iter().map(|v| (**v).clone()).collect::<Vec<_>>().as_slice(),
        Some(pre_condition_span),
        &mut new_referenced_variable_ids,
    );

    if !reconcilable_while_types.is_empty() {
        reconcile_keyed_types(
            context,
            &reconcilable_while_types,
            active_while_types,
            loop_context,
            &mut HashSet::default(),
            &new_referenced_variable_ids,
            &pre_condition_span,
            first_application,
            false,
        );
    }

    if is_do {
        return Ok(HashSet::default());
    }

    if !loop_context.clauses.is_empty() {
        let mut loop_context_clauses = loop_context.clauses.clone();

        for variable_id in &always_assigned_before_loop_body_variables {
            loop_context_clauses = BlockContext::filter_clauses(context, variable_id, loop_context_clauses, None);
        }

        loop_context.clauses = loop_context_clauses;
    }

    Ok(always_assigned_before_loop_body_variables)
}

fn update_loop_scope_contexts<'ctx, 'arena>(
    loop_scope: &mut LoopScope,
    loop_context: &mut BlockContext<'ctx>,
    continue_context: &mut BlockContext<'ctx>,
    pre_outer_context: &BlockContext<'ctx>,
    context: &Context<'ctx, 'arena>,
) {
    if !loop_scope.final_actions.contains(&ControlAction::Continue) {
        loop_context.locals = pre_outer_context.locals.clone();
    } else {
        for (variable_id, variable_type) in &loop_scope.redefined_loop_variables {
            continue_context.locals.insert(variable_id.clone(), Rc::new(variable_type.clone()));
        }

        for (variable_id, variable_type) in &loop_scope.possibly_redefined_loop_variables {
            if continue_context.has_variable(variable_id) {
                continue_context.locals.insert(
                    variable_id.clone(),
                    Rc::new(combine_union_types(
                        unsafe {
                            // SAFETY: we know that variable_id exists in continue_context.locals.
                            continue_context.locals.get(variable_id).unwrap_unchecked()
                        },
                        variable_type,
                        context.codebase,
                        false,
                    )),
                );
            }
        }
    }
}

fn get_and_expressions<'ast, 'arena>(cond: &'ast Expression<'arena>) -> Vec<&'ast Expression<'arena>> {
    if let Expression::Binary(binary) = &cond
        && let BinaryOperator::Or(_) | BinaryOperator::LowOr(_) = binary.operator
    {
        let mut anded = get_and_expressions(binary.lhs);
        anded.extend(get_and_expressions(binary.rhs));
        return anded;
    }

    vec![cond]
}

/// Analyzes the `foreach` iterator expression.
///
/// # Returns
///
/// A tuple containing:
///
/// - `bool`: `true` if the iterator is determined to always have at least one entry, `false` otherwise.
/// - `TUnion`: The combined type of the keys produced by the iterator.
/// - `TUnion`: The combined type of the values produced by the iterator.
///
/// Reports issues if the iterator type is problematic (e.g., null, scalar, non-traversable object).
fn analyze_iterator<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    iterator: &'ast Expression<'arena>,
    iterator_variable_id: Option<&String>,
    foreach: &'ast Foreach<'arena>,
) -> Result<(bool, TUnion, TUnion), AnalysisError> {
    let was_inside_general_use = block_context.inside_general_use;
    block_context.inside_general_use = true;
    iterator.analyze(context, block_context, artifacts)?;
    block_context.inside_general_use = was_inside_general_use;

    let iterator_type = if let Some(it_type) = artifacts.get_rc_expression_type(iterator).cloned() {
        it_type
    } else if let Some(var_type) = iterator_variable_id.and_then(|v| block_context.locals.get(v).cloned()) {
        var_type
    } else {
        context.collector.report_with_code(
            IssueCode::UnknownIteratorType,
            Issue::error("Cannot determine the type of the expression provided to `foreach`.")
                .with_annotation(
                    Annotation::primary(iterator.span())
                        .with_message("The type of this expression is unknown here"),
                )
                .with_note(
                    "Foreach loops require an array or an object implementing `Traversable` to iterate over."
                )
                .with_help(
                    "Ensure the expression is well-defined and has a known iterable type. Check for undefined variables or unresolvable function calls."
                )
        );

        return Ok((false, get_mixed(), get_mixed()));
    };

    if iterator_type.is_never() {
        return Ok((false, get_never(), get_never()));
    }

    if iterator_type.is_null() {
        context.collector.report_with_code(
            IssueCode::NullIterator,
            Issue::error("Iterating over `null` in `foreach`.")
                .with_annotation(Annotation::primary(iterator.span()).with_message("This expression is `null`"))
                .with_annotation(Annotation::secondary(foreach.body.span()).with_message("This `foreach` will not be executed"))
                .with_note("In PHP, iterating over `null` with `foreach` behaves like iterating an empty array; the loop body will not execute")
                .with_note("This can hide uninitialized variables or logic errors.")
                .with_help("Ensure the expression is initialized to an array or a Traversable object. If `null` is a possible expected state, consider an explicit check before the loop (e.g., `if ($iterable !== null)`).")
        );

        return Ok((false, get_never(), get_never()));
    }

    if iterator_type.is_false() {
        context.collector.report_with_code(
            IssueCode::FalseIterator,
            Issue::error("Iterating over `false` in `foreach`.")
                .with_annotation(Annotation::primary(iterator.span()).with_message("This expression is `false`"))
                .with_annotation(Annotation::secondary(foreach.span()).with_message("This `foreach` will not be executed"))
                .with_note("In PHP, iterating over `false` with `foreach` behaves like iterating an empty array; the loop body will not execute.")
                .with_note("This often indicates a function call that failed or an unintended boolean value.")
                .with_help("Ensure the expression evaluates to an array or a Traversable object. Check the return value of functions if this `false` is unexpected.")
        );

        return Ok((false, get_arraykey(), get_never()));
    }

    if iterator_type.is_nullable() && !iterator_type.ignore_nullable_issues {
        context.collector.report_with_code(
            IssueCode::PossiblyNullIterator,
            Issue::warning(format!("Expression being iterated (type `{}`) might be `null` at runtime.", iterator_type.get_id()))
                .with_annotation(Annotation::primary(iterator.span()).with_message("This might be `null`"))
                .with_annotation(Annotation::secondary(foreach.span()).with_message("This `foreach` might not be executed"))
                .with_note("If this expression is `null`, it will be treated as an empty array, and the loop body will not execute.")
                .with_help("Consider checking for `null` before the loop if this is not intended."),
        );
    }

    if iterator_type.is_falsable() && !iterator_type.ignore_falsable_issues {
        context.collector.report_with_code(
            IssueCode::PossiblyFalseIterator,
            Issue::warning(format!("Expression being iterated (type `{}`) might be `false` at runtime.", iterator_type.get_id()))
                .with_annotation(Annotation::primary(iterator.span()).with_message("This might be `false`"))
                .with_annotation(Annotation::secondary(foreach.span()).with_message("This `foreach` might not be executed"))
                .with_note("If this expression is `false`, it will be treated as an empty array, and the loop body will not execute.")
                .with_help("Consider checking for `false` or truthiness before the loop if this is not intended."),
        );
    }

    let mut has_at_least_one_entry = false;
    let mut collected_key_atomics: Vec<TAtomic> = vec![];
    let mut collected_value_atomics: Vec<TAtomic> = vec![];
    let mut has_valid_iterable_type = false;
    let mut invalid_atomic_ids = Vec::with_capacity(iterator_type.types.len());

    for iterator_atomic_original in iterator_type.types.as_ref() {
        let iterator_atomic = if let TAtomic::GenericParameter(generic_parameter) = iterator_atomic_original {
            generic_parameter.get_constraint().get_single()
        } else {
            iterator_atomic_original
        };

        match iterator_atomic {
            TAtomic::Null | TAtomic::Scalar(TScalar::Bool(TBool { value: Some(false), .. })) => {
                continue;
            }
            TAtomic::Array(array) => {
                has_valid_iterable_type = true;
                if array.is_non_empty() {
                    has_at_least_one_entry = true;
                }

                let (k, v) = get_array_parameters(array, context.codebase);
                collected_key_atomics.extend(k.types.into_owned());
                collected_value_atomics.extend(v.types.into_owned());
            }
            TAtomic::Iterable(iterable) => {
                has_valid_iterable_type = true;
                has_at_least_one_entry = false;

                collected_key_atomics.extend(iterable.key_type.types.clone().into_owned());
                collected_value_atomics.extend(iterable.value_type.types.clone().into_owned());
            }
            TAtomic::Object(object) => {
                let (obj_key_type, obj_value_type) = match object {
                    TObject::Any => {
                        context.collector.report_with_code(
                            IssueCode::GenericObjectIteration,
                            Issue::warning("Iterating over a generic `object`. This will iterate its public properties.")
                                .with_annotation(Annotation::primary(iterator.span()).with_message("Iterating a generic `object` type"))
                                .with_note("When `foreach` is used on a generic `object` whose specific class is unknown, PHP will attempt to iterate over its public properties. The keys will be property names (strings) and values their types (typically `mixed` from a static analysis perspective).")
                                .with_help("For predictable and type-safe iteration, ensure the object is an instance of a class implementing `Iterator` or `IteratorAggregate`.")
                        );

                        (get_string(), get_mixed())
                    }
                    TObject::Named(atomic_object) => {
                        if let Some((k, v)) = get_iterable_parameters(iterator_atomic, context.codebase) {
                            (k, v)
                        } else {
                            let class_name = atomic_object.name;
                            let iterator_atomic_str = iterator_atomic.get_id();

                            context.collector.report_with_code(
                                IssueCode::NonIterableObjectIteration,
                                Issue::warning(format!(
                                    "Iterating over object of type `{class_name}` which does not implement `Iterator` or `IteratorAggregate`.",
                                ))
                                    .with_annotation(
                                        Annotation::primary(iterator.span()).with_message(format!("Iterating non-traversable object `{class_name}` of type `{iterator_atomic_str}`")),
                                    )
                                    .with_note(format!("PHP will iterate over the public properties of `{class_name}`."))
                                    .with_help("The keys will be property names (strings) and values will be their types (often `mixed` from a static analysis perspective).")
                                    .with_help("This might expose internal state or lead to unexpected behavior if properties change.")
                                    .with_help(format!("For controlled and type-safe iteration, implement the `Iterator` or `IteratorAggregate` interface on class `{class_name}`."))
                            );

                            (get_string(), get_mixed())
                        }
                    }
                    TObject::Enum(enum_instance) => {
                        has_at_least_one_entry = true;

                        let enum_name = enum_instance.get_name();
                        let enum_backing_type = get_enum(context.codebase, enum_instance.get_name_ref())
                            .and_then(|class_like| class_like.enum_type.as_ref());

                        context.collector.report_with_code(
                            IssueCode::EnumIteration,
                            Issue::warning(format!("Iterating directly over the enum enum `{enum_name}`. This will yield its public properties.",))
                                .with_annotation(
                                    Annotation::primary(iterator.span()).with_message("This enum instance is being iterated directly"),
                                )
                                .with_note(format!(
                                    "PHP allows iterating an enum case instance like an object, which exposes its public properties: `name` (string){}.",
                                    if enum_backing_type.is_some() { " and `value` (its scalar backing value)" } else { "" },
                                ))
                                .with_note(format!("This is different from iterating through all defined cases of the `{enum_name}` enum using `{enum_name}::cases()`, where each item would be an enum case instance itself."))
                                .with_note(format!(
                                    "If you only need the properties of this specific instance, consider accessing them directly (e.g., `$instance->name`{}) for better clarity, unless iterating its few properties is explicitly intended.",
                                    if enum_backing_type.is_some() { ", `$instance->value`" } else { "" }
                                ))
                                .with_help(format!("If your goal is to loop through all defined cases of the `{enum_name}` enum, use `{enum_name}::cases()` instead (e.g., `foreach ({enum_name}::cases() as $case)`).")),
                        );

                        match enum_backing_type {
                            Some(backing_type) => (
                                TUnion::from_vec(vec![
                                    TAtomic::Scalar(TScalar::literal_string(atom("name"))),
                                    TAtomic::Scalar(TScalar::literal_string(atom("value"))),
                                ]),
                                TUnion::from_vec(vec![
                                    TAtomic::Scalar(TScalar::non_empty_string()),
                                    backing_type.clone(),
                                ]),
                            ),
                            None => (get_literal_string(atom("name")), get_non_empty_string()),
                        }
                    }
                };

                has_valid_iterable_type = true;

                collected_key_atomics.extend(obj_key_type.types.into_owned());
                collected_value_atomics.extend(obj_value_type.types.into_owned());
            }
            _ => {
                let iterator_atomic_id = iterator_atomic.get_id();
                invalid_atomic_ids.push(iterator_atomic_id.as_str());
            }
        }
    }

    if !has_valid_iterable_type {
        let iterator_type_id_str = iterator_type.get_id();
        let problematic_types_str = if invalid_atomic_ids.is_empty() {
            format!("resolved to type `{iterator_type_id_str}` which is not iterable in this context")
        } else if invalid_atomic_ids.len() == 1 {
            format!("resolved to type `{}`, which is not iterable", invalid_atomic_ids[0])
        } else {
            format!(
                "could be one of the following non-iterable types: `{}` (overall type: `{}`)",
                invalid_atomic_ids.join("`, `"),
                iterator_type_id_str
            )
        };

        context.collector.report_with_code(
            IssueCode::InvalidIterator,
            Issue::error(format!(
                "The expression provided to `foreach` is not iterable. It {problematic_types_str}."
            ))
            .with_annotation(
                Annotation::primary(iterator.span())
                    .with_message("This expression cannot be iterated"),
            )
            .with_note(
                "A `foreach` loop requires an array or an object implementing the `Traversable` interface."
            )
            .with_note(
                "Attempting to iterate other types will result in a runtime error or the loop not executing."
            )
            .with_help(
                "Ensure the expression always evaluates to an array or a traversable object. Check variable types and function return values.",
            ),
        );

        return Ok((false, get_never(), get_never()));
    } else if !invalid_atomic_ids.is_empty() {
        let iterator_type_id_str = iterator_type.get_id();
        let problematic_types_list_str = invalid_atomic_ids.join("`, `");

        context.collector.report_with_code(
            IssueCode::PossiblyInvalidIterator,
            Issue::warning(format!(
                "The expression provided to `foreach` (type `{iterator_type_id_str}`) might not be iterable at runtime."
            ))
            .with_annotation(
                Annotation::primary(iterator.span())
                    .with_message("This expression has potentially non-iterable types"),
            )
            .with_note(format!(
                "It could evaluate to one of the following non-iterable types: `{problematic_types_list_str}`. If so, a runtime error will occur or the loop will not execute for that specific type."
            ))
            .with_help(
                "Ensure all possible types for this expression are iterable, or add checks to handle non-iterable cases before the loop. For analysis, key/value types will include `mixed` due to this uncertainty.",
            ),
        );

        return Ok((false, get_mixed(), get_mixed()));
    }

    let final_key_type = if collected_key_atomics.is_empty() {
        get_mixed()
    } else {
        TUnion::from_vec(combine(collected_key_atomics, context.codebase, false))
    };

    let final_value_type = if collected_value_atomics.is_empty() {
        get_mixed()
    } else {
        TUnion::from_vec(combine(collected_value_atomics, context.codebase, false))
    };

    Ok((has_at_least_one_entry, final_key_type, final_value_type))
}

/// Scrapes all direct variable names from an expression and indicates if a reference operator (`&`)
/// was encountered anywhere within the expression.
///
/// # Arguments
///
/// * `expression` - The AST expression node to scrape.
///
/// # Returns
///
/// A `HashSet<String>` contains the string names of all `Variable::Direct` found.
fn scrape_variables_from_expression<'arena>(expression: &Expression<'arena>) -> HashSet<&'arena str> {
    let mut set = HashSet::default();

    fn walk<'arena>(current_expression: &Expression<'arena>, current_set: &mut HashSet<&'arena str>) {
        match current_expression {
            Expression::UnaryPrefix(UnaryPrefix { operand, .. }) => {
                walk(operand, current_set);
            }
            Expression::Access(access) => match access {
                Access::Property(property_access) => {
                    walk(property_access.object, current_set);
                }
                Access::NullSafeProperty(null_safe_property_access) => {
                    walk(null_safe_property_access.object, current_set);
                }
                Access::StaticProperty(static_property_access) => {
                    walk(static_property_access.class, current_set);
                }
                _ => {}
            },
            Expression::Variable(Variable::Direct(var)) => {
                current_set.insert(var.name);
            }
            Expression::List(List { elements, .. })
            | Expression::Array(Array { elements, .. })
            | Expression::LegacyArray(LegacyArray { elements, .. }) => {
                for element in elements.iter() {
                    if let Some(key_expression) = element.get_key() {
                        walk(key_expression, current_set);
                    }

                    if let Some(value_expression) = element.get_value() {
                        walk(value_expression, current_set);
                    }
                }
            }
            _ => {}
        }
    }

    walk(expression, &mut set);

    set
}
