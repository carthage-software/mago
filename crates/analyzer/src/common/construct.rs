use std::borrow::Cow;

use mago_codex::ttype::TType;
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::union_comparator;
use mago_codex::ttype::get_mixed;
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
use crate::error::AnalysisError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstructInput<'ast, 'arena> {
    ArgumentList(Option<&'ast ArgumentList<'arena>>),
    Expression(&'ast Expression<'arena>),
    ExpressionList(&'ast [Expression<'arena>]),
}

/// Analyzes inputs for a language construct (e.g., `echo`, `exit`, `return`).
///
/// This function is generalized to handle inputs from various syntaxes by matching
/// on the `ConstructInput` enum. It dispatches to the type verifier for each
/// provided value and correctly sets the `is_argument` flag to ensure error
/// messages use the appropriate terminology ("argument" vs. "value").
pub fn analyze_construct_inputs<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    construct_kind: &'static str,
    construct_keyword: Span,
    inputs: ConstructInput<'ast, 'arena>,
    expected_type: TUnion,
    is_variadic: bool,
    has_default: bool,
    has_side_effects: bool,
) -> Result<(), AnalysisError> {
    if has_side_effects && block_context.scope.is_pure() {
        context.collector.report_with_code(
            IssueCode::ImpureConstruct,
            Issue::error(format!(
                "Impure use of `{construct_kind}` in a pure context."
            ))
                .with_annotation(Annotation::primary(construct_keyword).with_message("This operation has side effects"))
                .with_note(format!(
                    "Constructs like `{construct_kind}` can cause side effects (e.g., producing output, changing state, or terminating the script), which are disallowed in contexts marked as `@pure` or `@mutation-free`."
                ))
                .with_help(format!(
                    "To resolve this, either remove the call to `{construct_kind}` from this function, or if the side effects are intended, remove the `@pure` or `@mutation-free` annotation from the surrounding scope."
                )),
        );
    }

    match inputs {
        ConstructInput::ArgumentList(list_option) => {
            let Some(argument_list) = list_option.filter(|l| !l.arguments.is_empty()) else {
                if !has_default {
                    report_missing_input(context, construct_kind, &construct_keyword, true);
                }

                return Ok(());
            };

            argument_list.analyze(context, block_context, artifacts)?;

            for (index, argument) in argument_list.arguments.iter().enumerate() {
                if index > 0 && !is_variadic {
                    report_too_many_inputs(context, construct_kind, &construct_keyword, argument.span(), index, true);
                    continue;
                }

                let input_type = artifacts
                    .get_expression_type(argument)
                    .map(Cow::Borrowed)
                    .unwrap_or_else(|| Cow::Owned(get_mixed()));

                verify_construct_input_type(
                    context,
                    &input_type,
                    &expected_type,
                    index,
                    argument.value(),
                    construct_kind,
                    &construct_keyword,
                    true,
                );
            }
        }
        ConstructInput::Expression(expression) => {
            expression.analyze(context, block_context, artifacts)?;

            let input_type =
                artifacts.get_expression_type(expression).map(Cow::Borrowed).unwrap_or_else(|| Cow::Owned(get_mixed()));

            verify_construct_input_type(
                context,
                &input_type,
                &expected_type,
                0,
                expression,
                construct_kind,
                &construct_keyword,
                false,
            );
        }
        ConstructInput::ExpressionList(expressions) => {
            if expressions.is_empty() {
                if !has_default {
                    report_missing_input(context, construct_kind, &construct_keyword, false);
                }
                return Ok(());
            }

            for (index, expression) in expressions.iter().enumerate() {
                if index > 0 && !is_variadic {
                    report_too_many_inputs(
                        context,
                        construct_kind,
                        &construct_keyword,
                        expression.span(),
                        index,
                        false,
                    );

                    continue;
                }

                expression.analyze(context, block_context, artifacts)?;

                let input_type = artifacts
                    .get_expression_type(expression)
                    .map(Cow::Borrowed)
                    .unwrap_or_else(|| Cow::Owned(get_mixed()));

                verify_construct_input_type(
                    context,
                    &input_type,
                    &expected_type,
                    index,
                    expression,
                    construct_kind,
                    &construct_keyword,
                    false, // Not in a formal argument list
                );
            }
        }
    }

    Ok(())
}

/// Verifies the type of a single input passed to a language construct.
///
/// This function checks for a variety of type mismatches, such as `null` or `false`
/// being passed to parameters that don't accept them, and general type
/// incompatibilities. It uses a set of helper functions to report clear,
/// specific errors to the user.
///
/// # Arguments
///
/// * `context` - The shared analysis context.
/// * `input_type` - The type of the value being passed.
/// * `parameter_type` - The type expected by the language construct.
/// * `argument_offset` - The 0-based index of the input, used for error messages.
/// * `input_expression` - The AST node of the input expression for highlighting.
/// * `construct_kind` - A string name of the construct (e.g., "echo", "exit").
/// * `construct_keyword` - The span of the construct's keyword for highlighting.
/// * `is_argument` - Controls terminology in errors. `true` uses "argument", `false` uses "value".
fn verify_construct_input_type<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    input_type: &TUnion,
    parameter_type: &TUnion,
    argument_offset: usize,
    input_expression: &'ast Expression<'arena>,
    construct_kind: &'static str,
    construct_keyword: &Span,
    is_argument: bool,
) {
    if input_type.is_never() {
        return report_never_input(
            context,
            construct_kind,
            input_expression,
            argument_offset,
            construct_keyword,
            is_argument,
        );
    }

    let input_type_str = input_type.get_id();
    let parameter_type_str = parameter_type.get_id();

    if !parameter_type.accepts_null() {
        if input_type.is_null() {
            return report_null_input(
                context,
                construct_kind,
                input_expression,
                argument_offset,
                &parameter_type_str,
                construct_keyword,
                is_argument,
            );
        }
        if input_type.is_nullable() && !input_type.ignore_nullable_issues {
            report_possibly_null_input(
                context,
                construct_kind,
                input_expression,
                argument_offset,
                &input_type_str,
                &parameter_type_str,
                construct_keyword,
                is_argument,
            );
        }
    }

    if !parameter_type.accepts_false() {
        if input_type.is_false() {
            return report_false_input(
                context,
                construct_kind,
                input_expression,
                argument_offset,
                &parameter_type_str,
                construct_keyword,
                is_argument,
            );
        }
        if input_type.is_falsable() && !input_type.ignore_falsable_issues {
            report_possibly_false_input(
                context,
                construct_kind,
                input_expression,
                argument_offset,
                &input_type_str,
                &parameter_type_str,
                construct_keyword,
                is_argument,
            );
        }
    }

    let mut comparison = ComparisonResult::new();
    if union_comparator::is_contained_by(
        context.codebase,
        input_type,
        parameter_type,
        true,
        true,
        false,
        &mut comparison,
    ) {
        return;
    }

    if input_type.is_mixed() {
        return report_mixed_input(
            context,
            construct_kind,
            input_expression,
            argument_offset,
            &input_type_str,
            &parameter_type_str,
            construct_keyword,
            is_argument,
        );
    }

    if comparison.type_coerced.unwrap_or(false) {
        report_less_specific_input(
            context,
            &comparison,
            construct_kind,
            input_expression,
            argument_offset,
            &input_type_str,
            &parameter_type_str,
            construct_keyword,
            is_argument,
        );
    } else {
        report_invalid_or_possibly_invalid_input(
            context,
            input_type,
            parameter_type,
            construct_kind,
            input_expression,
            argument_offset,
            &input_type_str,
            &parameter_type_str,
            construct_keyword,
            is_argument,
        )
    }
}

fn get_ordinal(index: usize) -> Cow<'static, str> {
    match index {
        0 => "first".into(),
        1 => "second".into(),
        2 => "third".into(),
        n => format!("{}th", n + 1).into(),
    }
}

fn report_missing_input<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    kind: &str,
    keyword: &Span,
    is_argument: bool,
) {
    let term = if is_argument { "argument" } else { "value" };
    context.collector.report_with_code(
        IssueCode::TooFewArguments,
        Issue::error(format!("`{kind}` expects at least one {term}."))
            .with_annotation(Annotation::primary(keyword.span()).with_message(format!("{term} is missing here"))),
    );
}

fn report_too_many_inputs<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    kind: &str,
    keyword: &Span,
    input_span: Span,
    index: usize,
    is_argument: bool,
) {
    let term = if is_argument { "argument" } else { "value" };
    let position = get_ordinal(index);
    context.collector.report_with_code(
        IssueCode::TooManyArguments,
        Issue::error(format!("Unexpected {position} {term} provided to `{kind}`."))
            .with_annotation(Annotation::primary(input_span).with_message(format!("Unexpected {term}")))
            .with_annotation(Annotation::secondary(keyword.span()).with_message(format!("`{kind}` defined here"))),
    );
}

fn report_never_input<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    kind: &str,
    expr: &'ast Expression<'arena>,
    offset: usize,
    keyword: &Span,
    is_argument: bool,
) {
    let term = if is_argument { "argument" } else { "value" };
    let position = get_ordinal(offset);
    context.collector.report_with_code(
        IssueCode::NoValue,
        Issue::error(format!("The {position} {term} for `{kind}` cannot produce a value."))
            .with_annotation(Annotation::primary(expr.span()).with_message("This expression has type `never`"))
            .with_annotation(Annotation::secondary(keyword.span()).with_message(format!("`{kind}` called here")))
            .with_note("An expression of type `never` cannot complete, often because it throws or exits."),
    );
}

fn report_null_input<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    kind: &str,
    expr: &'ast Expression<'arena>,
    offset: usize,
    param_type: &str,
    keyword: &Span,
    is_argument: bool,
) {
    let term = if is_argument { "argument" } else { "value" };
    let position = get_ordinal(offset);
    context.collector.report_with_code(
        IssueCode::NullArgument,
        Issue::error(format!("The {position} {term} for `{kind}` is `null`."))
            .with_annotation(
                Annotation::primary(expr.span()).with_message(format!("Expected `{param_type}`, but got `null`")),
            )
            .with_annotation(Annotation::secondary(keyword.span()).with_message(format!("`{kind}` called here")))
            .with_help("Provide a non-null value."),
    );
}

fn report_possibly_null_input<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    kind: &str,
    expr: &'ast Expression<'arena>,
    offset: usize,
    in_type: &str,
    param_type: &str,
    keyword: &Span,
    is_argument: bool,
) {
    let term = if is_argument { "argument" } else { "value" };
    let position = get_ordinal(offset);
    context.collector.report_with_code(
        IssueCode::PossiblyNullArgument,
        Issue::error(format!("The {position} {term} for `{kind}` might be `null`."))
            .with_annotation(
                Annotation::primary(expr.span()).with_message(format!("This is type `{in_type}` which can be `null`")),
            )
            .with_annotation(Annotation::secondary(keyword.span()).with_message(format!("`{kind}` called here")))
            .with_note(format!("The expected type `{param_type}` does not accept `null`."))
            .with_help("Add a `null` check to guard this input."),
    );
}

fn report_false_input<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    kind: &str,
    expr: &'ast Expression<'arena>,
    offset: usize,
    param_type: &str,
    keyword: &Span,
    is_argument: bool,
) {
    let term = if is_argument { "argument" } else { "value" };
    let position = get_ordinal(offset);

    context.collector.report_with_code(
        IssueCode::FalseArgument,
        Issue::error(format!("The {position} {term} for `{kind}` is `false`."))
            .with_annotation(
                Annotation::primary(expr.span()).with_message(format!("Expected `{param_type}`, but got `false`")),
            )
            .with_annotation(Annotation::secondary(keyword.span()).with_message(format!("`{kind}` called here")))
            .with_help("Provide a non-false value."),
    );
}

fn report_possibly_false_input<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    kind: &str,
    expr: &'ast Expression<'arena>,
    offset: usize,
    in_type: &str,
    param_type: &str,
    keyword: &Span,
    is_argument: bool,
) {
    let term = if is_argument { "argument" } else { "value" };
    let position = get_ordinal(offset);

    context.collector.report_with_code(
        IssueCode::PossiblyFalseArgument,
        Issue::error(format!("The {position} {term} for `{kind}` might be `false`."))
            .with_annotation(
                Annotation::primary(expr.span()).with_message(format!("This is type `{in_type}` which can be `false`")),
            )
            .with_annotation(Annotation::secondary(keyword.span()).with_message(format!("`{kind}` called here")))
            .with_note(format!("The expected type `{param_type}` does not accept `false`."))
            .with_help("Add a `false` check to guard this input."),
    );
}

fn report_mixed_input<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    kind: &str,
    expr: &'ast Expression<'arena>,
    offset: usize,
    in_type: &str,
    param_type: &str,
    keyword: &Span,
    is_argument: bool,
) {
    let term = if is_argument { "argument" } else { "value" };
    let position = get_ordinal(offset);

    context.collector.report_with_code(
        IssueCode::MixedArgument,
        Issue::error(format!("The {position} {term} for `{kind}` is too general."))
            .with_annotation(Annotation::primary(expr.span()).with_message(format!("Type `{in_type}` is too broad")))
            .with_annotation(Annotation::secondary(keyword.span()).with_message(format!("`{kind}` called here")))
            .with_note(format!("The expected type `{param_type}` is more specific."))
            .with_help("Add a specific type hint or assertion for this value."),
    );
}

fn report_less_specific_input<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    comparison: &ComparisonResult,
    kind: &str,
    expr: &'ast Expression<'arena>,
    offset: usize,
    in_type: &str,
    param_type: &str,
    keyword: &Span,
    is_argument: bool,
) {
    let term = if is_argument { "argument" } else { "value" };
    let position = get_ordinal(offset);
    let (code, reason) = if comparison.type_coerced_from_nested_mixed.unwrap_or(false) {
        (IssueCode::LessSpecificNestedArgumentType, "due to nested `mixed`")
    } else {
        (IssueCode::LessSpecificArgument, "it is a wider type")
    };

    context.collector.report_with_code(
        code,
        Issue::error(format!("The {position} {term} type `{in_type}` is less specific than expected `{param_type}`."))
            .with_annotation(
                Annotation::primary(expr.span()).with_message(format!("This type is too general {reason}")),
            )
            .with_annotation(Annotation::secondary(keyword.span()).with_message(format!("`{kind}` called here")))
            .with_help("Provide a more specific type."),
    );
}

fn report_invalid_or_possibly_invalid_input<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    in_type: &TUnion,
    param_type: &TUnion,
    kind: &str,
    expr: &'ast Expression<'arena>,
    offset: usize,
    in_type_str: &str,
    param_type_str: &str,
    keyword: &Span,
    is_argument: bool,
) {
    let term = if is_argument { "argument" } else { "value" };
    let position = get_ordinal(offset);

    let issue =
        if union_comparator::can_expression_types_be_identical(context.codebase, in_type, param_type, false, false) {
            Issue::error(format!("The {position} {term} for `{kind}` might have the wrong type."))
                .with_code(IssueCode::PossiblyInvalidArgument)
                .with_annotation(Annotation::primary(expr.span()).with_message(format!(
                    "This is `{in_type_str}`, which only sometimes overlaps with `{param_type_str}`"
                )))
                .with_annotation(Annotation::secondary(keyword.span()).with_message(format!("`{kind}` called here")))
                .with_help("Add a type check to ensure the value is what you expect.")
        } else {
            Issue::error(format!("Invalid type for the {position} {term}."))
                .with_code(IssueCode::InvalidArgument)
                .with_annotation(
                    Annotation::primary(expr.span())
                        .with_message(format!("This is type `{in_type_str}`, but expected `{param_type_str}`")),
                )
                .with_annotation(Annotation::secondary(keyword.span()).with_message(format!("`{kind}` called here")))
                .with_help("Adjust the value to match the expected type.")
        };

    context.collector.report(issue);
}
