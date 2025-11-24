use mago_atom::atom;
use mago_atom::concat_atom;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::callable::TCallable;
use mago_codex::ttype::atomic::callable::TCallableSignature;
use mago_codex::ttype::atomic::callable::parameter::TCallableParameter;
use mago_codex::ttype::template::TemplateResult;
use mago_codex::ttype::template::inferred_type_replacer;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::invocation::InvocationTargetParameter;

pub mod function_partial_application;
pub mod method_partial_application;
pub mod static_method_partial_application;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for PartialApplication<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
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
    argument_name: mago_atom::Atom,
) -> Option<usize> {
    let argument_variable_name = concat_atom!("$", argument_name);
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
/// A TAtomic::Callable containing the new closure signature with only placeholder parameters
/// and all template types replaced with their inferred concrete types
fn create_closure_from_partial_application<'a>(
    callable_signature: TCallableSignature,
    argument_list: &PartialArgumentList<'_>,
    original_parameters: &[InvocationTargetParameter<'a>],
    template_result: &TemplateResult,
    codebase: &CodebaseMetadata,
) -> TAtomic {
    let parameters = callable_signature.get_parameters();
    let arguments = &argument_list.arguments;

    let mut parameter_offset = 0;
    let mut new_parameters = Vec::new();

    for argument in arguments.iter() {
        match argument {
            PartialArgument::Placeholder(_) => {
                if let Some(param) = parameters.get(parameter_offset) {
                    let mut new_param = param.clone();

                    if let Some(type_sig) = new_param.get_type_signature()
                        && (template_result.has_template_types() || !template_result.lower_bounds.is_empty())
                    {
                        let substituted_type = inferred_type_replacer::replace(type_sig, template_result, codebase);
                        new_param = TCallableParameter::new(
                            Some(Box::new(substituted_type)),
                            new_param.is_by_reference(),
                            new_param.is_variadic(),
                            new_param.has_default(),
                        );
                    }

                    new_parameters.push(new_param);
                    parameter_offset += 1;
                }
            }
            PartialArgument::NamedPlaceholder(named_placeholder) => {
                let param_name = atom(named_placeholder.name.value);
                let param_index = find_parameter_index_by_name(original_parameters, param_name);

                if let Some(index) = param_index
                    && let Some(param) = parameters.get(index)
                {
                    let mut new_param = param.clone();

                    if let Some(type_sig) = new_param.get_type_signature()
                        && (template_result.has_template_types() || !template_result.lower_bounds.is_empty())
                    {
                        let substituted_type = inferred_type_replacer::replace(type_sig, template_result, codebase);
                        new_param = TCallableParameter::new(
                            Some(Box::new(substituted_type)),
                            new_param.is_by_reference(),
                            new_param.is_variadic(),
                            new_param.has_default(),
                        );
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
                            last_param.get_type_signature().map(|t| Box::new(t.clone())),
                            false,
                            true,
                            false,
                        )
                    };

                    if let Some(type_sig) = new_param.get_type_signature()
                        && (template_result.has_template_types() || !template_result.lower_bounds.is_empty())
                    {
                        let substituted_type = inferred_type_replacer::replace(type_sig, template_result, codebase);
                        new_param = TCallableParameter::new(
                            Some(Box::new(substituted_type)),
                            new_param.is_by_reference(),
                            new_param.is_variadic(),
                            new_param.has_default(),
                        );
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

    let return_type = if let Some(ret_type) = &callable_signature.return_type {
        if template_result.has_template_types() || !template_result.lower_bounds.is_empty() {
            Some(Box::new(inferred_type_replacer::replace(ret_type, template_result, codebase)))
        } else {
            Some(ret_type.clone())
        }
    } else {
        None
    };

    let new_signature = TCallableSignature::new(callable_signature.is_pure(), true)
        .with_parameters(new_parameters)
        .with_return_type(return_type);

    TAtomic::Callable(TCallable::Signature(new_signature))
}
