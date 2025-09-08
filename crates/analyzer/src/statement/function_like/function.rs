use mago_atom::atom;
use mago_codex::context::ScopeContext;
use mago_codex::get_function;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::heuristic;
use crate::statement::attributes::AttributeTarget;
use crate::statement::attributes::analyze_attributes;
use crate::statement::function_like::FunctionLikeBody;
use crate::statement::function_like::analyze_function_like;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for Function<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        analyze_attributes(
            context,
            block_context,
            artifacts,
            self.attribute_lists.as_slice(),
            AttributeTarget::Function,
        )?;

        let function_name = atom(context.resolved_names.get(&self.name));

        if context.settings.diff && context.codebase.safe_symbols.contains(&function_name) {
            return Ok(());
        }

        let Some(function_metadata) = get_function(context.codebase, &function_name) else {
            return Err(AnalysisError::InternalError(
                format!("Function metadata for `{function_name}` not found."),
                self.span(),
            ));
        };

        let mut scope = ScopeContext::new();
        scope.set_class_like(block_context.scope.get_class_like());
        scope.set_function_like(Some(function_metadata));

        analyze_function_like(
            context,
            artifacts,
            &mut BlockContext::new(scope),
            function_metadata,
            &self.parameter_list,
            FunctionLikeBody::Statements(self.body.statements.as_slice(), self.body.span()),
            None,
        )?;

        heuristic::check_function_like(
            function_metadata,
            self.parameter_list.parameters.as_slice(),
            FunctionLikeBody::Statements(self.body.statements.as_slice(), self.body.span()),
            context,
        );

        Ok(())
    }
}
