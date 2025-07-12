use std::borrow::Cow;
use std::rc::Rc;

use ahash::HashMap;
use ahash::RandomState;
use indexmap::IndexMap;
use itertools::Itertools;

use mago_codex::data_flow::graph::DataFlowGraph;
use mago_codex::data_flow::graph::GraphKind;
use mago_codex::data_flow::node::DataFlowNode;
use mago_codex::data_flow::node::DataFlowNodeId;
use mago_codex::data_flow::node::DataFlowNodeKind;
use mago_codex::data_flow::path::PathKind;
use mago_codex::get_class_like;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::is_instance_of;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_codex::misc::GenericParent;
use mago_codex::ttype::add_union_type;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::callable::TCallable;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::class_like_string::TClassLikeString;
use mago_codex::ttype::atomic::scalar::int::TInteger;
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::atomic_comparator;
use mago_codex::ttype::comparator::union_comparator;
use mago_codex::ttype::comparator::union_comparator::can_expression_types_be_identical;
use mago_codex::ttype::comparator::union_comparator::is_contained_by;
use mago_codex::ttype::expander;
use mago_codex::ttype::expander::StaticClassType;
use mago_codex::ttype::expander::TypeExpansionOptions;
use mago_codex::ttype::expander::get_signature_of_function_like_identifier;
use mago_codex::ttype::template::TemplateBound;
use mago_codex::ttype::template::TemplateResult;
use mago_codex::ttype::template::inferred_type_replacer;
use mago_codex::ttype::template::standin_type_replacer::StandinOptions;
use mago_codex::ttype::template::standin_type_replacer::get_most_specific_type_from_bounds;
use mago_codex::ttype::template::standin_type_replacer::insert_bound_type;
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
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::expression::array_access::add_array_access_dataflow;
use crate::invocation::Invocation;
use crate::invocation::InvocationArgument;
use crate::invocation::InvocationArgumentsSource;
use crate::invocation::InvocationTarget;
use crate::invocation::InvocationTargetParameter;
use crate::invocation::MethodTargetContext;
use crate::issue::TypingIssueKind;
use crate::utils::misc::unique_vec;
use crate::utils::template::get_template_types_for_class_member;

/// Analyzes and verifies arguments passed to a function, method, callable, or language construct.
///
/// Performs a multi-pass analysis:
///
/// 1. Separates arguments into positional (non-callable), callable, and unpacked categories.
/// 2. **Pass 1:** Analyzes non-callable arguments and infers initial template bounds into `template_result`.
/// 3. **Pass 2:** Analyzes callable arguments. It resolves the expected parameter signature using bounds
///    inferred in Pass 1, verifies the provided callable against the resolved signature (respecting variance),
///    and infers any remaining template bounds (e.g., for return types) *without* overriding bounds
///    set in Pass 1.
/// 4. **Refinement:** Applies class/function template definitions to the `template_result`.
/// 5. **Pass 3:** Verifies all positional arguments (non-callable and callable) against their
///    parameter types, which are now fully resolved using the final `template_result`.
/// 6. Verifies unpacked arguments against the (resolved) variadic parameter type.
/// 7. Performs a final consistency check on the inferred template bounds in `template_result`.
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

    populate_template_result_from_invocation(invocation, template_result);

    let parameter_refs = invocation.target.get_parameters();
    let mut analyzed_argument_types: HashMap<usize, (TUnion, Span)> = HashMap::default();

    let mut non_callable_arguments: Vec<(usize, InvocationArgument<'_>)> = Vec::new();
    let mut closure_arguments: Vec<(usize, InvocationArgument<'_>)> = Vec::new();
    let mut unpacked_arguments: Vec<InvocationArgument<'_>> = Vec::new();
    for (offset, argument) in invocation.arguments_source.get_arguments().into_iter().enumerate() {
        if argument.is_unpacked() {
            unpacked_arguments.push(argument);
        } else if matches!(
            argument.value(),
            Expression::Closure(_) | Expression::ArrowFunction(_) | Expression::ClosureCreation(_)
        ) {
            closure_arguments.push((offset, argument));
        } else {
            non_callable_arguments.push((offset, argument));
        }
    }

    let calling_class_like_metadata =
        calling_class_like.and_then(|(id, _)| get_class_like(context.codebase, context.interner, &id));
    let base_class_metadata =
        invocation.target.get_method_context().map(|ctx| ctx.class_like_metadata).or(calling_class_like_metadata);
    let method_call_context = invocation.target.get_method_context();

    for (argument_offset, argument) in &non_callable_arguments {
        let argument_expression = argument.value();
        let parameter = get_parameter_of_argument(context, &parameter_refs, argument, *argument_offset);

        analyze_and_store_argument_type(
            context,
            block_context,
            artifacts,
            argument_expression,
            *argument_offset,
            &mut analyzed_argument_types,
            parameter.is_some_and(|p| p.1.is_by_reference()),
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
                infer_templates_from_argument_and_parameter_types(
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

        analyze_and_store_argument_type(
            context,
            block_context,
            artifacts,
            argument_expression,
            *argument_offset,
            &mut analyzed_argument_types,
            parameter.is_some_and(|p| p.1.is_by_reference()),
        )?;

        if let Some(argument_type) = analyzed_argument_types.get(argument_offset)
            && let Some((_, parameter_ref)) = parameter
        {
            let base_parameter_type = get_parameter_type(
                context,
                Some(parameter_ref),
                base_class_metadata,
                calling_class_like_metadata,
                calling_class_like.and_then(|(_, atomic)| atomic),
            );

            if base_parameter_type.has_template_types() {
                let resolved_parameter_type = inferred_type_replacer::replace(
                    &base_parameter_type,
                    template_result,
                    context.codebase,
                    context.interner,
                );

                infer_templates_from_argument_and_parameter_types(
                    context,
                    &resolved_parameter_type,
                    &argument_type.0,
                    template_result,
                    *argument_offset,
                    argument_type.1,
                    true,
                );
            }
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
        non_callable_arguments.iter().chain(closure_arguments.iter()).sorted_by(|(a, _), (b, _)| a.cmp(b))
    {
        let argument_expression = argument.value();
        let (argument_value_type, _) = analyzed_argument_types
            .get(argument_offset)
            .cloned()
            .unwrap_or_else(|| (get_mixed_any(), argument_expression.span()));

        let parameter_ref = get_parameter_of_argument(context, &parameter_refs, argument, *argument_offset);
        if let Some((parameter_offset, parameter_ref)) = parameter_ref {
            if let Some(parameter_name) = parameter_ref.get_name() {
                parameter_types.insert(parameter_name.0, argument_value_type.clone());
            }

            if let Some(named_argument) = argument.get_named_argument() {
                if let Some(previous_span) = assigned_parameters_by_name.get(&named_argument.name.value) {
                    context.buffer.report(
                        TypingIssueKind::DuplicateNamedArgument,
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
                            context.buffer.report(
                                TypingIssueKind::NamedArgumentOverridesPositional,
                                Issue::error(format!(
                                    "Named argument `${}` for {} `{}` targets a parameter already provided positionally.",
                                    context.interner.lookup(&named_argument.name.value), target_kind_str, target_name_str
                                ))
                                .with_annotation(Annotation::primary(named_argument.name.span()).with_message("This named argument"))
                                .with_annotation(Annotation::secondary(*previous_span).with_message("Parameter already filled by positional argument here"))
                                .with_help("Provide the argument either positionally or by name, but not both."),
                            );
                        } else {
                            context.buffer.report(
                                TypingIssueKind::NamedArgumentForVariadicAfterPositional,
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

            let final_parameter_type = inferred_type_replacer::replace(
                &base_parameter_type,
                template_result,
                context.codebase,
                context.interner,
            );

            verify_argument_type(
                context,
                block_context,
                &mut artifacts.data_flow_graph,
                &mut artifacts.type_variable_bounds,
                &argument_value_type,
                &final_parameter_type,
                *argument_offset,
                argument_expression,
                &parameter_ref,
                &invocation.target,
            );
        } else if let Some(named_argument) = argument.get_named_argument() {
            let argument_name = context.interner.lookup(&named_argument.name.value);

            context.buffer.report(
                TypingIssueKind::InvalidNamedArgument,
                Issue::error(format!(
                    "Unknown named argument `${argument_name}` in call to {target_kind_str} `{target_name_str}`.",
                ))
                .with_annotation(
                    Annotation::primary(named_argument.name.span())
                        .with_message(format!("Unknown argument name `${argument_name}`",)),
                )
                .with_annotation(
                    Annotation::secondary(invocation.target.span()).with_message(format!("For this {target_kind_str}")),
                )
                .with_help(
                    if parameter_refs.is_empty() || !invocation.target.allows_named_arguments() {
                        format!(
                            "The {target_kind_str} `{target_name_str}` expects no arguments or named arguments are not supported.",
                        )
                    } else {
                        format!(
                            "Available parameters are: `{}`.",
                            parameter_refs
                                .iter()
                                .filter_map(|p| p.get_name())
                                .map(|n| format!("${}", context.interner.lookup(&n.0).trim_start_matches('$')))
                                .collect::<Vec<_>>()
                                .join("`, `"),
                        )
                    },
                ),
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

            let Some(parameter_name) = unused_parameter.get_name() else {
                continue;
            };

            if parameter_types.contains_key(&parameter_name.0) {
                continue;
            }

            let Some(default_type) = unused_parameter.get_default_type() else {
                break;
            };

            parameter_types.insert(parameter_name.0, default_type.clone());
        }
    }

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
                            argument_expression,
                            usize::MAX,
                            &mut analyzed_argument_types,
                            last_parameter_ref.is_by_reference(),
                        )?;
                    }

                    let argument_value_type =
                        artifacts.get_expression_type(argument_expression).cloned().unwrap_or_else(get_mixed_any); // Get type of the iterable

                    let unpacked_element_type = get_unpacked_argument_type(
                        context,
                        &artifacts.expression_types,
                        &mut artifacts.data_flow_graph,
                        &argument_value_type,
                        argument_expression.span(),
                    );

                    verify_argument_type(
                        context,
                        block_context,
                        &mut artifacts.data_flow_graph,
                        &mut artifacts.type_variable_bounds,
                        &unpacked_element_type,
                        &final_variadic_parameter_type,
                        parameter_refs.len() - 1,
                        argument_expression,
                        &last_parameter_ref,
                        &invocation.target,
                    );
                }
            } else {
                context.buffer.report(
                    TypingIssueKind::TooManyArguments,
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
            context.buffer.report(
                TypingIssueKind::TooManyArguments,
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

    let max_params = parameter_refs.len();
    let number_of_required_parameters =
        parameter_refs.iter().filter(|parameter| !parameter.has_default() && !parameter.is_variadic()).count();
    let number_of_provided_parameters = non_callable_arguments.len() + closure_arguments.len();
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
            InvocationArgumentsSource::LanguageConstructExpressions(_) => issue.with_annotation(
                Annotation::secondary(invocation.target.span()).with_message("For this language construct"),
            ),
            InvocationArgumentsSource::None(constructor_or_attribute_span) => issue.with_annotation(
                Annotation::secondary(constructor_or_attribute_span)
                    .with_message(format!("For this {target_kind_str}")),
            ),
        };

        issue = issue.with_help("Provide all required arguments.");
        context.buffer.report(TypingIssueKind::TooFewArguments, issue);
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
        } else if let InvocationArgumentsSource::LanguageConstructExpressions { .. } = invocation.arguments_source {
            issue = issue.with_annotation(
                Annotation::secondary(invocation.target.span())
                    .with_message(format!("For this {target_name_str} construct")),
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

        context.buffer.report(TypingIssueKind::TooManyArguments, issue);
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
pub fn populate_template_result_from_invocation(invocation: &Invocation<'_>, template_result: &mut TemplateResult) {
    if let InvocationTarget::FunctionLike { metadata, method_context, .. } = &invocation.target {
        for (template_name, template_details) in metadata.template_types.iter() {
            template_result.template_types.insert(*template_name, template_details.clone());
        }

        if let Some(method_metadata) = &metadata.method_metadata
            && !method_metadata.is_static()
            && let Some(method_context) = method_context
        {
            for (template_name, template_defaults) in method_context.class_like_metadata.template_types.iter() {
                template_result
                    .template_types
                    .entry(*template_name)
                    .or_default()
                    .extend(template_defaults.iter().cloned());
            }

            if let StaticClassType::Object(object_type) = &method_context.class_type
                && let TObject::Named(named_object) = object_type
                && let Some(type_parameters) = &named_object.type_parameters
            {
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
        }
    }
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
    argument_expression: &Expression,
    argument_offset: usize,
    analyzed_argument_types: &mut HashMap<usize, (TUnion, Span)>,
    referenced_parameter: bool,
) -> Result<(), AnalysisError> {
    if argument_offset != usize::MAX && analyzed_argument_types.contains_key(&argument_offset) {
        return Ok(());
    }

    let was_inside_general_use = block_context.inside_general_use;
    let was_inside_call = block_context.inside_call;
    let was_inside_variable_reference = block_context.inside_variable_reference;

    block_context.inside_general_use = true;
    block_context.inside_call = true;
    block_context.inside_variable_reference = referenced_parameter;

    argument_expression.analyze(context, block_context, artifacts)?;

    block_context.inside_general_use = was_inside_general_use;
    block_context.inside_call = was_inside_call;
    block_context.inside_variable_reference = was_inside_variable_reference;

    let argument_type = artifacts.get_expression_type(argument_expression).cloned().unwrap_or_else(get_mixed_any);

    if argument_offset != usize::MAX {
        analyzed_argument_types.insert(argument_offset, (argument_type, argument_expression.span()));
    }

    Ok(())
}

/// Infers template types by comparing a function's parameter type against a provided argument type.
///
/// This function is responsible for determining the concrete types of generic parameters (like `T`)
/// based on the actual argument passed to a function.
///
/// A key feature of this algorithm is its ability to produce more precise inferences by "subtracting"
/// concrete types that are common to both the parameter and the argument before matching generics.
///
/// ### Example of the core logic
///
/// - If a function parameter's type is `string | int | T`.
/// - And the passed argument's type is `string | int | float`.
///
/// A naive inference might resolve `T` to the entire argument type `string | int | float`.
///
/// This function, however, identifies that `string` and `int` are concrete types present in both.
/// It removes them from consideration, leaving `T` from the parameter and `float` from the argument.
/// This leads to a much more precise inference where `T` is correctly resolved to `float`.
///
/// This same reconciliation logic is applied recursively for nested types like arrays, callables, and objects.
///
/// ### Arguments
///
/// - `context`: The shared analysis context, containing the codebase and interner.
/// - `parameter_type`: The type of the function/method parameter, which may contain generics.
/// - `argument_type`: The type of the argument that was actually passed.
/// - `template_result`: A mutable struct where the results of the inference (the "lower bounds" for each template) will be stored.
/// - `argument_offset`: The argument's position in the function call.
/// - `argument_span`: The source code location of the argument, used for error reporting.
/// - `is_callable_argument`: A flag indicating if the argument is a callable, which has slightly different inference rules for bounds.
fn infer_templates_from_argument_and_parameter_types(
    context: &mut Context<'_>,
    parameter_type: &TUnion,
    argument_type: &TUnion,
    template_result: &mut TemplateResult,
    argument_offset: usize,
    argument_span: Span,
    is_callable_argument: bool,
) {
    if argument_type.is_mixed_with_any(&mut false) {
        return;
    }

    let (generic_parameter_parts, concrete_parameter_parts) = parameter_type.types.iter().partition::<Vec<_>, _>(|t| {
        matches!(
            t,
            TAtomic::GenericParameter(_)
                | TAtomic::Array(_)
                | TAtomic::Iterable(_)
                | TAtomic::Object(TObject::Named(_))
                | TAtomic::Callable(_)
                | TAtomic::Scalar(TScalar::ClassLikeString(_))
        )
    });

    let residual_argument_type = TUnion::new(
        argument_type
            .types
            .iter()
            .filter(|argument_atomic| {
                !concrete_parameter_parts.iter().any(|parameter_atomic| {
                    atomic_comparator::is_contained_by(
                        context.codebase,
                        context.interner,
                        parameter_atomic,
                        argument_atomic,
                        false,
                        &mut ComparisonResult::default(),
                    )
                })
            })
            .cloned()
            .collect::<Vec<_>>(),
    );

    // A map to hold potential violations, to be processed only if no other valid inference is found.
    let mut potential_template_violations = std::collections::HashMap::new();

    for parameter_atomic in generic_parameter_parts {
        match parameter_atomic {
            TAtomic::GenericParameter(parameter_generic_parameter) => {
                let template_parameter_name = &parameter_generic_parameter.parameter_name;

                let should_add_bound = !is_callable_argument
                    || template_result
                        .lower_bounds
                        .get(template_parameter_name)
                        .and_then(|map| map.get(&parameter_generic_parameter.defining_entity))
                        .is_none_or(|bounds| bounds.is_empty());

                if should_add_bound {
                    let mut has_violation = false;

                    if let Some(template_types) = template_result.template_types.get_mut(template_parameter_name) {
                        for (_, template_type) in template_types {
                            if !is_contained_by(
                                context.codebase,
                                context.interner,
                                &residual_argument_type,
                                template_type,
                                false,
                                false,
                                false,
                                &mut ComparisonResult::default(),
                            ) {
                                potential_template_violations
                                    .entry((*template_parameter_name, parameter_generic_parameter.defining_entity))
                                    .or_insert_with(|| {
                                        (
                                            residual_argument_type.clone(),
                                            template_type.clone(),
                                            parameter_generic_parameter.clone(),
                                        )
                                    });

                                has_violation = true;
                                break;
                            }
                        }
                    }

                    if !has_violation {
                        insert_bound_type(
                            template_result,
                            *template_parameter_name,
                            &parameter_generic_parameter.defining_entity,
                            residual_argument_type.clone(),
                            StandinOptions { appearance_depth: 1, ..Default::default() },
                            Some(argument_offset),
                            Some(argument_span),
                        );
                    }
                }
            }
            TAtomic::Array(parameter_array) => {
                for argument_atomic in &residual_argument_type.types {
                    if let TAtomic::Array(argument_array) = argument_atomic {
                        match (parameter_array, argument_array) {
                            (TArray::List(_), TArray::List(_)) => {
                                infer_templates_from_argument_and_parameter_types(
                                    context,
                                    &get_array_value_parameter(parameter_array, context.codebase, context.interner),
                                    &get_array_value_parameter(argument_array, context.codebase, context.interner),
                                    template_result,
                                    argument_offset,
                                    argument_span,
                                    is_callable_argument,
                                );
                            }
                            (TArray::Keyed(_), TArray::Keyed(_)) => {
                                let (parameter_key, parameter_value) =
                                    get_array_parameters(parameter_array, context.codebase, context.interner);
                                let (argument_key, argument_value) =
                                    get_array_parameters(argument_array, context.codebase, context.interner);

                                infer_templates_from_argument_and_parameter_types(
                                    context,
                                    &parameter_key,
                                    &argument_key,
                                    template_result,
                                    argument_offset,
                                    argument_span,
                                    is_callable_argument,
                                );

                                infer_templates_from_argument_and_parameter_types(
                                    context,
                                    &parameter_value,
                                    &argument_value,
                                    template_result,
                                    argument_offset,
                                    argument_span,
                                    is_callable_argument,
                                );
                            }
                            (TArray::List(_), TArray::Keyed(_)) => {
                                let parameter_value_type =
                                    get_array_value_parameter(parameter_array, context.codebase, context.interner);
                                let (_, argument_value_type) =
                                    get_array_parameters(argument_array, context.codebase, context.interner);

                                infer_templates_from_argument_and_parameter_types(
                                    context,
                                    &parameter_value_type,
                                    &argument_value_type,
                                    template_result,
                                    argument_offset,
                                    argument_span,
                                    is_callable_argument,
                                );
                            }
                            (TArray::Keyed(_), TArray::List(_)) => {
                                let (parameter_key_type, parameter_value_type) =
                                    get_array_parameters(parameter_array, context.codebase, context.interner);
                                let argument_value_type =
                                    get_array_value_parameter(argument_array, context.codebase, context.interner);

                                infer_templates_from_argument_and_parameter_types(
                                    context,
                                    &parameter_key_type,
                                    &TUnion::new(vec![TAtomic::Scalar(TScalar::Integer(TInteger::non_negative()))]),
                                    template_result,
                                    argument_offset,
                                    argument_span,
                                    is_callable_argument,
                                );

                                infer_templates_from_argument_and_parameter_types(
                                    context,
                                    &parameter_value_type,
                                    &argument_value_type,
                                    template_result,
                                    argument_offset,
                                    argument_span,
                                    is_callable_argument,
                                );
                            }
                        }
                    }
                }
            }
            TAtomic::Iterable(parameter_iterable) => {
                for argument_atomic in &residual_argument_type.types {
                    let Some((argument_key, argument_value)) =
                        get_iterable_parameters(argument_atomic, context.codebase, context.interner)
                    else {
                        return;
                    };

                    infer_templates_from_argument_and_parameter_types(
                        context,
                        parameter_iterable.get_key_type(),
                        &argument_key,
                        template_result,
                        argument_offset,
                        argument_span,
                        is_callable_argument,
                    );

                    infer_templates_from_argument_and_parameter_types(
                        context,
                        parameter_iterable.get_value_type(),
                        &argument_value,
                        template_result,
                        argument_offset,
                        argument_span,
                        is_callable_argument,
                    );
                }
            }
            TAtomic::Callable(parameter_callable) => {
                let parameter_signature = match parameter_callable {
                    TCallable::Signature(signature) => Cow::Borrowed(signature),
                    TCallable::Alias(id) => {
                        let Some(signature) =
                            get_signature_of_function_like_identifier(id, context.codebase, context.interner)
                        else {
                            continue;
                        };

                        Cow::Owned(signature)
                    }
                };

                for argument_atomic in &residual_argument_type.types {
                    let argument_signature = match argument_atomic {
                        TAtomic::Callable(TCallable::Signature(argument_signature)) => {
                            Cow::Borrowed(argument_signature)
                        }
                        TAtomic::Callable(TCallable::Alias(id)) => {
                            let Some(signature) =
                                get_signature_of_function_like_identifier(id, context.codebase, context.interner)
                            else {
                                continue;
                            };

                            Cow::Owned(signature)
                        }
                        _ => continue,
                    };

                    let parameter_parameters = parameter_signature.get_parameters();
                    let argument_parameters = argument_signature.get_parameters();

                    let parameter_count = parameter_parameters.len();
                    let argument_count = argument_parameters.iter().filter(|s| !s.has_default()).count();
                    let minimum_count = std::cmp::min(parameter_count, argument_count);
                    for i in 0..minimum_count {
                        let Some(parameter_parameter) = parameter_parameters.get(i) else {
                            continue;
                        };

                        let Some(argument_parameter) = argument_parameters.get(i) else {
                            continue;
                        };

                        let Some(parameter_parameter_type) = parameter_parameter.get_type_signature() else {
                            continue;
                        };

                        let Some(argument_parameter_type) = argument_parameter.get_type_signature() else {
                            continue;
                        };

                        infer_templates_from_argument_and_parameter_types(
                            context,
                            parameter_parameter_type,
                            argument_parameter_type,
                            template_result,
                            argument_offset,
                            argument_span,
                            true,
                        );
                    }

                    let Some(parameter_return) = parameter_signature.get_return_type() else {
                        continue;
                    };

                    let Some(argument_return) = argument_signature.get_return_type() else {
                        continue;
                    };

                    infer_templates_from_argument_and_parameter_types(
                        context,
                        parameter_return,
                        argument_return,
                        template_result,
                        argument_offset,
                        argument_span,
                        true,
                    );
                }
            }
            TAtomic::Object(TObject::Named(parameter_object)) if parameter_object.has_type_parameters() => {
                let Some(parameter_class_metadata) =
                    get_class_like(context.codebase, context.interner, &parameter_object.name)
                else {
                    return;
                };

                let Some(parameter_type_parameters) = parameter_object.get_type_parameters() else {
                    return;
                };

                for argument_atomic in &residual_argument_type.types {
                    let TAtomic::Object(TObject::Named(argument_object)) = argument_atomic else {
                        continue;
                    };

                    let Some(argument_class_metadata) =
                        get_class_like(context.codebase, context.interner, &argument_object.name)
                    else {
                        continue;
                    };

                    if !is_instance_of(
                        context.codebase,
                        context.interner,
                        &argument_object.name,
                        &parameter_object.name,
                    ) {
                        continue;
                    }

                    for (index, parameter_template_union) in parameter_type_parameters.iter().enumerate() {
                        let generic_parameters =
                            parameter_template_union.types.iter().filter_map(|atomic| match atomic {
                                TAtomic::GenericParameter(generic_parameter) => Some(generic_parameter),
                                _ => None,
                            });

                        for generic_parameter in generic_parameters {
                            let Some((template_name, _)) = parameter_class_metadata.template_types.get(index) else {
                                continue;
                            };

                            if let Some(inferred_bound) = get_specialized_template_type(
                                context.codebase,
                                context.interner,
                                context.interner.lookup(template_name),
                                &parameter_class_metadata.name,
                                argument_class_metadata,
                                argument_object.get_type_parameters(),
                            ) {
                                if !is_contained_by(
                                    context.codebase,
                                    context.interner,
                                    &inferred_bound,
                                    &generic_parameter.constraint,
                                    false,
                                    false,
                                    false,
                                    &mut ComparisonResult::default(),
                                ) {
                                    context.buffer.report(
                                        TypingIssueKind::TemplateConstraintViolation,
                                        Issue::error(format!(
                                            "Inferred type for template `{}` does not satisfy its constraint.",
                                            context.interner.lookup(template_name),
                                        ))
                                        .with_annotation(
                                            Annotation::primary(argument_span)
                                                .with_message(format!(
                                                    "This argument's type `{}` provides `{}` for the template parameter `{}`, but the constraint is `{}`.",
                                                    context.interner.lookup(&argument_class_metadata.original_name),
                                                    inferred_bound.get_id(Some(context.interner)),
                                                    context.interner.lookup(template_name),
                                                    generic_parameter.constraint.get_id(Some(context.interner)),
                                                )),
                                        )
                                        .with_note(format!(
                                            "The template parameter `{}` is constrained to be `{}`, but the inferred type `{}` does not satisfy this constraint.",
                                            context.interner.lookup(template_name),
                                            generic_parameter.constraint.get_id(Some(context.interner)),
                                            inferred_bound.get_id(Some(context.interner)),
                                        ))
                                        .with_help("Ensure the provided argument specializes the template with a type that matches the constraint."),
                                    );
                                }

                                insert_bound_type(
                                    template_result,
                                    generic_parameter.parameter_name,
                                    &generic_parameter.defining_entity,
                                    inferred_bound,
                                    StandinOptions { appearance_depth: 1, ..Default::default() },
                                    Some(argument_offset),
                                    Some(argument_span),
                                );
                            }
                        }
                    }
                }
            }
            TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::Generic {
                parameter_name,
                defining_entity,
                ..
            })) => {
                let should_add_bound = !is_callable_argument
                    || template_result
                        .lower_bounds
                        .get(parameter_name)
                        .and_then(|map| map.get(defining_entity))
                        .is_none_or(|bounds| bounds.is_empty());

                let mut argument_object_atomics = vec![];
                for argument_atomic in residual_argument_type.types.iter() {
                    let TAtomic::Scalar(TScalar::ClassLikeString(class_string)) = argument_atomic else {
                        continue;
                    };

                    argument_object_atomics.push(class_string.get_object_type());
                }

                let mut lower_bound_type = TUnion::new(argument_object_atomics);

                if should_add_bound {
                    if let Some(template_types) = template_result.template_types.get_mut(parameter_name) {
                        for (_, template_type) in template_types {
                            if !is_contained_by(
                                context.codebase,
                                context.interner,
                                &lower_bound_type,
                                template_type,
                                false,
                                false,
                                false,
                                &mut ComparisonResult::default(),
                            ) {
                                lower_bound_type = template_type.clone();

                                context.buffer.report(
                                    TypingIssueKind::TemplateConstraintViolation,
                                    Issue::error(format!(
                                        "Argument type mismatch for class string of `{}`.",
                                        context.interner.lookup(parameter_name),
                                    ))
                                    .with_annotation(
                                        Annotation::primary(argument_span)
                                            .with_message(format!(
                                                "This argument has type `{}`, which is not compatible with the required class string constraint `{}`.",
                                                argument_type.get_id(Some(context.interner)),
                                                template_type.get_id(Some(context.interner))
                                            ))
                                    )
                                    .with_note(format!(
                                        "Class string parameter `{}` is constrained with `{}`.",
                                        context.interner.lookup(parameter_name),
                                        template_type.get_id(Some(context.interner))
                                    ))
                                    .with_help("Ensure the argument's type satisfies the class string constraint."),
                                );
                            }
                        }
                    }

                    insert_bound_type(
                        template_result,
                        *parameter_name,
                        defining_entity,
                        lower_bound_type,
                        StandinOptions { appearance_depth: 1, ..Default::default() },
                        Some(argument_offset),
                        Some(argument_span),
                    );
                }
            }
            _ => {}
        }
    }

    for ((template_parameter_name, defining_entity), (inferred_type, constraint, _)) in potential_template_violations {
        let is_unresolved = template_result
            .lower_bounds
            .get(&template_parameter_name)
            .and_then(|map| map.get(&defining_entity))
            .is_none_or(|bounds| bounds.is_empty());

        if is_unresolved {
            context.buffer.report(
                TypingIssueKind::TemplateConstraintViolation,
                Issue::error(format!(
                    "Argument type mismatch for template `{}`.",
                    context.interner.lookup(&template_parameter_name),
                ))
                .with_annotation(Annotation::primary(argument_span).with_message(format!(
                    "This argument has type `{}`, which is not compatible with the required template constraint `{}`.",
                    inferred_type.get_id(Some(context.interner)),
                    constraint.get_id(Some(context.interner))
                )))
                .with_note(format!(
                    "Template parameter `{}` is constrained with `{}`.",
                    context.interner.lookup(&template_parameter_name),
                    constraint.get_id(Some(context.interner))
                ))
                .with_help("Ensure the argument's type satisfies the template constraint."),
            );

            insert_bound_type(
                template_result,
                template_parameter_name,
                &defining_entity,
                constraint,
                StandinOptions { appearance_depth: 1, ..Default::default() },
                Some(argument_offset),
                Some(argument_span),
            );
        }
    }
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
    block_context: &mut BlockContext,
    data_flow_graph: &mut DataFlowGraph,
    type_variable_bounds: &mut HashMap<String, (Vec<TemplateBound>, Vec<TemplateBound>)>,
    input_type: &TUnion,
    parameter_type: &TUnion,
    argument_offset: usize,
    input_expression: &Expression,
    invocation_target_parameter: &InvocationTargetParameter<'_>,
    invocation_target: &InvocationTarget<'_>,
) {
    let target_kind_str = invocation_target.guess_kind();
    let target_name_str = invocation_target.guess_name(context.interner);

    if input_type.is_never() {
        context.buffer.report(
            TypingIssueKind::NoValue,
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

    let mut union_comparison_result = ComparisonResult::new();
    let type_match_found = is_contained_by(
        context.codebase,
        context.interner,
        input_type,
        parameter_type,
        input_type.ignore_nullable_issues,
        input_type.ignore_falsable_issues,
        false,
        &mut union_comparison_result,
    );

    add_argument_dataflow(
        context,
        block_context,
        data_flow_graph,
        argument_offset,
        input_expression,
        input_type,
        invocation_target_parameter,
        invocation_target.get_function_like_identifier(),
        invocation_target.get_method_context(),
    );

    if !type_match_found {
        let call_site = Annotation::secondary(invocation_target.span())
            .with_message(format!("Arguments to this {} are incorrect", invocation_target.guess_kind(),));

        let input_type_str = input_type.get_id(Some(context.interner));
        let parameter_type_str = parameter_type.get_id(Some(context.interner));

        if !parameter_type.is_mixed() {
            let mut mixed_from_any = false;
            if input_type.is_mixed_with_any(&mut mixed_from_any) {
                for origin in &input_type.parent_nodes {
                    data_flow_graph.add_mixed_data(origin, input_expression.span());
                }

                context.buffer.report(
                    if mixed_from_any { TypingIssueKind::MixedAnyArgument } else { TypingIssueKind::MixedArgument },
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
        }

        if union_comparison_result.type_coerced.unwrap_or(false) && !input_type.is_mixed() {
            let issue_kind;
            let annotation_msg;
            let note_msg;

            if union_comparison_result.type_coerced_from_nested_any.unwrap_or(false) {
                issue_kind = TypingIssueKind::LessSpecificNestedAnyArgumentType;
                annotation_msg = format!("Provided type `{input_type_str}` is too general due to nested `any`.");
                note_msg = "The structure contains `any`, making it incompatible with the specific structure expected."
                    .to_string();
            } else if union_comparison_result.type_coerced_from_nested_mixed.unwrap_or(false) {
                issue_kind = TypingIssueKind::LessSpecificNestedArgumentType;
                annotation_msg = format!("Provided type `{input_type_str}` is too general due to nested `mixed`.");
                note_msg = "The structure contains `mixed`, making it incompatible.".to_string();
            } else {
                issue_kind = TypingIssueKind::LessSpecificArgument;
                annotation_msg = format!("Provided type `{input_type_str}` is too general.");
                note_msg = format!(
                    "The provided type `{input_type_str}` can be assigned to `{parameter_type_str}`, but is wider (less specific)."
                )
                .to_string();
            }

            context.buffer.report(
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
            );

            if types_can_be_identical {
                context.buffer.report(
                    TypingIssueKind::PossiblyInvalidArgument,
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
                context.buffer.report(
                    TypingIssueKind::InvalidArgument,
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
                        if !invocation_target.is_language_construct() {
                            format!("Change the argument value to match `{parameter_type_str}`, or update the parameter's type declaration.")
                        } else {
                            format!("Change the argument value to match `{parameter_type_str}`.")
                        }
                    ),
                );
            }
        }
    }

    if type_match_found || union_comparison_result.type_coerced.unwrap_or(false) {
        for (name, mut bound) in union_comparison_result.type_variable_lower_bounds {
            let name_str = context.interner.lookup(&name);
            bound.span = Some(input_expression.span());

            type_variable_bounds.entry(name_str.to_string()).or_insert_with(|| (Vec::new(), Vec::new())).0.push(bound);
        }
        for (name, mut bound) in union_comparison_result.type_variable_upper_bounds {
            let name_str = context.interner.lookup(&name);
            bound.span = Some(input_expression.span());
            type_variable_bounds.entry(name_str.to_string()).or_insert_with(|| (Vec::new(), Vec::new())).1.push(bound);
        }
    }
}

/// Analyzes all arguments within an `ArgumentList` in an arbitrary call context,
/// setting context flags once for the entire list analysis.
///
/// This function iterates through each argument in the list and analyzes its
/// value expression. It's used when the specific call target signature is
/// unknown or irrelevant, ensuring all argument expressions are processed.
///
/// # Arguments
///
/// * `context` - The overall analysis context.
/// * `artifacts` - Mutable store for analysis results.
/// * `block_context` - Mutable context for the current code block.
/// * `argument_list` - The AST node representing the list of arguments.
///
/// # Returns
///
/// * `Ok(())` if analysis of all arguments completes successfully.
/// * `Err(AnalysisError)` if an error occurs during the analysis of any argument's value.
pub(crate) fn evaluate_arbitrary_argument_list<'a>(
    context: &mut Context<'a>,
    artifacts: &mut AnalysisArtifacts,
    block_context: &mut BlockContext<'a>,
    argument_list: &ArgumentList,
) -> Result<(), AnalysisError> {
    let was_inside_call = block_context.inside_call;
    let was_inside_general_use = block_context.inside_general_use;

    block_context.inside_call = true;
    block_context.inside_general_use = true;

    for argument in argument_list.arguments.iter() {
        argument.value().analyze(context, block_context, artifacts)?;
    }

    block_context.inside_call = was_inside_call;
    block_context.inside_general_use = was_inside_general_use;

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
            parent_class: base_class_metadata.and_then(|meta| meta.get_direct_parent_class_ref()),
            function_is_final: calling_class_like_metadata.is_some_and(|meta| meta.is_final()),
            file_path: Some(&context.source.identifier),
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
        function_like_metadata.get_template_types(),
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
/// * `block_context` - Context for the current code block (currently unused but kept for signature consistency).
/// * `expression_types` - Map of expression ranges to their inferred types (currently unused).
/// * `data_flow_graph` - The data flow graph, used for adding taint/mixed sources.
/// * `argument_value_type` - The inferred type union of the expression being unpacked.
/// * `span` - The span of the unpacked argument expression (`...$arg`) for error reporting.
///
/// # Returns
///
/// A `TUnion` representing the combined type of the elements within the unpacked iterable.
fn get_unpacked_argument_type(
    context: &mut Context<'_>,
    expression_types: &HashMap<(usize, usize), Rc<TUnion>>,
    data_flow_graph: &mut DataFlowGraph,
    argument_value_type: &TUnion,
    span: Span,
) -> TUnion {
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
            TAtomic::Mixed(mixed) if mixed.is_any() => {
                if !reported_an_error {
                    context.buffer.report(
                        TypingIssueKind::MixedAnyArgument,
                        Issue::error(format!(
                            "Cannot unpack argument of type `{}` because it is not guaranteed to be iterable.",
                            atomic_type.get_id(Some(context.interner))
                        ))
                        .with_annotation(Annotation::primary(span).with_message("Expected an `iterable` for unpacking"))
                        .with_note("Argument unpacking `...` requires an `iterable` (e.g., `array` or `Traversable`).")
                        .with_note("The type `any` provides no guarantee of iterability.")
                        .with_help("Ensure the value is an `iterable` using type hints, checks, or assertions."),
                    );

                    reported_an_error = true;
                }

                for origin in &argument_value_type.parent_nodes {
                    data_flow_graph.add_mixed_data(origin, span);
                }

                potential_element_types.push(get_mixed_any());
            }
            TAtomic::Mixed(_) => {
                if !reported_an_error {
                    context.buffer.report(
                        TypingIssueKind::MixedArgument,
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

                for origin in &argument_value_type.parent_nodes {
                    data_flow_graph.add_mixed_data(origin, span);
                }

                potential_element_types.push(get_mixed());
            }
            _ => {
                if !reported_an_error {
                    let type_str = atomic_type.get_id(Some(context.interner));
                    context.buffer.report(
                        TypingIssueKind::InvalidArgument,
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

    let mut result_type = if potential_element_types.is_empty() {
        get_never()
    } else if potential_element_types.len() == 1 {
        potential_element_types.pop().unwrap()
    } else {
        let mut combined_type = potential_element_types.pop().unwrap();
        for element_type in potential_element_types {
            combined_type = add_union_type(combined_type, &element_type, context.codebase, context.interner, false);
        }

        combined_type
    };

    add_array_access_dataflow(expression_types, data_flow_graph, span, None, &mut result_type, &mut get_arraykey());

    result_type
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
                    TypingIssueKind::MixedArgument
                } else {
                    TypingIssueKind::InvalidArgument
                };

                context.buffer.report(
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
                    context.buffer.report(
                        TypingIssueKind::ConflictingTemplateEqualityBounds,
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
                        context.buffer.report(
                            TypingIssueKind::IncompatibleTemplateLowerBound,
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

/// Adds data flow edges connecting the sources of an argument's value (`input_type`)
/// to the node representing the corresponding function/method parameter (`param_node`).
///
/// This function models the flow of data from the argument expression into the parameter.
/// In `WholeProgram` analysis mode, it also adds edges to account for potential method
/// overrides in descendant classes and the actual declaring method, which is crucial for
/// features like taint tracking across the entire application.
///
/// # Arguments
///
/// * `context` - Analysis context, providing access to codebase and interner.
/// * `block_context` - Context for the current code block (used to determine node kind in `FunctionBody` mode).
/// * `data_flow_graph` - The data flow graph to be modified.
/// * `argument_offset` - The zero-based index of the argument.
/// * `input_expression` - The AST node for the argument expression (used for span info in some cases).
/// * `input_type` - The inferred type (`TUnion`) of the argument value, containing its source nodes (`parent_nodes`).
/// * `parameter_ref` - A reference (`CallParameterRef`) to the parameter's definition, used to get its span.
/// * `function_like_identifier` - Optional identifier of the function/method being called. If None, data flow cannot be added.
/// * `method_context` - Optional context specific to method calls, containing info about the declaring method and class hierarchy.
fn add_argument_dataflow(
    context: &mut Context<'_>,
    block_context: &mut BlockContext,
    data_flow_graph: &mut DataFlowGraph,
    argument_offset: usize,
    input_expression: &Expression,
    input_type: &TUnion,
    invocation_target_parameter: &InvocationTargetParameter<'_>,
    function_like_identifier: Option<&FunctionLikeIdentifier>,
    method_target_context: Option<&MethodTargetContext<'_>>,
) {
    let Some(function_like_id) = function_like_identifier else {
        return;
    };

    let param_node = {
        let Some(parameter_span) = invocation_target_parameter.get_name_span() else {
            return;
        };

        DataFlowNode {
            id: DataFlowNodeId::FunctionLikeArg(*function_like_id, argument_offset as u8),
            kind: if data_flow_graph.kind == GraphKind::FunctionBody && block_context.inside_general_use {
                DataFlowNodeKind::VariableUseSink { span: parameter_span }
            } else {
                DataFlowNodeKind::Vertex { span: Some(parameter_span), is_specialized: false }
            },
        }
    };

    if let GraphKind::WholeProgram = data_flow_graph.kind
        && let FunctionLikeIdentifier::Method(_, method_name) = function_like_id
        && let Some(method_info) = method_target_context
    {
        if let Some(dependent_class_likes) =
            context.codebase.all_class_like_descendants.get(&method_info.class_like_metadata.name)
            && !context.interner.lookup(method_name).eq_ignore_ascii_case("__construct")
        {
            for dependent_class_like_id in dependent_class_likes {
                if context.codebase.declaring_method_exists(dependent_class_like_id, method_name) {
                    let descendant_method_id = FunctionLikeIdentifier::Method(*dependent_class_like_id, *method_name);
                    let descendant_param_node = DataFlowNode::get_for_method_argument(
                        descendant_method_id,
                        argument_offset,
                        invocation_target_parameter.get_name_span(),
                        None,
                    );
                    data_flow_graph.add_node(descendant_param_node.clone());
                    data_flow_graph.add_path(&param_node, &descendant_param_node, PathKind::Default);
                }
            }
        }

        if let Some(declaring_id) = method_info.declaring_method_id {
            let declaring_function_id =
                FunctionLikeIdentifier::Method(*declaring_id.get_class_name(), *declaring_id.get_method_name());
            if &declaring_function_id != function_like_id {
                let declaring_method_param_node = DataFlowNode::get_for_method_argument(
                    declaring_function_id,
                    argument_offset,
                    invocation_target_parameter.get_name_span(),
                    Some(input_expression.span()),
                );

                data_flow_graph.add_node(declaring_method_param_node.clone());
                data_flow_graph.add_path(&param_node, &declaring_method_param_node, PathKind::Default);
            }
        }
    }

    for parent_node in &input_type.parent_nodes {
        if data_flow_graph.get_node(&parent_node.id).is_some() {
            data_flow_graph.add_path(parent_node, &param_node, PathKind::Default);
        }
    }

    data_flow_graph.add_node(param_node);
}
