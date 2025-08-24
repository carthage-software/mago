use std::borrow::Cow;

use ahash::HashMap;
use itertools::Itertools;

use mago_atom::Atom;
use mago_atom::AtomMap;
use mago_atom::concat_atom;
use mago_codex::get_class_like;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::expander;
use mago_codex::ttype::expander::StaticClassType;
use mago_codex::ttype::expander::TypeExpansionOptions;
use mago_codex::ttype::template::TemplateResult;
use mago_codex::ttype::template::inferred_type_replacer;
use mago_codex::ttype::union::TUnion;
use mago_codex::ttype::*;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::invocation::Invocation;
use crate::invocation::InvocationArgument;
use crate::invocation::InvocationArgumentsSource;
use crate::invocation::InvocationTargetParameter;
use crate::invocation::arguments::analyze_and_store_argument_type;
use crate::invocation::arguments::get_unpacked_argument_type;
use crate::invocation::arguments::verify_argument_type;
use crate::invocation::template_inference::infer_parameter_templates_from_argument;
use crate::invocation::template_inference::infer_parameter_templates_from_default;
use crate::invocation::template_result::check_template_result;
use crate::invocation::template_result::get_class_template_parameters_from_result;
use crate::invocation::template_result::populate_template_result_from_invocation;
use crate::invocation::template_result::refine_template_result_for_function_like;

/// Analyzes and verifies arguments passed to a function, method, or callable.
///
/// # Arguments
///
/// * `context` - Analysis context.
/// * `block_context` - Context for the current code block.
/// * `artifacts` - Function analysis data store.
/// * `invocation` - The invocation being analyzed.
/// * `calling_class_like` - Optional info about the class context if called via `parent::` etc.
/// * `template_result` - Stores inferred template types; assumed empty initially, populated during analysis.
pub fn analyze_invocation<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    invocation: &Invocation<'ctx, 'ast, 'arena>,
    calling_class_like: Option<(Atom, Option<&TAtomic>)>,
    template_result: &mut TemplateResult,
    parameter_types: &mut AtomMap<TUnion>,
) -> Result<(), AnalysisError> {
    fn get_parameter_of_argument<'invocation, 'ast, 'arena>(
        parameters: &[InvocationTargetParameter<'invocation>],
        argument: &InvocationArgument<'ast, 'arena>,
        mut argument_offset: usize,
    ) -> Option<(usize, InvocationTargetParameter<'invocation>)> {
        if let Some(named_argument) = argument.get_named_argument() {
            let argument_variable_name = concat_atom!("$", named_argument.name.value);

            let named_offset = parameters.iter().position(|parameter| {
                let Some(parameter_name) = parameter.get_name() else {
                    return false;
                };

                argument_variable_name == parameter_name.0
            })?;

            argument_offset = named_offset;
        }

        if argument_offset >= parameters.len()
            && let Some(last_parameter) = parameters.last()
            && last_parameter.is_variadic()
        {
            argument_offset = parameters.len() - 1;
        }

        parameters.get(argument_offset).copied().map(|parameter| (argument_offset, parameter))
    }

    populate_template_result_from_invocation(context, invocation, template_result);

    let parameter_refs = invocation.target.get_parameters();
    let mut analyzed_argument_types: HashMap<usize, (TUnion, Span)> = HashMap::default();

    let mut non_closure_arguments: Vec<(usize, InvocationArgument<'ast, 'arena>)> = Vec::new();
    let mut closure_arguments: Vec<(usize, InvocationArgument<'ast, 'arena>)> = Vec::new();
    let mut unpacked_arguments: Vec<InvocationArgument<'ast, 'arena>> = Vec::new();
    for (offset, argument) in invocation.arguments_source.get_arguments().into_iter().enumerate() {
        if argument.is_unpacked() {
            unpacked_arguments.push(argument);
        } else if matches!(argument.value(), Expression::Closure(_) | Expression::ArrowFunction(_)) {
            closure_arguments.push((offset, argument));
        } else {
            non_closure_arguments.push((offset, argument));
        }
    }

    let calling_class_like_metadata = calling_class_like.and_then(|(id, _)| get_class_like(context.codebase, &id));
    let base_class_metadata =
        invocation.target.get_method_context().map(|ctx| ctx.class_like_metadata).or(calling_class_like_metadata);
    let method_call_context = invocation.target.get_method_context();

    for (argument_offset, argument) in &non_closure_arguments {
        let argument_expression = argument.value();
        let parameter = get_parameter_of_argument(&parameter_refs, argument, *argument_offset);

        analyze_and_store_argument_type(
            context,
            block_context,
            artifacts,
            &invocation.target,
            argument_expression,
            *argument_offset,
            &mut analyzed_argument_types,
            parameter.is_some_and(|p| p.1.is_by_reference()),
            None,
        )?;

        if let Some(argument_type) = analyzed_argument_types.get(argument_offset)
            && let Some((_, parameter_ref)) = parameter
        {
            let parameter_type = get_parameter_type(
                context,
                Some(parameter_ref),
                base_class_metadata,
                calling_class_like_metadata,
                calling_class_like.and_then(|(_, atomic)| atomic),
            );

            if parameter_type.has_template_types() {
                infer_parameter_templates_from_argument(
                    context,
                    &parameter_type,
                    &argument_type.0,
                    template_result,
                    *argument_offset,
                    argument_type.1,
                    false,
                );
            }
        }
    }

    for (argument_offset, argument) in &closure_arguments {
        let argument_expression = argument.value();
        let parameter = get_parameter_of_argument(&parameter_refs, argument, *argument_offset);
        let mut parameter_type_had_template_types = false;
        let parameter_type = if let Some((_, parameter_ref)) = parameter {
            let base_parameter_type = get_parameter_type(
                context,
                Some(parameter_ref),
                base_class_metadata,
                calling_class_like_metadata,
                calling_class_like.and_then(|(_, atomic)| atomic),
            );

            if base_parameter_type.has_template_types() {
                parameter_type_had_template_types = true;

                Some(inferred_type_replacer::replace(&base_parameter_type, template_result, context.codebase))
            } else {
                Some(base_parameter_type)
            }
        } else {
            None
        };

        analyze_and_store_argument_type(
            context,
            block_context,
            artifacts,
            &invocation.target,
            argument_expression,
            *argument_offset,
            &mut analyzed_argument_types,
            parameter.is_some_and(|p| p.1.is_by_reference()),
            parameter_type.as_ref(),
        )?;

        if parameter_type_had_template_types
            && let Some(argument_type) = analyzed_argument_types.get(argument_offset)
            && let Some(parameter_type) = parameter_type
        {
            infer_parameter_templates_from_argument(
                context,
                &parameter_type,
                &argument_type.0,
                template_result,
                *argument_offset,
                argument_type.1,
                true,
            );
        }
    }

    if let Some(function_like_metadata) = invocation.target.get_function_like_metadata() {
        let class_generic_parameters = get_class_template_parameters_from_result(template_result, context);
        refine_template_result_for_function_like(
            template_result,
            context,
            method_call_context,
            base_class_metadata,
            calling_class_like_metadata,
            function_like_metadata,
            &class_generic_parameters,
        );
    }

    let mut assigned_parameters_by_name = HashMap::default();
    let mut assigned_parameters_by_position = HashMap::default();

    let target_kind_str = invocation.target.guess_kind();
    let target_name_str = invocation.target.guess_name();
    let mut has_too_many_arguments = false;
    let mut last_argument_offset: isize = -1;
    for (argument_offset, argument) in
        non_closure_arguments.iter().chain(closure_arguments.iter()).sorted_by(|(a, _), (b, _)| a.cmp(b))
    {
        let argument_expression = argument.value();
        let (argument_value_type, _) = analyzed_argument_types
            .get(argument_offset)
            .cloned()
            .unwrap_or_else(|| (get_mixed(), argument_expression.span()));

        let parameter_ref = get_parameter_of_argument(&parameter_refs, argument, *argument_offset);
        if let Some((parameter_offset, parameter_ref)) = parameter_ref {
            if let Some(parameter_name) = parameter_ref.get_name() {
                parameter_types.insert(parameter_name.0, argument_value_type.clone());
            }

            if let Some(named_argument) = argument.get_named_argument() {
                if let Some(previous_span) = assigned_parameters_by_name.get(&named_argument.name.value) {
                    context.collector.report_with_code(
                        IssueCode::DuplicateNamedArgument,
                        Issue::error(format!(
                            "Duplicate named argument `${}` in call to {} `{}`.",
                            named_argument.name.value, target_kind_str, target_name_str
                        ))
                        .with_annotation(
                            Annotation::primary(named_argument.name.span()).with_message("Duplicate argument name"),
                        )
                        .with_annotation(
                            Annotation::secondary(*previous_span)
                                .with_message("Argument previously provided by name here"),
                        )
                        .with_help("Remove one of the duplicate named arguments."),
                    );
                } else {
                    if let Some(previous_span) = assigned_parameters_by_position.get(&parameter_offset) {
                        if !parameter_ref.is_variadic() {
                            context.collector.report_with_code(
                                IssueCode::NamedArgumentOverridesPositional,
                                Issue::error(format!(
                                    "Named argument `${}` for {} `{}` targets a parameter already provided positionally.",
                                    named_argument.name.value, target_kind_str, target_name_str
                                ))
                                .with_annotation(Annotation::primary(named_argument.name.span()).with_message("This named argument"))
                                .with_annotation(Annotation::secondary(*previous_span).with_message("Parameter already filled by positional argument here"))
                                .with_help("Provide the argument either positionally or by name, but not both."),
                            );
                        } else {
                            context.collector.report_with_code(
                                IssueCode::NamedArgumentAfterPositional,
                                 Issue::warning(format!(
                                    "Named argument `${}` for {} `{}` targets a variadic parameter that has already captured positional arguments.",
                                    named_argument.name.value, target_kind_str, target_name_str
                                ))
                                .with_annotation(Annotation::primary(named_argument.name.span()).with_message("Named argument for variadic parameter"))
                                .with_annotation(Annotation::secondary(*previous_span).with_message("Positional arguments already captured by variadic here"))
                                .with_note("Mixing positional and named arguments for the same variadic parameter can be confusing and may lead to unexpected behavior depending on PHP version and argument unpacking.")
                                .with_help("Consider providing all arguments for the variadic parameter either positionally or via unpacking a named array."),
                            );
                        }
                    }

                    assigned_parameters_by_name.insert(named_argument.name.value, named_argument.name.span());
                }
            } else {
                assigned_parameters_by_position.insert(parameter_offset, argument.span());
            }

            let base_parameter_type = get_parameter_type(
                context,
                Some(parameter_ref),
                base_class_metadata,
                calling_class_like_metadata,
                calling_class_like.and_then(|(_, atomic)| atomic),
            );

            let final_parameter_type = if template_result.has_template_types() {
                inferred_type_replacer::replace(&base_parameter_type, template_result, context.codebase)
            } else {
                base_parameter_type
            };

            verify_argument_type(
                context,
                &argument_value_type,
                &final_parameter_type,
                *argument_offset,
                argument_expression,
                &invocation.target,
            );
        } else if let Some(named_argument) = argument.get_named_argument() {
            let argument_name = named_argument.name.value;

            context.collector.report_with_code(
                IssueCode::InvalidNamedArgument,
                Issue::error(format!(
                    "Invalid named argument `${argument_name}` for {target_kind_str} `{target_name_str}`"
                ))
                .with_annotation(
                    Annotation::primary(named_argument.name.span())
                        .with_message("Unknown argument name `${argument_name}`"),
                )
                .with_annotation(
                    Annotation::secondary(invocation.target.span())
                        .with_message(format!("Call to {target_kind_str} is here")),
                )
                .with_help(if !invocation.target.allows_named_arguments() {
                    format!("The {target_kind_str} `{target_name_str}` does not support named arguments.")
                } else if !parameter_refs.is_empty() {
                    format!(
                        "Available parameters are: `{}`.",
                        parameter_refs
                            .iter()
                            .filter_map(|p| p.get_name())
                            .map(|n| n.0.trim_start_matches('$'))
                            .collect::<Vec<_>>()
                            .join("`, `")
                    )
                } else {
                    format!("The {target_kind_str} `{target_name_str}` has no parameters.")
                }),
            );

            break;
        } else if *argument_offset >= parameter_refs.len() {
            has_too_many_arguments = true;
            continue;
        }

        last_argument_offset = *argument_offset as isize;
    }

    if !has_too_many_arguments {
        loop {
            last_argument_offset += 1;
            if last_argument_offset as usize >= parameter_refs.len() {
                break;
            }

            let Some(unused_parameter) = parameter_refs.get(last_argument_offset as usize).copied() else {
                break;
            };

            if !unused_parameter.has_default() {
                continue;
            }

            let parameter_type = get_parameter_type(
                context,
                Some(unused_parameter),
                base_class_metadata,
                calling_class_like_metadata,
                calling_class_like.and_then(|(_, atomic)| atomic),
            );

            let default_type =
                unused_parameter.get_default_type().map(Cow::Borrowed).unwrap_or_else(|| Cow::Owned(get_mixed()));

            infer_parameter_templates_from_default(context, &parameter_type, &default_type, template_result);

            let Some(parameter_name) = unused_parameter.get_name() else {
                continue;
            };

            if parameter_types.contains_key(&parameter_name.0) {
                continue;
            }

            parameter_types.insert(parameter_name.0, default_type.into_owned());
        }
    }

    let max_params = parameter_refs.len();
    let number_of_required_parameters = parameter_refs.iter().filter(|p| !p.has_default() && !p.is_variadic()).count();
    let mut number_of_provided_parameters = non_closure_arguments.len() + closure_arguments.len();

    if !unpacked_arguments.is_empty() {
        if let Some(last_parameter_ref) = parameter_refs.last().copied() {
            if last_parameter_ref.is_variadic() {
                let base_variadic_parameter_type = get_parameter_type(
                    context,
                    Some(last_parameter_ref),
                    base_class_metadata,
                    calling_class_like_metadata,
                    calling_class_like.and_then(|(_, atomic)| atomic),
                );

                let final_variadic_parameter_type =
                    inferred_type_replacer::replace(&base_variadic_parameter_type, template_result, context.codebase);

                for unpacked_argument in unpacked_arguments {
                    let argument_expression = unpacked_argument.value();
                    if artifacts.get_expression_type(argument_expression).is_none() {
                        analyze_and_store_argument_type(
                            context,
                            block_context,
                            artifacts,
                            &invocation.target,
                            argument_expression,
                            usize::MAX,
                            &mut analyzed_argument_types,
                            last_parameter_ref.is_by_reference(),
                            None,
                        )?;
                    }

                    let argument_value_type =
                        artifacts.get_expression_type(argument_expression).cloned().unwrap_or_else(get_mixed); // Get type of the iterable

                    let mut sizes = vec![];
                    for argument_atomic in argument_value_type.types.as_ref() {
                        let TAtomic::Array(array) = argument_atomic else {
                            sizes.push(0);

                            continue;
                        };

                        sizes.push(array.get_minimum_size());
                    }

                    number_of_provided_parameters += sizes.into_iter().min().unwrap_or(0);

                    let unpacked_element_type =
                        get_unpacked_argument_type(context, &argument_value_type, argument_expression.span());

                    verify_argument_type(
                        context,
                        &unpacked_element_type,
                        &final_variadic_parameter_type,
                        parameter_refs.len() - 1,
                        argument_expression,
                        &invocation.target,
                    );
                }
            } else {
                context.collector.report_with_code(
                    IssueCode::TooManyArguments,
                    Issue::error(format!(
                        "Cannot unpack arguments into non-variadic {} `{}`.",
                        invocation.target.guess_kind(),
                        invocation.target.guess_name(),
                    ))
                    .with_annotation(
                        Annotation::primary(unpacked_arguments[0].span())
                            .with_message("Argument unpacking requires a variadic parameter"),
                    )
                    .with_note(format!("Function expects exactly {} arguments.", parameter_refs.len()))
                    .with_help("Remove the argument unpacking (`...`) or make the last parameter variadic."),
                );
            }
        } else if !unpacked_arguments.is_empty() {
            context.collector.report_with_code(
                IssueCode::TooManyArguments,
                Issue::error(format!(
                    "Cannot unpack arguments into {} `{}` which expects no arguments.",
                    invocation.target.guess_kind(),
                    invocation.target.guess_name()
                ))
                .with_annotation(
                    Annotation::primary(unpacked_arguments[0].span()).with_message("Unexpected argument unpacking"),
                )
                .with_help("Remove the argument unpacking (`...`)."),
            );
        }
    }

    if number_of_provided_parameters < number_of_required_parameters {
        let primary_annotation_span = invocation.arguments_source.span();

        let main_message = match invocation.arguments_source {
            InvocationArgumentsSource::PipeInput(_) => format!(
                "Too few arguments for {target_kind_str} `{target_name_str}` when used with the pipe operator `|>`. Pipe provides 1, but at least {number_of_required_parameters} required."
            ),
            _ => format!("Too few arguments provided for {target_kind_str} `{target_name_str}`."),
        };

        let mut issue = Issue::error(main_message)
            .with_annotation(Annotation::primary(primary_annotation_span).with_message("More arguments expected here"))
            .with_note(format!(
                "Expected at least {number_of_required_parameters} argument(s) for non-optional parameters, but received {number_of_provided_parameters}.",
            ));

        issue = match invocation.arguments_source {
            InvocationArgumentsSource::ArgumentList(_) => issue.with_annotation(
                Annotation::secondary(invocation.target.span())
                    .with_message(format!("For this {target_kind_str} call")),
            ),
            InvocationArgumentsSource::PipeInput(pipe) => issue
                .with_annotation(Annotation::secondary(pipe.callable.span()).with_message(format!(
                    "This {target_kind_str} requires at least {number_of_required_parameters} argument(s)",
                )))
                .with_annotation(
                    Annotation::secondary(pipe.input.span()).with_message("This value is passed as the first argument"),
                ),
            InvocationArgumentsSource::None(constructor_or_attribute_span) => issue.with_annotation(
                Annotation::secondary(constructor_or_attribute_span)
                    .with_message(format!("For this {target_kind_str}")),
            ),
        };

        issue = issue.with_help("Provide all required arguments.");
        context.collector.report_with_code(IssueCode::TooFewArguments, issue);
    } else if has_too_many_arguments
        || (!parameter_refs.last().is_some_and(|p| p.is_variadic())
            && number_of_provided_parameters > max_params
            && max_params > 0)
    {
        let first_extra_arg_span = invocation
            .arguments_source
            .get_arguments()
            .get(max_params)
            .map(|arg| arg.span())
            .unwrap_or_else(|| invocation.arguments_source.span());

        let main_message = match invocation.arguments_source {
            InvocationArgumentsSource::PipeInput(_) => format!(
                "The {target_kind_str} `{target_name_str}` used with pipe operator `|>` expects 0 arguments, but 1 (the piped value) is provided."
            ),
            _ => format!("Too many arguments provided for {target_kind_str} `{target_name_str}`."),
        };

        let mut issue = Issue::error(main_message).with_annotation(
            Annotation::primary(first_extra_arg_span).with_message("Unexpected argument provided here"),
        );

        if let InvocationArgumentsSource::PipeInput(pipe) = invocation.arguments_source {
            issue = issue
                .with_annotation(
                    Annotation::secondary(pipe.callable.span())
                        .with_message(format!("This {target_kind_str} expects 0 arguments")),
                )
                .with_annotation(
                    Annotation::secondary(pipe.operator).with_message("Pipe operator provides this as an argument"),
                );
        } else {
            issue = issue.with_annotation(
                Annotation::secondary(invocation.target.span())
                    .with_message(format!("For this {target_kind_str} call")),
            );
        }

        issue = issue
            .with_note(format!("Expected {max_params} argument(s), but received {number_of_provided_parameters}."))
            .with_help("Remove the extra argument(s).");

        context.collector.report_with_code(IssueCode::TooManyArguments, issue);
    }

    check_template_result(context, template_result, invocation.span);

    Ok(())
}

/// Gets the effective parameter type from a potential parameter reference,
/// expanding `self`, `static`, and `parent` type hints based on the call context.
///
/// If no specific parameter type is found (e.g., missing parameter reference or
/// no type hint on the parameter), it defaults to `mixed|any`.
///
/// # Arguments
///
/// * `context` - The analysis context, providing codebase metadata.
/// * `parameter_ref` - An optional reference to the parameter's definition (either `FunctionLike` or `Callable`).
/// * `base_class_metadata` - Optional metadata for the class where the method is *defined*. Used for resolving `self::` and `parent::`.
/// * `calling_class_like_metadata` - Optional metadata for the class context from which the call is made. Used for resolving `static::`.
/// * `calling_instance_type` - Optional specific atomic type of the calling instance (`$this`). Used for resolving `static::` more precisely when available.
///
/// # Returns
///
/// A `TUnion` representing the resolved type of the parameter in the context of the call.
fn get_parameter_type<'ctx, 'arena>(
    context: &Context<'ctx, 'arena>,
    invocation_target_parameter: Option<InvocationTargetParameter<'_>>,
    base_class_metadata: Option<&'ctx ClassLikeMetadata>,
    calling_class_like_metadata: Option<&'ctx ClassLikeMetadata>,
    calling_instance_type: Option<&TAtomic>,
) -> TUnion {
    let Some(invocation_target_parameter) = invocation_target_parameter else {
        return get_mixed();
    };

    let Some(parameter_type) = invocation_target_parameter.get_type() else {
        return get_mixed();
    };

    let mut resolved_parameter_type = parameter_type.clone();

    expander::expand_union(
        context.codebase,
        &mut resolved_parameter_type,
        &TypeExpansionOptions {
            self_class: base_class_metadata.map(|meta| meta.name),
            static_class_type: match calling_class_like_metadata {
                Some(calling_meta) => {
                    if let Some(TAtomic::Object(instance_type)) = calling_instance_type {
                        StaticClassType::Object(instance_type.clone())
                    } else {
                        StaticClassType::Name(calling_meta.name)
                    }
                }
                None => StaticClassType::None,
            },
            parent_class: base_class_metadata.and_then(|meta| meta.direct_parent_class),
            function_is_final: calling_class_like_metadata.is_some_and(|meta| meta.flags.is_final()),
            ..Default::default()
        },
    );

    resolved_parameter_type
}
