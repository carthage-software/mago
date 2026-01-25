use mago_atom::AtomMap;
use mago_atom::ascii_lowercase_atom;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::callable::TCallable;
use mago_codex::ttype::expander::get_signature_of_function_like_identifier;
use mago_codex::ttype::get_mixed_closure;
use mago_codex::ttype::get_never;
use mago_codex::ttype::template::TemplateResult;
use mago_codex::ttype::union::TUnion;
use mago_span::HasSpan;
use mago_syntax::ast::StaticMethodPartialApplication;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::expression::partial_application::create_closure_from_partial_application;
use crate::invocation::Invocation;
use crate::invocation::InvocationArgumentsSource;
use crate::invocation::InvocationTarget;
use crate::invocation::InvocationTargetParameter;
use crate::invocation::analyzer::analyze_invocation;
use crate::resolver::static_method::resolve_static_method_targets;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for StaticMethodPartialApplication<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let method_resolution =
            resolve_static_method_targets(context, block_context, artifacts, self.class, &self.method, self.span())?;

        let mut identifiers = vec![];
        for resolved_method in &method_resolution.resolved_methods {
            let class_name = ascii_lowercase_atom(resolved_method.classname.as_ref());
            let method_name = *resolved_method.method_identifier.get_method_name();
            artifacts.symbol_references.add_reference_to_class_member(
                &block_context.scope,
                (class_name, method_name),
                false,
            );

            identifiers.push(FunctionLikeIdentifier::Method(
                resolved_method.classname,
                *resolved_method.method_identifier.get_method_name(),
            ));
        }

        let resulting_type = if self.argument_list.is_first_class_callable() {
            if identifiers.is_empty() {
                if method_resolution.has_invalid_target { get_never() } else { get_mixed_closure() }
            } else {
                TUnion::from_vec(
                    identifiers.into_iter().map(|identifier| TAtomic::Callable(TCallable::Alias(identifier))).collect(),
                )
            }
        } else {
            let mut closure_types = Vec::new();
            for resolved_method in method_resolution.resolved_methods {
                let identifier = FunctionLikeIdentifier::Method(
                    resolved_method.classname,
                    *resolved_method.method_identifier.get_method_name(),
                );

                let Some(signature) = get_signature_of_function_like_identifier(&identifier, context.codebase) else {
                    continue;
                };

                let Some(metadata) = context.codebase.get_function_like(&identifier) else {
                    continue;
                };

                let original_parameters: Vec<_> =
                    metadata.parameters.iter().map(InvocationTargetParameter::FunctionLike).collect();

                let invocation_target = InvocationTarget::FunctionLike {
                    identifier,
                    metadata,
                    inferred_return_type: None,
                    method_context: None,
                    span: self.method.span(),
                };

                let invocation = Invocation::new(
                    invocation_target,
                    InvocationArgumentsSource::PartialArgumentList(&self.argument_list),
                    self.span(),
                );

                let mut template_result = TemplateResult::default();
                let mut parameter_types = AtomMap::default();

                analyze_invocation(
                    context,
                    block_context,
                    artifacts,
                    &invocation,
                    Some((resolved_method.classname, None)),
                    &mut template_result,
                    &mut parameter_types,
                )?;

                closure_types.push(create_closure_from_partial_application(
                    &signature,
                    &self.argument_list,
                    &original_parameters,
                    &template_result,
                    context.codebase,
                ));
            }

            if closure_types.is_empty() {
                if method_resolution.has_invalid_target { get_never() } else { get_mixed_closure() }
            } else {
                TUnion::from_vec(closure_types)
            }
        };

        artifacts.set_expression_type(self, resulting_type);

        Ok(())
    }
}
