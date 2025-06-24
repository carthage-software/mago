use ahash::HashMap;

use mago_codex::context::ScopeContext;
use mago_codex::get_method_by_id;
use mago_codex::identifier::method::MethodIdentifier;
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

impl Analyzable for Method {
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
            AttributeTarget::Method,
        )?;

        let MethodBody::Concrete(concrete_body) = &self.body else { return Ok(()) };

        let Some(class_like_metadata) = block_context.scope.get_class_like() else {
            return Err(AnalysisError::InternalError(
                "Method analysis requires class-like context.".to_string(),
                self.span(),
            ));
        };

        let method_name = context.interner.lowered(&self.name.value);
        if context.settings.diff
            && context.codebase.safe_symbol_members.contains(&(class_like_metadata.name, method_name))
        {
            return Ok(());
        }

        let Some(method_metadata) = get_method_by_id(
            context.codebase,
            context.interner,
            &MethodIdentifier::new(class_like_metadata.name, method_name),
        ) else {
            return Err(AnalysisError::InternalError(
                format!("Method metadata for `{method_name}` not found."),
                self.span(),
            ));
        };

        let mut scope = ScopeContext::new();
        scope.set_class_like(Some(class_like_metadata));
        scope.set_function_like(Some(method_metadata));
        scope.set_static(self.is_static());

        analyze_function_like(
            context,
            artifacts,
            scope,
            method_metadata,
            &self.parameter_list,
            FunctionLikeBody::Statements(concrete_body.statements.as_slice()),
            HashMap::default(),
        )?;

        Ok(())
    }
}
