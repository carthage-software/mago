use std::borrow::Cow;

use ahash::HashMap;
use itertools::Itertools;

use mago_atom::Atom;
use mago_atom::AtomMap;
use mago_atom::concat_atom;
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
use mago_syntax::ast::*;

use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::invocation::Invocation;
use crate::invocation::InvocationArgument;
use crate::invocation::InvocationArgumentsSource;
use crate::invocation::InvocationTarget;
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

/// Finds the parameter that corresponds to a given argument.
fn get_parameter_of_argument<'invocation, 'ast, 'arena>(
    parameters: &[InvocationTargetParameter<'invocation>],
    argument: &InvocationArgument<'ast, 'arena>,
    mut argument_offset: usize,
) -> Option<(usize, InvocationTargetParameter<'invocation>)> {
    if let Some(named_argument) = argument.get_named_argument() {
        argument_offset = find_named_parameter_offset(parameters, named_argument.name.value.into())?;
    }

    argument_offset = adjust_offset_for_variadic(parameters, argument_offset);
    parameters.get(argument_offset).copied().map(|parameter| (argument_offset, parameter))
}

/// Finds the offset of a named parameter in the parameter list.
fn find_named_parameter_offset(
    parameters: &[InvocationTargetParameter<'_>],
    argument_name: mago_atom::Atom,
) -> Option<usize> {
    let argument_variable_name = concat_atom!("$", argument_name);
    parameters
        .iter()
        .position(|parameter| parameter.get_name().is_some_and(|param_name| argument_variable_name == param_name.0))
}

/// Adjusts argument offset for variadic parameters.
fn adjust_offset_for_variadic(parameters: &[InvocationTargetParameter<'_>], argument_offset: usize) -> usize {
    if argument_offset >= parameters.len() && parameters.last().is_some_and(|last_param| last_param.is_variadic()) {
        parameters.len() - 1
    } else {
        argument_offset
    }
}

/// Analyzes and verifies arguments passed to a function, method, or callable.
pub fn analyze_invocation<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    invocation: &Invocation<'ctx, 'ast, 'arena>,
    calling_class_like: Option<(Atom, Option<&TAtomic>)>,
    template_result: &mut TemplateResult,
    parameter_types: &mut AtomMap<TUnion>,
) -> Result<(), AnalysisError> {
    populate_template_result_from_invocation(context, invocation, template_result);

    let parameter_refs = invocation.target.get_parameters();
    let arguments = invocation.arguments_source.get_arguments();
    let arg_count = arguments.len();
    let mut analyzed_argument_types = HashMap::default();

    let mut non_closure_arguments = Vec::with_capacity(arg_count);
    let mut closure_arguments = Vec::with_capacity(arg_count);
    let mut unpacked_arguments = Vec::new();

    for (offset, argument) in arguments.into_iter().enumerate() {
        if argument.is_unpacked() {
            unpacked_arguments.push(argument);
        } else if matches!(argument.value(), Expression::Closure(_) | Expression::ArrowFunction(_)) {
            closure_arguments.push((offset, argument));
        } else {
            non_closure_arguments.push((offset, argument));
        }
    }

    let calling_class_like_metadata = calling_class_like.and_then(|(id, _)| context.codebase.get_class_like(&id));
    let method_call_context = invocation.target.get_method_context();
    let base_class_metadata = method_call_context.map(|ctx| ctx.class_like_metadata).or(calling_class_like_metadata);
    let calling_instance_type = calling_class_like.and_then(|(_, atomic)| atomic);

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
                calling_instance_type,
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
                calling_instance_type,
            );

            if base_parameter_type.has_template_types() {
                parameter_type_had_template_types = true;
                let mut replaced_type =
                    inferred_type_replacer::replace(&base_parameter_type, template_result, context.codebase);
                if replaced_type.is_expandable() {
                    expander::expand_union(
                        context.codebase,
                        &mut replaced_type,
                        &TypeExpansionOptions {
                            self_class: base_class_metadata.map(|meta| meta.name),
                            ..Default::default()
                        },
                    );
                }

                Some(replaced_type)
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
    let all_arguments =
        non_closure_arguments.iter().chain(closure_arguments.iter()).sorted_by(|(a, _), (b, _)| a.cmp(b));

    for (argument_offset, argument) in all_arguments {
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
                } else if let Some(previous_span) = assigned_parameters_by_position.get(&parameter_offset) {
                    if !parameter_ref.is_variadic() {
                        context.collector.report_with_code(
                            IssueCode::NamedArgumentOverridesPositional,
                            Issue::error(format!(
                                "Named argument `${}` for {} `{}` targets a parameter already provided positionally.",
                                named_argument.name.value, target_kind_str, target_name_str
                            ))
                            .with_annotation(
                                Annotation::primary(named_argument.name.span()).with_message("This named argument"),
                            )
                            .with_annotation(
                                Annotation::secondary(*previous_span)
                                    .with_message("Parameter already filled by positional argument here"),
                            )
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
                    assigned_parameters_by_name.insert(named_argument.name.value, named_argument.name.span());
                } else {
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
                calling_instance_type,
            );

            let final_parameter_type =
                if template_result.has_template_types() || !template_result.lower_bounds.is_empty() {
                    let mut final_parameter_type =
                        inferred_type_replacer::replace(&base_parameter_type, template_result, context.codebase);

                    expander::expand_union(
                        context.codebase,
                        &mut final_parameter_type,
                        &TypeExpansionOptions {
                            self_class: base_class_metadata.map(|meta| meta.name),
                            ..Default::default()
                        },
                    );

                    final_parameter_type
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

            // For variadic functions, allow extra named arguments
            let has_variadic_parameter = parameter_refs.last().is_some_and(|p| p.is_variadic());

            if !has_variadic_parameter {
                context.collector.report_with_code(
                    IssueCode::InvalidNamedArgument,
                    Issue::error(format!(
                        "Invalid named argument `${argument_name}` for {target_kind_str} `{target_name_str}`"
                    ))
                    .with_annotation(
                        Annotation::primary(named_argument.name.span())
                            .with_message(format!("Unknown argument name `${argument_name}`")),
                    )
                    .with_annotation(
                        Annotation::secondary(invocation.target.span())
                            .with_message(format!("Call to {target_kind_str} is here")),
                    )
                    .with_help({
                        if !invocation.target.allows_named_arguments() {
                            format!("The {target_kind_str} `{target_name_str}` does not support named arguments.")
                        } else if parameter_refs.is_empty() {
                            format!("The {target_kind_str} `{target_name_str}` has no parameters.")
                        } else {
                            let available_params: Vec<_> = parameter_refs
                                .iter()
                                .filter_map(|p| p.get_name())
                                .map(|n| n.0.trim_start_matches('$'))
                                .collect();
                            format!("Available parameters are: `{}`.", available_params.join("`, `"))
                        }
                    }),
                );
            }

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
                calling_instance_type,
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

    for (template_name, constraints) in invocation.target.get_template_types().into_iter().flatten() {
        for (generic_parent, type_constraint) in constraints {
            if template_result.has_lower_bound(template_name, generic_parent) {
                continue;
            }

            template_result.add_lower_bound(*template_name, *generic_parent, type_constraint.clone());
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
                    calling_instance_type,
                );

                let final_variadic_parameter_type =
                    if template_result.has_template_types() || !template_result.lower_bounds.is_empty() {
                        let mut replaced_type = inferred_type_replacer::replace(
                            &base_variadic_parameter_type,
                            template_result,
                            context.codebase,
                        );

                        if replaced_type.is_expandable() {
                            expander::expand_union(
                                context.codebase,
                                &mut replaced_type,
                                &TypeExpansionOptions {
                                    self_class: base_class_metadata.map(|meta| meta.name),
                                    ..Default::default()
                                },
                            );
                        }

                        replaced_type
                    } else {
                        base_variadic_parameter_type
                    };

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

                    let (sizes, has_keyed_array_with_named_args) = argument_value_type.types.as_ref().iter().fold(
                        (Vec::new(), false),
                        |(mut sizes, mut has_named_args), argument_atomic| {
                            let TAtomic::Array(array) = argument_atomic else {
                                sizes.push(0);
                                return (sizes, has_named_args);
                            };

                            sizes.push(array.get_minimum_size());

                            // Check if this is a keyed array with named arguments
                            if !has_named_args
                                && let mago_codex::ttype::atomic::array::TArray::Keyed(keyed_array) = array
                                && let Some(known_items) = &keyed_array.known_items
                            {
                                has_named_args = known_items.iter().any(|(array_key, _)| {
                                    matches!(array_key, mago_codex::ttype::atomic::array::key::ArrayKey::String(_))
                                });
                            }

                            (sizes, has_named_args)
                        },
                    );

                    number_of_provided_parameters += sizes.into_iter().min().unwrap_or(0);

                    // Skip union-based validation for keyed arrays with named arguments.
                    // Individual elements are properly validated in validate_keyed_array_elements.
                    if !has_keyed_array_with_named_args {
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
                }
            } else {
                let all_arguments = invocation.arguments_source.get_arguments();
                let mut current_parameter_position = 0;

                for argument in all_arguments.into_iter() {
                    if argument.is_unpacked() {
                        let argument_expression = argument.value();
                        if artifacts.get_expression_type(argument_expression).is_none() {
                            analyze_and_store_argument_type(
                                context,
                                block_context,
                                artifacts,
                                &invocation.target,
                                argument_expression,
                                usize::MAX,
                                &mut analyzed_argument_types,
                                false,
                                None,
                            )?;
                        }

                        let argument_value_type =
                            artifacts.get_expression_type(argument_expression).cloned().unwrap_or_else(get_mixed);

                        // Count the number of elements that would be unpacked
                        let mut sizes = vec![];
                        for argument_atomic in argument_value_type.types.as_ref() {
                            let TAtomic::Array(array) = argument_atomic else {
                                sizes.push(0);
                                continue;
                            };
                            sizes.push(array.get_minimum_size());
                        }

                        let unpacked_count = sizes.into_iter().min().unwrap_or(0);
                        number_of_provided_parameters += unpacked_count;

                        validate_unpacked_argument_elements(
                            context,
                            &argument_value_type,
                            argument_expression,
                            &parameter_refs,
                            base_class_metadata,
                            calling_class_like_metadata,
                            calling_instance_type,
                            &invocation.target,
                            template_result,
                            current_parameter_position,
                            target_kind_str,
                            &target_name_str,
                        );

                        current_parameter_position += unpacked_count;
                    } else {
                        current_parameter_position += 1;
                    }
                }
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

        issue = match invocation.arguments_source {
            InvocationArgumentsSource::PipeInput(pipe) => issue
                .with_annotation(
                    Annotation::secondary(pipe.callable.span())
                        .with_message(format!("This {target_kind_str} expects 0 arguments")),
                )
                .with_annotation(
                    Annotation::secondary(pipe.operator).with_message("Pipe operator provides this as an argument"),
                ),
            _ => issue.with_annotation(
                Annotation::secondary(invocation.target.span())
                    .with_message(format!("For this {target_kind_str} call")),
            ),
        };

        issue = issue
            .with_note(format!("Expected {max_params} argument(s), but received {number_of_provided_parameters}."))
            .with_help("Remove the extra argument(s).");

        context.collector.report_with_code(IssueCode::TooManyArguments, issue);
    }

    check_template_result(context, template_result, invocation.span);

    Ok(())
}

/// Gets the effective parameter type, expanding class-relative types based on call context.
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

    let parameter_type = invocation_target_parameter.get_type().cloned().unwrap_or_else(get_mixed);

    let mut resolved_parameter_type = parameter_type;

    expander::expand_union(
        context.codebase,
        &mut resolved_parameter_type,
        &TypeExpansionOptions {
            self_class: base_class_metadata.map(|meta| meta.name),
            static_class_type: calling_class_like_metadata.map_or(StaticClassType::None, |calling_meta| {
                calling_instance_type
                    .and_then(|instance| match instance {
                        TAtomic::Object(obj) => Some(StaticClassType::Object(obj.clone())),
                        _ => None,
                    })
                    .unwrap_or(StaticClassType::Name(calling_meta.name))
            }),
            parent_class: base_class_metadata.and_then(|meta| meta.direct_parent_class),
            function_is_final: calling_class_like_metadata.is_some_and(|meta| meta.flags.is_final()),
            ..Default::default()
        },
    );

    resolved_parameter_type
}

/// Validates individual elements within unpacked arrays against their corresponding parameters.
fn validate_unpacked_argument_elements<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    argument_value_type: &TUnion,
    argument_expression: &Expression<'arena>,
    parameter_refs: &[InvocationTargetParameter<'_>],
    base_class_metadata: Option<&'ctx ClassLikeMetadata>,
    calling_class_like_metadata: Option<&'ctx ClassLikeMetadata>,
    calling_instance_type: Option<&TAtomic>,
    invocation_target: &InvocationTarget<'_>,
    template_result: &TemplateResult,
    starting_parameter_position: usize,
    target_kind_str: &str,
    target_name_str: &str,
) {
    use mago_codex::ttype::atomic::array::TArray;
    use mago_codex::ttype::template::inferred_type_replacer;

    for argument_atomic in argument_value_type.types.as_ref() {
        let TAtomic::Array(array) = argument_atomic else {
            continue;
        };

        match array {
            TArray::List(list) => {
                if let Some(known_elements) = &list.known_elements {
                    for (array_index, (_, element_type)) in known_elements.iter() {
                        let parameter_position = starting_parameter_position + array_index;
                        if parameter_position >= parameter_refs.len() {
                            break;
                        }

                        if let Some(parameter_ref) = parameter_refs.get(parameter_position) {
                            let base_parameter_type = get_parameter_type(
                                context,
                                Some(*parameter_ref),
                                base_class_metadata,
                                calling_class_like_metadata,
                                calling_instance_type,
                            );

                            let final_parameter_type = if template_result.has_template_types() {
                                let mut replaced_type = inferred_type_replacer::replace(
                                    &base_parameter_type,
                                    template_result,
                                    context.codebase,
                                );

                                if replaced_type.is_expandable() {
                                    expander::expand_union(
                                        context.codebase,
                                        &mut replaced_type,
                                        &TypeExpansionOptions {
                                            self_class: base_class_metadata.map(|meta| meta.name),
                                            ..Default::default()
                                        },
                                    );
                                }

                                replaced_type
                            } else {
                                base_parameter_type
                            };

                            verify_argument_type(
                                context,
                                element_type,
                                &final_parameter_type,
                                parameter_position,
                                argument_expression,
                                invocation_target,
                            );
                        }
                    }
                } else {
                    let element_type = list.get_element_type();
                    let min_size = list.known_count.unwrap_or(0);
                    let max_parameters_to_check =
                        std::cmp::min(min_size, parameter_refs.len().saturating_sub(starting_parameter_position));

                    for i in 0..max_parameters_to_check {
                        let parameter_position = starting_parameter_position + i;
                        if let Some(parameter_ref) = parameter_refs.get(parameter_position) {
                            let base_parameter_type = get_parameter_type(
                                context,
                                Some(*parameter_ref),
                                base_class_metadata,
                                calling_class_like_metadata,
                                calling_instance_type,
                            );

                            let final_parameter_type = if template_result.has_template_types() {
                                let mut replaced_type = inferred_type_replacer::replace(
                                    &base_parameter_type,
                                    template_result,
                                    context.codebase,
                                );

                                if replaced_type.is_expandable() {
                                    expander::expand_union(
                                        context.codebase,
                                        &mut replaced_type,
                                        &TypeExpansionOptions {
                                            self_class: base_class_metadata.map(|meta| meta.name),
                                            ..Default::default()
                                        },
                                    );
                                }

                                replaced_type
                            } else {
                                base_parameter_type
                            };

                            verify_argument_type(
                                context,
                                element_type,
                                &final_parameter_type,
                                parameter_position,
                                argument_expression,
                                invocation_target,
                            );
                        }
                    }
                }
            }
            TArray::Keyed(keyed_array) => {
                validate_keyed_array_elements(
                    context,
                    keyed_array,
                    argument_expression,
                    parameter_refs,
                    base_class_metadata,
                    calling_class_like_metadata,
                    calling_instance_type,
                    invocation_target,
                    template_result,
                    target_kind_str,
                    target_name_str,
                );
            }
        }
    }
}

fn validate_keyed_array_elements<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    keyed_array: &mago_codex::ttype::atomic::array::keyed::TKeyedArray,
    argument_expression: &Expression<'arena>,
    parameter_refs: &[InvocationTargetParameter<'_>],
    base_class_metadata: Option<&'ctx ClassLikeMetadata>,
    calling_class_like_metadata: Option<&'ctx ClassLikeMetadata>,
    calling_instance_type: Option<&TAtomic>,
    invocation_target: &InvocationTarget<'_>,
    template_result: &TemplateResult,
    target_kind_str: &str,
    target_name_str: &str,
) {
    use mago_atom::concat_atom;
    use mago_codex::ttype::atomic::array::key::ArrayKey;
    use mago_codex::ttype::template::inferred_type_replacer;

    let Some(known_items) = &keyed_array.known_items else {
        return;
    };

    for (array_key, (_, element_type)) in known_items.iter() {
        let parameter_name = match array_key {
            ArrayKey::String(key_str) => concat_atom!("$", key_str),
            ArrayKey::Integer(key_int) => {
                if let Some(parameter_ref) = parameter_refs.get(*key_int as usize) {
                    if let Some(param_name) = parameter_ref.get_name() {
                        param_name.0
                    } else {
                        continue;
                    }
                } else {
                    continue;
                }
            }
        };

        if let Some(parameter_ref) =
            parameter_refs.iter().find(|p| p.get_name().is_some_and(|param_name| param_name.0 == parameter_name))
        {
            let base_parameter_type = get_parameter_type(
                context,
                Some(*parameter_ref),
                base_class_metadata,
                calling_class_like_metadata,
                calling_instance_type,
            );

            let final_parameter_type =
                if template_result.has_template_types() || !template_result.lower_bounds.is_empty() {
                    let mut replaced_type =
                        inferred_type_replacer::replace(&base_parameter_type, template_result, context.codebase);

                    if replaced_type.is_expandable() {
                        expander::expand_union(
                            context.codebase,
                            &mut replaced_type,
                            &TypeExpansionOptions {
                                self_class: base_class_metadata.map(|meta| meta.name),
                                ..Default::default()
                            },
                        );
                    }

                    replaced_type
                } else {
                    base_parameter_type
                };

            let parameter_position = parameter_refs
                .iter()
                .position(|p| if let Some(param_name) = p.get_name() { param_name.0 == parameter_name } else { false })
                .unwrap_or(0);

            verify_argument_type(
                context,
                element_type,
                &final_parameter_type,
                parameter_position,
                argument_expression,
                invocation_target,
            );
        } else if let ArrayKey::String(key_str) = array_key {
            let argument_name = key_str.as_str();

            // For variadic functions, allow extra named arguments
            let has_variadic_parameter = parameter_refs.last().is_some_and(|p| p.is_variadic());

            if !has_variadic_parameter {
                context.collector.report_with_code(
                    IssueCode::InvalidNamedArgument,
                    Issue::error(format!(
                        "Invalid named argument `${argument_name}` for {target_kind_str} `{target_name_str}`"
                    ))
                    .with_annotation(
                        Annotation::primary(argument_expression.span())
                            .with_message(format!("Unknown argument name `${argument_name}` in unpacked array")),
                    )
                    .with_annotation(
                        Annotation::secondary(invocation_target.span())
                            .with_message(format!("Call to {target_kind_str} is here")),
                    )
                    .with_help(if !invocation_target.allows_named_arguments() {
                        format!("The {target_kind_str} `{target_name_str}` does not support named arguments.")
                    } else if !parameter_refs.is_empty() {
                        let available_params: Vec<String> = parameter_refs
                            .iter()
                            .filter_map(|p| {
                                p.get_name().map(|name| format!("${}", name.0.as_str().trim_start_matches('$')))
                            })
                            .collect();

                        if available_params.is_empty() {
                            format!("The {target_kind_str} `{target_name_str}` does not accept any named arguments.")
                        } else {
                            format!("Available named arguments are: {}.", available_params.join(", "))
                        }
                    } else {
                        format!("The {target_kind_str} `{target_name_str}` does not accept any arguments.")
                    }),
                );
            }
        }
    }
}
