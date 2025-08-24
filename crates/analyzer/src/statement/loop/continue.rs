use std::rc::Rc;

use ahash::HashSet;

use mago_codex::ttype::TType;
use mago_codex::ttype::add_optional_union_type;
use mago_codex::ttype::combine_union_types;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::context::scope::control_action::ControlAction;
use crate::error::AnalysisError;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for Continue<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let levels = match self.level.as_ref() {
            Some(expression) => {
                if let Expression::Literal(Literal::Integer(LiteralInteger { value: Some(literal_integer), .. })) =
                    expression
                {
                    *literal_integer
                } else {
                    expression.analyze(context, block_context, artifacts)?;

                    context.collector.report_with_code(
                        IssueCode::InvalidContinue,
                        Issue::error("Continue level must be an integer literal.").with_annotation(
                            Annotation::primary(expression.span()).with_message(format!(
                                "Expected an integer literal here, found an expression of type `{}`.",
                                artifacts
                                    .get_expression_type(expression)
                                    .map(|union| union.get_id().as_str())
                                    .unwrap_or_else(|| "unknown")
                            )),
                        ),
                    );

                    1
                }
            }
            None => 1,
        };

        let mut i = levels;
        let mut loop_scope_ref = artifacts.loop_scope.as_mut();
        let mut loop_spans = vec![];
        while let Some(loop_scope) = loop_scope_ref.take() {
            loop_spans.push(loop_scope.span);

            if i > 1 && loop_scope.parent_loop.is_some() {
                i -= 1;
                loop_scope_ref = loop_scope.parent_loop.as_deref_mut();
            } else if i > 1 && loop_scope.parent_loop.is_none() {
                let actual_levels_available = levels - i + 1;
                let error_message = format!(
                    "Cannot continue {} levels - only {} enclosing loop{} available.",
                    levels,
                    actual_levels_available,
                    if actual_levels_available == 1 { "" } else { "s" }
                );

                let mut issue = Issue::error(error_message);
                if let Some(level) = &self.level {
                    issue = issue.with_annotation(Annotation::primary(level.span()).with_message(format!(
                        "Continue level must be less than or equal to {actual_levels_available}."
                    )));
                }

                for (i, loop_span) in loop_spans.into_iter().enumerate() {
                    issue = issue.with_annotation(
                        Annotation::secondary(loop_span)
                            .with_message(format!("This is the {} enclosing loop.", get_ordinal_string(i + 1))),
                    );
                }

                context.collector.report_with_code(IssueCode::InvalidContinue, issue);

                block_context.has_returned = true;

                return Ok(());
            } else {
                loop_scope_ref = Some(loop_scope);

                break;
            }
        }

        let Some(loop_scope) = loop_scope_ref else {
            context.collector.report_with_code(
                IssueCode::InvalidContinue,
                Issue::error("Continue statement used outside of loop.").with_annotation(
                    Annotation::primary(self.span())
                        .with_message("Continue statement must be inside a loop.".to_string()),
                ),
            );

            block_context.has_returned = true;

            return Ok(());
        };

        if block_context.break_types.last().is_some_and(|last_break_type| last_break_type.is_switch()) && levels < 2 {
            loop_scope.final_actions.insert(ControlAction::LeaveSwitch);
        } else {
            loop_scope.final_actions.insert(ControlAction::Continue);
        }

        let mut removed_var_ids = HashSet::default();
        let redefined_vars =
            block_context.get_redefined_locals(&loop_scope.parent_context_variables, false, &mut removed_var_ids);

        loop_scope.redefined_loop_variables.retain(|redefined_var, current_redefined_type| {
            match redefined_vars.get(redefined_var) {
                Some(outer_redefined_type) => {
                    *current_redefined_type =
                        combine_union_types(outer_redefined_type, current_redefined_type, context.codebase, false);

                    true
                }
                None => false,
            }
        });

        for (var_id, var_type) in redefined_vars {
            loop_scope.possibly_redefined_loop_variables.insert(
                var_id.clone(),
                add_optional_union_type(
                    var_type,
                    loop_scope.possibly_redefined_loop_variables.get(&var_id),
                    context.codebase,
                ),
            );
        }

        if let Some(finally_scope) = block_context.finally_scope.clone() {
            let mut finally_scope = (*finally_scope).borrow_mut();
            for (var_id, var_type) in &block_context.locals {
                if let Some(finally_type) = finally_scope.locals.get_mut(var_id) {
                    *finally_type = Rc::new(combine_union_types(finally_type, var_type, context.codebase, false));
                } else {
                    finally_scope.locals.insert(var_id.clone(), var_type.clone());
                }
            }
        }

        block_context.has_returned = true;

        Ok(())
    }
}

fn get_ordinal_string(n: usize) -> String {
    match n {
        1 => "first".to_string(),
        2 => "second".to_string(),
        3 => "third".to_string(),
        4 => "fourth".to_string(),
        5 => "fifth".to_string(),
        _ => {
            // Handle general cases with suffixes
            let suffix = match n % 10 {
                1 if n % 100 != 11 => "st",
                2 if n % 100 != 12 => "nd",
                3 if n % 100 != 13 => "rd",
                _ => "th",
            };

            format!("{n}{suffix}")
        }
    }
}
