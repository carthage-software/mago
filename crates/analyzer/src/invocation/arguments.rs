use ahash::HashMap;

use mago_codex::ttype::add_union_type;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::callable::TCallable;
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::union_comparator::can_expression_types_be_identical;
use mago_codex::ttype::comparator::union_comparator::is_contained_by;
use mago_codex::ttype::union::TUnion;
use mago_codex::ttype::*;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::invocation::InvocationTarget;

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
pub fn analyze_and_store_argument_type<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    invocation_target: &InvocationTarget<'ctx>,
    argument_expression: &Expression<'arena>,
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
        for closure_parameter_atomic in closure_parameter_type.types.as_ref() {
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

    if referenced_parameter {
        let is_referenceable = argument_expression.is_referenceable(false)
            || (argument_expression.is_referenceable(true) && argument_type.by_reference);

        if !is_referenceable {
            let target_kind_str = invocation_target.guess_kind();
            let target_name_str = invocation_target.guess_name();

            context.collector.report_with_code(
                IssueCode::InvalidPassByReference,
                Issue::error(format!(
                    "Invalid argument for by-reference parameter #{} in call to {} `{}`.",
                    argument_offset + 1,
                    target_kind_str,
                    target_name_str,
                ))
                .with_annotation(
                    Annotation::primary(argument_expression.span())
                        .with_message("This expression cannot be passed by reference."),
                )
                .with_note(
                    "You can only pass variables, properties, array elements, or the result of another function that itself returns a reference."
                )
                .with_help("To fix this, assign this value to a variable first, and then pass that variable to the function."),
            );
        }
    }

    if argument_offset != usize::MAX {
        analyzed_argument_types.insert(argument_offset, (argument_type, argument_expression.span()));
    }

    Ok(())
}

/// Verifies a single argument's type against the resolved parameter type for a function/method/callable call.
///
/// This function compares the `input_type` (actual argument type) against the `parameter_type`
/// (expected type after template resolution). It reports various type mismatch errors
/// (e.g., invalid type, possibly invalid, mixed argument, less specific argument)
/// with appropriate severity and context. It also adds data flow edges from the argument
/// sources to the parameter representation in the data flow graph.
pub fn verify_argument_type<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    input_type: &TUnion,
    parameter_type: &TUnion,
    argument_offset: usize,
    input_expression: &'ast Expression<'arena>,
    invocation_target: &InvocationTarget<'_>,
) {
    let target_kind_str = invocation_target.guess_kind();
    let target_name_str = invocation_target.guess_name();

    if input_type.is_never() {
        context.collector.report_with_code(
            IssueCode::NoValue,
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

    let input_type_str = input_type.get_id();
    let parameter_type_str = parameter_type.get_id();

    if !parameter_type.accepts_null() {
        if input_type.is_null() {
            context.collector.report_with_code(
                IssueCode::NullArgument,
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
                IssueCode::PossiblyNullArgument,
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
                IssueCode::FalseArgument,
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
                IssueCode::PossiblyFalseArgument,
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
    let type_match_found =
        is_contained_by(context.codebase, input_type, parameter_type, true, true, false, &mut union_comparison_result);

    if type_match_found {
        return;
    }

    if input_type.is_mixed() {
        context.collector.report_with_code(
            IssueCode::MixedArgument,
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
            issue_kind = IssueCode::LessSpecificNestedArgumentType;
            annotation_msg = format!("Provided type `{input_type_str}` is too general due to nested `mixed`.");
            note_msg = "The structure contains `mixed`, making it incompatible.".to_string();
        } else {
            issue_kind = IssueCode::LessSpecificArgument;
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
        let types_can_be_identical =
            can_expression_types_be_identical(context.codebase, input_type, parameter_type, false, false);

        if types_can_be_identical {
            context.collector.report_with_code(
                IssueCode::PossiblyInvalidArgument,
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
                IssueCode::InvalidArgument,
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
/// * `context` - Analysis context, used for reporting issues and accessing codebase.
/// * `argument_value_type` - The inferred type union of the expression being unpacked.
/// * `span` - The span of the unpacked argument expression (`...$arg`) for error reporting.
///
/// # Returns
///
/// A `TUnion` representing the combined type of the elements within the unpacked iterable.
pub fn get_unpacked_argument_type<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    argument_value_type: &TUnion,
    span: Span,
) -> TUnion {
    let mut potential_element_types = Vec::new();
    let mut reported_an_error = false;

    for atomic_type in argument_value_type.types.as_ref() {
        if let Some(value_parameter) = get_iterable_value_parameter(atomic_type, context.codebase) {
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
                        IssueCode::MixedArgument,
                        Issue::error(format!(
                            "Cannot unpack argument of type `{}` because it is not guaranteed to be iterable.",
                            atomic_type.get_id()
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
                    let type_str = atomic_type.get_id();
                    context.collector.report_with_code(
                        IssueCode::InvalidArgument,
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
            combined_type = add_union_type(combined_type, &element_type, context.codebase, false);
        }

        combined_type
    } else {
        get_never()
    }
}
