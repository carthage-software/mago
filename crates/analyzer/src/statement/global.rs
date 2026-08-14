use mago_allocator::Arena;
use std::rc::Rc;

use mago_codex::ttype::get_mixed;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::cst::Global;
use mago_word::Word;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::common::global::get_global_variable_type;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::context::block::ReferenceConstraint;
use crate::context::block::ReferenceConstraintSource;
use crate::error::AnalysisError;
use crate::utils::docblock::get_type_from_var_docblock;
use crate::utils::expression::get_variable_id;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for Global<'arena> {
    fn analyze<'ctx, A>(
        &'ast self,
        context: &mut Context<'ctx, 'arena, A>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError>
    where
        A: Arena,
    {
        if block_context.is_global_scope() {
            context.collector.report_with_code(
                IssueCode::InvalidGlobal,
                Issue::error("The 'global' keyword has no effect in the global scope.")
                    .with_annotation(Annotation::primary(self.span()).with_message("This statement is redundant here."))
                    .with_note("The 'global' keyword is used *inside* functions or methods to import variables from the global scope into the local scope.")
                    .with_help("Consider removing this 'global' statement as it does not do anything in this context."),
            );
        }

        for variable in &self.variables {
            if let Some(var_id) = get_variable_id(variable) {
                block_context.locals.insert(Word::from(var_id), Rc::new(get_mixed()));
            }
        }

        for variable in &self.variables {
            let Some(var_id) = get_variable_id(variable) else {
                continue;
            };

            let var_id_atom = mago_word::word(var_id);
            let is_argc_or_argv = var_id == b"$argc" || var_id == b"$argv";
            let known_type = get_global_variable_type(var_id);
            let docblock_type =
                get_type_from_var_docblock(context, block_context, artifacts, Some(var_id), self.variables.len() == 1)
                    .map(|(docblock_type, docblock_type_span)| (Rc::new(docblock_type), docblock_type_span));

            // Incompatibilities between the docblock and a known superglobal type are already
            // reported by the generic `@var` pass, which sees them in scope before this point.
            let variable_type = match (&docblock_type, known_type) {
                (Some((docblock_type, _)), _) => Rc::clone(docblock_type),
                (None, Some(known_type)) => known_type,
                (None, None) => Rc::new(get_mixed()),
            };

            block_context.locals.insert(var_id_atom, variable_type);

            if !is_argc_or_argv {
                block_context.variables_possibly_in_scope.insert(var_id_atom);
                block_context.by_reference_constraints.insert(
                    var_id_atom,
                    match &docblock_type {
                        Some((docblock_type, docblock_type_span)) => ReferenceConstraint::new(
                            *docblock_type_span,
                            ReferenceConstraintSource::Global,
                            Some(Rc::clone(docblock_type)),
                        ),
                        None => ReferenceConstraint::new(variable.span(), ReferenceConstraintSource::Global, None),
                    },
                );
            }

            block_context.references_to_external_scope.insert(var_id_atom);

            if block_context.references_in_scope.contains_key(&var_id_atom) {
                block_context.decrement_reference_count(var_id_atom.as_bytes());
                block_context.references_in_scope.remove(&var_id_atom);
            }
        }

        Ok(())
    }
}
