use mago_atom::ascii_lowercase_atom;
use mago_atom::atom;
use mago_codex::context::ScopeContext;
use mago_codex::get_method_by_id;
use mago_codex::identifier::method::MethodIdentifier;
use mago_codex::is_method_overriding;
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

impl<'ast, 'arena> Analyzable<'ast, 'arena> for Method<'arena> {
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
            AttributeTarget::Method,
        )?;

        let MethodBody::Concrete(concrete_body) = &self.body else { return Ok(()) };

        let Some(class_like_metadata) = block_context.scope.get_class_like() else {
            tracing::error!("Attempted to analyze method `{}` without class-like context.", self.name.value);

            return Ok(());
        };

        let method_name = atom(self.name.value);
        let lowercase_method_name = ascii_lowercase_atom(self.name.value);
        if context.settings.diff
            && context.codebase.safe_symbol_members.contains(&(class_like_metadata.name, lowercase_method_name))
        {
            return Ok(());
        }

        let Some(method_metadata) =
            get_method_by_id(context.codebase, &MethodIdentifier::new(class_like_metadata.name, lowercase_method_name))
        else {
            tracing::error!(
                "Failed to find method metadata for `{}` in class `{}`.",
                self.name.value,
                class_like_metadata.original_name
            );

            return Ok(());
        };

        let mut scope = ScopeContext::new();
        scope.set_class_like(Some(class_like_metadata));
        scope.set_function_like(Some(method_metadata));
        scope.set_static(self.is_static());

        analyze_function_like(
            context,
            artifacts,
            &mut BlockContext::new(scope),
            method_metadata,
            &self.parameter_list,
            FunctionLikeBody::Statements(concrete_body.statements.as_slice(), concrete_body.span()),
            None,
        )?;

        if !is_method_overriding(context.codebase, &class_like_metadata.name, &method_name) {
            heuristic::check_function_like(
                method_metadata,
                self.parameter_list.parameters.as_slice(),
                FunctionLikeBody::Statements(concrete_body.statements.as_slice(), concrete_body.span()),
                context,
            );
        }

        Ok(())
    }
}
