use mago_atom::ascii_lowercase_atom;
use mago_atom::atom;
use mago_atom::concat_atom;
use mago_codex::context::ScopeContext;

use mago_codex::identifier::method::MethodIdentifier;

use mago_span::HasSpan;
use mago_syntax::ast::Method;
use mago_syntax::ast::MethodBody;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::statement::attributes::AttributeTarget;
use crate::statement::attributes::analyze_attributes;
use crate::statement::function_like::FunctionLikeBody;
use crate::statement::function_like::analyze_function_like;
use crate::statement::function_like::check_unused_function_template_parameters;
use crate::statement::function_like::unused_parameter;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for Method<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        analyze_attributes(context, block_context, artifacts, self.attribute_lists.as_slice(), AttributeTarget::Method);

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
            context.codebase.get_method_by_id(&MethodIdentifier::new(class_like_metadata.name, lowercase_method_name))
        else {
            tracing::error!(
                "Failed to find method metadata for `{}` in class `{}`.",
                self.name.value,
                class_like_metadata.original_name
            );

            return Ok(());
        };

        // Skip duplicate methods; semantics reports the error
        if method_metadata.span != self.span() {
            return Ok(());
        }

        let mut scope = ScopeContext::new();
        scope.set_class_like(Some(class_like_metadata));
        scope.set_function_like(Some(method_metadata));
        scope.set_static(self.is_static());

        let mut method_block_context = BlockContext::new(scope, context.settings.register_super_globals);

        method_block_context.flags.set_collect_initializations(true);

        analyze_function_like(
            context,
            artifacts,
            &mut method_block_context,
            method_metadata,
            &self.parameter_list,
            FunctionLikeBody::Statements(concrete_body.statements.as_slice(), concrete_body.span()),
            None,
        )?;

        let method_key = (class_like_metadata.name, lowercase_method_name);

        artifacts
            .method_initialized_properties
            .insert(method_key, method_block_context.definitely_initialized_properties.clone());

        artifacts.method_calls_this_methods.insert(method_key, method_block_context.definitely_called_methods.clone());

        if method_block_context.flags.calls_parent_constructor() {
            artifacts.method_calls_parent_constructor.insert(method_key, true);
        }

        if let Some(parent_initializer_name) = method_block_context.calls_parent_initializer {
            artifacts.method_calls_parent_initializer.insert(method_key, parent_initializer_name);
        }

        check_unused_function_template_parameters(
            context,
            method_metadata,
            self.name.span(),
            "method",
            concat_atom!(&class_like_metadata.original_name, "::", &method_name),
        );

        if context.settings.find_unused_parameters
            && !context.codebase.method_is_overriding(&class_like_metadata.name, &method_name)
        {
            unused_parameter::check_unused_params(
                method_metadata,
                self.parameter_list.parameters.as_slice(),
                FunctionLikeBody::Statements(concrete_body.statements.as_slice(), concrete_body.span()),
                context,
            );
        }

        // Check for missing type hints
        for parameter in &self.parameter_list.parameters {
            crate::utils::missing_type_hints::check_parameter_type_hint(
                context,
                Some(class_like_metadata),
                method_metadata,
                parameter,
            );
        }

        crate::utils::missing_type_hints::check_return_type_hint(
            context,
            Some(class_like_metadata),
            method_metadata,
            self.name.value,
            self.return_type_hint.as_ref(),
            self.span(),
        );

        Ok(())
    }
}
