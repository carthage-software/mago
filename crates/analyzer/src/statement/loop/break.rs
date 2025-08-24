use std::rc::Rc;

use ahash::HashMap;
use ahash::HashSet;

use mago_codex::ttype::TType;
use mago_codex::ttype::add_optional_union_type;
use mago_codex::ttype::combine_optional_union_types;
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

impl<'ast, 'arena> Analyzable<'ast, 'arena> for Break<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let levels = match self.level.as_ref() {
            Some(expression) => {
                if let Expression::Literal(Literal::Integer(LiteralInteger { value: Some(integer_value), .. })) =
                    expression
                {
                    *integer_value
                } else {
                    expression.analyze(context, block_context, artifacts)?;

                    context.collector.report_with_code(
                        IssueCode::InvalidBreak,
                        Issue::error("Break level must be an integer literal.").with_annotation(
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
        while let Some(loop_scope) = loop_scope_ref.take() {
            if i > 1 && loop_scope.parent_loop.is_some() {
                i -= 1;
                loop_scope_ref = loop_scope.parent_loop.as_deref_mut();
            } else {
                loop_scope_ref = Some(loop_scope);

                break;
            }
        }

        let mut leaving_switch = true;
        let mut leaving_loop = false;
        if let Some(loop_scope) = loop_scope_ref {
            if block_context.break_types.last().is_some_and(|last_break_type| last_break_type.is_switch()) && levels < 2
            {
                loop_scope.final_actions.insert(ControlAction::LeaveSwitch);
            } else {
                leaving_switch = false;
                leaving_loop = true;
                loop_scope.final_actions.insert(ControlAction::Break);
            }

            let mut removed_var_ids = HashSet::default();
            let redefined_vars =
                block_context.get_redefined_locals(&loop_scope.parent_context_variables, false, &mut removed_var_ids);

            for (var_id, var_type) in redefined_vars {
                loop_scope.possibly_redefined_loop_parent_variables.insert(
                    var_id.clone(),
                    Rc::new(add_optional_union_type(
                        var_type,
                        loop_scope.possibly_redefined_loop_parent_variables.get(&var_id).map(|rc| rc.as_ref()),
                        context.codebase,
                    )),
                );
            }

            if loop_scope.iteration_count == 0 {
                for (var_id, var_type) in &block_context.locals {
                    if !loop_scope.parent_context_variables.contains_key(var_id) {
                        loop_scope.possibly_defined_loop_parent_variables.insert(
                            var_id.clone(),
                            add_optional_union_type(
                                var_type.as_ref().clone(),
                                loop_scope.possibly_defined_loop_parent_variables.get(var_id),
                                context.codebase,
                            ),
                        );
                    }
                }
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
        }

        if let Some(case_scope) = artifacts.case_scopes.last_mut() {
            if leaving_switch {
                let mut new_break_vars = case_scope.break_vars.clone().unwrap_or(HashMap::default());

                for (var_id, var_type) in &block_context.locals {
                    new_break_vars.insert(
                        var_id.clone(),
                        combine_optional_union_types(Some(var_type), new_break_vars.get(var_id), context.codebase),
                    );
                }

                case_scope.break_vars = Some(new_break_vars);
            }
        } else if !leaving_loop {
            // `break` outside of a loop or switch
            context.collector.report_with_code(
                IssueCode::InvalidBreak,
                Issue::error("Break statement outside of a loop or switch.").with_annotation(
                    Annotation::primary(self.span()).with_message("This break statement is not valid here."),
                ),
            );
        }

        block_context.has_returned = true;

        Ok(())
    }
}
