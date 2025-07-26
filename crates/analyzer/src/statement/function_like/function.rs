use ahash::HashMap;

use mago_codex::context::ScopeContext;
use mago_codex::get_function;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::statement::attributes::AttributeTarget;
use crate::statement::attributes::analyze_attributes;
use crate::statement::function_like::FunctionLikeBody;
use crate::statement::function_like::analyze_function_like;

impl Analyzable for Function {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        analyze_attributes(
            context,
            block_context,
            artifacts,
            self.attribute_lists.as_slice(),
            AttributeTarget::Function,
        )?;

        let function_name = context.resolved_names.get(&self.name);

        if context.settings.diff && context.codebase.safe_symbols.contains(function_name) {
            return Ok(());
        }

        let Some(function_metadata) = get_function(context.codebase, context.interner, function_name) else {
            return Err(AnalysisError::InternalError(
                format!("Function metadata for `{}` not found.", context.interner.lookup(function_name)),
                self.span(),
            ));
        };

        let mut scope = ScopeContext::new();
        scope.set_class_like(block_context.scope.get_class_like());
        scope.set_function_like(Some(function_metadata));

        analyze_function_like(
            context,
            artifacts,
            scope,
            function_metadata,
            &self.parameter_list,
            FunctionLikeBody::Statements(self.body.statements.as_slice()),
            HashMap::default(),
        )?;

        Ok(())
    }
}
