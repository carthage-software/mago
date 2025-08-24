use std::ops::Deref;
use std::rc::Rc;

use ahash::HashMap;
use ahash::HashSet;
use indexmap::IndexMap;
use itertools::Itertools;

use mago_algebra::clause::Clause;
use mago_algebra::disjoin_clauses;
use mago_algebra::find_satisfying_assignments;
use mago_algebra::negate_formula;
use mago_algebra::saturate_clauses;
use mago_codex::ttype::combine_union_types;
use mago_codex::ttype::union::TUnion;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::common::synthetic::new_synthetic_call;
use crate::common::synthetic::new_synthetic_negation;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::context::scope::conditional_scope::IfConditionalScope;
use crate::context::scope::control_action::ControlAction;
use crate::context::scope::if_scope::IfScope;
use crate::context::utils::inherit_branch_context_properties;
use crate::error::AnalysisError;
use crate::formula;
use crate::formula::negate_or_synthesize;
use crate::reconciler::reconcile_keyed_types;
use crate::statement::analyze_statements;
use crate::utils::conditional;
use crate::utils::expression::is_derived_access_path;
use crate::utils::misc::check_for_paradox;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for If<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let mut if_scope = IfScope::new();

        // We need to clone the original context for later use if we're exiting in this if conditional
        if is_obvious_boolean_condition(self.condition) {
            let final_actions =
                ControlAction::from_statements(self.body.statements().iter().collect(), vec![], Some(artifacts), true);

            let has_leaving_statements = final_actions.len() == 1 && final_actions.contains(&ControlAction::End)
                || (!final_actions.is_empty() && !final_actions.contains(&ControlAction::None));

            if has_leaving_statements {
                if_scope.post_leaving_if_context = Some(block_context.clone());
            }
        }

        let (mut if_conditional_scope, applied_block_context) =
            conditional::analyze(context, block_context.clone(), artifacts, &mut if_scope, self.condition, true)?;
        *block_context = applied_block_context;

        let mut if_block_context = if_conditional_scope.if_body_context.clone();

        let post_if_block_context = if_conditional_scope.post_if_context.clone();

        let mut mixed_variables = vec![];
        for (variable_id, variable_type) in if_block_context.locals.iter() {
            if variable_type.is_mixed() && block_context.locals.contains_key(variable_id) {
                mixed_variables.push(variable_id.clone());
            }
        }

        let mut if_clauses = formula::get_formula(
            self.condition.span(),
            self.condition.span(),
            self.condition,
            context.get_assertion_context_from_block(block_context),
            artifacts,
        ).unwrap_or_else(|| {
            context.collector.report_with_code(
                IssueCode::ConditionIsTooComplex,
                Issue::warning("Condition is too complex for precise type analysis.")
                    .with_annotation(
                        Annotation::primary(self.condition.span())
                            .with_message("This `if` condition is too complex for the analyzer to fully understand"),
                    )
                    .with_annotation(
                        Annotation::secondary(self.body.span())
                            .with_message("Type inference within this branch may be inaccurate as a result"),
                    )
                    .with_note(
                        "The analyzer limits the number of logical paths it explores for a single condition to prevent performance issues."
                    )
                    .with_note(
                        "Because this limit was exceeded, type assertions from the condition will not be applied inside the `if` block, which may lead to incorrect type information."
                    )
                    .with_help(
                        "Consider refactoring this complex condition into smaller, intermediate boolean variables or nested `if` statements.",
                    ),
            );

            vec![]
        });

        for clause in &mut if_clauses {
            let keys = clause.possibilities.keys().cloned().collect::<Vec<String>>();
            mixed_variables.retain(|i| !keys.contains(i));

            'outer: for key in keys {
                for mixed_var_id in &mixed_variables {
                    if is_derived_access_path(&key, mixed_var_id) {
                        *clause = Clause::new(
                            Default::default(),
                            self.condition.span(),
                            self.condition.span(),
                            Some(true),
                            Some(true),
                            Some(false),
                        );

                        break 'outer;
                    }
                }
            }
        }

        check_for_paradox(&mut context.collector, &block_context.clauses, &if_clauses, self.condition.span());

        if_clauses = saturate_clauses(if_clauses.iter());
        let combined_clauses = if block_context.clauses.is_empty() {
            if_clauses.clone()
        } else {
            saturate_clauses(if_clauses.iter().chain(block_context.clauses.iter().map(Rc::deref)))
        };

        if_block_context.clauses = combined_clauses.into_iter().map(Rc::new).collect();

        if !if_block_context.reconciled_expression_clauses.is_empty() {
            if_block_context.clauses.retain(|clause| !if_block_context.reconciled_expression_clauses.contains(clause));

            if if_block_context.clauses.len() == 1
                && if_block_context.clauses[0].wedge
                && if_block_context.clauses[0].possibilities.is_empty()
            {
                if_block_context.clauses.clear();
                if_block_context.reconciled_expression_clauses.clear();
            }
        }

        if_scope.reasonable_clauses = if_block_context.clauses.to_vec();
        if_scope.negated_clauses = negate_or_synthesize(
            if_clauses,
            self.condition,
            context.get_assertion_context_from_block(block_context),
            artifacts,
        );

        let all_negated_clauses =
            saturate_clauses(block_context.clauses.iter().map(Rc::deref).chain(if_scope.negated_clauses.iter()));

        if_scope.negated_types =
            find_satisfying_assignments(all_negated_clauses.iter().as_slice(), None, &mut HashSet::default()).0;

        let mut temporary_else_context = post_if_block_context.clone();
        let mut changed_variable_ids: HashSet<String> = HashSet::default();

        if !if_scope.negated_types.is_empty() {
            reconcile_keyed_types(
                context,
                &if_scope.negated_types,
                IndexMap::new(),
                &mut temporary_else_context,
                &mut changed_variable_ids,
                &HashSet::default(),
                &self.condition.span(),
                false,
                false,
            );
        }

        let pre_assignment_else_redefined_locals: HashMap<String, TUnion> = temporary_else_context
            .get_redefined_locals(&if_block_context.locals, true, &mut HashSet::default())
            .into_iter()
            .filter(|(k, _)| changed_variable_ids.contains(k))
            .collect();

        analyze_if_statement_block(
            context,
            &mut if_scope,
            &mut if_conditional_scope,
            if_block_context,
            block_context,
            artifacts,
            &pre_assignment_else_redefined_locals,
            self,
        )?;

        let mut else_block_context = if let Some(post_leaving_if_context) = if_scope.post_leaving_if_context.take() {
            post_leaving_if_context
        } else {
            post_if_block_context
        };

        else_block_context.clauses =
            saturate_clauses(else_block_context.clauses.iter().map(Rc::deref).chain(if_scope.negated_clauses.iter()))
                .into_iter()
                .map(Rc::new)
                .collect();

        for clause in self.body.else_if_clauses() {
            analyze_else_if_clause(context, &mut if_scope, &mut else_block_context, block_context, artifacts, clause)?;
        }

        analyze_else_statements(
            context,
            &mut if_scope,
            &mut else_block_context,
            block_context,
            artifacts,
            self.body.else_statements(),
            self.span(),
        )?;

        let has_returned = !if_scope.final_actions.contains(&ControlAction::None);

        if !if_scope.if_actions.is_empty()
            && !if_scope.if_actions.contains(&ControlAction::None)
            && !self.body.has_else_if_clauses()
        {
            block_context.clauses = else_block_context.clauses;
            for (variable_id, variable_type) in else_block_context.locals {
                block_context.locals.insert(variable_id, variable_type);
            }
        }

        if let Some(loop_scope) = artifacts.loop_scope.as_mut() {
            loop_scope.final_actions.extend(if_scope.final_actions);
        }

        block_context.variables_possibly_in_scope.extend(if_scope.new_variables_possibly_in_scope);
        block_context.possibly_assigned_variable_ids.extend(if_scope.possibly_assigned_variable_ids);
        block_context.assigned_variable_ids.extend(if_scope.assigned_variable_ids.unwrap_or_default());

        if let Some(new_variables) = if_scope.new_variables {
            for (variable_id, variable_type) in new_variables {
                block_context.locals.insert(variable_id, Rc::new(variable_type));
            }
        }

        if let Some(redefined_variables) = if_scope.redefined_variables {
            for (variable_id, variable_type) in redefined_variables {
                block_context.locals.insert(variable_id.clone(), Rc::new(variable_type));

                if !if_scope.reasonable_clauses.is_empty() {
                    if_scope.reasonable_clauses = BlockContext::filter_clauses(
                        context,
                        &variable_id,
                        if_scope.reasonable_clauses,
                        block_context.locals.get(&variable_id).map(|rc| rc.as_ref()),
                    );
                }

                if_scope.updated_variables.insert(variable_id);
            }
        }

        if !if_scope.reasonable_clauses.is_empty()
            && (if_scope.reasonable_clauses.len() > 1 || !if_scope.reasonable_clauses[0].wedge)
        {
            block_context.clauses = saturate_clauses(
                if_scope.reasonable_clauses.iter().map(Rc::deref).chain(block_context.clauses.iter().map(Rc::deref)),
            )
            .into_iter()
            .map(Rc::new)
            .collect();
        }

        for (variable_id, variable_type) in if_scope.possibly_redefined_variables {
            let Some(existing_type) = block_context.locals.remove(&variable_id).map(Rc::unwrap_or_clone) else {
                continue;
            };

            if if_scope.updated_variables.contains(&variable_id) {
                block_context.locals.insert(variable_id, Rc::new(existing_type));

                continue;
            }

            let new_type = combine_union_types(&existing_type, &variable_type, context.codebase, false);

            if !new_type.eq(&existing_type) {
                block_context.remove_descendants(context, &variable_id, &existing_type, Some(&new_type));
            }

            block_context.locals.insert(variable_id.clone(), Rc::new(new_type));
        }

        if has_returned {
            block_context.has_returned = true;
        }

        Ok(())
    }
}

#[inline]
const fn is_obvious_boolean_condition(condition: &Expression) -> bool {
    match condition {
        Expression::Binary(_) => true,
        Expression::UnaryPrefix(UnaryPrefix { operand, operator: UnaryPrefixOperator::Not(_) }) => {
            is_obvious_boolean_condition(operand)
        }
        _ => false,
    }
}

fn analyze_if_statement_block<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    if_scope: &mut IfScope<'ctx>,
    if_conditional_scope: &mut IfConditionalScope<'ctx>,
    mut if_block_context: BlockContext<'ctx>,
    outer_block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    pre_assignment_else_redefined_locals: &HashMap<String, TUnion>,
    if_statement: &If<'arena>,
) -> Result<(), AnalysisError> {
    let mut conditionally_referenced_variable_ids = if_block_context.conditionally_referenced_variable_ids.clone();
    let (reconcilable_if_types, active_if_types) = find_satisfying_assignments(
        if_block_context.clauses.iter().map(Rc::as_ref).cloned().collect_vec().as_slice(),
        Some(if_statement.condition.span()),
        &mut conditionally_referenced_variable_ids,
    );

    if outer_block_context.clauses.iter().any(|clause| !clause.possibilities.is_empty()) {
        let mut omit_keys = HashSet::default();
        for clause in outer_block_context.clauses.iter() {
            omit_keys.extend(clause.possibilities.keys());
        }

        let (outer_truthes, _) = find_satisfying_assignments(
            outer_block_context.clauses.iter().map(Rc::as_ref).cloned().collect_vec().as_slice(),
            None,
            &mut HashSet::default(),
        );

        omit_keys.retain(|key| !outer_truthes.contains_key(*key));
        conditionally_referenced_variable_ids.retain(|key| !omit_keys.contains(key));
    }

    if !reconcilable_if_types.is_empty() {
        let mut changed_variable_ids = HashSet::default();

        reconcile_keyed_types(
            context,
            &reconcilable_if_types,
            active_if_types,
            &mut if_block_context,
            &mut changed_variable_ids,
            &conditionally_referenced_variable_ids,
            &if_statement.condition.span(),
            false,
            false,
        );

        for (variable_id, _) in reconcilable_if_types.iter() {
            if_block_context.variables_possibly_in_scope.insert(variable_id.clone());
        }

        if !changed_variable_ids.is_empty() {
            if_block_context.clauses = BlockContext::remove_reconciled_clauses(
                &if_block_context.clauses.iter().map(Rc::deref).cloned().collect(),
                &changed_variable_ids,
            )
            .0
            .into_iter()
            .map(Rc::new)
            .collect();

            let mut variables_to_remove = vec![];
            for changed_variable_id in &changed_variable_ids {
                for (variable_id, _) in if_block_context.locals.iter() {
                    if is_derived_access_path(variable_id, changed_variable_id)
                        && !changed_variable_ids.contains(variable_id)
                        && !conditionally_referenced_variable_ids.contains(variable_id)
                    {
                        variables_to_remove.push(variable_id.clone());
                    }
                }
            }

            for variable_id in variables_to_remove {
                if_block_context.remove_possible_reference(&variable_id);
            }
        }

        if_scope.conditionally_changed_variable_ids = changed_variable_ids;
    }

    if_block_context.reconciled_expression_clauses = vec![];
    outer_block_context.variables_possibly_in_scope.extend(if_block_context.variables_possibly_in_scope.clone());

    let old_if_block_context = if_block_context.clone();
    let assigned_variable_ids = std::mem::take(&mut if_block_context.assigned_variable_ids);
    let possibly_assigned_variable_ids = std::mem::take(&mut if_block_context.possibly_assigned_variable_ids);

    analyze_statements(if_statement.body.statements(), context, &mut if_block_context, artifacts)?;

    for variable_id in &if_block_context.parent_conflicting_clause_variables {
        outer_block_context.remove_variable_from_conflicting_clauses(context, variable_id, None);
    }

    let final_actions = ControlAction::from_statements(
        if_statement.body.statements().iter().collect::<Vec<_>>(),
        vec![],
        Some(artifacts),
        true,
    );

    let has_ending_statements = final_actions.len() == 1 && final_actions.contains(&ControlAction::End);
    let has_break_statement = final_actions.len() == 1 && final_actions.contains(&ControlAction::Break);
    let has_continue_statement = final_actions.len() == 1 && final_actions.contains(&ControlAction::Continue);
    let has_leaving_statements =
        has_ending_statements || (!final_actions.is_empty() && !final_actions.contains(&ControlAction::None));

    if_scope.if_actions = final_actions.iter().copied().collect();
    if_scope.final_actions = final_actions.iter().copied().collect();

    let new_assigned_variable_ids = if_block_context.assigned_variable_ids.clone();
    let new_possibly_assigned_variable_ids = if_block_context.possibly_assigned_variable_ids.clone();

    if_block_context.assigned_variable_ids.extend(assigned_variable_ids);
    if_block_context.possibly_assigned_variable_ids.extend(possibly_assigned_variable_ids);

    inherit_branch_context_properties(context, outer_block_context, &if_block_context);

    if !has_leaving_statements {
        let new_assigned_variable_ids_keys = new_assigned_variable_ids.keys().cloned().collect::<Vec<String>>();

        update_if_scope(
            context,
            if_scope,
            &mut if_block_context,
            outer_block_context,
            new_assigned_variable_ids,
            new_possibly_assigned_variable_ids,
            if_scope.conditionally_changed_variable_ids.clone(),
            true,
        );

        if !if_scope.reasonable_clauses.is_empty() {
            for variable_id in new_assigned_variable_ids_keys {
                let previous_clauses = std::mem::take(&mut if_scope.reasonable_clauses);
                if_scope.reasonable_clauses = BlockContext::filter_clauses(
                    context,
                    &variable_id,
                    previous_clauses,
                    if_block_context.locals.get(&variable_id).map(|rc| rc.as_ref()),
                );
            }
        }
    } else if !has_break_statement {
        if_scope.reasonable_clauses = vec![];

        if let Some(post_leaving_if_context) = if_scope.post_leaving_if_context.as_mut()
            && !if_conditional_scope.assigned_in_conditional_variable_ids.is_empty()
        {
            add_conditionally_assigned_variables_to_context(
                context,
                artifacts,
                post_leaving_if_context,
                outer_block_context,
                if_statement.condition,
                &if_conditional_scope.assigned_in_conditional_variable_ids,
            )?;
        }
    }

    if !if_scope.negated_types.is_empty() {
        let variables_to_update = if_scope
            .negated_types
            .keys()
            .filter(|key| pre_assignment_else_redefined_locals.contains_key(*key))
            .cloned()
            .collect::<HashSet<String>>();

        outer_block_context.update(
            context,
            &old_if_block_context,
            &mut if_block_context,
            has_leaving_statements,
            variables_to_update,
            &mut if_scope.updated_variables,
        );
    }

    if !has_ending_statements {
        let variables_possibly_in_scope = if_block_context
            .variables_possibly_in_scope
            .iter()
            .filter(|id| !outer_block_context.variables_possibly_in_scope.contains(*id))
            .cloned()
            .collect::<HashSet<String>>();

        if let Some(loop_scope) = artifacts.loop_scope.as_mut() {
            if !has_continue_statement && !has_break_statement {
                if_scope.new_variables_possibly_in_scope = variables_possibly_in_scope.clone();
            }

            loop_scope.variables_possibly_in_scope.extend(variables_possibly_in_scope);
        } else if !has_leaving_statements {
            if_scope.new_variables_possibly_in_scope = variables_possibly_in_scope;
        }
    }

    outer_block_context.update_references_possibly_from_confusing_scope(&if_block_context);

    Ok(())
}

fn analyze_else_if_clause<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    if_scope: &mut IfScope<'ctx>,
    else_block_context: &mut BlockContext<'ctx>,
    outer_block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    else_if_clause: (&'ast Expression<'arena>, &'ast [Statement<'arena>]),
) -> Result<(), AnalysisError> {
    let (if_conditional_scope, applied_else_block_context) =
        conditional::analyze(context, else_block_context.clone(), artifacts, if_scope, else_if_clause.0, true)?;
    *else_block_context = applied_else_block_context;

    let mut else_if_block_context = if_conditional_scope.if_body_context;
    let mut conditionally_referenced_variable_ids = if_conditional_scope.conditionally_referenced_variable_ids;
    let assigned_in_conditional_variable_ids = if_conditional_scope.assigned_in_conditional_variable_ids;

    let mut mixed_variables = vec![];
    for (variable_id, variable_type) in else_if_block_context.locals.iter() {
        if variable_type.is_mixed() && outer_block_context.locals.contains_key(variable_id) {
            mixed_variables.push(variable_id.clone());
        }
    }

    let mut else_if_clauses = formula::get_formula(
        else_if_clause.0.span(),
        else_if_clause.0.span(),
        else_if_clause.0,
        context.get_assertion_context_from_block(&else_if_block_context),
        artifacts,
    ).unwrap_or_else(|| {
        let clauses_statements_span = match (else_if_clause.1.first(), else_if_clause.1.last()) {
            (Some(first_statement), Some(last_statement)) => Some(first_statement.span().join(last_statement.span())),
            _ => None,
        };

        if let Some(clauses_statements_span) = clauses_statements_span {
            context.collector.report_with_code(
                IssueCode::ConditionIsTooComplex,
                Issue::warning("Condition is too complex for precise type analysis.")
                    .with_annotation(
                        Annotation::primary(else_if_clause.0.span())
                            .with_message("This `elseif` condition is too complex for the analyzer to fully understand"),
                    )
                    .with_annotation(
                        Annotation::secondary(clauses_statements_span)
                            .with_message("Type inference within this branch may be inaccurate as a result"),
                    )
                    .with_note(
                        "The analyzer limits the number of logical paths it explores for a single condition to prevent performance issues."
                    )
                    .with_note(
                        "Because this limit was exceeded, type assertions from the condition will not be applied to the following code, which may lead to incorrect type information."
                    )
                    .with_help(
                        "Consider refactoring this complex condition into smaller, intermediate boolean variables or nested `if` statements.",
                    ),
            );
        }

        vec![]
    });

    for clause in &mut else_if_clauses {
        let keys = clause.possibilities.keys().cloned().collect::<Vec<String>>();
        mixed_variables.retain(|i| !keys.contains(i));

        'outer: for key in keys {
            for mixed_var_id in &mixed_variables {
                if !is_derived_access_path(&key, mixed_var_id) {
                    continue;
                }

                *clause = Clause::new(
                    Default::default(),
                    else_if_clause.0.span(),
                    else_if_clause.0.span(),
                    Some(true),
                    Some(true),
                    Some(false),
                );

                break 'outer;
            }
        }
    }

    let mut entry_clauses = vec![];
    for mut clause in if_conditional_scope.entry_clauses {
        'set_clause: for key in clause.possibilities.keys() {
            for conditional_assigned_variable_id in assigned_in_conditional_variable_ids.keys() {
                if !is_derived_access_path(key, conditional_assigned_variable_id) {
                    continue;
                }

                clause = Clause::new(
                    Default::default(),
                    else_if_clause.0.span(),
                    else_if_clause.0.span(),
                    Some(true),
                    Some(true),
                    Some(false),
                );

                break 'set_clause;
            }
        }

        entry_clauses.push(Rc::new(clause));
    }

    check_for_paradox(&mut context.collector, &entry_clauses, &else_if_clauses, else_if_clause.0.span());

    let else_if_clauses = saturate_clauses(else_if_clauses.iter());
    else_if_block_context.clauses = if entry_clauses.is_empty() {
        else_if_clauses.clone().into_iter().map(Rc::new).collect()
    } else {
        saturate_clauses(else_if_clauses.iter().chain(entry_clauses.iter().map(Rc::deref)))
            .into_iter()
            .map(Rc::new)
            .collect()
    };

    if !else_if_block_context.reconciled_expression_clauses.is_empty() {
        let reconciled_expression_clauses = else_if_block_context
            .reconciled_expression_clauses
            .iter()
            .map(|clause| clause.hash)
            .collect::<HashSet<u32>>();

        else_if_block_context.clauses.retain(|clause| !reconciled_expression_clauses.contains(&clause.hash));
    }

    if entry_clauses.iter().any(|clause| !clause.possibilities.is_empty()) {
        let omit_keys =
            entry_clauses.iter().flat_map(|clause| clause.possibilities.keys()).cloned().collect::<HashSet<String>>();

        let (outer_truthes, _) = find_satisfying_assignments(
            outer_block_context.clauses.iter().map(Rc::as_ref).cloned().collect_vec().as_slice(),
            None,
            &mut HashSet::default(),
        );

        let omit_keys =
            omit_keys.into_iter().filter(|key| !outer_truthes.contains_key(key)).collect::<HashSet<String>>();

        conditionally_referenced_variable_ids.retain(|key| !omit_keys.contains(key));
    }

    let (reconcilable_else_if_types, active_else_if_types) = find_satisfying_assignments(
        else_if_block_context.clauses.iter().map(Rc::as_ref).cloned().collect_vec().as_slice(),
        Some(else_if_clause.0.span()),
        &mut conditionally_referenced_variable_ids,
    );

    let negated_if_clauses = negate_or_synthesize(
        else_if_clauses.clone(),
        else_if_clause.0,
        context.get_assertion_context_from_block(&else_if_block_context),
        artifacts,
    );

    let (negated_else_if_types, _) =
        find_satisfying_assignments(negated_if_clauses.as_slice(), None, &mut HashSet::default());

    let all_negated_variables =
        HashSet::from_iter(negated_else_if_types.keys().cloned().chain(if_scope.negated_types.keys().cloned()));

    for negated_variable_id in all_negated_variables {
        if let Some(negated_variable_type_assertions) = negated_else_if_types.get(&negated_variable_id) {
            if let Some(negated_elseif_type_assertions) = if_scope.negated_types.swap_remove(&negated_variable_id) {
                if_scope.negated_types.insert(
                    negated_variable_id.clone(),
                    negated_variable_type_assertions
                        .iter()
                        .cloned()
                        .chain(negated_elseif_type_assertions.into_iter())
                        .collect(),
                );
            } else {
                if_scope.negated_types.insert(negated_variable_id.clone(), negated_variable_type_assertions.clone());
            }
        }
    }

    let mut newly_reconciled_variable_ids = HashSet::default();
    if !reconcilable_else_if_types.is_empty() {
        reconcile_keyed_types(
            context,
            &reconcilable_else_if_types,
            active_else_if_types,
            &mut else_if_block_context,
            &mut newly_reconciled_variable_ids,
            &conditionally_referenced_variable_ids,
            &else_if_clause.0.span(),
            false,
            false,
        );

        if !newly_reconciled_variable_ids.is_empty() {
            else_if_block_context.clauses = BlockContext::remove_reconciled_clauses(
                &else_if_block_context.clauses.iter().map(Rc::deref).cloned().collect(),
                &newly_reconciled_variable_ids,
            )
            .0
            .into_iter()
            .map(Rc::new)
            .collect();

            let mut variables_to_remove = vec![];
            for (changed_variable_id, _) in reconcilable_else_if_types.iter() {
                for (variable_id, _) in else_if_block_context.locals.iter() {
                    if is_derived_access_path(variable_id, changed_variable_id)
                        && !newly_reconciled_variable_ids.contains(variable_id)
                        && !conditionally_referenced_variable_ids.contains(variable_id)
                    {
                        variables_to_remove.push(variable_id.clone());
                    }
                }
            }

            for variable_id in variables_to_remove {
                else_if_block_context.remove_possible_reference(&variable_id);
            }
        }
    }

    let pre_assigned_variable_ids = std::mem::take(&mut else_if_block_context.assigned_variable_ids);
    let pre_possibly_assigned_variable_ids = std::mem::take(&mut else_if_block_context.possibly_assigned_variable_ids);

    analyze_statements(else_if_clause.1, context, &mut else_if_block_context, artifacts)?;

    for variable_id in else_if_block_context.parent_conflicting_clause_variables.iter() {
        outer_block_context.remove_variable_from_conflicting_clauses(context, variable_id, None);
    }

    let new_assigned_variable_ids = else_if_block_context.assigned_variable_ids.clone();
    let new_possibly_assigned_variable_ids = else_if_block_context.possibly_assigned_variable_ids.clone();
    else_if_block_context.assigned_variable_ids.extend(pre_assigned_variable_ids);
    else_if_block_context.possibly_assigned_variable_ids.extend(pre_possibly_assigned_variable_ids);

    inherit_branch_context_properties(context, outer_block_context, &else_if_block_context);

    let final_actions =
        ControlAction::from_statements(else_if_clause.1.iter().collect(), vec![], Some(artifacts), true);

    let has_actions = !final_actions.is_empty();
    let has_ending_statements;
    let has_break_statement;
    let has_continue_statement;
    let has_leaving_statements;

    if !has_actions {
        has_ending_statements = false;
        has_break_statement = false;
        has_continue_statement = false;
        has_leaving_statements = false;
    } else {
        has_ending_statements = final_actions.len() == 1 && final_actions.contains(&ControlAction::End);
        has_break_statement = final_actions.len() == 1 && final_actions.contains(&ControlAction::Break);
        has_continue_statement = final_actions.len() == 1 && final_actions.contains(&ControlAction::Continue);
        has_leaving_statements = has_ending_statements || !final_actions.contains(&ControlAction::None);
    }

    if_scope.if_actions.extend(final_actions.iter().copied());

    if !has_leaving_statements {
        update_if_scope(
            context,
            if_scope,
            &mut else_if_block_context,
            outer_block_context,
            new_assigned_variable_ids.into_iter().chain(assigned_in_conditional_variable_ids).collect(),
            new_possibly_assigned_variable_ids.clone(),
            newly_reconciled_variable_ids,
            false,
        );

        if if_scope.reasonable_clauses.is_empty() || else_if_clauses.is_empty() {
            if_scope.reasonable_clauses = vec![];
        } else {
            let previous_clauses = std::mem::take(&mut if_scope.reasonable_clauses);

            if_scope.reasonable_clauses = disjoin_clauses(
                previous_clauses.into_iter().map(Rc::unwrap_or_clone).collect(),
                else_if_clauses.clone(),
                else_if_clause.0.span(),
            )
            .into_iter()
            .map(Rc::new)
            .collect();
        }
    } else {
        if_scope.reasonable_clauses = vec![];

        if !negated_else_if_types.is_empty() {
            let mut implied_outer_context = else_if_block_context.clone();

            reconcile_keyed_types(
                context,
                &negated_else_if_types,
                IndexMap::new(),
                &mut implied_outer_context,
                &mut HashSet::default(),
                &HashSet::default(),
                &else_if_clause.0.span(),
                false,
                false,
            );
        }
    }

    if !has_ending_statements {
        let variables_possibly_in_scope = else_if_block_context
            .variables_possibly_in_scope
            .iter()
            .filter(|id| !outer_block_context.variables_possibly_in_scope.contains(*id))
            .cloned()
            .collect::<HashSet<String>>();

        let possibly_assigned_variable_ids = new_possibly_assigned_variable_ids;

        if let Some(loop_scope) = artifacts.loop_scope.as_mut() {
            if has_leaving_statements {
                if !has_continue_statement && !has_break_statement {
                    if_scope.new_variables_possibly_in_scope.extend(variables_possibly_in_scope.clone());
                    if_scope.possibly_assigned_variable_ids.extend(possibly_assigned_variable_ids);
                }

                loop_scope.variables_possibly_in_scope.extend(variables_possibly_in_scope);
            }
        } else if !has_leaving_statements {
            if_scope.new_variables_possibly_in_scope.extend(variables_possibly_in_scope);
            if_scope.possibly_assigned_variable_ids.extend(possibly_assigned_variable_ids);
        }
    }

    if_scope.negated_clauses = match negate_formula(else_if_clauses) {
        Some(negated_formula) => saturate_clauses(if_scope.negated_clauses.iter().chain(negated_formula.iter())),
        None => vec![],
    };

    outer_block_context.update_references_possibly_from_confusing_scope(&else_if_block_context);

    Ok(())
}

fn analyze_else_statements<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    if_scope: &mut IfScope<'ctx>,
    else_block_context: &mut BlockContext<'ctx>,
    outer_block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    else_statements: Option<&[Statement<'arena>]>,
    if_span: Span,
) -> Result<(), AnalysisError> {
    if else_statements.is_none() && if_scope.negated_clauses.is_empty() && else_block_context.clauses.is_empty() {
        if_scope.final_actions.insert(ControlAction::None);
        if_scope.assigned_variable_ids = None;
        if_scope.new_variables = None;
        if_scope.redefined_variables = None;
        if_scope.reasonable_clauses = vec![];

        return Ok(());
    }

    let else_span = match else_statements {
        Some(else_statements) => match (else_statements.first(), else_statements.last()) {
            (Some(first_statement), Some(last_statement)) => first_statement.span().join(last_statement.span()),
            _ => if_span,
        },
        None => if_span,
    };

    else_block_context.clauses =
        saturate_clauses(else_block_context.clauses.iter().map(Rc::deref).chain(if_scope.negated_clauses.iter()))
            .into_iter()
            .map(Rc::new)
            .collect();

    let (else_types, _) = find_satisfying_assignments(
        else_block_context.clauses.iter().map(Rc::deref).cloned().collect_vec().as_slice(),
        None,
        &mut HashSet::default(),
    );

    let mut original_context = else_block_context.clone();

    if !else_types.is_empty() {
        let mut changed_variable_ids = HashSet::default();

        reconcile_keyed_types(
            context,
            &else_types,
            IndexMap::new(),
            else_block_context,
            &mut changed_variable_ids,
            &HashSet::default(),
            &else_span,
            false,
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

        let mut variables_to_remove = vec![];
        for changed_variable_id in &changed_variable_ids {
            for (variable_id, _) in else_block_context.locals.iter() {
                if variable_id.eq(changed_variable_id) {
                    continue;
                }

                if is_derived_access_path(variable_id, changed_variable_id)
                    && !changed_variable_ids.contains(variable_id)
                {
                    variables_to_remove.push(variable_id.clone());
                }
            }
        }

        for variable_id in variables_to_remove {
            else_block_context.remove_possible_reference(&variable_id);
        }
    }

    let old_else_context = else_block_context.clone();
    let pre_assigned_variable_ids = std::mem::take(&mut else_block_context.assigned_variable_ids);
    let pre_possibly_assigned_variable_ids = std::mem::take(&mut else_block_context.possibly_assigned_variable_ids);

    if let Some(statements) = else_statements {
        analyze_statements(statements, context, else_block_context, artifacts)?;
    }

    for variable_id in &else_block_context.parent_conflicting_clause_variables {
        outer_block_context.remove_variable_from_conflicting_clauses(context, variable_id, None);
    }

    let new_assigned_variable_ids = else_block_context.assigned_variable_ids.clone();
    let new_possibly_assigned_variable_ids = else_block_context.possibly_assigned_variable_ids.clone();
    else_block_context.assigned_variable_ids.extend(pre_assigned_variable_ids);
    else_block_context.possibly_assigned_variable_ids.extend(pre_possibly_assigned_variable_ids);

    if else_statements.is_some() {
        inherit_branch_context_properties(context, outer_block_context, else_block_context);
    }

    let final_actions = match else_statements {
        Some(else_statements) => {
            ControlAction::from_statements(else_statements.iter().collect(), vec![], Some(artifacts), true)
        }
        None => HashSet::from_iter([ControlAction::None]),
    };

    let has_actions = !final_actions.is_empty();
    let has_ending_statements;
    let has_break_statement;
    let has_continue_statement;
    let has_leaving_statements;

    if !has_actions {
        has_ending_statements = false;
        has_break_statement = false;
        has_continue_statement = false;
        has_leaving_statements = false;
    } else {
        has_ending_statements = final_actions.len() == 1 && final_actions.contains(&ControlAction::End);
        has_break_statement = final_actions.len() == 1 && final_actions.contains(&ControlAction::Break);
        has_continue_statement = final_actions.len() == 1 && final_actions.contains(&ControlAction::Continue);
        has_leaving_statements = has_ending_statements || !final_actions.contains(&ControlAction::None);
    }

    if_scope.final_actions.extend(final_actions);

    if !has_leaving_statements {
        update_if_scope(
            context,
            if_scope,
            else_block_context,
            &mut original_context,
            new_assigned_variable_ids,
            new_possibly_assigned_variable_ids.clone(),
            if_scope.conditionally_changed_variable_ids.clone(),
            true,
        );

        if_scope.reasonable_clauses = vec![];
    }

    if !if_scope.negated_types.is_empty() {
        let variables_to_update = if_scope.negated_types.keys().cloned().collect::<HashSet<String>>();

        outer_block_context.update(
            context,
            &old_else_context,
            else_block_context,
            has_leaving_statements,
            variables_to_update,
            &mut if_scope.updated_variables,
        );
    }

    if !has_ending_statements {
        let variables_possibly_in_scope = else_block_context
            .variables_possibly_in_scope
            .iter()
            .filter(|&id| !outer_block_context.variables_possibly_in_scope.contains(id))
            .cloned()
            .collect::<HashSet<String>>();

        if has_leaving_statements {
            if let Some(loop_scope) = artifacts.loop_scope.as_mut() {
                if !has_continue_statement && !has_break_statement {
                    if_scope.new_variables_possibly_in_scope.extend(variables_possibly_in_scope.clone());
                }

                loop_scope.variables_possibly_in_scope.extend(variables_possibly_in_scope);
            }
        } else {
            if_scope.new_variables_possibly_in_scope.extend(variables_possibly_in_scope.clone());
            if_scope.possibly_assigned_variable_ids.extend(new_possibly_assigned_variable_ids);
        }
    }

    outer_block_context.update_references_possibly_from_confusing_scope(else_block_context);

    Ok(())
}

fn add_conditionally_assigned_variables_to_context<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    artifacts: &mut AnalysisArtifacts,
    post_leaving_if_block_context: &mut BlockContext<'ctx>,
    post_if_block_context: &mut BlockContext<'ctx>,
    condition: &Expression<'arena>,
    assigned_in_conditional_variable_ids: &HashMap<String, u32>,
) -> Result<(), AnalysisError> {
    if assigned_in_conditional_variable_ids.is_empty() {
        return Ok(());
    }

    let old_expression_types = artifacts.expression_types.clone();
    let expressions = get_definitely_evaluated_ored_expressions(condition);

    let (result, _) = context.record(|context| {
        for expression in expressions {
            let negated_expression = new_synthetic_negation(context.arena, expression);
            let assertion = new_synthetic_call(context.arena, "assert", negated_expression);

            let was_inside_negation = post_leaving_if_block_context.inside_negation;
            post_leaving_if_block_context.inside_negation = true;

            assertion.analyze(context, post_leaving_if_block_context, artifacts)?;

            post_leaving_if_block_context.inside_negation = was_inside_negation;
        }

        Ok(())
    });

    result?;

    artifacts.expression_types = old_expression_types;

    for variable_id in assigned_in_conditional_variable_ids.keys() {
        if let Some(variable_type) = post_leaving_if_block_context.locals.get(variable_id) {
            post_if_block_context.locals.insert(variable_id.clone(), variable_type.clone());
        }
    }

    Ok(())
}

fn update_if_scope<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    if_scope: &mut IfScope<'ctx>,
    if_block_context: &mut BlockContext<'ctx>,
    outer_block_context: &mut BlockContext<'ctx>,
    new_assigned_variable_ids: HashMap<String, u32>,
    new_possibly_assigned_variable_ids: HashSet<String>,
    newly_reconciled_variable_ids: HashSet<String>,
    update_new_variables: bool,
) {
    let mut redefined_variables =
        if_block_context.get_redefined_locals(&outer_block_context.locals, false, &mut HashSet::default());

    match &mut if_scope.new_variables {
        Some(new_variables) => {
            let mut to_remove = vec![];
            for (new_variable, new_variable_type) in new_variables.iter_mut() {
                if !if_block_context.has_variable(new_variable) {
                    to_remove.push(new_variable.clone());
                } else if let Some(variable_type) = if_block_context.locals.get(new_variable) {
                    *new_variable_type = combine_union_types(new_variable_type, variable_type, context.codebase, false);
                } else {
                    unreachable!("variable is known to be in if_block_context");
                }
            }

            for new_variable in to_remove {
                new_variables.remove(&new_variable);
            }
        }
        None => {
            if update_new_variables {
                if_scope.new_variables = Some(
                    if_block_context
                        .locals
                        .iter()
                        .filter(|(variable_id, _)| !outer_block_context.locals.contains_key(*variable_id))
                        .map(|(variable_id, variable_type)| (variable_id.clone(), variable_type.as_ref().clone()))
                        .collect(),
                );
            }
        }
    }

    let mut possibly_redefined_variables = HashMap::default();
    for (variable_id, variable_type) in redefined_variables.iter() {
        if new_possibly_assigned_variable_ids.contains(variable_id)
            || !newly_reconciled_variable_ids.contains(variable_id)
        {
            possibly_redefined_variables.insert(variable_id.clone(), variable_type.clone());
        }
    }

    match &mut if_scope.assigned_variable_ids {
        Some(assigned_variable_ids) => {
            assigned_variable_ids.extend(new_assigned_variable_ids);
        }
        None => {
            if_scope.assigned_variable_ids = Some(new_assigned_variable_ids);
        }
    }

    if_scope.possibly_assigned_variable_ids.extend(new_possibly_assigned_variable_ids);

    match &mut if_scope.redefined_variables {
        Some(redefined_scope_variables) => {
            let mut variables_to_remove: Vec<String> = vec![];
            for (redefined_variable, variable_type) in redefined_scope_variables.iter_mut() {
                let Some(redefined_type) = redefined_variables.remove(redefined_variable) else {
                    variables_to_remove.push(redefined_variable.clone());
                    continue;
                };

                *variable_type = combine_union_types(&redefined_type, variable_type, context.codebase, false);
            }

            for variable in variables_to_remove {
                redefined_scope_variables.remove(&variable);
            }

            for (variable_id, variable_type) in possibly_redefined_variables {
                let resulting_type = match if_scope.possibly_redefined_variables.get(&variable_id) {
                    Some(existing_type) => combine_union_types(&variable_type, existing_type, context.codebase, false),
                    None => variable_type,
                };

                if_scope.possibly_redefined_variables.insert(variable_id, resulting_type);
            }
        }
        None => {
            if_scope.redefined_variables = Some(redefined_variables);
            if_scope.possibly_redefined_variables = possibly_redefined_variables;
        }
    }
}

fn get_definitely_evaluated_ored_expressions<'ast, 'arena>(
    expression: &'ast Expression<'arena>,
) -> Vec<&'ast Expression<'arena>> {
    if let Expression::Binary(Binary {
        lhs,
        operator: BinaryOperator::Or(_) | BinaryOperator::LowOr(_) | BinaryOperator::LowXor(_),
        rhs,
    }) = expression
    {
        return get_definitely_evaluated_ored_expressions(lhs)
            .into_iter()
            .chain(get_definitely_evaluated_ored_expressions(rhs))
            .collect();
    }

    vec![expression]
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::code::IssueCode;
    use crate::test_analysis;

    test_analysis! {
        name = conditional_negation,
        code = indoc! {r#"
            <?php

            /**
             * @param int<0, max> $offset
             *
             * @return false|int<0, max>
             */
            function strpos(string $haystack, string $needle, int $offset = 0): false|int
            {
                return strpos($haystack, $needle, $offset);
            }

            /**
             * @param int<0, max> $offset
             *
             * @return null|int<0, max>
             */
            function search1(string $haystack, string $needle, int $offset = 0): null|int
            {
                if ('' === $needle) {
                    return null;
                }

                if (false === ($pos = strpos($haystack, $needle, $offset))) {
                    return null;
                } else {
                    return $pos;
                }
            }

            /**
             * @param int<0, max> $offset
             *
             * @return null|int<0, max>
             */
            function search2(string $haystack, string $needle, int $offset = 0): null|int
            {
                if ('' === $needle) {
                    return null;
                }

                if (false === ($pos = strpos($haystack, $needle, $offset))) {
                    $_tmp = null;
                } else {
                    $_tmp = $pos;
                }

                return $_tmp;
            }
        "#},
    }

    test_analysis! {
        name = simple_if_statement,
        code = indoc! {r#"
            <?php

            $a = 1;
            $b = 2;

            if ($a > $b) {
                echo "a is greater than b";
            } elseif ($a < $b) {
                echo "a is less than b";
            } else {
                echo "a is equal to b";
            }
        "#},
        issues = [
            IssueCode::ImpossibleCondition, // `if ($a > $b)` is never executed
            IssueCode::RedundantComparison, // `$a > $b` is always false
            IssueCode::RedundantCondition, // `if ($a < $b)` is always executed
            IssueCode::RedundantComparison, // `$a < $b` is always true
        ],
    }

    test_analysis! {
        name = if_narrowing_string_or_null_to_string,
        code = indoc! {r#"
            <?php
            function takes_string(string $_s): void {}

            function process_string_or_null(?string $input): void {
                if ($input !== null) {
                    takes_string($input);
                }
            }
        "#},
    }

    test_analysis! {
        name = if_else_different_assignments_widening,
        code = indoc! {r#"
            <?php

            /** @param int|string $_is */
            function takes_int_or_string(mixed $_is): void {}

            function get_int_or_string(bool $condition): void {
                if ($condition) {
                    $value = "hello";
                } else {
                    $value = 123;
                }

                takes_int_or_string($value);
            }
        "#},
    }

    test_analysis! {
        name = if_var_becomes_never_due_to_impossible_condition,
        code = indoc! {r#"
            <?php
            function takes_string(string $_s): void {}

            /** @param 'a' $input */
            function test_var_becomes_never(string $input): void {
                if ($input !== 'a') {
                    // $input is 'a' and $input is not 'a' => impossible
                    // so $input is 'never' in this block
                    takes_string($input);
                }
            }
        "#},
        issues = [
            IssueCode::RedundantComparison,
            IssueCode::ImpossibleCondition,
            IssueCode::NoValue,
        ],
    }

    test_analysis! {
        name = if_var_narrowed_then_used_after_with_original_possibilities,
        code = indoc! {r#"
            <?php
            /** @param 'a'|'b' $_ab */
            function expect_a_or_b(string $_ab): void {
                if ($_ab === 'a') {
                    expect_a($_ab);
                } else {
                    expect_b($_ab);
                }

                expect_a_or_b($_ab); // $_ab is still 'a'|'b' here
            }

            /** @param 'a' $_a */
            function expect_a(string $_a): void {}

            /** @param 'b' $_b */
            function expect_b(string $_b): void {}

        "#},
    }

    test_analysis! {
        name = if_assign_in_condition_and_used_after,
        code = indoc! {r#"
            <?php

            /** @param 'hello' $_s */
            function takes_string(string $_s): void {}

            /** @param null $_n */
            function takes_null($_n): void {}

            /** @param 'hello'|null $_sn */
            function takes_string_or_null(?string $_sn): void {
                if ($_sn !== null) {
                    takes_string($_sn);
                } else {
                    takes_null($_sn);
                }
            }

            /** @param bool $cond */
            function test_assign_in_cond_used_after(bool $cond): void {
                if ($cond) {
                    $val = "hello";
                } else {
                    $val = null;
                }

                if ($val) {
                    takes_string($val); // $val is "hello" here
                }

                takes_string_or_null($val); // $val is "hello"|null here
            }
        "#},
    }

    test_analysis! {
        name = if_condition_on_array_element_type,
        code = indoc! {r#"
            <?php
            /** @param string $_s */
            function takes_string(string $_s): void {}

            /** @param array{'data': string|int} $arr */
            function test_array_element_condition(array $arr): void {
                if ($arr['data'] === "text") {
                    takes_string($arr['data']);
                }
            }
        "#},
    }

    test_analysis! {
        name = if_array_is_not_empty_narrowing,
        code = indoc! {r#"
            <?php
            /** @param non-empty-array<int, string> $_non_empty_arr */
            function expect_non_empty_arr(array $_non_empty_arr): void {}

            /** @param array<int, string> $input_arr */
            function process_array_check_not_empty(array $input_arr): void {
                if ($input_arr !== []) {
                    expect_non_empty_arr($input_arr);
                }
            }
        "#},
        issues = []
    }

    test_analysis! {
        name = if_array_is_empty_narrowing,
        code = indoc! {r#"
            <?php

            /** @param array{} $_empty_arr */
            function expect_empty_arr(array $_empty_arr): void {}

            /** @param array<int, string> $input_arr */
            function process_array_check_empty(array $input_arr): void {
                if ($input_arr === []) {
                    expect_empty_arr($input_arr);
                }
            }
        "#},
    }

    test_analysis! {
        name = if_else_both_define_var_type_is_union,
        code = indoc! {r#"
            <?php
            /** @param 1|2 $_one_or_two */
            function expect_one_or_two(int $_one_or_two): void {}

            function test_defined_in_both_if_else(bool $condition): void {
                if ($condition) {
                    $num = 1;
                } else {
                    $num = 2;
                }

                expect_one_or_two($num);
            }
        "#},
    }

    test_analysis! {
        name = if_variable_defined_only_in_if_is_possibly_undefined_after,
        code = indoc! {r#"
            <?php
            /** @param string|null $_s */
            function expect_string_or_null(?string $_s): void {}

            function test_defined_in_if(bool $condition): void {
                if ($condition) {
                    $message = "defined";
                } else {
                    $message = null;
                }

                expect_string_or_null($message);
            }
        "#},
    }

    test_analysis! {
        name = if_not_exhaustive_union_remains_union,
        code = indoc! {r#"
            <?php
            /** @param 'a'|'b'|'c' $_abc */
            function expect_abc(string $_abc): void {}

            /** @param 'a' $_a */
            function expect_a(string $_a): void {}

            /** @param 'a'|'b'|'c' $input */
            function test_not_exhaustive_union(string $input): void {
                if ($input === "a") {
                    expect_a($input);
                }

                // $input is still 'a'|'b'|'c' after this if block
                expect_abc($input);
            }
        "#},
    }

    test_analysis! {
        name = if_else_exhaustive_literal_union,
        code = indoc! {r#"
            <?php

            /** @param 'x'|'y' $_s */
            function expect_x_or_y(string $_s): void {}

            /** @param 'z' $_z_only */
            function expect_z_only(string $_z_only): void {}

            /** @param 'x'|'y'|'z' $_xyz */
            function expect_xyz(string $_xyz): void {}

            /** @param 'x'|'y'|'z' $input */
            function test_exhaustive_literal_union(string $input): void {
                if ($input === 'x' || $input === 'y') {
                    $result = $input;
                    expect_x_or_y($result);
                } else {
                    $result = 'z';
                    expect_z_only($input);
                }

                expect_xyz($result);
                expect_x_or_y($result);
            }
        "#},
        issues = [
            IssueCode::PossiblyInvalidArgument, // `expect_x_or_y` expects 'x'|'y', but $result is 'x'|'y'|'z'
        ]
    }

    test_analysis! {
        name = if_or_condition_lhs,
        code = indoc! {r#"
            <?php

            /** @param 'text'|'other' $_s */
            function takes_string(string $_s): void
            {
            }

            /** @param string|int $input */
            function test_or_lhs_true($input): void
            {
                if ($input === 'other' || $input === 'text') {
                    takes_string($input);
                }

                if ($input === 'text') {
                    takes_string($input);
                }

                if ($input === 'other') {
                    takes_string($input);

                    if ($input === 'text') {
                        takes_string($input);
                    }
                }
            }

            test_or_lhs_true('text');
        "#},
        issues = [
            IssueCode::RedundantComparison, // `$input === 'text'` is always true
            IssueCode::ImpossibleCondition, // `if ($input === 'text')` is never executed
            IssueCode::NoValue, // `takes_string($input)` is called with 'never' type
        ]
    }

    test_analysis! {
        name = if_and_condition_lhs_false_makes_whole_false,
        code = indoc! {r#"
            <?php
            /** @param 'text' $_s */
            function takes_string_unreachable(string $_s): void {}

            /** @param string|null $input */
            function test_and_lhs_false($input): void {
                if ($input !== null && $input === "text") {
                    takes_string_unreachable($input);
                }
            }

            test_and_lhs_false(null);
        "#}
    }

    test_analysis! {
        name = if_assignment_in_condition_nullable_string,
        code = indoc! {r#"
            <?php

            function get_string_or_null(): null|string
            {
                return null;
            }

            function takes_string(string $_s): void
            {
            }

            function takes_null(null $_n): void
            {
            }

            function takes_string_or_null(null|string $_s): void
            {
            }

            function test_assignment_in_condition_is_maybe_null(): void
            {
                if (($message = get_string_or_null()) !== null) {
                    takes_string($message);
                } else {
                    takes_null($message);
                }

                takes_string_or_null($message);
            }
        "#},
        issues = []
    }

    test_analysis! {
        name = if_assignment_in_condition_always_false_literal_empty_string,
        code = indoc! {r#"
            <?php

            /** @param non-empty-string $_s */
            function takes_non_empty_string(string $_s): void
            {
            }

            /** @param '' $_s */
            function takes_empty_string(string $_s): void
            {
            }

            function test_assignment_in_condition_is_falsy(): void
            {
                if ($message = "") {
                    takes_non_empty_string($message);
                } else {
                    takes_empty_string($message);
                }
            }
        "#},
        issues = [
            IssueCode::ImpossibleCondition, // `if ($message = "")` is never executed
            IssueCode::NoValue, // `takes_string($message)` is called with 'never' type
        ]
    }

    test_analysis! {
        name = nested_if_deep_narrowing_nullable_union,
        code = indoc! {r#"
            <?php

            /** @param 'target' $_s */
            function expect_target_string(string $_s): void {}

            /** @param array{'data': array{'value': 'target'|'other'|null }|null}|null $obj */
            function process_deep_nullable_union($obj): void {
                if ($obj !== null) {
                    if ($obj['data'] !== null) {
                        if ($obj['data']['value'] === 'target') {
                            expect_target_string($obj['data']['value']);
                        }
                    }
                }
            }
        "#}
    }

    test_analysis! {
        name = if_impossible_condition_literal_false,
        code = indoc! {r#"
            <?php
            /** @param string $_s */
            function takes_string_unreachable(string $_s): void {}

            function test_impossible_literal_false(): void {
                if (false) {
                    takes_string_unreachable("unreachable");
                }
            }
        "#},
        issues = [
            IssueCode::ImpossibleCondition, // `if (false)` is never executed
        ]
    }

    test_analysis! {
        name = if_redundant_condition_literal_true,
        code = indoc! {r#"
            <?php
            /** @param string $_s */
            function takes_string_reachable(string $_s): void {}
            function test_redundant_literal_true(): void {
                if (true) {
                    takes_string_reachable("always_reached");
                }
            }
        "#},
        issues = [
            IssueCode::RedundantCondition, // `if (true)` is always executed
        ]
    }

    test_analysis! {
        name = if_impossible_condition_var_is_a_and_b,
        code = indoc! {r#"
            <?php
            /** @param string $_s */
            function takes_string_unreachable(string $_s): void {}

            /** @param 'a'|'b' $input */
            function test_var_a_and_b(string $input): void {
                if ($input === 'a' && $input === 'b') {
                    takes_string_unreachable("unreachable");
                }
            }
        "#},
        issues = [
            IssueCode::RedundantComparison, // `$input === 'b'` is always false after `$input === 'a'`
            IssueCode::RedundantLogicalOperation, // `$input === 'a' && $input === 'b'` is false because `$input === 'b'` is false.
            IssueCode::ImpossibleCondition, // `if ($input === 'a' && $input === 'b')` is never executed
        ]
    }

    test_analysis! {
        name = if_else_redundant_check_in_else_for_null,
        code = indoc! {r#"
            <?php
            /** @param string $_s */
            function takes_string_unreachable(string $_s): void {}

            /** @param string|null $input */
            function test_redundant_else_check_is_null($input): void {
                if ($input === null) {
                    // $input is null
                } else {
                    // $input is string here
                    if ($input === null) { // Impossible here
                        takes_string_unreachable("unreachable_in_else");
                    }
                }
            }
        "#},
        issues = [
            IssueCode::RedundantComparison, // `$input === null` is always false in the else block
            IssueCode::ImpossibleCondition, // `if ($input === null)` in the else block is never executed
        ]
    }

    test_analysis! {
        name = if_always_falsy_type_is_null,
        code = indoc! {r#"
            <?php
            /** @param string $_s */
            function takes_string_unreachable(string $_s): void {}
            function test_always_falsy_is_null(): void {
                $val = null;
                if ($val) { // null is always falsy
                    takes_string_unreachable("unreachable");
                }
            }
        "#},
        issues = [
            IssueCode::ImpossibleCondition, // `if ($val)` is never executed because $val is null
        ]
    }

    test_analysis! {
        name = if_always_truthy_type_is_non_empty_string_literal,
        code = indoc! {r#"
            <?php
            /** @param string $_s */
            function takes_string_reachable(string $_s): void {}
            function test_always_truthy_non_empty_string(): void {
                $val = "hello";
                if ($val) { // "hello" is always truthy
                    takes_string_reachable("always_reached");
                }
            }
        "#},
        issues = [
            IssueCode::RedundantCondition, // `if ($val)` is always executed because $val is truthy
        ]
    }

    test_analysis! {
        name = formula_generated,
        code = indoc! {r#"
            <?php

            final class Tokenizer
            {
                public static function isSpecial(string $char): bool
                {
                    if (
                        $char === '<' ||
                            $char === '>' ||
                            $char === '|' ||
                            $char === '?' ||
                            $char === ',' ||
                            $char === '{' ||
                            $char === '}' ||
                            $char === '[' ||
                            $char === ']' ||
                            $char === '(' ||
                            $char === ')' ||
                            $char === ' ' ||
                            $char === '&' ||
                            $char === '='
                    ) {
                        return true;
                    } else {
                        return false;
                    }
                }
            }
        "#},
    }

    test_analysis! {
        name = eliminate_null_from_variable,
        code = indoc! {r#"
            <?php

            function get_maybe_null(): ?string {
                return null;
            }

            /**
             * @template T
             * @param T $value
             * @return T
             */
            function identity(mixed $value): mixed {
                return $value;
            }

            function eliminate_null1(): string {
                $var = get_maybe_null();
                if ($var === null) {
                    return '';
                }

                return identity($var);
            }

            function eliminate_null2(): string {
                $var = get_maybe_null();
                if (null === $var) {
                    return '';
                }

                return identity($var);
            }

            function eliminate_null3(): string {
                $var = get_maybe_null();
                if ($var !== null) {
                    return identity($var);
                }

                return '';
            }

            function eliminate_null4(): string {
                $var = get_maybe_null();
                if (null !== $var) {
                    return identity($var);
                }

                return '';
            }
        "#},
    }

    test_analysis! {
        name = eliminate_true_and_false_from_scalar,
        code = indoc! {r#"
            <?php

            function i_take_non_bool_scalar(int|float|string $_val): void {}

            function i_take_non_false_scalar(true|int|float|string $_val): void {}

            function i_take_non_true_scalar(false|int|float|string $_val): void {}

            function i_take_false(bool $_val): void {}

            function i_take_true(bool $_val): void {}

            /**
             * @param scalar $val
             */
            function remove_false(mixed $val): void {
                if ($val === false) {
                    i_take_false($val);
                    return;
                }

                i_take_non_false_scalar($val);
            }

            /**
             * @param scalar $val
             */
            function remove_true(mixed $val): void {
                if ($val === true) {
                    i_take_true($val);
                    return;
                }

                i_take_non_true_scalar($val);
            }

            /**
             * @param scalar $val
             */
            function remove_true_and_false(mixed $val): void {
                if ($val === true || $val === false) {
                    return;
                }

                i_take_non_false_scalar($val);
                i_take_non_true_scalar($val);
                i_take_non_bool_scalar($val);
            }
        "#},
    }

    test_analysis! {
        name = remove_enum_case_in_else,
        code = indoc! {r#"
            <?php

            enum Color
            {
                case Transparent;
                case Black;
                case White;
                case Red;
                case Green;
                case Blue;
            }

            function get_rgba_1(Color $color): string
            {
                if ($color === Color::Transparent) {
                    return 'rgba(0, 0, 0, 0)';
                }

                [$r, $g, $b] = match ($color) {
                    Color::Black => ['0', '0', '0'],
                    Color::White => ['255', '255', '255'],
                    Color::Red => ['255', '0', '0'],
                    Color::Green => ['0', '255', '0'],
                    Color::Blue => ['0', '0', '255'],
                };

                return "rgba($r, $g, $b, 1)";
            }

            function get_rgba_2(Color $color): string
            {
                if (Color::Transparent === $color) {
                    return 'rgba(0, 0, 0, 0)';
                }

                [$r, $g, $b] = match ($color) {
                    Color::Black => ['0', '0', '0'],
                    Color::White => ['255', '255', '255'],
                    Color::Red => ['255', '0', '0'],
                    Color::Green => ['0', '255', '0'],
                    Color::Blue => ['0', '0', '255'],
                };

                return "rgba($r, $g, $b, 1)";
            }

            function get_rgba_3(Color $color): string
            {
                if (Color::Transparent !== $color) {
                    [$r, $g, $b] = match ($color) {
                        Color::Black => ['0', '0', '0'],
                        Color::White => ['255', '255', '255'],
                        Color::Red => ['255', '0', '0'],
                        Color::Green => ['0', '255', '0'],
                        Color::Blue => ['0', '0', '255'],
                    };

                    return "rgba($r, $g, $b, 1)";
                }

                return 'rgba(0, 0, 0, 0)';
            }

            function get_rgba_4(Color $color): string
            {
                if ($color !== Color::Transparent) {
                    [$r, $g, $b] = match ($color) {
                        Color::Black => ['0', '0', '0'],
                        Color::White => ['255', '255', '255'],
                        Color::Red => ['255', '0', '0'],
                        Color::Green => ['0', '255', '0'],
                        Color::Blue => ['0', '0', '255'],
                    };

                    return "rgba($r, $g, $b, 1)";
                }

                return 'rgba(0, 0, 0, 0)';
            }
        "#},
    }

    test_analysis! {
        name = if_condition_is_too_complex,
        code = indoc! {r#"
            <?php

            function is_special_case(int $id, int $count, float $score, float $threshold, bool $is_active, bool $is_admin, string $name, string $role, string $permission, string $category): bool {
                if (
                    ($id > 1000 && $count < 5 || $score >= 99.5 && $threshold < $score || $name === 'azjezz' && $role !== 'guest') &&
                    ($is_active && !$is_admin || $permission === 'write' && ($category === 'critical' || $category === 'urgent')) ||
                    !($count === 0 || $id < 0) && (
                        $role === 'admin' && $is_admin ||
                        $name !== 'guest' && $permission !== 'none' ||
                        ($score - $threshold) > 5.0 && $count > 1
                    ) && (
                        $category === 'general' || $category === 'special' ||
                        ($is_active && $is_admin && $id % 2 === 0) ||
                        ($name !== 'system' && $role !== 'user' && $score < 50.0)
                    ) || (
                        $id < 0 && $count > 100 ||
                        ($score < 10.0 && $threshold > 20.0) ||
                        ($is_active && $is_admin && $name === 'root') ||
                        ($role === 'guest' && $permission === 'read' && $category === 'public')
                    )
                ) {
                    return true;
                }

                return false;
            }
        "#},
        issues = [
            IssueCode::ConditionIsTooComplex,
        ]
    }

    test_analysis! {
        name = elseif_condition_is_too_complex,
        code = indoc! {r#"
            <?php

            function is_special_case(int $id, int $count, float $score, float $threshold, bool $is_active, bool $is_admin, string $name, string $role, string $permission, string $category): bool {
                if ($id === 0 && $name === 'admin' && $role === 'system') {
                    return true;
                } elseif (
                    ($id > 1000 && $count < 5 || $score >= 99.5 && $threshold < $score || $name === 'azjezz' && $role !== 'guest') &&
                    ($is_active && !$is_admin || $permission === 'write' && ($category === 'critical' || $category === 'urgent')) ||
                    !($count === 0 || $id < 0) && (
                        $role === 'admin' && $is_admin ||
                        $name !== 'guest' && $permission !== 'none' ||
                        ($score - $threshold) > 5.0 && $count > 1
                    ) && (
                        $category === 'general' || $category === 'special' ||
                        ($is_active && $is_admin && $id % 2 === 0) ||
                        ($name !== 'system' && $role !== 'user' && $score < 50.0)
                    ) || (
                        $id < 0 && $count > 100 ||
                        ($score < 10.0 && $threshold > 20.0) ||
                        ($is_active && $is_admin && $name === 'root') ||
                        ($role === 'guest' && $permission === 'read' && $category === 'public')
                    )
                ) {
                    return true;
                }

                return false;
            }
        "#},
        issues = [
            IssueCode::ConditionIsTooComplex,
        ]
    }
}
