use std::rc::Rc;

use mago_codex::ttype::get_mixed;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::Global;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::Code;
use crate::common::global::get_global_variable_type;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::context::block::ReferenceConstraint;
use crate::context::block::ReferenceConstraintSource;
use crate::error::AnalysisError;
use crate::utils::expression::get_variable_id;

impl Analyzable for Global {
    fn analyze(
        &self,
        context: &mut Context<'_>,
        block_context: &mut BlockContext,
        _artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        if block_context.is_global_scope() {
            context.collector.report_with_code(
                Code::INVALID_GLOBAL,
                Issue::error("The 'global' keyword has no effect in the global scope.")
                    .with_annotation(Annotation::primary(self.span()).with_message("This statement is redundant here."))
                    .with_note("The 'global' keyword is used *inside* functions or methods to import variables from the global scope into the local scope.")
                    .with_help("Consider removing this 'global' statement as it does not do anything in this context."),
            );
        }

        for variable in self.variables.iter() {
            if let Some(var_id) = get_variable_id(variable, context.interner) {
                block_context.locals.insert(var_id, Rc::new(get_mixed()));
            }
        }

        for variable in self.variables.iter() {
            let Some(var_id) = get_variable_id(variable, context.interner) else {
                continue;
            };

            let is_argc_or_argv = var_id == "$argc" || var_id == "$argv";
            let global_type = get_global_variable_type(&var_id).unwrap_or_else(get_mixed);

            block_context.locals.insert(var_id.clone(), Rc::new(global_type));

            if !is_argc_or_argv {
                block_context.variables_possibly_in_scope.insert(var_id.clone());
                block_context.by_reference_constraints.insert(
                    var_id.clone(),
                    ReferenceConstraint::new(variable.span(), ReferenceConstraintSource::Global, None),
                );
            }

            block_context.references_to_external_scope.insert(var_id.clone());

            if block_context.references_in_scope.contains_key(&var_id) {
                block_context.decrement_reference_count(&var_id);
                block_context.references_in_scope.remove(&var_id);
            }
        }

        Ok(())
    }
}
