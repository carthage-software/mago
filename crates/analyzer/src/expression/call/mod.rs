use mago_atom::AtomMap;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::ttype::TType;
use mago_codex::ttype::add_optional_union_type;
use mago_codex::ttype::combine_union_types;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_null;
use mago_codex::ttype::get_void;
use mago_codex::ttype::template::TemplateResult;
use mago_codex::ttype::union::TUnion;
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
use crate::context::scope::control_action::ControlAction;
use crate::error::AnalysisError;
use crate::invocation::Invocation;
use crate::invocation::InvocationArgumentsSource;
use crate::invocation::InvocationTarget;
use crate::invocation::analyzer::analyze_invocation;
use crate::invocation::post_process::post_invocation_process;
use crate::invocation::return_type_fetcher::fetch_invocation_return_type;

pub mod function_call;
pub mod method_call;
pub mod pipe;
pub mod static_method_call;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for Call<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        match self {
            Call::Function(call) => call.analyze(context, block_context, artifacts),
            Call::Method(call) => call.analyze(context, block_context, artifacts),
            Call::NullSafeMethod(call) => call.analyze(context, block_context, artifacts),
            Call::StaticMethod(call) => call.analyze(context, block_context, artifacts),
        }
    }
}

fn analyze_invocation_targets<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    mut template_result: TemplateResult,
    invocation_targets: Vec<InvocationTarget<'ctx>>,
    invocation_arguments: InvocationArgumentsSource<'ast, 'arena>,
    call_span: Span,
    encountered_invalid_targets: bool,
    encountered_mixed_targets: bool,
    should_add_null: bool,
) -> Result<(), AnalysisError> {
    let mut resulting_type = None;
    for target in invocation_targets {
        if let InvocationTarget::FunctionLike { metadata, .. } = &target
            && let Some(name) = metadata.name
        {
            match true {
                _ if name.eq_ignore_ascii_case("mago\\inspect") => {
                    inspect_arguments(context, block_context, artifacts, &target, &invocation_arguments)?;

                    resulting_type =
                        Some(add_optional_union_type(get_void(), resulting_type.as_ref(), context.codebase));

                    continue;
                }
                _ if name.eq_ignore_ascii_case("mago\\confirm") => {
                    confirm_argument_type(context, block_context, artifacts, &target, &invocation_arguments)?;

                    resulting_type =
                        Some(add_optional_union_type(get_void(), resulting_type.as_ref(), context.codebase));

                    continue;
                }
                _ => {}
            }
        }

        let invocation: Invocation<'ctx, 'ast, 'arena> = Invocation::new(target, invocation_arguments, call_span);
        let mut argument_types = AtomMap::default();

        analyze_invocation(
            context,
            block_context,
            artifacts,
            &invocation,
            None,
            &mut template_result,
            &mut argument_types,
        )?;

        resulting_type = Some(add_optional_union_type(
            fetch_invocation_return_type(
                context,
                block_context,
                artifacts,
                &invocation,
                &template_result,
                &argument_types,
            ),
            resulting_type.as_ref(),
            context.codebase,
        ));

        post_invocation_process(
            context,
            block_context,
            artifacts,
            &invocation,
            None,
            &template_result,
            &argument_types,
            true,
        )?;
    }

    let resulting_type = match resulting_type {
        Some(resulting_type) => {
            if encountered_invalid_targets {
                return Ok(());
            } else if encountered_mixed_targets {
                get_mixed()
            } else if should_add_null {
                combine_union_types(&resulting_type, &get_null(), context.codebase, false)
            } else {
                resulting_type
            }
        }
        None => {
            match invocation_arguments {
                InvocationArgumentsSource::ArgumentList(argument_list) => {
                    argument_list.analyze(context, block_context, artifacts)?;
                }
                InvocationArgumentsSource::PipeInput(pipe) => {
                    let was_inside_call = block_context.inside_call;
                    let was_inside_general_use = block_context.inside_general_use;
                    block_context.inside_call = true;
                    block_context.inside_general_use = true;
                    pipe.input.analyze(context, block_context, artifacts)?;
                    block_context.inside_call = was_inside_call;
                    block_context.inside_general_use = was_inside_general_use;
                }
                _ => {}
            };

            if encountered_mixed_targets {
                get_mixed()
            } else {
                return Ok(());
            }
        }
    };

    if resulting_type.is_never() && !block_context.inside_loop {
        artifacts.set_expression_type(&call_span, resulting_type);

        block_context.has_returned = true;
        block_context.control_actions.insert(ControlAction::End);
        return Ok(());
    } else {
        artifacts.set_expression_type(&call_span, resulting_type);
    }

    Ok(())
}

fn get_function_like_target<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    function_like: FunctionLikeIdentifier,
    alternative: Option<FunctionLikeIdentifier>,
    span: Span,
    inferred_return_type: Option<Box<TUnion>>,
) -> Result<Option<InvocationTarget<'ctx>>, AnalysisError> {
    let mut identifier = function_like;

    let metadata = context.codebase.get_function_like(&identifier).or_else(|| {
        if let Some(alternative) = alternative {
            context.codebase.get_function_like(&alternative).inspect(|_| {
                identifier = alternative;
            })
        } else {
            None
        }
    });

    let Some(metadata) = metadata else {
        let title_str = function_like.title_kind_str();
        let kind_str = function_like.kind_str();
        let name_str = function_like.as_string();

        let issue = if let Some(alt_id) = alternative {
            let alt_name_str = alt_id.as_string();

            Issue::error(format!(
                "Could not find definition for {kind_str} `{name_str}` (also tried as `{alt_name_str}` in a broader scope)."
            )).with_annotation(
                Annotation::primary(span).with_message(format!("Attempted to use {kind_str} `{name_str}` which is undefined")),
            ).with_note(
                format!("Neither `{name_str}` (e.g., in current namespace) nor `{alt_name_str}` (e.g., global fallback) could be resolved."),
            )
        } else {
            Issue::error(format!("{title_str} `{name_str}` could not be found.")).with_annotation(
                Annotation::primary(span).with_message(format!("Undefined {kind_str} `{name_str}` called here")),
            )
        };

        context.collector.report_with_code(
            IssueCode::NonExistentFunction,
            issue.with_note("This often means the function/method is misspelled, not imported correctly (e.g., missing `use` statement for namespaced functions), or not defined/autoloaded.")
                .with_help(format!("Check for typos in `{name_str}`. Verify namespace imports if applicable, and ensure the {kind_str} is defined and accessible."))
        );

        return Ok(None);
    };

    Ok(Some(InvocationTarget::FunctionLike { identifier, metadata, inferred_return_type, method_context: None, span }))
}

fn inspect_arguments<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    target: &InvocationTarget<'ctx>,
    invocation_arguments: &InvocationArgumentsSource<'ast, 'arena>,
) -> Result<(), AnalysisError> {
    match invocation_arguments {
        InvocationArgumentsSource::ArgumentList(argument_list) => {
            argument_list.analyze(context, block_context, artifacts)?;
        }
        InvocationArgumentsSource::PipeInput(pipe) => {
            let was_inside_call = block_context.inside_call;
            let was_inside_general_use = block_context.inside_general_use;
            block_context.inside_call = true;
            block_context.inside_general_use = true;
            pipe.input.analyze(context, block_context, artifacts)?;
            block_context.inside_call = was_inside_call;
            block_context.inside_general_use = was_inside_general_use;
        }
        _ => {}
    };

    let mut argument_annotations = vec![];
    for (idx, argument) in invocation_arguments.get_arguments().iter().enumerate() {
        let argument_expression = argument.value();
        let argument_span = argument_expression.span();
        let argument_type_string =
            artifacts.get_expression_type(argument_expression).map_or("<unknown type>", |t| t.get_id().as_str());

        argument_annotations.push(
            Annotation::secondary(argument_span)
                .with_message(format!("Argument #{} type: `{argument_type_string}`", idx + 1,)),
        );
    }

    let mut issue = Issue::help("Type information for arguments of `Mago\\inspect()` call.")
        .with_annotation(Annotation::primary(target.span()).with_message("Type inspection point"));

    for annotation in argument_annotations {
        issue = issue.with_annotation(annotation);
    }

    context.collector.report_with_code(
        IssueCode::TypeInspection,
        issue
            .with_note(
                "The `Mago\\inspect()` function is a static analysis debugging utility; it has no effect at runtime.",
            )
            .with_help("Remember to remove `Mago\\inspect()` calls before deploying to production."),
    );

    Ok(())
}

fn confirm_argument_type<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    target: &InvocationTarget<'ctx>,
    invocation_arguments: &InvocationArgumentsSource<'ast, 'arena>,
) -> Result<(), AnalysisError> {
    match invocation_arguments {
        InvocationArgumentsSource::ArgumentList(argument_list) => {
            argument_list.analyze(context, block_context, artifacts)?;
        }
        InvocationArgumentsSource::PipeInput(pipe) => {
            let was_inside_call = block_context.inside_call;
            let was_inside_general_use = block_context.inside_general_use;
            block_context.inside_call = true;
            block_context.inside_general_use = true;
            pipe.input.analyze(context, block_context, artifacts)?;
            block_context.inside_call = was_inside_call;
            block_context.inside_general_use = was_inside_general_use;
        }
        _ => {}
    };

    let arguments = invocation_arguments.get_arguments();

    if arguments.len() != 2 {
        context.collector.report_with_code(
            IssueCode::TypeConfirmation,
            Issue::error(format!(
                "`Mago\\confirm()` expects exactly 2 arguments (a value and an expected type string), but {} {} provided.",
                arguments.len(),
                if arguments.len() == 1 { "was" } else { "were" }
            ))
            .with_annotation(Annotation::primary(target.span())
                .with_message(if arguments.len() < 2 {
                    "Too few arguments provided: expected a value and a type string."
                } else {
                    "Too many arguments provided: expected only a value and a type string."
                }))
            .with_note("The `Mago\\confirm()` function is a debugging utility and requires these two specific arguments to function.")
            .with_help("Usage: `Mago\\confirm($value_to_check, \"ExpectedTypeAsString\");`. Remember to remove before committing."),
        );

        return Ok(());
    }

    let value_to_check_argument = &arguments[0];
    let expected_type_string_argument = &arguments[1];

    let value_expression = value_to_check_argument.value();
    let expected_type_expression = expected_type_string_argument.value();
    let Some(actual_argument_type) = artifacts.get_expression_type(value_expression) else {
        context.collector.report_with_code(
            IssueCode::TypeConfirmation,
            Issue::error("Cannot determine the type of the first argument passed to `Mago\\confirm()`.")
                .with_annotation(
                    Annotation::primary(value_expression.span())
                        .with_message("The type of this expression could not be determined here"),
                )
                .with_annotation(Annotation::secondary(target.span()).with_message(
                    "`Mago\\confirm()` expects a value to check and a string representing the expected type.",
                ))
                .with_note("`Mago\\confirm()` needs to know the type of the value to perform the confirmation.")
                .with_note("This debugging utility (`Mago\\confirm()`) should be removed before committing code.")
                .with_help("Ensure the expression is well-formed and its type can be inferred by the analyzer."),
        );

        return Ok(());
    };

    let Some(expected_type_expression_type) = artifacts.get_expression_type(expected_type_expression) else {
        context.collector.report_with_code(
            IssueCode::TypeConfirmation,
            Issue::error(
                "Cannot determine the type of the second argument (the expected type string) passed to `Mago\\confirm()`."
            )
            .with_annotation(
                Annotation::primary(expected_type_expression.span())
                    .with_message("The type of this expression (expected to be a literal string) is unknown"),
            )
            .with_annotation(Annotation::secondary(target.span())
                .with_message("`Mago\\confirm()` expects a value to check and a string representing the expected type."))
            .with_note("`Mago\\confirm()` requires the second argument to be a literal string representing the type.")
            .with_note("This debugging utility (`Mago\\confirm()`) should be removed before committing code.")
            .with_help("Ensure the second argument is a literal string (e.g., `\"int\"`)."),
        );

        return Ok(());
    };

    let Some(expected_type_literal_string) = expected_type_expression_type.get_single_literal_string_value() else {
        context.collector.report_with_code(
            IssueCode::TypeConfirmation,
            Issue::error(format!(
                "Second argument to `Mago\\confirm()` must be a literal string, but found type `{}`.",
                expected_type_expression_type.get_id()
            ))
            .with_annotation(Annotation::primary(expected_type_expression.span())
                .with_message(format!("Expected a literal string here, not type `{}`", expected_type_expression_type.get_id())))
            .with_annotation(Annotation::secondary(target.span())
                .with_message("`Mago\\confirm()` expects a value to check and a string representing the expected type."))
            .with_note("`Mago\\confirm()` uses the second argument as a string representation of the expected type for comparison.")
            .with_note("This debugging utility (`Mago\\confirm()`) should be removed before committing code.")
            .with_help("Provide the expected type as a literal string, e.g., `\"int\"`, `\"literal-string\"`, or `\"Collection<int>\"`."),
        );

        return Ok(());
    };

    let actual_argument_type_string = actual_argument_type.get_id();
    let is_match = expected_type_literal_string.eq_ignore_ascii_case(&actual_argument_type_string);

    if is_match {
        context.collector.report_with_code(
            IssueCode::TypeConfirmation,
            Issue::help(format!("Type of expression is `{actual_argument_type_string}` as expected.",))
                .with_annotation(
                    Annotation::primary(value_expression.span())
                        .with_message(format!("Confirmed type: `{actual_argument_type_string}`")),
                )
                .with_annotation(
                    Annotation::secondary(expected_type_expression.span())
                        .with_message(format!("Matches expected type: `{expected_type_literal_string}`")),
                )
                .with_note("`Mago\\confirm()` successfully confirmed the type of the expression.")
                .with_help("This debugging utility (`Mago\\confirm()`) should be removed before committing code."),
        );
    } else {
        context.collector.report_with_code(
            IssueCode::TypeConfirmation,
            Issue::error(format!(
                "Type of expression is `{actual_argument_type_string}`, but expected `{expected_type_literal_string}`."
            ))
            .with_annotation(
                Annotation::primary(value_expression.span())
                    .with_message(format!("Actual type: `{actual_argument_type_string}`")),
            )
            .with_annotation(
                Annotation::secondary(expected_type_expression.span())
                    .with_message(format!("Expected type: `{expected_type_literal_string}`")),
            )
            .with_note("`Mago\\confirm()` failed to confirm the type of the expression.")
            .with_help("This debugging utility (`Mago\\confirm()`) should be removed before committing code."),
        );
    }

    Ok(())
}
