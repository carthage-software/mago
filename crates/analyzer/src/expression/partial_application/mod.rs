use mago_allocator::Arena;
use std::sync::Arc;

use mago_codex::misc::VariableIdentifier;
use mago_codex::ttype::TType;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::callable::TCallable;
use mago_codex::ttype::atomic::callable::TCallableConstraint;
use mago_codex::ttype::atomic::callable::TCallableSignature;
use mago_codex::ttype::atomic::callable::parameter::TCallableParameter;
use mago_codex::ttype::template::TemplateResult;
use mago_codex::ttype::union::TUnion;
use mago_syntax::cst::PartialApplication;
use mago_syntax::cst::PartialArgument;
use mago_syntax::cst::PartialArgumentList;
use mago_word::WordMap;
use mago_word::concat_word;
use mago_word::word;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::invocation::Invocation;
use crate::invocation::InvocationTargetParameter;
use crate::invocation::resolve_invocation_type;

pub mod function_partial_application;
pub mod method_partial_application;
pub mod static_method_partial_application;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for PartialApplication<'arena> {
    fn analyze<'ctx, A>(
        &'ast self,
        context: &mut Context<'ctx, 'arena, A>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError>
    where
        A: Arena,
    {
        match self {
            PartialApplication::Function(function_partial_application) => {
                function_partial_application.analyze(context, block_context, artifacts)
            }
            PartialApplication::Method(method_partial_application) => {
                method_partial_application.analyze(context, block_context, artifacts)
            }
            PartialApplication::StaticMethod(static_method_partial_application) => {
                static_method_partial_application.analyze(context, block_context, artifacts)
            }
        }
    }
}

/// Finds the index of a parameter by name in the parameter list.
///
/// This is used for named placeholders in partial function application to resolve
/// the actual parameter position when named arguments can reorder parameters.
fn find_parameter_index_by_name(
    parameters: &[InvocationTargetParameter<'_>],
    argument_name: mago_word::Word,
) -> Option<usize> {
    let argument_variable_name = concat_word!("$", argument_name);
    parameters
        .iter()
        .position(|parameter| parameter.get_name().is_some_and(|param_name| argument_variable_name == param_name.0))
}

/// Creates a closure type for partial function application by filtering parameters
/// based on placeholders in the argument list and applying template substitutions.
///
/// This function examines the partial argument list and the callable's signature to build
/// a new closure signature that only includes parameters corresponding to placeholders.
/// It also applies any template type substitutions from the template result.
///
/// For named placeholders (e.g., `foo(i: ?, s: ?)`), this function correctly resolves
/// parameter names to their actual positions in the signature, ensuring proper type
/// mapping when named arguments reorder parameters.
///
/// # Parameters
///
/// - `callable_signature`: The resolved signature of the function being partially applied
/// - `argument_list`: The partial argument list containing placeholders and bound arguments
/// - `original_parameters`: The original parameters with names from the function metadata
/// - `template_result`: Template inference results containing concrete types for template parameters
/// - `codebase`: The codebase for looking up type information during substitution
///
/// # Returns
///
/// A `TAtomic::Callable` containing the new closure signature with only placeholder parameters
/// and all template types replaced with their inferred concrete types
fn create_closure_from_partial_application<'ctx, 'arena, A>(
    context: &Context<'ctx, 'arena, A>,
    invocation: &Invocation<'ctx, '_, 'arena>,
    callable_signature: &TCallableSignature,
    argument_list: &PartialArgumentList<'_>,
    original_parameters: &[InvocationTargetParameter<'_>],
    template_result: &TemplateResult,
    parameter_types: &WordMap<TUnion>,
) -> TAtomic
where
    A: Arena,
{
    let parameters = callable_signature.get_parameters();
    let arguments = &argument_list.arguments;

    let mut parameter_offset = 0;
    let mut new_parameters = Vec::new();

    for argument in arguments {
        match argument {
            PartialArgument::Placeholder(_) => {
                if let Some(param) = parameters.get(parameter_offset) {
                    let mut new_param = param.clone();

                    if let Some(type_signature) = new_param.get_type_signature() {
                        let resolved_type = resolve_invocation_type(
                            context,
                            invocation,
                            template_result,
                            parameter_types,
                            type_signature.clone(),
                        );
                        new_param = new_param.with_type_signature(Some(Arc::new(resolved_type)));
                    }

                    new_parameters.push(new_param);
                    parameter_offset += 1;
                }
            }
            PartialArgument::NamedPlaceholder(named_placeholder) => {
                let param_name = word(named_placeholder.name.value);
                let param_index = find_parameter_index_by_name(original_parameters, param_name);

                if let Some(index) = param_index
                    && let Some(param) = parameters.get(index)
                {
                    let mut new_param = param.clone();

                    if let Some(type_signature) = new_param.get_type_signature() {
                        let resolved_type = resolve_invocation_type(
                            context,
                            invocation,
                            template_result,
                            parameter_types,
                            type_signature.clone(),
                        );
                        new_param = new_param.with_type_signature(Some(Arc::new(resolved_type)));
                    }

                    new_parameters.push(new_param);
                }

                parameter_offset += 1;
            }
            PartialArgument::VariadicPlaceholder(_) => {
                if let Some(last_param) = parameters.get(parameter_offset) {
                    let mut new_param = if last_param.is_variadic() {
                        last_param.clone()
                    } else {
                        TCallableParameter::new(
                            last_param.get_type_signature().map(|t| Arc::new(t.clone())),
                            false,
                            true,
                            false,
                        )
                        .with_name(last_param.get_name().copied())
                    };

                    if let Some(type_signature) = new_param.get_type_signature() {
                        let resolved_type = resolve_invocation_type(
                            context,
                            invocation,
                            template_result,
                            parameter_types,
                            type_signature.clone(),
                        );
                        new_param = new_param.with_type_signature(Some(Arc::new(resolved_type)));
                    }

                    new_parameters.push(new_param);
                }

                break;
            }
            PartialArgument::Positional(_) | PartialArgument::Named(_) => {
                parameter_offset += 1;
            }
        }
    }

    let return_type = callable_signature.return_type.as_ref().map(|return_type| {
        Arc::new(resolve_invocation_type(
            context,
            invocation,
            template_result,
            parameter_types,
            (**return_type).clone(),
        ))
    });

    let placeholder_names: Vec<_> =
        new_parameters.iter().filter_map(|parameter| parameter.get_name().map(|name| name.0)).collect();
    let mut constraints = Vec::new();

    for (index, original_parameter) in original_parameters.iter().enumerate() {
        let Some(parameter_name) = original_parameter.get_name() else {
            continue;
        };
        if placeholder_names.contains(&parameter_name.0) {
            continue;
        }

        let Some(input_type) = parameter_types.get(&parameter_name.0) else {
            continue;
        };
        let Some(parameter_type) = parameters.get(index).and_then(TCallableParameter::get_type_signature) else {
            continue;
        };

        let parameter_type =
            resolve_invocation_type(context, invocation, template_result, parameter_types, parameter_type.clone());
        let mut dependent_names: Vec<VariableIdentifier> = Vec::new();
        for node in parameter_type.get_all_child_nodes() {
            let mago_codex::ttype::TypeRef::Atomic(TAtomic::Variable(variable)) = node else {
                continue;
            };
            let variable = VariableIdentifier(*variable);
            if placeholder_names.contains(&variable.0) && !dependent_names.contains(&variable) {
                dependent_names.push(variable);
            }
        }

        if !dependent_names.is_empty() {
            constraints.push(TCallableConstraint::new(
                dependent_names,
                Arc::new(input_type.clone()),
                Arc::new(parameter_type),
            ));
        }
    }

    let new_signature = TCallableSignature::new(callable_signature.is_pure(), true)
        .with_parameters(new_parameters)
        .with_return_type(return_type)
        .with_constraints(constraints);

    TAtomic::Callable(TCallable::Signature(new_signature))
}
