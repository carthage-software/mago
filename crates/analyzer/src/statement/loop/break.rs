use std::rc::Rc;

use mago_allocator::Arena;
use mago_codex::ttype::add_optional_union_type;
use mago_codex::ttype::combine_union_types;
use mago_codex::ttype::combiner::CombinerOptions;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::cst::Break;
use mago_word::WordSet;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::context::scope::control_action::ControlAction;
use crate::error::AnalysisError;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for Break<'arena> {
    fn analyze<'ctx, A>(
        &'ast self,
        context: &mut Context<'ctx, 'arena, A>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError>
    where
        A: Arena,
    {
        let levels = super::parse_control_flow_level(
            self.level,
            context,
            block_context,
            artifacts,
            IssueCode::InvalidBreak,
            "Break level must be an integer literal.",
        )?;

        let available_levels = block_context.break_types.len() as u64;
        if available_levels == 0 {
            context.collector.report_with_code(
                IssueCode::InvalidBreak,
                Issue::error("Break statement outside of a loop or switch.").with_annotation(
                    Annotation::primary(self.span()).with_message("This break statement is not valid here."),
                ),
            );

            block_context.flags.set_has_returned(true);

            return Ok(());
        }

        if levels == 0 {
            context.collector.report_with_code(
                IssueCode::InvalidBreak,
                Issue::error("Break level must be greater than zero.").with_annotation(
                    Annotation::primary(self.level.as_ref().map_or_else(|| self.span(), |level| level.span()))
                        .with_message("Specify a positive break level."),
                ),
            );

            block_context.flags.set_has_returned(true);

            return Ok(());
        }

        if levels > available_levels {
            let error_message = format!(
                "Cannot break {} levels - only {} enclosing loop{} or switch{} available.",
                levels,
                available_levels,
                if available_levels == 1 { "" } else { "s" },
                if available_levels == 1 { "" } else { "es" }
            );

            let mut issue = Issue::error(error_message);
            if let Some(level) = &self.level {
                issue = issue.with_annotation(
                    Annotation::primary(level.span())
                        .with_message(format!("Break level must be less than or equal to {available_levels}.")),
                );
            }

            for (index, break_context) in block_context.break_types.iter().rev().enumerate() {
                let kind = if break_context.is_switch() { "switch" } else { "loop" };
                issue =
                    issue.with_annotation(Annotation::secondary(break_context.span()).with_message(format!(
                        "This is the {} enclosing {kind}.",
                        super::get_ordinal_string(index + 1)
                    )));
            }

            context.collector.report_with_code(IssueCode::InvalidBreak, issue);

            block_context.flags.set_has_returned(true);

            return Ok(());
        }

        let target_index = block_context.break_types.len() - levels as usize;
        let target_is_switch = block_context.break_types[target_index].is_switch();
        let loops_inside_target = block_context.break_types[(target_index + 1)..]
            .iter()
            .filter(|break_context| !break_context.is_switch())
            .count();
        let mut loop_scope_ref = artifacts.loop_scope.as_mut();
        for _ in 0..loops_inside_target.saturating_sub(usize::from(target_is_switch)) {
            loop_scope_ref = loop_scope_ref.and_then(|loop_scope| loop_scope.parent_loop.as_deref_mut());
        }

        if target_is_switch && loops_inside_target > 0 {
            loop_scope_ref = loop_scope_ref.and_then(|loop_scope| {
                loop_scope.final_actions.insert(ControlAction::Break);

                loop_scope.parent_loop.as_deref_mut()
            });
        }

        let has_loop_scope = loop_scope_ref.is_some();
        if let Some(loop_scope) = loop_scope_ref {
            if target_is_switch {
                loop_scope.final_actions.insert(ControlAction::LeaveSwitch);
            } else {
                loop_scope.final_actions.insert(ControlAction::Break);
            }

            let mut removed_var_ids = WordSet::default();
            let redefined_vars =
                block_context.get_redefined_locals(&loop_scope.parent_context_variables, false, &mut removed_var_ids);

            for var_id in loop_scope.parent_context_variables.keys() {
                if !redefined_vars.contains_key(var_id)
                    && let Some(current_type) = block_context.locals.get(var_id)
                {
                    loop_scope.possibly_redefined_loop_parent_variables.insert(
                        *var_id,
                        Rc::new(add_optional_union_type(
                            (**current_type).clone(),
                            loop_scope
                                .possibly_redefined_loop_parent_variables
                                .get(var_id)
                                .map(std::convert::AsRef::as_ref),
                            context.codebase,
                        )),
                    );
                }
            }

            for (var_id, var_type) in redefined_vars {
                loop_scope.possibly_redefined_loop_parent_variables.insert(
                    var_id,
                    match loop_scope.possibly_redefined_loop_parent_variables.get(&var_id) {
                        Some(existing_type) => Rc::new(combine_union_types(
                            existing_type,
                            &var_type,
                            context.codebase,
                            CombinerOptions::default(),
                        )),
                        None => Rc::clone(&var_type),
                    },
                );
            }

            if loop_scope.iteration_count == 0 {
                for (var_id, var_type) in &block_context.locals {
                    if !loop_scope.parent_context_variables.contains_key(var_id) {
                        loop_scope.possibly_defined_loop_parent_variables.insert(
                            *var_id,
                            match loop_scope.possibly_defined_loop_parent_variables.get(var_id) {
                                Some(existing_type) => Rc::new(combine_union_types(
                                    existing_type,
                                    var_type,
                                    context.codebase,
                                    CombinerOptions::default(),
                                )),
                                None => Rc::clone(var_type),
                            },
                        );
                    }
                }
            }

            if let Some(finally_scope) = block_context.finally_scope.clone() {
                let mut finally_scope = (*finally_scope).borrow_mut();
                for (var_id, var_type) in &block_context.locals {
                    if let Some(finally_type) = finally_scope.locals.get_mut(var_id) {
                        *finally_type = Rc::new(combine_union_types(
                            finally_type,
                            var_type,
                            context.codebase,
                            CombinerOptions::default(),
                        ));
                    } else {
                        finally_scope.locals.insert(*var_id, Rc::clone(var_type));
                    }
                }
            }
        }

        if target_is_switch {
            let switches_to_target = block_context.break_types[target_index..]
                .iter()
                .filter(|break_context| break_context.is_switch())
                .count();
            if let Some(case_scope_index) = artifacts.case_scopes.len().checked_sub(switches_to_target)
                && let Some(case_scope) = artifacts.case_scopes.get_mut(case_scope_index)
            {
                case_scope.record_break(&block_context.locals, context.codebase);
            }
        } else if !has_loop_scope {
            context.collector.report_with_code(
                IssueCode::InvalidBreak,
                Issue::error("Break statement outside of a loop or switch.").with_annotation(
                    Annotation::primary(self.span()).with_message("This break statement is not valid here."),
                ),
            );
        }

        block_context.flags.set_has_returned(true);

        Ok(())
    }
}
