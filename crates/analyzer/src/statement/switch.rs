use std::collections::BTreeMap;
use std::rc::Rc;

use ahash::HashMap;
use ahash::HashSet;
use indexmap::IndexMap;

use mago_algebra::clause::Clause;
use mago_codex::ttype::TType;
use mago_codex::ttype::combine_optional_union_types;
use mago_codex::ttype::combine_union_types;
use mago_codex::ttype::comparator::union_comparator::can_expression_types_be_identical;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::union::TUnion;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Statement;
use mago_syntax::ast::Switch;
use mago_syntax::ast::SwitchCase;
use mago_syntax::ast::SwitchExpressionCase;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::common::synthetic::new_synthetic_disjunctive_equality;
use crate::common::synthetic::new_synthetic_equals;
use crate::common::synthetic::new_synthetic_or;
use crate::common::synthetic::new_synthetic_variable;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::context::block::BreakContext;
use crate::context::scope::case_scope::CaseScope;
use crate::context::scope::control_action::BreakType;
use crate::context::scope::control_action::ControlAction;
use crate::context::utils::inherit_branch_context_properties;
use crate::error::AnalysisError;
use crate::expression::binary::utils::is_always_identical_to;
use crate::formula::get_formula;
use crate::formula::negate_or_synthesize;
use crate::reconciler::reconcile_keyed_types;
use crate::statement::analyze_statements;
use crate::utils::expression::get_expression_id;
use crate::utils::expression::get_root_expression_id;
use crate::utils::misc::check_for_paradox;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for Switch<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        SwitchAnalyzer::new(context, block_context, artifacts).analyze(self)
    }
}

#[derive(Debug)]
struct SwitchAnalyzer<'anlyz, 'ctx, 'arena> {
    context: &'anlyz mut Context<'ctx, 'arena>,
    block_context: &'anlyz mut BlockContext<'ctx>,
    artifacts: &'anlyz mut AnalysisArtifacts,
    new_locals: Option<BTreeMap<String, Rc<TUnion>>>,
    redefined_variables: Option<HashMap<String, Rc<TUnion>>>,
    possibly_redefined_variables: Option<BTreeMap<String, TUnion>>,
    leftover_statements: Vec<Statement<'arena>>,
    leftover_case_equality_expression: Option<Expression<'arena>>,
    negated_clauses: Vec<Clause>,
    new_assigned_variable_ids: HashMap<String, u32>,
    last_case_exit_type: ControlAction,
    case_exit_types: HashMap<usize, ControlAction>,
    case_actions: HashMap<usize, HashSet<ControlAction>>,
    has_default_case: bool,
}

impl<'anlyz, 'ctx, 'arena> SwitchAnalyzer<'anlyz, 'ctx, 'arena> {
    const SYNTHETIC_SWITCH_VAR_PREFIX: &'static str = "$-tmp-switch-";

    pub fn new(
        context: &'anlyz mut Context<'ctx, 'arena>,
        block_context: &'anlyz mut BlockContext<'ctx>,
        artifacts: &'anlyz mut AnalysisArtifacts,
    ) -> Self {
        Self {
            context,
            block_context,
            artifacts,
            new_locals: None,
            redefined_variables: None,
            possibly_redefined_variables: None,
            leftover_statements: vec![],
            leftover_case_equality_expression: None,
            negated_clauses: vec![],
            new_assigned_variable_ids: HashMap::default(),
            last_case_exit_type: ControlAction::Break,
            case_exit_types: HashMap::default(),
            case_actions: HashMap::default(),
            has_default_case: false,
        }
    }

    pub fn analyze(mut self, switch: &Switch<'arena>) -> Result<(), AnalysisError> {
        let was_inside_conditional = self.block_context.inside_conditional;
        self.block_context.inside_conditional = true;
        switch.expression.analyze(self.context, self.block_context, self.artifacts)?;
        self.block_context.inside_conditional = was_inside_conditional;

        let subject_type = match self.artifacts.get_rc_expression_type(&switch.expression).cloned() {
            Some(t) => t,
            None => Rc::new(get_mixed()),
        };

        let (is_synthetic, subject_id, root_subject_id, subject_for_conditions) =
            self.get_subject_info(switch, &subject_type);

        let original_context = self.block_context.clone();

        let cases = switch.body.cases();
        if cases.is_empty() {
            return Ok(());
        }

        let indexed_cases = cases.iter().enumerate().collect::<IndexMap<_, _>>();

        let mut last_case_index = cases.len() - 1;
        for (i, case) in indexed_cases.iter() {
            if case.is_default() {
                self.has_default_case = true;
                last_case_index = *i;
                break;
            }
        }

        for (i, case) in indexed_cases.iter().rev() {
            self.update_case_exit_map(case, *i);
        }

        let mut all_options_returned = true;
        let mut previous_empty_cases = vec![];

        let mut previously_matching_case = None;
        for (i, case) in indexed_cases {
            let is_last = i == last_case_index;
            let case_exit_type = &self.case_exit_types[&i];
            if case_exit_type != &ControlAction::Return {
                all_options_returned = false;
            }

            if let SwitchCase::Expression(switch_case) = case
                && case.statements().is_empty()
                && !is_last
            {
                previous_empty_cases.push(switch_case);
                continue;
            };

            let is_matching = self.analyze_case(
                switch,
                &subject_for_conditions,
                is_synthetic,
                &subject_id,
                case,
                &previous_empty_cases,
                &original_context,
                is_last,
                i,
                previously_matching_case,
            )?;

            if let Some(true) = is_matching {
                previously_matching_case = Some((all_options_returned, case.span()));
            }

            previous_empty_cases = vec![];
        }

        let mut possibly_redefined_vars = self.possibly_redefined_variables.unwrap_or_default();
        if let Some(new_locals) = self.new_locals {
            possibly_redefined_vars.retain(|k, _| !new_locals.contains_key(k));
            self.block_context.locals.extend(new_locals);
        }

        if let Some(redefined_vars) = self.redefined_variables {
            possibly_redefined_vars.retain(|k, _| !redefined_vars.contains_key(k));
            self.block_context.locals.extend(redefined_vars.clone());
        }

        for (var_id, var_type) in possibly_redefined_vars {
            if let Some(context_type) = self.block_context.locals.get(&var_id).cloned() {
                self.block_context.locals.insert(
                    var_id.clone(),
                    Rc::new(combine_union_types(&var_type, &context_type, self.context.codebase, false)),
                );
            }
        }

        let is_exhaustive = self.has_default_case || {
            let mut final_else_context = original_context.clone();
            let final_else_clauses: Vec<_> =
                final_else_context.clauses.iter().map(|c| (**c).clone()).chain(self.negated_clauses).collect();

            let mut final_else_referenced_ids = HashSet::default();
            let (reconcilable_types, _) =
                mago_algebra::find_satisfying_assignments(&final_else_clauses, None, &mut final_else_referenced_ids);

            if !reconcilable_types.is_empty() {
                reconcile_keyed_types(
                    self.context,
                    &reconcilable_types,
                    Default::default(),
                    &mut final_else_context,
                    &mut HashSet::default(),
                    &final_else_referenced_ids,
                    &switch.span(),
                    false,
                    false,
                );
            }

            final_else_context.locals.get(&subject_id).is_some_and(|t| t.is_never())
                || root_subject_id
                    .as_ref()
                    .is_some_and(|id| final_else_context.locals.get(id).is_some_and(|t| t.is_never()))
        };

        self.artifacts.fully_matched_switch_offsets.insert(switch.start_position().offset);
        self.block_context.assigned_variable_ids.extend(self.new_assigned_variable_ids);
        self.block_context.has_returned = all_options_returned && is_exhaustive;

        Ok(())
    }

    pub(crate) fn analyze_case<'ast>(
        &mut self,
        switch: &Switch,
        switch_condition: &'ast Expression<'arena>,
        condition_is_synthetic: bool,
        switch_var_id: &String,
        switch_case: &'ast SwitchCase<'arena>,
        previous_empty_cases: &Vec<&'ast SwitchExpressionCase<'arena>>,
        original_block_context: &BlockContext<'ctx>,
        is_last: bool,
        case_index: usize,
        previously_matching_case: Option<(bool, Span)>,
    ) -> Result<Option<bool>, AnalysisError> {
        if let Some((_, previously_matching_case_span)) = previously_matching_case {
            if switch_case.is_default() {
                self.context.collector.report_with_code(
                    IssueCode::UnreachableSwitchDefault,
                    Issue::error("Unreachable default case")
                        .with_annotation(
                            Annotation::primary(switch_case.span()).with_message("this default case is unreachable"),
                        )
                        .with_annotation(
                            Annotation::secondary(previously_matching_case_span)
                                .with_message("this previous case always matches, making subsequent cases unreachable"),
                        )
                        .with_note("Because a previous case always matches the subject, this default case can never be reached.")
                        .with_help("Remove this default case or reorder the cases."),
                );
            } else {
                self.context.collector.report_with_code(
                    IssueCode::UnreachableSwitchCase,
                    Issue::error("Unreachable switch case")
                        .with_annotation(
                            Annotation::primary(switch_case.span()).with_message("this case is unreachable"),
                        )
                        .with_annotation(
                            Annotation::secondary(previously_matching_case_span)
                                .with_message("this previous case always matches, making subsequent cases unreachable"),
                        )
                        .with_note(
                            "Because a previous case always matches the subject, this case can never be reached.",
                        )
                        .with_help("Remove this case or reorder the cases to ensure it can be reached."),
                );
            }

            return Ok(Some(false));
        }

        let mut result = None;

        let case_actions = &self.case_actions[&case_index];
        let case_exit_type = self.case_exit_types[&case_index];

        let has_ending_statements = case_actions.len() == 1 && case_actions.contains(&ControlAction::End);
        let has_leaving_statements =
            has_ending_statements || (!case_actions.is_empty() && !case_actions.contains(&ControlAction::None));

        let mut case_block_context = original_block_context.clone();

        let mut old_expression_types = self.artifacts.expression_types.clone();
        let mut case_equality_expression = None;

        if condition_is_synthetic {
            self.artifacts.set_expression_type(
                switch_condition,
                if let Some(t) = self.block_context.locals.get(switch_var_id) { (**t).clone() } else { get_mixed() },
            );
        }

        let switch_condition_type =
            self.artifacts.get_rc_expression_type(switch_condition).cloned().unwrap_or(Rc::new(get_mixed()));

        if switch_condition_type.is_never() {
            result = Some(false);

            let (code, message, annotation_message) = if switch_case.is_default() {
                (IssueCode::UnreachableSwitchDefault, "Unreachable default case", "this default case is unreachable")
            } else {
                (IssueCode::UnreachableSwitchCase, "Unreachable switch case", "this case is unreachable")
            };

            self.context.collector.report_with_code(
                code,
                Issue::error(message)
                    .with_annotation(Annotation::primary(switch_case.span()).with_message(annotation_message))
                    .with_annotation(
                        Annotation::secondary(switch.expression.span())
                            .with_message("The switch subject's type has been fully exhausted by previous cases."),
                    )
                    .with_note("The switch subject's type has been fully exhausted by previous cases.")
                    .with_help("Remove this case or ensure that the switch subject's type can still match it."),
            );
        }

        if let Some(case_condition) = switch_case.expression() {
            case_condition.analyze(self.context, self.block_context, self.artifacts)?;

            if result.is_none()
                && let Some(condition_type) = self.artifacts.get_rc_expression_type(case_condition)
            {
                if (switch_condition_type.is_true() && condition_type.is_always_falsy())
                    || !can_expression_types_be_identical(
                        self.context.codebase,
                        switch_condition_type.as_ref(),
                        condition_type.as_ref(),
                        false,
                        true,
                    )
                {
                    result = Some(false);

                    self.context.collector.report_with_code(
                        IssueCode::NeverMatchingSwitchCase,
                        Issue::error("Switch case condition will never match")
                            .with_annotation(Annotation::primary(case_condition.span()).with_message(format!(
                                "This case with type `{}` will never match the subject type.",
                                condition_type.get_id()
                            )))
                            .with_annotation(
                                Annotation::secondary(switch.expression.span()).with_message(format!(
                                    "Switch subject has type `{}`.",
                                    switch_condition_type.get_id()
                                )),
                            )
                            .with_note("This case condition will never match the switch subject's type.")
                            .with_help("Remove this case or ensure that the switch subject's type can still match it."),
                    );
                } else if !is_last
                    && ((switch_condition_type.is_true() && condition_type.is_always_truthy())
                        || is_always_identical_to(condition_type.as_ref(), switch_condition_type.as_ref()))
                {
                    result = Some(true);

                    self.context.collector.report_with_code(
                        IssueCode::AlwaysMatchingSwitchCase,
                        Issue::error("This switch case will always match, making subsequent cases unreachable.")
                            .with_annotation(
                                Annotation::primary(case_condition.span())
                                    .with_message("This case will always match the subject."),
                            )
                            .with_annotation(
                                Annotation::secondary(switch.expression.span()).with_message(format!(
                                    "Switch subject has type `{}`.",
                                    switch_condition_type.get_id()
                                )),
                            )
                            .with_note("All subsequent `case` and `default` statements are unreachable.")
                            .with_help(
                                "Remove this case or rearrange the switch cases to ensure that this case is last.",
                            ),
                    );
                }
            }

            case_equality_expression = Some(if !previous_empty_cases.is_empty() {
                for previous_empty_case in previous_empty_cases {
                    previous_empty_case.expression.analyze(self.context, self.block_context, self.artifacts)?;
                }

                new_synthetic_disjunctive_equality(
                    self.context.arena,
                    switch_condition,
                    case_condition,
                    previous_empty_cases.clone().into_iter().map(|c| c.expression).collect::<Vec<_>>(),
                )
            } else if switch_condition_type.is_true() {
                case_condition.clone()
            } else {
                new_synthetic_equals(self.context.arena, switch_condition, case_condition)
            });
        } else if result.is_none() {
            result = Some(true);
        }

        let mut case_stmts = self.leftover_statements.clone();

        case_stmts.extend(switch_case.statements().iter().cloned());

        if !has_leaving_statements && !is_last {
            let case_equality_expression = unsafe {
                // SAFETY: this is safe for non-defaults, and defaults are always last
                case_equality_expression.unwrap_unchecked()
            };

            self.leftover_case_equality_expression =
                Some(if let Some(leftover_case_equality_expr) = &self.leftover_case_equality_expression {
                    new_synthetic_or(self.context.arena, leftover_case_equality_expr, &case_equality_expression)
                } else {
                    case_equality_expression
                });

            self.leftover_statements = case_stmts;

            self.artifacts.expression_types = old_expression_types;

            return Ok(result);
        }

        if let Some(leftover_case_equality_expr) = &self.leftover_case_equality_expression {
            case_equality_expression = Some(new_synthetic_or(
                self.context.arena,
                leftover_case_equality_expr,
                &case_equality_expression
                    .unwrap_or_else(|| new_synthetic_equals(self.context.arena, switch_condition, switch_condition)),
            ));
        }

        case_block_context.break_types.push(BreakContext::Switch);

        self.leftover_statements = vec![];
        self.leftover_case_equality_expression = None;

        let assertion_context = self.context.get_assertion_context_from_block(self.block_context);

        let case_clauses = if let Some(case_equality_expr) = &case_equality_expression {
            let span = if let Some(case_condition) = switch_case.expression() {
                case_condition.span()
            } else {
                switch_case.span()
            };

            // todo: complexity!!
            get_formula(span, span, case_equality_expr, assertion_context, self.artifacts).unwrap_or_default()
        } else {
            vec![]
        };

        let mut entry_clauses = if !self.negated_clauses.is_empty() && self.negated_clauses.len() < 50 {
            let mut c = original_block_context.clauses.iter().map(|v| &**v).collect::<Vec<_>>();
            c.extend(self.negated_clauses.iter());

            mago_algebra::saturate_clauses(c)
        } else {
            original_block_context.clauses.iter().map(|v| (**v).clone()).collect::<Vec<_>>()
        };

        case_block_context.clauses = if !case_clauses.is_empty() {
            if let Some(case_condition) = switch_case.expression() {
                check_for_paradox(
                    &mut self.context.collector,
                    &entry_clauses.iter().map(|v| Rc::new(v.clone())).collect::<Vec<_>>(),
                    &case_clauses,
                    case_condition.span(),
                );

                entry_clauses.extend(case_clauses.clone());

                if entry_clauses.len() < 50 {
                    mago_algebra::saturate_clauses(entry_clauses.iter())
                } else {
                    entry_clauses
                }
            } else {
                entry_clauses
            }
        } else {
            entry_clauses
        }
        .into_iter()
        .map(|v| Rc::new(v.clone()))
        .collect();

        let (reconcilable_if_types, _) = mago_algebra::find_satisfying_assignments(
            &case_block_context.clauses.iter().map(|v| v.as_ref().clone()).collect::<Vec<_>>(),
            None,
            &mut HashSet::default(),
        );

        if !reconcilable_if_types.is_empty() {
            let mut changed_var_ids = HashSet::default();

            reconcile_keyed_types(
                self.context,
                &reconcilable_if_types,
                IndexMap::new(),
                &mut case_block_context,
                &mut changed_var_ids,
                &if !switch_case.is_default() {
                    HashSet::from_iter([switch_var_id.clone()])
                } else {
                    HashSet::default()
                },
                &switch_case.span(),
                true,
                false,
            );

            for (var_id, _) in reconcilable_if_types {
                case_block_context.variables_possibly_in_scope.insert(var_id);
            }

            if !changed_var_ids.is_empty() {
                case_block_context.clauses =
                    BlockContext::remove_reconciled_clause_refs(&case_block_context.clauses, &changed_var_ids).0;
            }
        }

        if !case_clauses.is_empty()
            && let Some(case_equality_expr) = &case_equality_expression
        {
            let assertion_context = self.context.get_assertion_context_from_block(self.block_context);

            self.negated_clauses.extend(negate_or_synthesize(
                case_clauses,
                case_equality_expr,
                assertion_context,
                self.artifacts,
            ));
        }

        self.artifacts.case_scopes.push(CaseScope::new());

        analyze_statements(&case_stmts, self.context, &mut case_block_context, self.artifacts)?;

        let Some(case_scope) = self.artifacts.case_scopes.pop() else {
            return Ok(result);
        };

        let new_expression_types = self.artifacts.expression_types.clone();
        old_expression_types.extend(new_expression_types);
        self.artifacts.expression_types = old_expression_types;

        if !matches!(case_exit_type, ControlAction::Return) {
            self.handle_non_returning_case(&case_block_context, original_block_context, case_exit_type)?;
        }

        inherit_branch_context_properties(self.context, self.block_context, &case_block_context);

        if let Some(break_vars) = &case_scope.break_vars {
            if let Some(ref mut possibly_redefined_var_ids) = self.possibly_redefined_variables {
                for (var_id, var_type) in break_vars {
                    possibly_redefined_var_ids.insert(
                        var_id.clone(),
                        combine_optional_union_types(
                            Some(var_type),
                            possibly_redefined_var_ids.get(var_id),
                            self.context.codebase,
                        ),
                    );
                }
            } else {
                self.possibly_redefined_variables = Some(
                    break_vars
                        .iter()
                        .filter(|(var_id, _)| self.block_context.locals.contains_key(*var_id))
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect(),
                );
            }

            if let Some(ref mut new_locals) = self.new_locals {
                for (var_id, var_type) in new_locals.clone() {
                    if let Some(break_var_type) = break_vars.get(&var_id) {
                        if case_block_context.locals.contains_key(&var_id) {
                            new_locals.insert(
                                var_id.clone(),
                                Rc::new(combine_union_types(break_var_type, &var_type, self.context.codebase, false)),
                            );
                        } else {
                            new_locals.remove(&var_id);
                        }
                    } else {
                        new_locals.remove(&var_id);
                    }
                }
            }

            if let Some(ref mut redefined_vars) = self.redefined_variables {
                for (var_id, var_type) in redefined_vars.clone() {
                    if let Some(break_var_type) = break_vars.get(&var_id) {
                        redefined_vars.insert(
                            var_id.clone(),
                            Rc::new(combine_union_types(break_var_type, &var_type, self.context.codebase, false)),
                        );
                    } else {
                        redefined_vars.remove(&var_id);
                    }
                }
            }
        }

        Ok(result)
    }

    fn handle_non_returning_case(
        &mut self,
        case_block_context: &BlockContext<'ctx>,
        original_block_context: &BlockContext<'ctx>,
        case_exit_type: ControlAction,
    ) -> Result<(), AnalysisError> {
        if matches!(case_exit_type, ControlAction::Continue) {
            return Ok(());
        }

        let mut removed_var_ids = HashSet::default();
        let case_redefined_vars =
            case_block_context.get_redefined_locals(&original_block_context.locals, false, &mut removed_var_ids);

        if let Some(possibly_redefined_var_ids) = &mut self.possibly_redefined_variables {
            for (var_id, var_type) in &case_redefined_vars {
                possibly_redefined_var_ids.insert(
                    var_id.clone(),
                    combine_optional_union_types(
                        Some(var_type),
                        possibly_redefined_var_ids.get(var_id),
                        self.context.codebase,
                    ),
                );
            }
        } else {
            self.possibly_redefined_variables = Some(
                case_redefined_vars
                    .clone()
                    .into_iter()
                    .filter(|(var_id, _)| self.block_context.locals.contains_key(var_id))
                    .collect(),
            );
        }

        if let Some(redefined_vars) = &mut self.redefined_variables {
            for (var_id, var_type) in redefined_vars.clone() {
                if let Some(break_var_type) = case_redefined_vars.get(&var_id) {
                    redefined_vars.insert(
                        var_id.clone(),
                        Rc::new(combine_union_types(break_var_type, &var_type, self.context.codebase, false)),
                    );
                } else {
                    redefined_vars.remove(&var_id);
                }
            }
        } else {
            self.redefined_variables = Some(case_redefined_vars.into_iter().map(|(k, v)| (k, Rc::new(v))).collect());
        }

        if let Some(new_locals) = &mut self.new_locals {
            for (var_id, var_type) in new_locals.clone() {
                if let Some(existing_var_type) = case_block_context.locals.get(&var_id) {
                    new_locals.insert(
                        var_id.clone(),
                        Rc::new(combine_union_types(existing_var_type, &var_type, self.context.codebase, false)),
                    );
                } else {
                    new_locals.remove(&var_id);
                }
            }
        } else {
            self.new_locals = Some(
                case_block_context
                    .locals
                    .clone()
                    .into_iter()
                    .filter(|(k, _)| !self.block_context.locals.contains_key(k))
                    .collect(),
            );
        }

        Ok(())
    }

    fn get_subject_info(
        &mut self,
        switch: &Switch<'arena>,
        subject_type: &Rc<TUnion>,
    ) -> (bool, String, Option<String>, Expression<'arena>) {
        if let Some(id) = get_expression_id(
            switch.expression,
            self.block_context.scope.get_class_like_name(),
            self.context.resolved_names,
            Some(self.context.codebase),
        ) {
            (false, id, get_root_expression_id(switch.expression), switch.expression.clone())
        } else {
            let subject_id = format!("{}{}", Self::SYNTHETIC_SWITCH_VAR_PREFIX, switch.expression.span().start.offset);
            self.block_context.locals.insert(subject_id.clone(), subject_type.clone());
            let subject_for_conditions =
                new_synthetic_variable(self.context.arena, &subject_id, switch.expression.span());

            (true, subject_id, None, subject_for_conditions)
        }
    }

    fn update_case_exit_map(&mut self, case: &SwitchCase, case_index: usize) {
        let raw_actions = ControlAction::from_statements(
            case.statements().iter().collect(),
            vec![BreakType::Switch],
            Some(self.artifacts),
            true,
        );

        let actions_set: HashSet<ControlAction> = raw_actions.into_iter().collect();
        let effective_action = Self::get_last_action(&actions_set);

        if let Some(action) = effective_action {
            self.last_case_exit_type = action;
        }

        self.case_exit_types.insert(case_index, self.last_case_exit_type);
        self.case_actions.insert(case_index, actions_set);
    }

    fn get_last_action(case_actions: &HashSet<ControlAction>) -> Option<ControlAction> {
        match (
            case_actions.len(),
            case_actions.contains(&ControlAction::None),
            case_actions.contains(&ControlAction::End),
            case_actions.contains(&ControlAction::Continue),
            case_actions.contains(&ControlAction::LeaveSwitch),
        ) {
            (1, false, true, _, _) => Some(ControlAction::Return),
            (1, false, _, true, _) => Some(ControlAction::Continue),
            (_, false, _, _, true) => Some(ControlAction::Break),
            (len, true, _, _, _) if len > 1 => Some(ControlAction::Break),
            _ => None,
        }
    }
}
