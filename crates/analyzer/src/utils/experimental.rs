use mago_allocator::Arena;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::Span;
use mago_word::Word;

use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;

/// Checks if using an experimental class-like from a non-experimental context and reports a warning.
pub fn check_experimental_class_like<A>(
    context: &mut Context<'_, '_, A>,
    block_context: &BlockContext<'_>,
    class_name: Word,
    span: Span,
) where
    A: Arena,
{
    if !context.settings.check_experimental {
        return;
    }

    let Some(metadata) = context.codebase.get_class_like(class_name.as_bytes()) else {
        return;
    };

    if !metadata.flags.is_experimental() {
        return;
    }

    if is_current_context_experimental(block_context) {
        return;
    }

    context.collector.report_with_code(
        IssueCode::ExperimentalUsage,
        Issue::warning(format!("Usage of experimental class-like `{}`.", metadata.original_name))
            .with_annotation(
                Annotation::primary(span)
                    .with_message(format!("`{}` is marked as `@experimental`.", metadata.original_name)),
            )
            .with_note("Experimental APIs may change or be removed without notice.")
            .with_help("Mark the current function or class as `@experimental` to suppress this warning."),
    );
}

/// Checks if calling an experimental function from a non-experimental context and reports a warning.
///
/// Accepts pre-resolved metadata to avoid redundant lookups.
pub fn check_experimental_function_with_metadata<A>(
    context: &mut Context<'_, '_, A>,
    block_context: &BlockContext<'_>,
    metadata: &FunctionLikeMetadata,
    span: Span,
) where
    A: Arena,
{
    if !context.settings.check_experimental {
        return;
    }

    if !metadata.flags.is_experimental() {
        return;
    }

    if is_current_context_experimental(block_context) {
        return;
    }

    let display_name = metadata.original_name;

    context.collector.report_with_code(
        IssueCode::ExperimentalUsage,
        Issue::warning(format!("Usage of experimental function `{display_name}`."))
            .with_annotation(
                Annotation::primary(span).with_message(format!("`{display_name}` is marked as `@experimental`.")),
            )
            .with_note("Experimental APIs may change or be removed without notice.")
            .with_help("Mark the current function or class as `@experimental` to suppress this warning."),
    );
}

/// Checks if using an experimental constant from a non-experimental context and reports a warning.
///
/// Accepts pre-resolved flags to avoid redundant lookups.
pub fn check_experimental_constant<A>(
    context: &mut Context<'_, '_, A>,
    block_context: &BlockContext<'_>,
    flags: &mago_codex::metadata::flags::MetadataFlags,
    constant_name: &[u8],
    span: Span,
) where
    A: Arena,
{
    if !context.settings.check_experimental {
        return;
    }

    if !flags.is_experimental() {
        return;
    }

    if is_current_context_experimental(block_context) {
        return;
    }

    let constant_name = mago_bytes::BytesDisplay(constant_name);
    context.collector.report_with_code(
        IssueCode::ExperimentalUsage,
        Issue::warning(format!("Usage of experimental constant `{constant_name}`."))
            .with_annotation(
                Annotation::primary(span).with_message(format!("`{constant_name}` is marked as `@experimental`.")),
            )
            .with_note("Experimental APIs may change or be removed without notice.")
            .with_help("Mark the current function or class as `@experimental` to suppress this warning."),
    );
}

/// Checks if the current analysis context is itself marked as experimental.
fn is_current_context_experimental(block_context: &BlockContext<'_>) -> bool {
    if let Some(class_like) = block_context.scope.get_class_like()
        && class_like.flags.is_experimental()
    {
        return true;
    }

    // Check if the current function/method is experimental
    if let Some(function_meta) = block_context.scope.get_function_like()
        && function_meta.flags.is_experimental()
    {
        return true;
    }

    false
}
