use std::borrow::Cow;

use ahash::HashMap;
use ahash::RandomState;
use indexmap::IndexMap;
use itertools::Itertools;

use mago_codex::get_class_like;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_codex::misc::GenericParent;
use mago_codex::ttype::add_union_type;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::callable::TCallable;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::union_comparator;
use mago_codex::ttype::comparator::union_comparator::can_expression_types_be_identical;
use mago_codex::ttype::comparator::union_comparator::is_contained_by;
use mago_codex::ttype::expander;
use mago_codex::ttype::expander::StaticClassType;
use mago_codex::ttype::expander::TypeExpansionOptions;
use mago_codex::ttype::template::TemplateBound;
use mago_codex::ttype::template::TemplateResult;
use mago_codex::ttype::template::inferred_type_replacer;
use mago_codex::ttype::template::standin_type_replacer::get_most_specific_type_from_bounds;
use mago_codex::ttype::union::TUnion;
use mago_codex::ttype::*;
use mago_interner::StringIdentifier;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::Code;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::invocation::Invocation;
use crate::invocation::InvocationArgument;
use crate::invocation::InvocationArgumentsSource;
use crate::invocation::InvocationTarget;
use crate::invocation::InvocationTargetParameter;
use crate::invocation::MethodTargetContext;
use crate::invocation::template_inference::infer_parameter_templates_from_argument;
use crate::invocation::template_inference::infer_parameter_templates_from_default;
use crate::invocation::template_inference::infer_templates_for_method_call;
use crate::utils::misc::unique_vec;
use crate::utils::template::get_template_types_for_class_member;

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
pub fn analyze_invocation<'a>(
    context: &mut Context<'a>,
    block_context: &mut BlockContext<'a>,
    artifacts: &mut AnalysisArtifacts,
    invocation: &Invocation<'_>,
    calling_class_like: Option<(StringIdentifier, Option<&TAtomic>)>,
    template_result: &mut TemplateResult,
    parameter_types: &mut HashMap<StringIdentifier, TUnion>,
) -> Result<(), AnalysisError> {
    fn get_parameter_of_argument<'r>(
        context: &Context<'_>,
        parameters: &[InvocationTargetParameter<'r>],
        argument: &InvocationArgument<'_>,
        mut argument_offset: usize,
    ) -> Option<(usize, InvocationTargetParameter<'r>)> {
        if let Some(named_argument) = argument.get_named_argument() {
            let argument_name_str = context.interner.lookup(&named_argument.name.value);
            let argument_variable_name_str = format!("${argument_name_str}");
            let argument_variable_name = context.interner.intern(&argument_variable_name_str);

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

    let mut non_closure_arguments: Vec<(usize, InvocationArgument<'_>)> = Vec::new();
    let mut closure_arguments: Vec<(usize, InvocationArgument<'_>)> = Vec::new();
    let mut unpacked_arguments: Vec<InvocationArgument<'_>> = Vec::new();
    for (offset, argument) in invocation.arguments_source.get_arguments().into_iter().enumerate() {
        if argument.is_unpacked() {
            unpacked_arguments.push(argument);
        } else if matches!(argument.value(), Expression::Closure(_) | Expression::ArrowFunction(_)) {
            closure_arguments.push((offset, argument));
        } else {
            non_closure_arguments.push((offset, argument));
        }
    }

    let calling_class_like_metadata =
        calling_class_like.and_then(|(id, _)| get_class_like(context.codebase, context.interner, &id));
    let base_class_metadata =
        invocation.target.get_method_context().map(|ctx| ctx.class_like_metadata).or(calling_class_like_metadata);
    let method_call_context = invocation.target.get_method_context();

    for (argument_offset, argument) in &non_closure_arguments {
        let argument_expression = argument.value();
        let parameter = get_parameter_of_argument(context, &parameter_refs, argument, *argument_offset);

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
        let parameter = get_parameter_of_argument(context, &parameter_refs, argument, *argument_offset);
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

                Some(inferred_type_replacer::replace(
                    &base_parameter_type,
                    template_result,
                    context.codebase,
                    context.interner,
                ))
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
    let target_name_str = invocation.target.guess_name(context.interner);
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

        let parameter_ref = get_parameter_of_argument(context, &parameter_refs, argument, *argument_offset);
        if let Some((parameter_offset, parameter_ref)) = parameter_ref {
            if let Some(parameter_name) = parameter_ref.get_name() {
                parameter_types.insert(parameter_name.0, argument_value_type.clone());
            }

            if let Some(named_argument) = argument.get_named_argument() {
                if let Some(previous_span) = assigned_parameters_by_name.get(&named_argument.name.value) {
                    context.collector.report_with_code(
                        Code::DUPLICATE_NAMED_ARGUMENT,
                        Issue::error(format!(
                            "Duplicate named argument `${}` in call to {} `{}`.",
                            context.interner.lookup(&named_argument.name.value),
                            target_kind_str,
                            target_name_str
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
                                Code::NAMED_ARGUMENT_OVERRIDES_POSITIONAL,
                                Issue::error(format!(
                                    "Named argument `${}` for {} `{}` targets a parameter already provided positionally.",
                                    context.interner.lookup(&named_argument.name.value), target_kind_str, target_name_str
                                ))
                                .with_annotation(Annotation::primary(named_argument.name.span()).with_message("This named argument"))
                                .with_annotation(Annotation::secondary(*previous_span).with_message("Parameter already filled by positional argument here"))
                                .with_help("Provide the argument either positionally or by name, but not both."),
                            );
                        } else {
                            context.collector.report_with_code(
                                Code::NAMED_ARGUMENT_AFTER_POSITIONAL,
                                 Issue::warning(format!(
                                    "Named argument `${}` for {} `{}` targets a variadic parameter that has already captured positional arguments.",
                                    context.interner.lookup(&named_argument.name.value), target_kind_str, target_name_str
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
                inferred_type_replacer::replace(
                    &base_parameter_type,
                    template_result,
                    context.codebase,
                    context.interner,
                )
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
            let argument_name = context.interner.lookup(&named_argument.name.value);

            context.collector.report_with_code(
                Code::UNKNOWN_NAMED_ARGUMENT,
                Issue::error(format!(
                    "Unknown named argument `${argument_name}` for {target_kind_str} `{target_name_str}`"
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
                            .map(|n| context.interner.lookup(&n.0).trim_start_matches('$'))
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

                let final_variadic_parameter_type = inferred_type_replacer::replace(
                    &base_variadic_parameter_type,
                    template_result,
                    context.codebase,
                    context.interner,
                );

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
                    for argument_atomic in &argument_value_type.types {
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
                    Code::TOO_MANY_ARGUMENTS,
                    Issue::error(format!(
                        "Cannot unpack arguments into non-variadic {} `{}`.",
                        invocation.target.guess_kind(),
                        invocation.target.guess_name(context.interner),
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
                Code::TOO_MANY_ARGUMENTS,
                Issue::error(format!(
                    "Cannot unpack arguments into {} `{}` which expects no arguments.",
                    invocation.target.guess_kind(),
                    invocation.target.guess_name(context.interner)
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
        context.collector.report_with_code(Code::TOO_FEW_ARGUMENTS, issue);
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

        context.collector.report_with_code(Code::TOO_MANY_ARGUMENTS, issue);
    }

    check_template_result(context, template_result, invocation.span);

    Ok(())
}

/// Populates the `TemplateResult` with template types from the invocation target.
///
/// This function extracts template types from the metadata of the invocation target,
/// including any method context if applicable. It also adds lower bounds for
/// template types based on the class-like metadata and the type parameters of the class.
///
/// # Arguments
///
/// * `invocation` - The invocation whose target metadata is used to populate the template result.
/// * `template_result` - The mutable `TemplateResult` to be populated with template types and bounds.
///
/// # Note
///
/// This function assumes that the `TemplateResult` is initially empty and will be populated with
/// template types and bounds derived from the invocation's target metadata.
pub fn populate_template_result_from_invocation(
    context: &mut Context<'_>,
    invocation: &Invocation<'_>,
    template_result: &mut TemplateResult,
) {
    let InvocationTarget::FunctionLike { metadata, method_context, .. } = &invocation.target else {
        return;
    };

    for (template_name, template_details) in metadata.template_types.iter() {
        template_result.template_types.insert(*template_name, template_details.clone());
    }

    let Some(method_metadata) = &metadata.method_metadata else {
        return;
    };

    if method_metadata.is_static {
        return;
    }

    let Some(method_context) = method_context else {
        return;
    };

    let StaticClassType::Object(TObject::Named(instance_type)) = &method_context.class_type else {
        return;
    };

    if let Some(type_parameters) = &instance_type.type_parameters {
        for (template_index, template_type) in type_parameters.iter().enumerate() {
            let Some(template_name) = method_context
                .class_like_metadata
                .template_types
                .iter()
                .enumerate()
                .find_map(|(index, (name, _))| if index == template_index { Some(*name) } else { None })
            else {
                break;
            };

            template_result.add_lower_bound(
                template_name,
                GenericParent::ClassLike(method_context.class_like_metadata.name),
                template_type.clone(),
            );
        }
    }

    let Some(identifier) = method_context.declaring_method_id else {
        return;
    };

    let Some(metadata) = get_class_like(context.codebase, context.interner, identifier.get_class_name()) else {
        return;
    };

    infer_templates_for_method_call(context, instance_type, method_context, method_metadata, metadata, template_result);
}

/// Analyzes a single argument expression and stores its inferred type and span.
///
/// This function ensures an argument expression is analyzed within the correct
/// context (temporarily setting `inside_general_use` to true) and stores the
/// resulting type and the expression's span in the provided map for later use,
/// unless the argument is unpacked (indicated by `argument_offset == usize::MAX`).
/// It avoids re-analyzing if the argument type is already present in the map.
///
/// # Arguments
///
/// * `context` - The overall analysis context.
/// * `block_context` - Mutable context for the current code block.
/// * `artifacts` - Mutable store for analysis results, including expression types.
/// * `argument_expression` - The AST node for the argument's value expression.
/// * `argument_offset` - The zero-based index of the argument. Use `usize::MAX` to skip storing (e.g., for unpacked arguments analyzed just for side effects).
/// * `analyzed_argument_types` - The map where the inferred type and span are stored, keyed by argument offset.
///
/// # Returns
///
/// * `Ok(())` if analysis completes successfully.
/// * `Err(AnalysisError)` if an error occurs during the analysis of the argument's value.
fn analyze_and_store_argument_type<'a>(
    context: &mut Context<'a>,
    block_context: &mut BlockContext<'a>,
    artifacts: &mut AnalysisArtifacts,
    invocation_target: &InvocationTarget<'_>,
    argument_expression: &Expression,
    argument_offset: usize,
    analyzed_argument_types: &mut HashMap<usize, (TUnion, Span)>,
    referenced_parameter: bool,
    closure_parameter_type: Option<&TUnion>,
) -> Result<(), AnalysisError> {
    if argument_offset != usize::MAX && analyzed_argument_types.contains_key(&argument_offset) {
        return Ok(());
    }

    let mut inferred_parameter_types: Option<HashMap<usize, TUnion>> = None;
    if let Some(closure_parameter_type) = closure_parameter_type {
        let mut inferred_parameters = HashMap::default();
        for closure_parameter_atomic in &closure_parameter_type.types {
            let TAtomic::Callable(TCallable::Signature(callable)) = closure_parameter_atomic else {
                continue;
            };

            for (parameter_index, parameter) in callable.parameters.iter().enumerate() {
                let Some(parameter_type) = parameter.get_type_signature() else {
                    continue;
                };

                if !parameter_type.is_array() && !parameter_type.has_object() {
                    continue;
                }

                inferred_parameters.insert(parameter_index, parameter_type.clone());
            }
        }

        inferred_parameter_types = Some(inferred_parameters);
    }

    let inferred_parameter_types = std::mem::replace(&mut artifacts.inferred_parameter_types, inferred_parameter_types);
    let was_inside_general_use = block_context.inside_general_use;
    let was_inside_call = block_context.inside_call;
    let was_inside_variable_reference = block_context.inside_variable_reference;

    block_context.inside_general_use = true;
    block_context.inside_call = true;
    block_context.inside_variable_reference = referenced_parameter;

    argument_expression.analyze(context, block_context, artifacts)?;

    artifacts.inferred_parameter_types = inferred_parameter_types;
    block_context.inside_general_use = was_inside_general_use;
    block_context.inside_call = was_inside_call;
    block_context.inside_variable_reference = was_inside_variable_reference;

    let argument_type = artifacts.get_expression_type(argument_expression).cloned().unwrap_or_else(get_mixed);

    if argument_offset != usize::MAX {
        analyzed_argument_types.insert(argument_offset, (argument_type, argument_expression.span()));
    }

    if referenced_parameter && !argument_expression.is_referenceable() {
        let target_kind_str = invocation_target.guess_kind();
        let target_name_str = invocation_target.guess_name(context.interner);

        context.collector.report_with_code(
            Code::INVALID_PASS_BY_REFERENCE,
            Issue::error(format!(
                "Argument #{} of {} `{}` cannot be passed by reference.",
                argument_offset + 1,
                target_kind_str,
                target_name_str,
            ))
            .with_annotation(
                Annotation::primary(argument_expression.span())
                    .with_message("This expression is not a valid reference."),
            )
            .with_note(
                "Only variables, array elements, or object properties can be passed by reference. Literals and the results of most expressions cannot."
            )
            .with_help("To fix this, assign this value to a variable first, and then pass the variable to the function."),
        );
    }

    Ok(())
}

/// Extracts and resolves concrete types for class-level template parameters based on inferred lower bounds.
///
/// This function iterates through the `lower_bounds` collected in a `TemplateResult`.
/// For each template parameter that is defined by a class (`GenericParent::ClassLike`),
/// it calculates the most specific type derived from its lower bounds using
/// `get_most_specific_type_from_bounds`.
///
/// The result is a map where keys are template parameter names (`StringIdentifier`) and
/// values are vectors containing pairs of the defining class (`GenericParent`) and the
/// resolved concrete type (`TUnion`) for that template in the context of that class.
///
/// This map is typically used later to refine template standins within method/property signatures
/// belonging to the class or its children.
///
/// # Arguments
///
/// * `template_result` - The template result containing the inferred lower bounds.
/// * `context` - The analysis context, providing access to codebase and interner needed for type resolution.
///
/// # Returns
///
/// An `IndexMap` mapping class template parameter names to a vector of (Defining Entity, Resolved Type).
fn get_class_template_parameters_from_result(
    template_result: &TemplateResult,
    context: &Context<'_>,
) -> IndexMap<StringIdentifier, Vec<(GenericParent, TUnion)>, RandomState> {
    let mut class_generic_parameters: IndexMap<StringIdentifier, Vec<(GenericParent, TUnion)>, RandomState> =
        IndexMap::with_hasher(RandomState::new());

    for (template_name, type_map) in &template_result.lower_bounds {
        for (generic_parent, lower_bounds) in type_map {
            if matches!(generic_parent, GenericParent::ClassLike(_)) && !lower_bounds.is_empty() {
                let specific_bound_type =
                    get_most_specific_type_from_bounds(lower_bounds, context.codebase, context.interner);

                class_generic_parameters
                    .entry(*template_name)
                    .or_default()
                    .push((*generic_parent, specific_bound_type));
            }
        }
    }

    class_generic_parameters
}

/// Verifies a single argument's type against the resolved parameter type for a function/method/callable call.
///
/// This function compares the `input_type` (actual argument type) against the `parameter_type`
/// (expected type after template resolution). It reports various type mismatch errors
/// (e.g., invalid type, possibly invalid, mixed argument, less specific argument)
/// with appropriate severity and context. It also adds data flow edges from the argument
/// sources to the parameter representation in the data flow graph.
fn verify_argument_type(
    context: &mut Context<'_>,
    input_type: &TUnion,
    parameter_type: &TUnion,
    argument_offset: usize,
    input_expression: &Expression,
    invocation_target: &InvocationTarget<'_>,
) {
    let target_kind_str = invocation_target.guess_kind();
    let target_name_str = invocation_target.guess_name(context.interner);

    if input_type.is_never() {
        context.collector.report_with_code(
            Code::NO_VALUE,
            Issue::error(format!(
                "Argument #{} passed to {} `{}` has type `never`, meaning it cannot produce a value.",
                argument_offset + 1,
                target_kind_str,
                target_name_str
            ))
            .with_annotation(
                Annotation::primary(input_expression.span())
                    .with_message("This argument expression results in type `never`")
            )
            .with_note(
                "The `never` type indicates this expression will not complete to produce a value."
            )
            .with_note(
                "This often occurs in unreachable code, due to impossible conditional logic, or if an expression always exits (e.g., `throw`, `exit()`)."
            )
            .with_help(
                "Review preceding logic to ensure this argument can receive a value, or remove if unreachable."
            ),
        );

        return;
    }

    let call_site = Annotation::secondary(invocation_target.span())
        .with_message(format!("Arguments to this {} are incorrect", invocation_target.guess_kind()));

    let input_type_str = input_type.get_id(Some(context.interner));
    let parameter_type_str = parameter_type.get_id(Some(context.interner));

    if !parameter_type.accepts_null() {
        if input_type.is_null() {
            context.collector.report_with_code(
                Code::NULL_ARGUMENT,
                Issue::error(format!(
                    "Argument #{} of {} `{}` is `null`, but parameter type `{}` does not accept it.",
                    argument_offset + 1,
                    target_kind_str,
                    target_name_str,
                    parameter_type_str
                ))
                .with_annotation(Annotation::primary(input_expression.span()).with_message("This argument is `null`"))
                .with_annotation(call_site)
                .with_help(format!(
                    "Provide a non-null value, or declare the parameter as nullable (e.g., `{parameter_type_str}|null`)."
                )),
            );

            return;
        }

        if input_type.is_nullable() && !input_type.ignore_nullable_issues {
            context.collector.report_with_code(
                Code::POSSIBLY_NULL_ARGUMENT,
                Issue::error(format!(
                    "Argument #{} of {} `{}` is possibly `null`, but parameter type `{}` does not accept it.",
                    argument_offset + 1,
                    target_kind_str,
                    target_name_str,
                    parameter_type_str
                ))
                .with_annotation(
                    Annotation::primary(input_expression.span())
                        .with_message(format!("This argument of type `{input_type_str}` might be `null`")),
                )
                .with_annotation(call_site.clone())
                .with_help("Add a `null` check before this call to ensure the value is not `null`."),
            );
        }
    }

    if !parameter_type.accepts_false() {
        if input_type.is_false() {
            context.collector.report_with_code(
                Code::FALSE_ARGUMENT,
                Issue::error(format!(
                    "Argument #{} of {} `{}` is `false`, but parameter type `{}` does not accept it.",
                    argument_offset + 1,
                    target_kind_str,
                    target_name_str,
                    parameter_type_str
                ))
                .with_annotation(Annotation::primary(input_expression.span()).with_message("This argument is `false`"))
                .with_annotation(call_site)
                .with_help(format!(
                    "Provide a different value, or update the parameter type to accept false (e.g., `{parameter_type_str}|false`)."
                )),
            );

            return;
        }

        if input_type.is_falsable() && !input_type.ignore_falsable_issues {
            context.collector.report_with_code(
                Code::POSSIBLY_FALSE_ARGUMENT,
                Issue::error(format!(
                    "Argument #{} of {} `{}` is possibly `false`, but parameter type `{}` does not accept it.",
                    argument_offset + 1,
                    target_kind_str,
                    target_name_str,
                    parameter_type_str
                ))
                .with_annotation(
                    Annotation::primary(input_expression.span())
                        .with_message(format!("This argument of type `{input_type_str}` might be `false`")),
                )
                .with_annotation(call_site.clone())
                .with_help("Add a check to ensure the value is not `false` before this call."),
            );
        }
    }

    let mut union_comparison_result = ComparisonResult::new();
    let type_match_found = is_contained_by(
        context.codebase,
        context.interner,
        input_type,
        parameter_type,
        true,
        true,
        false,
        &mut union_comparison_result,
    );

    if type_match_found {
        return;
    }

    if input_type.is_mixed() {
        context.collector.report_with_code(
            Code::MIXED_ARGUMENT,
            Issue::error(format!(
                "Invalid argument type for argument #{} of `{}`: expected `{}`, but found `{}`.",
                argument_offset + 1,
                target_name_str,
                parameter_type_str,
                input_type_str
            ))
            .with_annotation(
                Annotation::primary(input_expression.span())
                    .with_message(format!("Argument has type `{input_type_str}`")),
            )
            .with_annotation(call_site)
            .with_note(format!(
                "The type `{input_type_str}` is too general and does not match the expected type `{parameter_type_str}`."
            ))
            .with_help("Add specific type hints or assertions to the argument value."),
        );

        return;
    }

    if union_comparison_result.type_coerced.unwrap_or(false) && !input_type.is_mixed() {
        let issue_kind;
        let annotation_msg;
        let note_msg;

        if union_comparison_result.type_coerced_from_nested_mixed.unwrap_or(false) {
            issue_kind = Code::LESS_SPECIFIC_NESTED_ARGUMENT_TYPE;
            annotation_msg = format!("Provided type `{input_type_str}` is too general due to nested `mixed`.");
            note_msg = "The structure contains `mixed`, making it incompatible.".to_string();
        } else {
            issue_kind = Code::LESS_SPECIFIC_ARGUMENT;
            annotation_msg = format!("Provided type `{input_type_str}` is too general.");
            note_msg = format!(
                    "The provided type `{input_type_str}` can be assigned to `{parameter_type_str}`, but is wider (less specific)."
                )
                .to_string();
        }

        context.collector.report_with_code(
            issue_kind,
            Issue::error(format!(
                "Argument type mismatch for argument #{} of `{}`: expected `{}`, but provided type `{}` is less specific.",
                argument_offset + 1, target_name_str, parameter_type_str, input_type_str
            ))
            .with_annotation(Annotation::primary(input_expression.span()).with_message(annotation_msg))
            .with_annotation(call_site)
            .with_note(note_msg)
            .with_help(format!("Provide a value that more precisely matches `{parameter_type_str}` or adjust the parameter type.")),
        );
    } else if !union_comparison_result.type_coerced.unwrap_or(false) {
        let types_can_be_identical = can_expression_types_be_identical(
            context.codebase,
            context.interner,
            input_type,
            parameter_type,
            false,
            false,
        );

        if types_can_be_identical {
            context.collector.report_with_code(
                Code::POSSIBLY_INVALID_ARGUMENT,
                Issue::error(format!(
                    "Possible argument type mismatch for argument #{} of `{}`: expected `{}`, but possibly received `{}`.",
                    argument_offset + 1, target_name_str, parameter_type_str, input_type_str
                ))
                .with_annotation(Annotation::primary(input_expression.span()).with_message(format!("This might not be type `{parameter_type_str}`")))
                .with_annotation(call_site)
                .with_note(format!("The provided type `{input_type_str}` overlaps with `{parameter_type_str}` but is not fully contained."))
                .with_help("Ensure the argument always has the expected type using checks or assertions."),
            );
        } else {
            context.collector.report_with_code(
                Code::INVALID_ARGUMENT,
                Issue::error(format!(
                    "Invalid argument type for argument #{} of `{}`: expected `{}`, but found `{}`.",
                    argument_offset + 1,
                    target_name_str,
                    parameter_type_str,
                    input_type_str
                ))
                .with_annotation(
                    Annotation::primary(input_expression.span())
                        .with_message(format!("This has type `{input_type_str}`")),
                )
                .with_annotation(call_site)
                .with_note(format!(
                    "The provided type `{input_type_str}` is not compatible with the expected type `{parameter_type_str}`."
                ))
                .with_help(
                    format!("Change the argument value to match `{parameter_type_str}`, or update the parameter's type declaration.")
                ),
            );
        }
    }
}

/// Gets the effective parameter type from a potential parameter reference,
/// expanding `self`, `static`, and `parent` type hints based on the call context.
///
/// If no specific parameter type is found (e.g., missing parameter reference or
/// no type hint on the parameter), it defaults to `mixed|any`.
///
/// # Arguments
///
/// * `context` - The analysis context, providing codebase and interner access.
/// * `parameter_ref` - An optional reference to the parameter's definition (either `FunctionLike` or `Callable`).
/// * `base_class_metadata` - Optional metadata for the class where the method is *defined*. Used for resolving `self::` and `parent::`.
/// * `calling_class_like_metadata` - Optional metadata for the class context from which the call is made. Used for resolving `static::`.
/// * `calling_instance_type` - Optional specific atomic type of the calling instance (`$this`). Used for resolving `static::` more precisely when available.
///
/// # Returns
///
/// A `TUnion` representing the resolved type of the parameter in the context of the call.
fn get_parameter_type(
    context: &Context<'_>,
    invocation_target_parameter: Option<InvocationTargetParameter<'_>>,
    base_class_metadata: Option<&ClassLikeMetadata>,
    calling_class_like_metadata: Option<&ClassLikeMetadata>,
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
        context.interner,
        &mut resolved_parameter_type,
        &TypeExpansionOptions {
            self_class: base_class_metadata.map(|meta| &meta.name),
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
            parent_class: base_class_metadata.and_then(|meta| meta.direct_parent_class.as_ref()),
            function_is_final: calling_class_like_metadata.is_some_and(|meta| meta.flags.is_final()),
            ..Default::default()
        },
    );

    resolved_parameter_type
}

/// Refines the template result by incorporating template definitions specific to the called function or method.
///
/// This function retrieves the applicable template type definitions (e.g., `@template T as array-key`
/// defined on the function/method itself or inherited) considering the class context.
///
/// If the `template_result` provided does not already contain template type definitions
/// (i.e., `template_result.template_types` is empty), this function populates it with
/// the definitions resolved by `get_template_types_for_class_member`.
///
/// **Note:** If `template_result.template_types` already contains entries (perhaps from
/// analyzing generic class types), this function currently does *not* merge or overwrite them.
/// It only initializes the map if it's empty.
fn refine_template_result_for_function_like(
    template_result: &mut TemplateResult,
    context: &Context<'_>,
    method_target_context: Option<&MethodTargetContext<'_>>,
    base_class_metadata: Option<&ClassLikeMetadata>,
    calling_class_like_metadata: Option<&ClassLikeMetadata>,
    function_like_metadata: &FunctionLikeMetadata,
    class_template_parameters: &IndexMap<StringIdentifier, Vec<(GenericParent, TUnion)>, RandomState>,
) {
    if !template_result.template_types.is_empty() {
        return;
    }

    let resolved_template_types = get_template_types_for_class_member(
        context,
        base_class_metadata,
        method_target_context.as_ref().map(|mci| &mci.class_like_metadata.name),
        calling_class_like_metadata,
        &function_like_metadata.template_types,
        class_template_parameters,
    );

    if resolved_template_types.is_empty() {
        return;
    }

    template_result.template_types = resolved_template_types
        .into_iter()
        .map(|(template_name, type_map)| (template_name, type_map.into_iter().collect()))
        .collect::<IndexMap<_, _, RandomState>>();
}

/// Determines the resulting element type when an argument is unpacked using the spread operator (`...`).
///
/// Iterates through the atomic types of the `$argument_value_type` (the variable being unpacked).
/// - For known iterable array types (`list`, `array`), it extracts the value type parameter.
/// - For `mixed` or `any`, it reports an error as iterability cannot be guaranteed and returns `mixed`/`any`.
/// - For `never`, it returns `never`.
/// - For any other non-iterable type, it reports an error and returns `mixed`.
///
/// The function combines the potential element types derived from all parts of the input union.
///
/// # Arguments
///
/// * `context` - Analysis context, used for reporting issues and accessing codebase/interner.
/// * `argument_value_type` - The inferred type union of the expression being unpacked.
/// * `span` - The span of the unpacked argument expression (`...$arg`) for error reporting.
///
/// # Returns
///
/// A `TUnion` representing the combined type of the elements within the unpacked iterable.
fn get_unpacked_argument_type(context: &mut Context<'_>, argument_value_type: &TUnion, span: Span) -> TUnion {
    let mut potential_element_types = Vec::new();
    let mut reported_an_error = false;

    for atomic_type in &argument_value_type.types {
        if let Some(value_parameter) = get_iterable_value_parameter(atomic_type, context.codebase, context.interner) {
            potential_element_types.push(value_parameter);

            continue;
        }

        match atomic_type {
            TAtomic::Never => {
                potential_element_types.push(get_never());
            }
            TAtomic::Mixed(_) => {
                if !reported_an_error {
                    context.collector.report_with_code(
                        Code::MIXED_ARGUMENT,
                        Issue::error(format!(
                            "Cannot unpack argument of type `{}` because it is not guaranteed to be iterable.",
                            atomic_type.get_id(Some(context.interner))
                        ))
                        .with_annotation(Annotation::primary(span).with_message("Expected an `iterable` for unpacking"))
                        .with_note("Argument unpacking `...` requires an `iterable` (e.g., `array` or `Traversable`).")
                        .with_note("The type `mixed` provides no guarantee of iterability.")
                        .with_help("Ensure the value is an `iterable` using type hints, checks, or assertions."),
                    );
                    reported_an_error = true;
                }

                potential_element_types.push(get_mixed());
            }
            _ => {
                if !reported_an_error {
                    let type_str = atomic_type.get_id(Some(context.interner));
                    context.collector.report_with_code(
                        Code::INVALID_ARGUMENT,
                        Issue::error(format!(
                            "Cannot unpack argument of type `{type_str}` because it is not an iterable type."
                        ))
                        .with_annotation(
                            Annotation::primary(span).with_message(format!("Type `{type_str}` is not `iterable`")),
                        )
                        .with_note("Argument unpacking `...` requires an `iterable` (e.g., `array` or `Traversable`).")
                        .with_help("Ensure the value being unpacked is an `iterable`."),
                    );

                    reported_an_error = true;
                }

                potential_element_types.push(get_mixed());
            }
        }
    }

    if let Some(mut combined_type) = potential_element_types.pop() {
        for element_type in potential_element_types {
            combined_type = add_union_type(combined_type, &element_type, context.codebase, context.interner, false);
        }

        combined_type
    } else {
        get_never()
    }
}

/// Checks the consistency of inferred template parameter bounds.
///
/// This function analyzes the collected lower bounds (`T >: X`) and upper bounds (`T <: Y`, `T = Z`)
/// for each template parameter (`T`) within a `TemplateResult`. It reports errors if conflicting
/// bounds are found, such as:
///
/// - A lower bound that is not a subtype of an upper bound (`X` not assignable to `Y`).
/// - Multiple incompatible equality bounds (`T = int` and `T = string`).
/// - A lower bound that is not compatible with an equality bound (`T >: float` and `T = int`).
///
/// # Arguments
///
/// * `context` - The analysis context, providing access to the codebase and interner.
/// * `template_result` - The result containing the bounds to check (will be mutated if bounds are added).
/// * `span` - The span (location) to associate with any reported errors (e.g., the call site).
fn check_template_result(context: &mut Context<'_>, template_result: &mut TemplateResult, span: Span) {
    if template_result.lower_bounds.is_empty() {
        return;
    }

    let codebase = context.codebase;
    let interner = context.interner;

    for (template_name, defining_map) in &template_result.upper_bounds {
        for (defining_entity, upper_bound) in defining_map {
            let lower_bounds = template_result
                .lower_bounds
                .entry(*template_name)
                .or_default()
                .entry(*defining_entity)
                .or_insert_with(|| vec![TemplateBound::of_type(upper_bound.bound_type.clone())]);

            let (lower_bound_type, upper_bound_type) = if template_result.upper_bounds_unintersectable_types.len() > 1 {
                (
                    Cow::Borrowed(&template_result.upper_bounds_unintersectable_types[0]),
                    Cow::Borrowed(&template_result.upper_bounds_unintersectable_types[1]),
                )
            } else {
                (
                    Cow::Owned(get_most_specific_type_from_bounds(lower_bounds, codebase, interner)),
                    Cow::Borrowed(&upper_bound.bound_type),
                )
            };

            let mut comparison_result = ComparisonResult::new();
            let is_contained = union_comparator::is_contained_by(
                codebase,
                interner,
                &lower_bound_type,
                &upper_bound_type,
                false,
                false,
                false,
                &mut comparison_result,
            );

            if !is_contained {
                let issue_kind = if comparison_result.type_coerced.unwrap_or(false)
                    && comparison_result.type_coerced_from_as_mixed.unwrap_or(false)
                {
                    Code::MIXED_ARGUMENT
                } else {
                    Code::INVALID_ARGUMENT
                };

                context.collector.report_with_code(
                    issue_kind,
                    Issue::error(format!("Incompatible template bounds for `{}`.", interner.lookup(template_name)))
                        .with_annotation(Annotation::primary(span).with_message(format!(
                            "Inferred type `{}` is not compatible with declared bound `{}`",
                            lower_bound_type.get_id(Some(interner)),
                            upper_bound_type.get_id(Some(interner)),
                        )))
                        .with_note(format!(
                            "Could not reconcile bounds for template parameter `{}`.",
                            interner.lookup(template_name),
                        ))
                        .with_help(
                            "Check the types used for arguments or properties related to this template parameter.",
                        ),
                );
            }
        }
    }

    for (template_name, lower_bounds_map) in &template_result.lower_bounds {
        for lower_bounds in lower_bounds_map.values() {
            if lower_bounds.len() <= 1 {
                continue;
            }

            let bounds_with_equality: Vec<_> =
                lower_bounds.iter().filter(|bound| bound.equality_bound_classlike.is_some()).collect();

            if !bounds_with_equality.is_empty() {
                let equality_types: Vec<String> =
                    unique_vec(bounds_with_equality.iter().map(|bound| bound.bound_type.get_id(Some(interner))));

                if equality_types.len() > 1 {
                    context.collector.report_with_code(
                        Code::CONFLICTING_TEMPLATE_EQUALITY_BOUNDS,
                        Issue::error(format!(
                            "Conflicting equality requirements found for template `{}`.",
                            interner.lookup(template_name)
                        ))
                        .with_annotation(Annotation::primary(span).with_message(format!(
                            "Template `{}` cannot be equal to all of: `{}`.",
                            interner.lookup(template_name),
                            equality_types.join("`, `"),
                        )))
                        .with_help(
                            "Check the argument types provided for this template parameter; they must resolve to a single compatible type."
                        ),
                    );

                    continue;
                }
            }

            if let Some(first_equality_bound) = bounds_with_equality.first() {
                for lower_bound in lower_bounds {
                    if lower_bound.equality_bound_classlike.is_some() {
                        continue;
                    }

                    let is_contained = union_comparator::is_contained_by(
                        codebase,
                        interner,
                        &lower_bound.bound_type,
                        &first_equality_bound.bound_type,
                        false,
                        false,
                        false,
                        &mut ComparisonResult::new(),
                    );

                    if !is_contained {
                        context.collector.report_with_code(
                            Code::INCOMPATIBLE_TEMPLATE_LOWER_BOUND,
                            Issue::error(format!(
                                "Incompatible bounds found for template `{}`.",
                                interner.lookup(template_name)
                            ))
                            .with_annotation(Annotation::primary(span).with_message(format!(
                                "Type `{}` required by a lower bound is not compatible with the required equality type `{}`.",
                                lower_bound.bound_type.get_id(Some(interner)),
                                first_equality_bound.bound_type.get_id(Some(interner)),
                            )))
                            .with_help(
                                "Check the argument types provided; they must satisfy all lower and equality bounds simultaneously."
                            ),
                        );
                    }
                }
            }
        }
    }
}
