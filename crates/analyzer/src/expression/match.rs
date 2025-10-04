use std::ops::Deref;
use std::rc::Rc;

use ahash::HashSet;

use mago_algebra::saturate_clauses;
use mago_atom::atom;
use mago_codex::ttype::TType;
use mago_codex::ttype::combine_optional_union_types;
use mago_codex::ttype::combine_union_types;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_never;
use mago_codex::ttype::union::TUnion;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::common::synthetic::new_synthetic_disjunctive_identity;
use crate::common::synthetic::new_synthetic_variable;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::context::utils::inherit_branch_context_properties;
use crate::error::AnalysisError;
use crate::formula::get_formula;
use crate::formula::negate_or_synthesize;
use crate::reconciler::reconcile_keyed_types;
use crate::utils::expression::get_expression_id;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for Match<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        MatchAnalyzer::new(self, context, block_context, artifacts).analyze()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum ArmExecutionStatus {
    Always,
    Never,
    Conditional,
}

struct MatchAnalyzer<'anlyz, 'ctx, 'ast, 'arena> {
    stmt: &'ast Match<'arena>,
    context: &'anlyz mut Context<'ctx, 'arena>,
    block_context: &'anlyz mut BlockContext<'ctx>,
    artifacts: &'anlyz mut AnalysisArtifacts,
}

impl<'anlyz, 'ctx, 'ast, 'arena> MatchAnalyzer<'anlyz, 'ctx, 'ast, 'arena> {
    const SYNTHETIC_MATCH_VAR_PREFIX: &'static str = "$-tmp-match-";

    fn new(
        stmt: &'ast Match<'arena>,
        context: &'anlyz mut Context<'ctx, 'arena>,
        block_context: &'anlyz mut BlockContext<'ctx>,
        artifacts: &'anlyz mut AnalysisArtifacts,
    ) -> Self {
        Self { stmt, context, block_context, artifacts }
    }

    fn analyze(&mut self) -> Result<(), AnalysisError> {
        let was_inside_conditional = self.block_context.inside_conditional;
        self.block_context.inside_conditional = true;
        self.stmt.expression.analyze(self.context, self.block_context, self.artifacts)?;
        self.block_context.inside_conditional = was_inside_conditional;

        let mut expression_arms = vec![];
        let mut first_default_arm = None;

        for arm in self.stmt.arms.iter() {
            match arm {
                MatchArm::Expression(expr_arm) => expression_arms.push(expr_arm),
                MatchArm::Default(default_arm) => {
                    if first_default_arm.is_none() {
                        first_default_arm = Some(default_arm);
                    }
                }
            }
        }

        if expression_arms.is_empty() {
            if let Some(default_arm) = first_default_arm {
                self.report_only_default_arm();
                default_arm.expression.analyze(self.context, self.block_context, self.artifacts)?;
                if let Some(expr_type) = self.artifacts.get_rc_expression_type(&default_arm.expression).cloned() {
                    self.artifacts.set_rc_expression_type(self.stmt, expr_type);
                }
            } else {
                self.report_empty_match();
                self.set_unhandled_match_error(None, self.stmt.span(), true);
                self.artifacts.set_expression_type(self.stmt, get_never());
            }
            return Ok(());
        }

        let subject_type = match self.artifacts.get_rc_expression_type(&self.stmt.expression).cloned() {
            Some(t) => t,
            None => {
                self.report_unknown_subject_type();
                Rc::new(get_mixed())
            }
        };

        if subject_type.is_never() {
            self.report_subject_is_never();
            self.artifacts.set_expression_type(self.stmt, get_never());
            return Ok(());
        }

        let (is_synthetic, subject_id, subject_for_conditions) = self.get_subject_info(&subject_type);

        let mut arm_body_types: Vec<Rc<TUnion>> = Vec::new();
        let mut arm_exit_contexts: Vec<BlockContext<'ctx>> = Vec::new();
        let mut running_else_context = self.block_context.clone();
        let last_expression_arm_index = expression_arms.len().saturating_sub(1);
        let mut previous_arms_executed = ArmExecutionStatus::Never;

        let mut changed_variables = HashSet::default();
        let mut is_exhaustive = false;
        for (i, expression_arm) in expression_arms.iter().enumerate() {
            is_exhaustive = is_exhaustive || {
                running_else_context.locals.get(&subject_id).is_some_and(|t| t.is_never())
                    || changed_variables
                        .iter()
                        .any(|var_id| running_else_context.locals.get(var_id).is_some_and(|t| t.is_never()))
            };

            let is_last_arm = i == last_expression_arm_index && first_default_arm.is_none();
            let arm_status = self.analyze_expression_arm(
                &subject_for_conditions,
                expression_arm,
                &mut running_else_context,
                &mut arm_body_types,
                &mut arm_exit_contexts,
                is_last_arm,
                is_exhaustive,
            )?;

            if arm_status != ArmExecutionStatus::Never {
                if previous_arms_executed == ArmExecutionStatus::Never {
                    previous_arms_executed = arm_status;
                } else {
                    previous_arms_executed = ArmExecutionStatus::Conditional;
                }

                if arm_status == ArmExecutionStatus::Always {
                    is_exhaustive = true;
                }
            }

            let mut else_referenced_ids = HashSet::default();
            let (reconcilable_else_types, _) = mago_algebra::find_satisfying_assignments(
                &running_else_context.clauses.iter().map(|c| (**c).clone()).collect::<Vec<_>>(),
                None,
                &mut else_referenced_ids,
            );

            if !reconcilable_else_types.is_empty() {
                reconcile_keyed_types(
                    self.context,
                    &reconcilable_else_types,
                    Default::default(),
                    &mut running_else_context,
                    &mut changed_variables,
                    &else_referenced_ids,
                    &expression_arm.span(),
                    false,
                    false,
                );
            }
        }

        is_exhaustive = is_exhaustive || {
            running_else_context.locals.get(&subject_id).is_some_and(|t| t.is_never())
                || changed_variables
                    .iter()
                    .any(|var_id| running_else_context.locals.get(var_id).is_some_and(|t| t.is_never()))
        };

        if let Some(default_arm) = first_default_arm {
            self.analyze_default_arm(
                default_arm,
                &mut running_else_context,
                &mut arm_body_types,
                &mut arm_exit_contexts,
                previous_arms_executed,
                is_exhaustive,
            )?;

            is_exhaustive = true;
        }

        if first_default_arm.is_none() {
            if !is_exhaustive {
                let unhandled_type =
                    running_else_context.locals.get(&subject_id).cloned().unwrap_or_else(|| Rc::new(get_mixed()));

                self.report_non_exhaustive(&subject_type, &unhandled_type);
                self.set_unhandled_match_error(Some(&mut running_else_context), self.stmt.span(), false);
            }

            arm_body_types.push(Rc::new(get_never()));
            arm_exit_contexts.push(running_else_context);
        }

        self.merge_match_contexts(&arm_exit_contexts);

        if is_synthetic {
            self.block_context.locals.remove(&subject_id);
        }

        let final_type = arm_body_types.into_iter().reduce(|acc, item| {
            Rc::new(combine_union_types(acc.as_ref(), item.as_ref(), self.context.codebase, false))
        });

        self.artifacts.set_rc_expression_type(self.stmt, final_type.unwrap_or_else(|| Rc::new(get_mixed())));

        Ok(())
    }

    fn get_subject_info(&mut self, subject_type: &Rc<TUnion>) -> (bool, String, Expression<'arena>) {
        if let Some(id) = get_expression_id(
            self.stmt.expression,
            self.block_context.scope.get_class_like_name(),
            self.context.resolved_names,
            Some(self.context.codebase),
        ) {
            (false, id, self.stmt.expression.clone())
        } else {
            let subject_id =
                format!("{}{}", Self::SYNTHETIC_MATCH_VAR_PREFIX, self.stmt.expression.span().start.offset);
            self.block_context.locals.insert(subject_id.clone(), subject_type.clone());
            let subject_for_conditions =
                new_synthetic_variable(self.context.arena, &subject_id, self.stmt.expression.span());

            (true, subject_id, subject_for_conditions)
        }
    }

    fn analyze_expression_arm(
        &mut self,
        subject_expr: &Expression<'arena>,
        expression_arm: &MatchExpressionArm<'arena>,
        running_else_context: &mut BlockContext<'ctx>,
        arm_body_types: &mut Vec<Rc<TUnion>>,
        arm_exit_contexts: &mut Vec<BlockContext<'ctx>>,
        is_last: bool,
        is_exhaustive: bool,
    ) -> Result<ArmExecutionStatus, AnalysisError> {
        if is_exhaustive {
            self.report_unreachable_arm(
                expression_arm,
                "All possible types for the subject have been handled by previous arms.",
            );

            return Ok(ArmExecutionStatus::Never);
        }

        let arm_condition = new_synthetic_disjunctive_identity(
            self.context.arena,
            subject_expr,
            expression_arm.conditions.get(0).unwrap(),
            expression_arm.conditions.iter().skip(1).collect(),
        );

        let was_inside_conditional = running_else_context.inside_conditional;
        running_else_context.inside_conditional = true;
        arm_condition.analyze(self.context, running_else_context, self.artifacts)?;
        running_else_context.inside_conditional = was_inside_conditional;

        let arm_status = if let Some(condition_type) = self.artifacts.get_rc_expression_type(&arm_condition).cloned() {
            if condition_type.is_always_truthy() {
                if !is_last {
                    self.report_always_matching_arm(expression_arm);
                }
                ArmExecutionStatus::Always
            } else if condition_type.is_always_falsy() {
                self.report_unreachable_arm(expression_arm, "The condition is always false in this context.");
                ArmExecutionStatus::Never
            } else {
                ArmExecutionStatus::Conditional
            }
        } else {
            ArmExecutionStatus::Conditional
        };

        if arm_status == ArmExecutionStatus::Never {
            return Ok(ArmExecutionStatus::Never);
        }

        let mut arm_body_context = running_else_context.clone();
        let assertion_context = self.context.get_assertion_context_from_block(&arm_body_context);

        let arm_clauses = get_formula(
            expression_arm.span(),
            expression_arm.span(),
            &arm_condition,
            assertion_context,
            self.artifacts,
        )
        .unwrap_or_default();

        let combined_clauses: Vec<_> =
            saturate_clauses(arm_clauses.iter().chain(arm_body_context.clauses.iter().map(Deref::deref)))
                .into_iter()
                .map(Rc::new)
                .collect();

        let mut arm_referenced_ids = HashSet::default();
        let (reconcilable_types, active_types) = mago_algebra::find_satisfying_assignments(
            &combined_clauses.iter().map(|c| (**c).clone()).collect::<Vec<_>>(),
            None,
            &mut arm_referenced_ids,
        );

        if !reconcilable_types.is_empty() {
            reconcile_keyed_types(
                self.context,
                &reconcilable_types,
                active_types,
                &mut arm_body_context,
                &mut HashSet::default(),
                &arm_referenced_ids,
                &arm_condition.span(),
                false,
                false,
            );
        }

        expression_arm.expression.analyze(self.context, &mut arm_body_context, self.artifacts)?;
        arm_body_types.push(
            self.artifacts
                .get_rc_expression_type(&expression_arm.expression)
                .cloned()
                .unwrap_or_else(|| Rc::new(get_mixed())),
        );
        arm_exit_contexts.push(arm_body_context);

        let negated_arm_clauses = negate_or_synthesize(
            arm_clauses,
            &arm_condition,
            self.context.get_assertion_context_from_block(running_else_context),
            self.artifacts,
        );

        running_else_context.clauses =
            saturate_clauses(running_else_context.clauses.iter().map(Deref::deref).chain(negated_arm_clauses.iter()))
                .into_iter()
                .map(Rc::new)
                .collect();

        Ok(arm_status)
    }

    fn analyze_default_arm(
        &mut self,
        default_arm: &'ast MatchDefaultArm<'arena>,
        running_else_context: &mut BlockContext<'ctx>,
        arm_body_types: &mut Vec<Rc<TUnion>>,
        arm_exit_contexts: &mut Vec<BlockContext<'ctx>>,
        previous_arms_executed: ArmExecutionStatus,
        is_exhaustive: bool,
    ) -> Result<(), AnalysisError> {
        if previous_arms_executed == ArmExecutionStatus::Never {
            self.report_default_always_executed(default_arm);
        }

        if is_exhaustive {
            self.report_unreachable_default_arm(default_arm);

            return Ok(());
        }

        let mut default_context = running_else_context.clone();
        default_arm.expression.analyze(self.context, &mut default_context, self.artifacts)?;

        arm_body_types.push(
            self.artifacts
                .get_rc_expression_type(&default_arm.expression)
                .cloned()
                .unwrap_or_else(|| Rc::new(get_mixed())),
        );
        arm_exit_contexts.push(default_context);

        Ok(())
    }

    fn set_unhandled_match_error(
        &mut self,
        block_context: Option<&mut BlockContext<'ctx>>,
        span: Span,
        always_throws: bool,
    ) {
        let block_context = block_context.unwrap_or(self.block_context);

        block_context.possibly_thrown_exceptions.entry(atom("UnhandledMatchError")).or_default().insert(span);

        if always_throws {
            block_context.has_returned = true;
        }
    }

    fn merge_match_contexts(&mut self, arm_exit_contexts: &[BlockContext<'ctx>]) {
        let reachable_contexts: Vec<_> = arm_exit_contexts.iter().filter(|c| !c.has_returned).collect();

        if reachable_contexts.is_empty() {
            self.block_context.has_returned = true;
            return;
        }

        let mut all_redefined_vars: HashSet<String> = HashSet::default();
        for ctx in &reachable_contexts {
            inherit_branch_context_properties(self.context, self.block_context, ctx);

            all_redefined_vars.extend(
                ctx.get_redefined_locals(&self.block_context.locals, false, &mut HashSet::default()).keys().cloned(),
            );
        }

        for var_id in all_redefined_vars {
            let mut final_type: Option<TUnion> = None;

            for arm_context in &reachable_contexts {
                let arm_type = arm_context.locals.get(&var_id).map(|rc| rc.as_ref());
                final_type = Some(combine_optional_union_types(final_type.as_ref(), arm_type, self.context.codebase));
            }

            if let Some(final_type) = final_type {
                self.block_context.locals.insert(var_id, Rc::new(final_type));
            }
        }

        for ctx in &reachable_contexts {
            self.block_context.variables_possibly_in_scope.extend(ctx.variables_possibly_in_scope.iter().cloned());
        }
    }

    fn report_empty_match(&mut self) {
        self.context.collector.report_with_code(
            IssueCode::EmptyMatchExpression,
            Issue::error("Match expression cannot be empty.")
                .with_annotation(Annotation::primary(self.stmt.span()).with_message("This match has no arms"))
                .with_note("In PHP, an empty `match` expression will result in a fatal `UnhandledMatchError`."),
        );
    }

    fn report_only_default_arm(&mut self) {
        self.context.collector.report_with_code(
            IssueCode::MatchExpressionOnlyDefaultArm,
            Issue::help("This match expression is redundant as it only contains a default arm.")
                .with_annotation(
                    Annotation::primary(self.stmt.span())
                        .with_message("This match will always execute the default arm"),
                )
                .with_help("Consider replacing the entire match expression with the body of the default arm."),
        );
    }

    fn report_unknown_subject_type(&mut self) {
        self.context.collector.report_with_code(
            IssueCode::UnknownMatchSubjectType,
            Issue::error("The type of the match subject expression is unknown.")
                .with_annotation(
                    Annotation::primary(self.stmt.expression.span())
                        .with_message("The type of the match subject expression could not be determined."),
                )
                .with_note("Ensure that the expression is well-formed and has a valid type."),
        );
    }

    fn report_subject_is_never(&mut self) {
        self.context.collector.report_with_code(
            IssueCode::MatchSubjectTypeIsNever,
            Issue::error("The match subject is of type `never`, making the match expression unreachable.")
                .with_annotation(
                    Annotation::primary(self.stmt.expression.span())
                        .with_message("The match subject expression evaluates to `never`"),
                )
                .with_note("This means the subject can never have a value at runtime."),
        );
    }

    fn report_unreachable_arm(&mut self, arm: &MatchExpressionArm, note: &str) {
        self.context.collector.report_with_code(
            IssueCode::UnreachableMatchArm,
            Issue::warning("This match arm is unreachable.")
                .with_annotation(Annotation::primary(arm.span()).with_message("This arm can never be reached"))
                .with_annotation(Annotation::secondary(self.stmt.span()).with_message("In this match expression"))
                .with_note(note.to_string()),
        );
    }

    fn report_always_matching_arm(&mut self, arm: &MatchExpressionArm) {
        self.context.collector.report_with_code(
            IssueCode::MatchArmAlwaysTrue,
            Issue::warning("This match arm is always true, making subsequent arms unreachable.")
                .with_annotation(
                    Annotation::primary(arm.span()).with_message("This arm covers all remaining cases for the subject"),
                )
                .with_annotation(Annotation::secondary(self.stmt.span()).with_message("In this match expression"))
                .with_note("Any arms after this one can never be reached."),
        );
    }

    fn report_unreachable_default_arm(&mut self, arm: &MatchDefaultArm) {
        self.context.collector.report_with_code(
            IssueCode::UnreachableMatchDefaultArm,
            Issue::warning("This default arm is unreachable.")
                .with_annotation(Annotation::primary(arm.span()).with_message("This default arm can never be reached"))
                .with_annotation(Annotation::secondary(self.stmt.span()).with_message("In this match expression"))
                .with_note("All possible types for the subject have been handled by previous arms."),
        );
    }

    fn report_default_always_executed(&mut self, arm: &MatchDefaultArm) {
        self.context.collector.report_with_code(
            IssueCode::MatchDefaultArmAlwaysExecuted,
            Issue::warning("This default arm is always executed because no other arms can match.")
                .with_annotation(Annotation::primary(arm.span()).with_message("This arm is always executed"))
                .with_annotation(Annotation::secondary(self.stmt.span()).with_message("In this match expression"))
                .with_note("None of the preceding conditions can be met."),
        );
    }

    fn report_non_exhaustive(&mut self, subject_type: &TUnion, unhandled_type: &TUnion) {
        self.context.collector.report_with_code(
            IssueCode::MatchNotExhaustive,
            Issue::error(format!(
                "Non-exhaustive `match` expression: subject of type `{}` is not fully handled.",
                subject_type.get_id()
            ))
            .with_annotation(Annotation::primary(self.stmt.expression.span()).with_message(format!(
                "Unhandled portion of subject: `{}`",
                unhandled_type.get_id()
            )))
            .with_annotation(
                Annotation::secondary(self.stmt.span()).with_message(
                    "The `match` arms here do not cover all possible types and lack a `default` arm.",
                ),
            )
            .with_note(
                "If the subject expression evaluates to one of the unhandled types at runtime, PHP will throw an `UnhandledMatchError`.",
            )
            .with_help(format!(
                "Add conditional arms to cover type(s) `{}` or include a `default` arm to handle all other possibilities.",
                unhandled_type.get_id()
            )),
        );
    }
}
