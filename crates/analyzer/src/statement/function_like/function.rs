use mago_atom::atom;
use mago_codex::context::ScopeContext;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::Function;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::plugin::context::HookContext;
use crate::statement::attributes::AttributeTarget;
use crate::statement::attributes::analyze_attributes;
use crate::statement::function_like::FunctionLikeBody;
use crate::statement::function_like::analyze_function_like;
use crate::statement::function_like::check_unused_function_template_parameters;
use crate::statement::function_like::unused_parameter;

/// Reports a duplicate function definition issue.
fn report_duplicate_function_definition(
    context: &mut Context<'_, '_>,
    name: &str,
    duplicate_span: Span,
    original_span: Span,
) {
    context.collector.report_with_code(
        IssueCode::DuplicateDefinition,
        Issue::error(format!("Function `{name}` is already defined elsewhere."))
            .with_annotation(Annotation::primary(duplicate_span).with_message("Duplicate function definition here"))
            .with_annotation(Annotation::secondary(original_span).with_message("Original function defined here"))
            .with_note("Each function must have a unique name within the same namespace.")
            .with_note("The duplicate definition will be ignored during analysis.")
            .with_help(
                "Consider using namespaces to avoid naming conflicts, or remove one of the duplicate definitions.",
            ),
    );
}

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
        );

        let function_name = atom(context.resolved_names.get(&self.name));

        if context.settings.diff && context.codebase.safe_symbols.contains(&function_name) {
            return Ok(());
        }

        let Some(function_metadata) = context.codebase.get_function(&function_name) else {
            return Err(AnalysisError::InternalError(
                format!("Function metadata for `{function_name}` not found."),
                self.span(),
            ));
        };

        if function_metadata.span != self.span() {
            report_duplicate_function_definition(context, function_name.as_ref(), self.span(), function_metadata.span);

            return Ok(());
        }

        // Call plugin on_enter_function hooks
        if context.plugin_registry.has_function_decl_hooks() {
            let mut hook_context = HookContext::new(context.codebase, block_context, artifacts);
            context.plugin_registry.on_enter_function(self, function_metadata, &mut hook_context)?;
            for reported in hook_context.take_issues() {
                context.collector.report_with_code(reported.code, reported.issue);
            }
        }

        let mut scope = ScopeContext::new();
        scope.set_class_like(block_context.scope.get_class_like());
        scope.set_function_like(Some(function_metadata));

        analyze_function_like(
            context,
            artifacts,
            &mut BlockContext::new(scope, context.settings.register_super_globals),
            function_metadata,
            &self.parameter_list,
            FunctionLikeBody::Statements(self.body.statements.as_slice(), self.body.span()),
            None,
        )?;

        // Call plugin on_leave_function hooks
        if context.plugin_registry.has_function_decl_hooks() {
            let mut hook_context = HookContext::new(context.codebase, block_context, artifacts);
            context.plugin_registry.on_leave_function(self, function_metadata, &mut hook_context)?;
            for reported in hook_context.take_issues() {
                context.collector.report_with_code(reported.code, reported.issue);
            }
        }

        check_unused_function_template_parameters(
            context,
            function_metadata,
            self.name.span(),
            "function",
            function_name,
        );

        if context.settings.find_unused_parameters {
            unused_parameter::check_unused_params(
                function_metadata,
                self.parameter_list.parameters.as_slice(),
                FunctionLikeBody::Statements(self.body.statements.as_slice(), self.body.span()),
                context,
            );
        }

        // Check for missing type hints
        for parameter in &self.parameter_list.parameters {
            crate::utils::missing_type_hints::check_parameter_type_hint(
                context,
                None, // Functions don't have a class context
                function_metadata,
                parameter,
            );
        }

        crate::utils::missing_type_hints::check_return_type_hint(
            context,
            None, // Functions don't have a class context
            function_metadata,
            self.name.value,
            self.return_type_hint.as_ref(),
            self.span(),
        );

        Ok(())
    }
}
