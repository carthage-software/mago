use std::borrow::Cow;

use foldhash::HashMap;

use mago_atom::Atom;
use mago_atom::AtomMap;
use mago_atom::concat_atom;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::ttype::TType;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::callable::TCallable;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::class_like_string::TClassLikeString;
use mago_codex::ttype::atomic::scalar::string::TString;
use mago_codex::ttype::atomic::scalar::string::TStringLiteral;
use mago_codex::ttype::expander;
use mago_codex::ttype::expander::StaticClassType;
use mago_codex::ttype::expander::TypeExpansionOptions;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::template::TemplateResult;
use mago_codex::ttype::template::inferred_type_replacer;
use mago_codex::ttype::union::TUnion;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::Expression;

use crate::artifacts::AnalysisArtifacts;
use crate::artifacts::ClosureBindScope;
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
fn get_parameter_of_argument<'target, 'ctx>(
    target: &'target InvocationTarget<'ctx>,
    argument: &InvocationArgument<'_, '_>,
    mut argument_offset: usize,
) -> Option<(usize, InvocationTargetParameter<'target>)>
where
    'ctx: 'target,
{
    // Handle both named arguments and named placeholders
    if let Some(parameter_name) = argument.get_parameter_name() {
        argument_offset = find_named_parameter_offset(target, parameter_name.into())?;
    }

    argument_offset = adjust_offset_for_variadic(target, argument_offset);
    target.get_parameter(argument_offset).map(|parameter| (argument_offset, parameter))
}

/// Finds the offset of a named parameter in the parameter list.
fn find_named_parameter_offset<'target, 'ctx>(
    target: &'target InvocationTarget<'ctx>,
    argument_name: mago_atom::Atom,
) -> Option<usize>
where
    'ctx: 'target,
{
    let argument_variable_name = concat_atom!("$", argument_name);
    target
        .iter_parameters()
        .position(|parameter| parameter.get_name().is_some_and(|param_name| argument_variable_name == param_name.0))
}

/// Adjusts argument offset for variadic parameters.
fn adjust_offset_for_variadic(target: &InvocationTarget<'_>, argument_offset: usize) -> usize {
    let parameter_count = target.parameter_count();
    if parameter_count > 0
        && argument_offset >= parameter_count
        && target.get_parameter(parameter_count - 1).is_some_and(|parameter| parameter.is_variadic())
    {
        parameter_count - 1
    } else {
        argument_offset
    }
}

/// Analyzes and verifies arguments passed to a function, method, or callable.
pub fn analyze_invocation<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    invocation: &Invocation<'ctx, '_, 'arena>,
    calling_class_like: Option<(Atom, Option<&TAtomic>)>,
    template_result: &mut TemplateResult,
    parameter_types: &mut AtomMap<TUnion>,
) -> Result<(), AnalysisError> {
    if !context.settings.allow_side_effects_in_conditions
        && block_context.flags.inside_conditional()
        && !invocation.target.is_pure_or_mutation_free()
    {
        context.collector.report_with_code(
            IssueCode::SideEffectsInCondition,
            Issue::warning("Impure function call inside condition.")
                .with_annotation(
                    Annotation::primary(invocation.span)
                        .with_message("This call is not marked `@pure` or `@mutation-free` and may have side effects"),
                )
                .with_note("Side effects inside conditions can silently alter variables used elsewhere in the same expression.")
                .with_note("PHP's evaluation order may not match reading order, leading to surprising behavior.")
                .with_help(
                    "Extract the call into a variable before the condition, or annotate the function with `@pure` or `@mutation-free` if it has no side effects.",
                ),
        );
    }

    populate_template_result_from_invocation(context, invocation, template_result);

    let arg_count = invocation.arguments_source.argument_count();
    let mut analyzed_argument_types = HashMap::default();

    let mut non_closure_arguments = Vec::with_capacity(arg_count);
    let mut closure_arguments = Vec::with_capacity(arg_count);
    let mut unpacked_arguments = Vec::new();

    for (offset, argument) in invocation.arguments_source.iter_arguments().enumerate() {
        if argument.is_unpacked() {
            unpacked_arguments.push(argument);
        } else if argument.is_placeholder() {
            non_closure_arguments.push((offset, argument));
        } else if matches!(argument.value(), Some(Expression::Closure(_) | Expression::ArrowFunction(_))) {
            closure_arguments.push((offset, argument));
        } else {
            non_closure_arguments.push((offset, argument));
        }
    }

    let calling_class_like_metadata = calling_class_like.and_then(|(id, _)| context.codebase.get_class_like(&id));
    let method_call_context = invocation.target.get_method_context();
    let base_class_metadata = method_call_context.map(|ctx| ctx.class_like_metadata).or(calling_class_like_metadata);
    let calling_instance_type = calling_class_like.and_then(|(_, atomic)| atomic);
    let method_class_type = method_call_context.map(|ctx| &ctx.class_type);

    for (argument_offset, argument) in &non_closure_arguments {
        let Some(argument_expression) = argument.value() else {
            continue;
        };

        let parameter = get_parameter_of_argument(&invocation.target, argument, *argument_offset);

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
                method_class_type,
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

    let closure_bind_scope = detect_closure_bind_scope(context, invocation, &analyzed_argument_types);
    let previous_bind_scope = artifacts.closure_bind_scope.take();
    artifacts.closure_bind_scope = closure_bind_scope;

    for (argument_offset, argument) in &closure_arguments {
        let Some(argument_expression) = argument.value() else {
            continue;
        };

        let parameter = get_parameter_of_argument(&invocation.target, argument, *argument_offset);
        let mut parameter_type_had_template_types = false;
        // Store the original unreplaced type for template inference, so closure return types
        // can contribute bounds even after other arguments have already inferred some bounds.
        // See: https://github.com/carthage-software/mago/issues/755
        let mut base_parameter_type_for_inference: Option<TUnion> = None;
        let parameter_type = if let Some((_, parameter_ref)) = parameter {
            let base_parameter_type = get_parameter_type(
                context,
                Some(parameter_ref),
                base_class_metadata,
                calling_class_like_metadata,
                calling_instance_type,
                method_class_type,
            );

            if base_parameter_type.has_template_types() {
                parameter_type_had_template_types = true;
                // Store the original type before replacement for inference
                base_parameter_type_for_inference = Some(base_parameter_type.clone());
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

        let parameter_type = if let Some(mut pt) = parameter_type {
            filter_array_filter_callback_type(&invocation.target, &mut pt, &analyzed_argument_types);
            Some(pt)
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

        // Use the original unreplaced type for inference to allow closure return types
        // to contribute bounds that can widen literal types from other arguments.
        if parameter_type_had_template_types
            && let Some(argument_type) = analyzed_argument_types.get(argument_offset)
            && let Some(base_type) = &base_parameter_type_for_inference
        {
            infer_parameter_templates_from_argument(
                context,
                base_type,
                &argument_type.0,
                template_result,
                *argument_offset,
                argument_type.1,
                true,
            );
        }
    }

    // Restore the previous closure bind scope after analyzing closure arguments
    artifacts.closure_bind_scope = previous_bind_scope;

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
    let target_name_str = invocation.target.guess_name(context);
    let mut has_too_many_arguments = false;
    let mut last_argument_offset: isize = -1;
    let mut non_closure_index = 0;
    let mut closure_index = 0;

    while non_closure_index < non_closure_arguments.len() || closure_index < closure_arguments.len() {
        let (argument_offset, argument) =
            match (non_closure_arguments.get(non_closure_index), closure_arguments.get(closure_index)) {
                (Some(non_closure), Some(closure)) if non_closure.0 <= closure.0 => {
                    non_closure_index += 1;
                    non_closure
                }
                (Some(_), Some(closure)) => {
                    closure_index += 1;
                    closure
                }
                (Some(non_closure), None) => {
                    non_closure_index += 1;
                    non_closure
                }
                (None, Some(closure)) => {
                    closure_index += 1;
                    closure
                }
                (None, None) => break,
            };

        let Some(argument_expression) = argument.value() else {
            continue;
        };

        let (argument_value_type, _) = analyzed_argument_types
            .get(argument_offset)
            .cloned()
            .unwrap_or_else(|| (get_mixed(), argument_expression.span()));

        let parameter_ref = get_parameter_of_argument(&invocation.target, argument, *argument_offset);
        if let Some((parameter_offset, parameter_ref)) = parameter_ref {
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
                    if parameter_ref.is_variadic() {
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
                    } else {
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
                method_class_type,
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

            if let Some(parameter_name) = parameter_ref.get_name() {
                parameter_types.insert(parameter_name.0, argument_value_type);
            }
        } else if let Some(named_argument) = argument.get_named_argument() {
            let argument_name = named_argument.name.value;

            // For variadic functions, allow extra named arguments
            let has_variadic_parameter = invocation
                .target
                .parameter_count()
                .checked_sub(1)
                .and_then(|idx| invocation.target.get_parameter(idx))
                .is_some_and(|parameter| parameter.is_variadic());

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
                        } else if invocation.target.parameter_count() == 0 {
                            format!("The {target_kind_str} `{target_name_str}` has no parameters.")
                        } else {
                            let available_params: Vec<_> = invocation
                                .target
                                .iter_parameters()
                                .filter_map(|parameter| parameter.get_name())
                                .map(|n| n.0.trim_start_matches('$'))
                                .collect();
                            format!("Available parameters are: `{}`.", available_params.join("`, `"))
                        }
                    }),
                );
            }

            break;
        } else if *argument_offset >= invocation.target.parameter_count() {
            has_too_many_arguments = true;
            continue;
        } else {
            // positional argument with no matching parameter and not over the limit; fall through to record offset
        }

        last_argument_offset = *argument_offset as isize;
    }

    if !has_too_many_arguments {
        loop {
            last_argument_offset += 1;
            if last_argument_offset as usize >= invocation.target.parameter_count() {
                break;
            }

            let Some(unused_parameter) = invocation.target.get_parameter(last_argument_offset as usize) else {
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
                method_class_type,
            );

            let default_type =
                unused_parameter.get_default_type().map_or_else(|| Cow::Owned(get_mixed()), Cow::Borrowed);

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

    if !unpacked_arguments.is_empty()
        && let Some(last_parameter_offset) = invocation.target.parameter_count().checked_sub(1)
        && let Some(last_parameter_ref) = invocation.target.get_parameter(last_parameter_offset)
        && last_parameter_ref.is_variadic()
    {
        let base_variadic_parameter_type = get_parameter_type(
            context,
            Some(last_parameter_ref),
            base_class_metadata,
            calling_class_like_metadata,
            calling_instance_type,
            method_class_type,
        );

        if base_variadic_parameter_type.has_template_types() {
            for unpacked_argument in &unpacked_arguments {
                let Some(argument_expression) = unpacked_argument.value() else {
                    continue;
                };

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
                    artifacts.get_expression_type(argument_expression).cloned().unwrap_or_else(get_mixed);

                let unpacked_element_type =
                    get_unpacked_argument_type(context, &argument_value_type, argument_expression.span());

                infer_parameter_templates_from_argument(
                    context,
                    &base_variadic_parameter_type,
                    &unpacked_element_type,
                    template_result,
                    last_parameter_offset,
                    argument_expression.span(),
                    false,
                );
            }
        }
    }

    if let Some(template_types) = invocation.target.get_template_types() {
        for (template_name, template) in template_types {
            if template_result.has_lower_bound(*template_name, &template.defining_entity) {
                continue;
            }

            template_result.add_lower_bound(*template_name, template.defining_entity, template.constraint.clone());
        }
    }

    let max_params = invocation.target.parameter_count();
    let number_of_required_parameters = invocation
        .target
        .iter_parameters()
        .filter(|parameter| !parameter.has_default() && !parameter.is_variadic())
        .count();
    let mut number_of_provided_parameters = non_closure_arguments.len() + closure_arguments.len();

    if !unpacked_arguments.is_empty() {
        if let Some(last_parameter_offset) = invocation.target.parameter_count().checked_sub(1)
            && let Some(last_parameter_ref) = invocation.target.get_parameter(last_parameter_offset)
        {
            if last_parameter_ref.is_variadic() {
                let base_variadic_parameter_type = get_parameter_type(
                    context,
                    Some(last_parameter_ref),
                    base_class_metadata,
                    calling_class_like_metadata,
                    calling_instance_type,
                    method_class_type,
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
                    let Some(argument_expression) = unpacked_argument.value() else {
                        continue;
                    };

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

                    let (minimum_unpacked_size, has_keyed_array_with_named_args) = argument_value_type
                        .types
                        .as_ref()
                        .iter()
                        .fold((None::<usize>, false), |(minimum_size, mut has_named_args), argument_atomic| {
                            let size = if let TAtomic::Array(array) = argument_atomic {
                                if !has_named_args
                                    && let mago_codex::ttype::atomic::array::TArray::Keyed(keyed_array) = array
                                    && let Some(known_items) = &keyed_array.known_items
                                {
                                    has_named_args = known_items.iter().any(|(array_key, _)| {
                                        matches!(array_key, mago_codex::ttype::atomic::array::key::ArrayKey::String(_))
                                    });
                                }

                                array.get_minimum_size()
                            } else {
                                0
                            };

                            (Some(minimum_size.map_or(size, |current| current.min(size))), has_named_args)
                        });

                    number_of_provided_parameters += minimum_unpacked_size.unwrap_or(0);

                    // Skip union-based validation for keyed arrays with named arguments.
                    // Individual elements are properly validated in validate_keyed_array_elements.
                    if !has_keyed_array_with_named_args {
                        let unpacked_element_type =
                            get_unpacked_argument_type(context, &argument_value_type, argument_expression.span());

                        verify_argument_type(
                            context,
                            &unpacked_element_type,
                            &final_variadic_parameter_type,
                            last_parameter_offset,
                            argument_expression,
                            &invocation.target,
                        );
                    }
                }
            } else {
                let mut current_parameter_position = 0;

                for argument in invocation.arguments_source.iter_arguments() {
                    if argument.is_unpacked() {
                        let Some(argument_expression) = argument.value() else {
                            continue;
                        };

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
                        let unpacked_count = argument_value_type
                            .types
                            .as_ref()
                            .iter()
                            .fold(None::<usize>, |minimum_size, argument_atomic| {
                                let size = if let TAtomic::Array(array) = argument_atomic {
                                    array.get_minimum_size()
                                } else {
                                    0
                                };

                                Some(minimum_size.map_or(size, |current| current.min(size)))
                            })
                            .unwrap_or(0);
                        number_of_provided_parameters += unpacked_count;

                        validate_unpacked_argument_elements(
                            context,
                            &argument_value_type,
                            argument_expression,
                            base_class_metadata,
                            calling_class_like_metadata,
                            calling_instance_type,
                            method_class_type,
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
                    invocation.target.guess_name(context)
                ))
                .with_annotation(
                    Annotation::primary(unpacked_arguments[0].span()).with_message("Unexpected argument unpacking"),
                )
                .with_help("Remove the argument unpacking (`...`)."),
            );
        } else {
            // target accepts no parameters and no unpacking was attempted; nothing to validate
        }
    }

    let should_check_argument_count = match invocation.arguments_source {
        InvocationArgumentsSource::PartialArgumentList(_) => {
            (non_closure_arguments.len() + closure_arguments.len()) < number_of_required_parameters
        }
        _ => number_of_provided_parameters < number_of_required_parameters,
    };

    if should_check_argument_count {
        let primary_annotation_span = invocation.arguments_source.span();

        let main_message = match invocation.arguments_source {
            InvocationArgumentsSource::PipeInput(_) => format!(
                "Too few arguments for {target_kind_str} `{target_name_str}` when used with the pipe operator `|>`. Pipe provides 1, but at least {number_of_required_parameters} required."
            ),
            _ => format!("Too few arguments provided for {target_kind_str} `{target_name_str}`."),
        };

        let total_positions = non_closure_arguments.len() + closure_arguments.len();
        let mut issue = Issue::error(main_message)
            .with_annotation(Annotation::primary(primary_annotation_span).with_message("More arguments expected here"))
            .with_note(format!(
                "Expected at least {number_of_required_parameters} argument(s) for non-optional parameters, but received {}.",
                if matches!(invocation.arguments_source, InvocationArgumentsSource::PartialArgumentList(_)) {
                    total_positions
                } else {
                    number_of_provided_parameters
                }
            ));

        issue = match invocation.arguments_source {
            InvocationArgumentsSource::ArgumentList(_) | InvocationArgumentsSource::PartialArgumentList(_) => issue
                .with_annotation(
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
        || (!invocation
            .target
            .parameter_count()
            .checked_sub(1)
            .and_then(|idx| invocation.target.get_parameter(idx))
            .is_some_and(|parameter| parameter.is_variadic())
            && number_of_provided_parameters > max_params
            && max_params > 0)
    {
        let first_extra_arg_span = invocation
            .arguments_source
            .get_argument(max_params)
            .map_or_else(|| invocation.arguments_source.span(), |argument| argument.span());

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
    } else {
        // argument count is within the parameter range; nothing to report
    }

    check_template_result(context, template_result, invocation.span);

    Ok(())
}

/// Gets the effective parameter type, expanding class-relative types based on call context.
fn get_parameter_type<'ctx>(
    context: &Context<'ctx, '_>,
    invocation_target_parameter: Option<InvocationTargetParameter<'_>>,
    base_class_metadata: Option<&'ctx ClassLikeMetadata>,
    calling_class_like_metadata: Option<&'ctx ClassLikeMetadata>,
    calling_instance_type: Option<&TAtomic>,
    method_class_type: Option<&StaticClassType>,
) -> TUnion {
    let Some(invocation_target_parameter) = invocation_target_parameter else {
        return get_mixed();
    };

    let parameter_type = invocation_target_parameter.get_type().cloned().unwrap_or_else(get_mixed);

    let mut resolved_parameter_type = parameter_type;

    let static_class_type = method_class_type
        .filter(|t| !matches!(t, StaticClassType::None))
        .cloned()
        .or_else(|| {
            calling_class_like_metadata.map(|calling_meta| {
                calling_instance_type
                    .and_then(|instance| match instance {
                        TAtomic::Object(obj) => Some(StaticClassType::Object(obj.clone())),
                        _ => None,
                    })
                    .unwrap_or(StaticClassType::Name(calling_meta.name))
            })
        })
        .unwrap_or(StaticClassType::None);

    expander::expand_union(
        context.codebase,
        &mut resolved_parameter_type,
        &TypeExpansionOptions {
            self_class: base_class_metadata.map(|meta| meta.name),
            static_class_type,
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
    base_class_metadata: Option<&'ctx ClassLikeMetadata>,
    calling_class_like_metadata: Option<&'ctx ClassLikeMetadata>,
    calling_instance_type: Option<&TAtomic>,
    method_class_type: Option<&StaticClassType>,
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
                    for (array_index, (_, element_type)) in known_elements {
                        let parameter_position = starting_parameter_position + array_index;
                        if parameter_position >= invocation_target.parameter_count() {
                            break;
                        }

                        if let Some(parameter_ref) = invocation_target.get_parameter(parameter_position) {
                            let base_parameter_type = get_parameter_type(
                                context,
                                Some(parameter_ref),
                                base_class_metadata,
                                calling_class_like_metadata,
                                calling_instance_type,
                                method_class_type,
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
                    let max_parameters_to_check = std::cmp::min(
                        min_size,
                        invocation_target.parameter_count().saturating_sub(starting_parameter_position),
                    );

                    for i in 0..max_parameters_to_check {
                        let parameter_position = starting_parameter_position + i;
                        if let Some(parameter_ref) = invocation_target.get_parameter(parameter_position) {
                            let base_parameter_type = get_parameter_type(
                                context,
                                Some(parameter_ref),
                                base_class_metadata,
                                calling_class_like_metadata,
                                calling_instance_type,
                                method_class_type,
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
                    base_class_metadata,
                    calling_class_like_metadata,
                    calling_instance_type,
                    method_class_type,
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
    base_class_metadata: Option<&'ctx ClassLikeMetadata>,
    calling_class_like_metadata: Option<&'ctx ClassLikeMetadata>,
    calling_instance_type: Option<&TAtomic>,
    method_class_type: Option<&StaticClassType>,
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

    for (array_key, (_, element_type)) in known_items {
        let parameter_name = match array_key {
            ArrayKey::String(key_str) => concat_atom!("$", key_str),
            ArrayKey::Integer(key_int) => {
                if let Some(parameter_ref) = invocation_target.get_parameter(*key_int as usize) {
                    if let Some(param_name) = parameter_ref.get_name() {
                        param_name.0
                    } else {
                        continue;
                    }
                } else {
                    continue;
                }
            }
            ArrayKey::ClassLikeConstant { .. } => {
                continue;
            }
        };

        if let Some((parameter_position, parameter_ref)) = invocation_target
            .iter_parameters()
            .enumerate()
            .find(|(_, p)| p.get_name().is_some_and(|param_name| param_name.0 == parameter_name))
        {
            let base_parameter_type = get_parameter_type(
                context,
                Some(parameter_ref),
                base_class_metadata,
                calling_class_like_metadata,
                calling_instance_type,
                method_class_type,
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
            let has_variadic_parameter = invocation_target
                .parameter_count()
                .checked_sub(1)
                .and_then(|idx| invocation_target.get_parameter(idx))
                .is_some_and(|parameter| parameter.is_variadic());

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
                    } else if invocation_target.parameter_count() > 0 {
                        let available_params: Vec<String> = invocation_target
                            .iter_parameters()
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
        } else {
            // integer array key with no matching parameter; already handled via continue above
        }
    }
}

/// Detects if the current invocation is a `Closure::bind` or `Closure::bindTo` call
/// and extracts the bound scope information from its arguments.
///
/// Returns `Some(ClosureBindScope)` if this is a Closure::bind/bindTo call with extractable scope,
/// `None` otherwise.
fn detect_closure_bind_scope<'ctx, 'arena>(
    context: &Context<'ctx, 'arena>,
    invocation: &Invocation<'ctx, '_, 'arena>,
    analyzed_argument_types: &HashMap<usize, (TUnion, mago_span::Span)>,
) -> Option<ClosureBindScope> {
    let identifier = invocation.target.get_function_like_identifier()?;

    let FunctionLikeIdentifier::Method(class_id, method_id) = identifier else {
        return None;
    };

    if !class_id.eq_ignore_ascii_case("closure") {
        return None;
    }

    let method_name = method_id.as_str().to_ascii_lowercase();

    let (new_this_offset, new_scope_offset) = if method_name.eq_ignore_ascii_case("bind") {
        (1, 2)
    } else {
        return None;
    };

    let new_this_type = analyzed_argument_types.get(&new_this_offset).map(|(t, _)| t);
    let has_this = new_this_type.is_some_and(|t| !t.is_null());

    let class_name = if let Some((scope_type, _)) = analyzed_argument_types.get(&new_scope_offset) {
        extract_class_name_from_scope_arg(context, scope_type)
    } else if has_this {
        new_this_type.and_then(extract_class_name_from_type)
    } else {
        None
    };

    if class_name.is_some() || has_this { Some(ClosureBindScope { class_name, has_this }) } else { None }
}

/// Extracts a class name from a scope argument (typically the 3rd argument to Closure::bind).
/// This handles cases like `Foo::class`, `'Foo'`, or a class instance type.
fn extract_class_name_from_scope_arg(context: &Context<'_, '_>, scope_type: &TUnion) -> Option<Atom> {
    for atomic in scope_type.types.as_ref() {
        match atomic {
            TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::Literal { value })) => {
                return Some(*value);
            }
            TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::OfType { constraint, .. }))
                if let Some(name) = extract_class_name_from_atomic(constraint) =>
            {
                return Some(name);
            }
            TAtomic::Scalar(TScalar::String(TString { literal: Some(TStringLiteral::Value(value)), .. }))
                if context.codebase.get_class_like(value).is_some() =>
            {
                return Some(*value);
            }
            TAtomic::Object(TObject::Named(named)) => {
                return Some(named.name);
            }
            TAtomic::Object(TObject::Enum(enum_obj)) => {
                return Some(enum_obj.name);
            }
            _ => {}
        }
    }

    None
}

/// Extracts a class name from a type union (typically for $this type).
fn extract_class_name_from_type(t: &TUnion) -> Option<Atom> {
    for atomic in t.types.as_ref() {
        if let Some(name) = extract_class_name_from_atomic(atomic) {
            return Some(name);
        }
    }
    None
}

/// Extracts a class name from an atomic type.
fn extract_class_name_from_atomic(atomic: &TAtomic) -> Option<Atom> {
    match atomic {
        TAtomic::Object(TObject::Named(named)) => Some(named.name),
        TAtomic::Object(TObject::Enum(enum_obj)) => Some(enum_obj.name),
        _ => None,
    }
}

/// Filters the callable parameter type for `array_filter` based on the `mode` argument.
/// For `array_filter`, narrows the callback parameter type to only the callable signature
/// matching the `mode` argument, so closure parameters get the correct inferred type.
///
/// mode 0 (default) → `callable(V): bool`, mode 1 → `callable(V, K): bool`, mode 2 → `callable(K): bool`
fn filter_array_filter_callback_type(
    target: &InvocationTarget<'_>,
    parameter_type: &mut TUnion,
    analyzed_argument_types: &HashMap<usize, (TUnion, mago_span::Span)>,
) {
    let is_array_filter = target.get_function_like_identifier().is_some_and(
        |id| matches!(id, FunctionLikeIdentifier::Function(name) if name.eq_ignore_ascii_case("array_filter")),
    );

    if !is_array_filter {
        return;
    }

    let mode = analyzed_argument_types
        .get(&2)
        .and_then(|(mode_type, _)| mode_type.get_single_literal_int_value())
        .unwrap_or(0);

    let expected_params: usize = if mode == 1 { 2 } else { 1 };

    let value_type_id = if expected_params == 1 {
        parameter_type.types.as_ref().iter().find_map(|atomic| {
            if let TAtomic::Callable(TCallable::Signature(sig)) = atomic
                && sig.parameters.len() == 2
            {
                sig.parameters.first().and_then(|p| p.get_type_signature()).map(|t| t.get_id())
            } else {
                None
            }
        })
    } else {
        None
    };

    let mut kept_one = false;
    parameter_type.types.to_mut().retain(|atomic| {
        let TAtomic::Callable(TCallable::Signature(sig)) = atomic else {
            return true;
        };

        if sig.parameters.len() != expected_params {
            return false;
        }

        if expected_params == 2 {
            return if kept_one {
                false
            } else {
                kept_one = true;
                true
            };
        }

        let is_value = sig.parameters.first().and_then(|p| p.get_type_signature()).map(|t| t.get_id()) == value_type_id;

        let want_value = mode != 2;
        if is_value == want_value && !kept_one {
            kept_one = true;
            true
        } else {
            false
        }
    });
}
