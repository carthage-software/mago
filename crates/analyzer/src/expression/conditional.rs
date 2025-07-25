use std::ops::Deref;
use std::rc::Rc;

use ahash::HashSet;

use mago_algebra::clause::Clause;
use mago_algebra::find_satisfying_assignments;
use mago_algebra::saturate_clauses;
use mago_codex::assertion::Assertion;
use mago_codex::ttype::combine_optional_union_types;
use mago_codex::ttype::combine_union_types;
use mago_codex::ttype::get_mixed;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::context::scope::if_scope::IfScope;
use crate::error::AnalysisError;
use crate::formula::get_formula;
use crate::formula::negate_or_synthesize;
use crate::issue::TypingIssueKind;
use crate::reconciler::ReconcilationContext;
use crate::reconciler::assertion_reconciler;
use crate::reconciler::reconcile_keyed_types;
use crate::utils::conditional;
use crate::utils::expression::is_derived_access_path;
use crate::utils::misc::check_for_paradox;

impl Analyzable for Conditional {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let mut if_scope = IfScope::new();
        let (if_conditional_scope, applied_block_context) =
            conditional::analyze(context, block_context.clone(), artifacts, &mut if_scope, &self.condition, false)?;

        *block_context = applied_block_context;

        let mut if_block_context = if_conditional_scope.if_body_context;
        let mut conditionally_referenced_variable_ids = if_conditional_scope.conditionally_referenced_variable_ids;
        let assigned_in_conditional_variable_ids = if_conditional_scope.assigned_in_conditional_variable_ids;

        let assertion_context = context.get_assertion_context_from_block(block_context);
        let mut if_clauses = get_formula(self.condition.span(), self.condition.span(), &self.condition, assertion_context, artifacts).unwrap_or_else(|| {
            context.buffer.report(
                TypingIssueKind::ConditionIsTooComplex,
                Issue::warning("Condition is too complex for precise type analysis.")
                    .with_annotation(
                        Annotation::primary(self.condition.span())
                            .with_message("This conditional expression is too complex for the analyzer to fully understand"),
                    )
                    .with_annotation(
                        Annotation::secondary(self.span())
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

        let entry_clauses = block_context.clauses.to_vec();

        check_for_paradox(
            context.interner,
            &mut context.buffer,
            &entry_clauses,
            &if_clauses,
            &self.condition.span(),
            &assigned_in_conditional_variable_ids,
        );

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
            conditional_context_clauses
                .retain(|clause| !if_block_context.reconciled_expression_clauses.contains(clause));

            if if_block_context.clauses.len() == 1
                && if_block_context.clauses[0].wedge
                && if_block_context.clauses[0].possibilities.is_empty()
            {
                if_block_context.clauses.clear();
                if_block_context.reconciled_expression_clauses.clear();
            }
        }

        if_scope.negated_clauses = negate_or_synthesize(
            if_clauses,
            self.condition.as_ref(),
            context.get_assertion_context_from_block(block_context),
            artifacts,
        );

        if_scope.negated_types = find_satisfying_assignments(
            saturate_clauses(block_context.clauses.iter().map(Rc::deref).chain(if_scope.negated_clauses.iter()))
                .as_slice(),
            None,
            &mut HashSet::default(),
        )
        .0;

        let (reconcilable_if_types, active_if_types) = find_satisfying_assignments(
            conditional_context_clauses.into_iter().map(|rc| (*rc).clone()).collect::<Vec<_>>().as_slice(),
            Some(self.condition.span()),
            &mut conditionally_referenced_variable_ids,
        );

        if !reconcilable_if_types.is_empty() {
            let mut changed_variable_ids = HashSet::default();
            let mut reconcilation_context =
                ReconcilationContext::new(context.interner, context.codebase, &mut context.buffer, artifacts);

            reconcile_keyed_types(
                &mut reconcilation_context,
                &reconcilable_if_types,
                active_if_types,
                &mut if_block_context,
                &mut changed_variable_ids,
                &conditionally_referenced_variable_ids,
                &self.condition.span(),
                true,
                false,
            );
        }

        let mut else_block_context = block_context.clone();

        if let Some(then) = self.then.as_ref() {
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
            let mut reconcilation_context =
                ReconcilationContext::new(context.interner, context.codebase, &mut context.buffer, artifacts);

            reconcile_keyed_types(
                &mut reconcilation_context,
                &if_scope.negated_types,
                Default::default(), // todo: this is sort of a hack, we should probably pass the active types here
                &mut else_block_context,
                &mut changed_variable_ids,
                &conditionally_referenced_variable_ids,
                &self.condition.span(),
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
        self.r#else.as_ref().analyze(context, &mut else_block_context, artifacts)?;
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
                Rc::new(combine_union_types(
                    if_type.as_ref(),
                    else_type.as_ref(),
                    context.codebase,
                    context.interner,
                    false,
                )),
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
                context.interner,
            );

            block_context.locals.insert(redefined_variable_id, Rc::new(combined_type));
        }

        for if_redefined_variable in if_redefined_variables {
            let Some(if_type) = if_block_context.locals.get(&if_redefined_variable) else {
                continue;
            };

            let Some(previous_type) = block_context.locals.get(&if_redefined_variable) else {
                continue;
            };

            let combined_type = combine_optional_union_types(
                Some(if_type.as_ref()),
                Some(previous_type.as_ref()),
                context.codebase,
                context.interner,
            );

            block_context.locals.insert(if_redefined_variable, Rc::new(combined_type));
        }

        for else_redefined_variable in else_redefined_variables {
            let Some(else_type) = else_block_context.locals.get(&else_redefined_variable) else {
                continue;
            };

            let Some(previous_type) = block_context.locals.get(&else_redefined_variable) else {
                continue;
            };

            let combined_type = combine_optional_union_types(
                Some(else_type.as_ref()),
                Some(previous_type.as_ref()),
                context.codebase,
                context.interner,
            );

            block_context.locals.insert(else_redefined_variable, Rc::new(combined_type));
        }

        block_context.variables_possibly_in_scope.extend(if_block_context.variables_possibly_in_scope);
        block_context.variables_possibly_in_scope.extend(else_block_context.variables_possibly_in_scope);

        block_context
            .conditionally_referenced_variable_ids
            .extend(if_block_context.conditionally_referenced_variable_ids);
        block_context
            .conditionally_referenced_variable_ids
            .extend(else_block_context.conditionally_referenced_variable_ids);

        let mut left_type = None;
        let condition_type = artifacts.get_rc_expression_type(&self.condition).cloned();
        if let Some(then_expression) = self.then.as_ref() {
            if let Some(then_type) = artifacts.get_expression_type(then_expression) {
                left_type = Some(then_type.clone());
            }
        } else if let Some(condition_type) = condition_type.as_ref() {
            let mut reconcilation_context =
                ReconcilationContext::new(context.interner, context.codebase, &mut context.buffer, artifacts);

            let if_return_type_reconciled = assertion_reconciler::reconcile(
                &mut reconcilation_context,
                &Assertion::Truthy,
                Some(condition_type.as_ref()),
                false,
                Some(&"".to_string()),
                block_context.inside_loop,
                Some(&self.condition.span()),
                true,
                false,
            );

            left_type = Some(if_return_type_reconciled);
        }

        let right_type = artifacts.get_rc_expression_type(self.r#else.as_ref()).cloned();
        let resulting_type = match condition_type {
            Some(condition_type) => match (left_type.clone(), right_type.clone()) {
                (Some(left), _) if condition_type.is_always_truthy() => Rc::new(left),
                (_, Some(right)) if condition_type.is_always_falsy() => right,
                (Some(left), Some(right)) => {
                    Rc::new(combine_union_types(&left, right.as_ref(), context.codebase, context.interner, false))
                }
                (None, Some(right)) if self.then.is_none() => Rc::new(combine_union_types(
                    condition_type.as_ref(),
                    right.as_ref(),
                    context.codebase,
                    context.interner,
                    false,
                )),
                _ => Rc::new(get_mixed()),
            },
            None => match (left_type.clone(), right_type.clone()) {
                (Some(left), Some(right)) => {
                    Rc::new(combine_union_types(&left, right.as_ref(), context.codebase, context.interner, false))
                }
                _ => Rc::new(get_mixed()),
            },
        };

        artifacts.set_rc_expression_type(self, resulting_type);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::issue::TypingIssueKind;
    use crate::test_analysis;

    test_analysis! {
        name = conditional_if_else,
        code = indoc! {r#"
            <?php

            function get_string_or_null(): ?string { return null; }
            /** @return 1 */
            function i_take_string(string $_s): int { return 1; }
            /** @return 2 */
            function i_take_null(null $_s): int { return 2; }

            /** @param 1|2 $_i */
            function i_take_one_or_two(int $_i): void {}

            function test(): void {
                $value = get_string_or_null();
                $result = $value === null ? i_take_null($value) : i_take_string($value);
                i_take_one_or_two($result);
            }
        "#}
    }

    test_analysis! {
        name = condition_is_too_complex,
        code = indoc! {r#"
            <?php

            function is_special_case(int $id, int $count, float $score, float $threshold, bool $is_active, bool $is_admin, string $name, string $role, string $permission, string $category): bool {
                return (
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
                ) ? true : false;
            }
        "#},
        issues = [
            TypingIssueKind::ConditionIsTooComplex,
        ]
    }

    test_analysis! {
        name = conditional_return_with_assignment_in_condition,
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

                return false === ($pos = strpos($haystack, $needle, $offset)) ? null : $pos;
            }
        "#},
    }
}
