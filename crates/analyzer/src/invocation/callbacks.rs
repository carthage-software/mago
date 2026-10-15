use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::parameter::CallableExceptionCatchSummary;
use mago_codex::metadata::parameter::CallableInvocationKind;
use mago_span::Span;
use mago_syntax::cst::Expression;
use mago_syntax::utils::unparenthesized_expression;
use mago_word::Word;

use crate::artifacts::AnalysisArtifacts;
use crate::artifacts::CallableEffectSummary;
use crate::artifacts::CallbackCallsiteContext;
use crate::context::block::BlockContext;
use crate::invocation::InvocationTarget;

/// Returns the invocation contract for a resolved function-like parameter.
#[must_use]
pub(crate) fn classify_callback(
    target: &InvocationTarget<'_>,
    parameter_offset: usize,
    argument_offset: usize,
    argument_count: usize,
    is_unpacked: bool,
) -> Option<CallableInvocationKind> {
    let metadata = target.get_function_like_metadata()?;
    let parameter = metadata.parameters.get(parameter_offset)?;
    let invocation_kind = parameter.callable_invocation_kind?;

    if invocation_kind == CallableInvocationKind::Immediately
        && parameter.flags.is_variadic()
        && metadata.flags.is_built_in()
        && let Some(callback_count) = target.get_function_like_identifier().and_then(variadic_builtin_callback_count)
        && (is_unpacked || argument_count < callback_count || argument_offset < argument_count - callback_count)
    {
        return None;
    }

    Some(invocation_kind)
}

fn variadic_builtin_callback_count(identifier: &FunctionLikeIdentifier) -> Option<usize> {
    let FunctionLikeIdentifier::Function(name) = identifier else {
        return None;
    };
    let name = name.as_bytes().rsplit(|byte| *byte == b'\\').next().unwrap_or(name.as_bytes());

    if [
        &b"array_diff_ukey"[..],
        b"array_udiff",
        b"array_udiff_assoc",
        b"array_diff_uassoc",
        b"array_uintersect_assoc",
        b"array_intersect_uassoc",
        b"array_intersect_ukey",
        b"array_uintersect",
    ]
    .iter()
    .any(|candidate| name.eq_ignore_ascii_case(candidate))
    {
        return Some(1);
    }

    if [b"array_udiff_uassoc".as_slice(), b"array_uintersect_uassoc"]
        .iter()
        .any(|candidate| name.eq_ignore_ascii_case(candidate))
    {
        return Some(2);
    }

    None
}

/// Returns catch filters inferred from the callee body or from a builtin callback contract.
///
/// `None` means neither the scanned function body nor current analysis artifacts provide a summary.
#[must_use]
pub(crate) fn callback_catch_filters(
    target: &InvocationTarget<'_>,
    parameter_offset: usize,
    artifacts: &AnalysisArtifacts,
    codebase: &CodebaseMetadata,
) -> Option<Vec<CallableExceptionCatchSummary>> {
    let metadata = target.get_function_like_metadata()?;
    if metadata.flags.is_built_in() {
        return Some(vec![CallableExceptionCatchSummary { catch_groups: Vec::new() }]);
    }

    let parameter = metadata.parameters.get(parameter_offset)?;
    let mut parameter_catches = parameter.callable_exception_catches.clone();
    if let Some(catches) = &mut parameter_catches {
        for (summary_offset, fallback_function) in &parameter.callable_exception_catch_global_fallbacks {
            if codebase.get_function_like(&FunctionLikeIdentifier::Function(*fallback_function)).is_some()
                && let Some(summary) = catches.get_mut(*summary_offset)
            {
                summary.catch_groups.clear();
            }
        }
    }
    let identifier = target.get_function_like_identifier()?;
    let analyzed_catches = artifacts.get_callback_parameter_effects(identifier, parameter_offset).map(<[_]>::to_vec);
    if analyzed_catches.as_ref().is_some_and(|summaries| {
        summaries.iter().flat_map(|summary| &summary.catch_groups).any(|group| group.overrides_pending_exceptions)
    }) {
        return analyzed_catches;
    }

    match (parameter_catches, analyzed_catches) {
        (Some(mut scanned), Some(analyzed)) => {
            for invocation_catch in analyzed {
                if !scanned.contains(&invocation_catch) {
                    scanned.push(invocation_catch);
                }
            }
            Some(scanned)
        }
        (Some(scanned), None) | (None, Some(scanned)) => Some(scanned),
        (None, None) => None,
    }
}

/// Returns whether a callback exception escapes all invocation sites represented by the callee contract.
#[must_use]
pub(crate) fn callback_exception_escapes(
    exception: Word,
    callsite: &CallbackCallsiteContext,
    codebase: &CodebaseMetadata,
) -> bool {
    callback_exception_escapes_filters(exception, callsite.caught_exceptions.as_deref(), codebase)
}

pub(crate) fn record_callable_effects(
    span: Span,
    effects: CallableEffectSummary,
    is_immediate_callback: bool,
    check_throws: bool,
    block_context: &mut BlockContext<'_>,
    artifacts: &mut AnalysisArtifacts,
    codebase: &CodebaseMetadata,
) {
    if is_immediate_callback
        && check_throws
        && let Some(callsite) = artifacts.get_callback_callsite(span)
    {
        for (exception, spans) in &effects.possibly_thrown_exceptions {
            if callback_exception_escapes(*exception, callsite, codebase) {
                block_context.possibly_thrown_exceptions.entry(*exception).or_default().extend(spans);
            }
        }
    }

    artifacts.record_callable_effects(span, effects);
}

#[must_use]
pub(crate) fn callback_exception_escapes_filters(
    exception: Word,
    invocation_catches: Option<&[CallableExceptionCatchSummary]>,
    codebase: &CodebaseMetadata,
) -> bool {
    let Some(invocation_catches) = invocation_catches else {
        return true;
    };

    invocation_catches.iter().any(|summary| {
        if summary.catch_groups.iter().any(|group| group.overrides_pending_exceptions) {
            return false;
        }

        let matches_type = |caught: &Word| {
            exception.as_bytes().eq_ignore_ascii_case(caught.as_bytes())
                || codebase.is_instance_of(exception.as_bytes(), caught.as_bytes())
        };

        for group in &summary.catch_groups {
            if let Some(arm) = group.arms.iter().find(|arm| arm.exception_types.iter().any(&matches_type)) {
                if arm.rethrows {
                    continue;
                }

                if arm.rethrows_types.iter().any(|thrown| exception.as_bytes().eq_ignore_ascii_case(thrown.as_bytes()))
                {
                    continue;
                }

                if arm.rethrows_caught_types.iter().any(|thrown| {
                    exception.as_bytes().eq_ignore_ascii_case(thrown.as_bytes())
                        || codebase.is_instance_of(exception.as_bytes(), thrown.as_bytes())
                }) {
                    continue;
                }

                if arm.rethrows_with_catches.iter().any(|thrown| {
                    let is_caught = thrown.caught_types.iter().any(|caught| {
                        if thrown.caught_variable {
                            matches_type(caught)
                        } else {
                            codebase.is_instance_of(thrown.exception_type.as_bytes(), caught.as_bytes())
                        }
                    });

                    let matches_rethrown_type = if thrown.caught_variable {
                        exception.as_bytes().eq_ignore_ascii_case(thrown.exception_type.as_bytes())
                            || codebase.is_instance_of(exception.as_bytes(), thrown.exception_type.as_bytes())
                    } else {
                        exception.as_bytes().eq_ignore_ascii_case(thrown.exception_type.as_bytes())
                    };

                    !is_caught && matches_rethrown_type
                }) {
                    continue;
                }

                return false;
            }
        }

        true
    })
}
/// Returns the parameter offset when an expression directly invokes an immediately invoked callable parameter.
#[must_use]
pub(crate) fn immediate_callable_parameter_offset(
    block_context: &BlockContext<'_>,
    expression: &Expression<'_>,
) -> Option<usize> {
    let Expression::Variable(variable) = unparenthesized_expression(expression) else {
        return None;
    };
    let variable_name = crate::utils::expression::get_variable_id(variable)?;
    let variable_name = variable_name.strip_prefix(b"$").unwrap_or(variable_name);
    let function_like = block_context.scope.get_function_like()?;

    function_like.parameters.iter().position(|parameter| {
        parameter.callable_invocation_kind == Some(CallableInvocationKind::Immediately)
            && parameter.get_name().0.as_bytes().strip_prefix(b"$").unwrap_or(parameter.get_name().0.as_bytes())
                == variable_name
    })
}
