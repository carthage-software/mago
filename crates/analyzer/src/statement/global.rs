use std::rc::Rc;

use mago_codex::ttype::get_mixed;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::Global;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::issue::TypingIssueKind;
use crate::utils::expression::get_variable_id;

impl Analyzable for Global {
    fn analyze(
        &self,
        context: &mut Context<'_>,
        block_context: &mut BlockContext,
        _artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        if block_context.is_global_scope() {
            context.buffer.report(
                TypingIssueKind::InvalidGlobal,
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

        Ok(())
    }
}
