use mago_allocator::Arena;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::template::TemplateResult;
use mago_codex::ttype::union::TUnion;
use mago_word::WordMap;

use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::invocation::Invocation;
use crate::invocation::resolver::resolve_invocation_type;

pub fn fetch_invocation_return_type<'ctx, 'arena, A>(
    context: &mut Context<'ctx, 'arena, A>,
    block_context: &BlockContext<'ctx>,
    artifacts: &AnalysisArtifacts,
    invocation: &Invocation<'ctx, '_, 'arena>,
    template_result: &TemplateResult,
    parameters: &WordMap<TUnion>,
) -> Result<TUnion, AnalysisError>
where
    A: Arena,
{
    if let Some(return_type) = fetch_invocation_provider_return_type(context, block_context, artifacts, invocation) {
        return Ok(return_type);
    }

    Ok(fetch_declared_invocation_return_type(context, invocation, template_result, parameters))
}

/// Requests a custom return type from registered providers and reports provider issues.
///
pub(crate) fn fetch_invocation_provider_return_type<'ctx, 'arena, A>(
    context: &mut Context<'ctx, 'arena, A>,
    block_context: &BlockContext<'ctx>,
    artifacts: &AnalysisArtifacts,
    invocation: &Invocation<'ctx, '_, 'arena>,
) -> Option<TUnion>
where
    A: Arena,
{
    let identifier = invocation.target.get_function_like_identifier()?;

    fetch_function_like_provider_return_type(context, block_context, artifacts, identifier, invocation)
}

/// Requests a custom return type for an explicit function-like identifier.
///
/// This permits dynamic method calls to match the method requested by the user
/// while retaining `__call` or `__callStatic` as the invocation's native target.
///
pub(crate) fn fetch_function_like_provider_return_type<'ctx, 'arena, A>(
    context: &mut Context<'ctx, 'arena, A>,
    block_context: &BlockContext<'ctx>,
    artifacts: &AnalysisArtifacts,
    identifier: &FunctionLikeIdentifier,
    invocation: &Invocation<'ctx, '_, 'arena>,
) -> Option<TUnion>
where
    A: Arena,
{
    if let Some(result) = context.plugin_registry.get_function_like_return_type(
        context.codebase,
        context.source_file,
        block_context,
        artifacts,
        identifier,
        invocation,
        context.external_analysis_session,
    ) {
        for reported_issue in result.issues {
            context.collector.report_with_code(reported_issue.code, reported_issue.issue);
        }

        if let Some(ty) = result.return_type {
            return Some(ty);
        }
    }

    None
}

/// Resolves the declared return type of an invocation without consulting providers.
pub(crate) fn fetch_declared_invocation_return_type<A>(
    context: &Context<'_, '_, A>,
    invocation: &Invocation<'_, '_, '_>,
    template_result: &TemplateResult,
    parameters: &WordMap<TUnion>,
) -> TUnion
where
    A: Arena,
{
    let target_return_type = if let Some(function_like_metadata) = invocation.target.get_function_like_metadata()
        && let Some(docblock_return) = function_like_metadata.return_type_metadata.as_ref()
        && docblock_return.from_docblock
        && !docblock_return.type_union.has_template_types()
        && !docblock_return.type_union.is_generic_parameter()
        && !docblock_return.type_union.types.iter().any(|t| {
            t.is_derived()
                || t.is_conditional()
                || matches!(t, TAtomic::Alias(_) | TAtomic::Reference(_) | TAtomic::GenericParameter(_))
        })
        && let Some(native_return) = function_like_metadata.return_type_declaration_metadata.as_ref()
        && ((native_return.type_union.has_object_type()
            && !docblock_return.type_union.has_object_type()
            && !docblock_return.type_union.has_mixed())
            || (native_return.type_union.is_void() && !docblock_return.type_union.is_void())
            || (native_return.type_union.is_array()
                && !docblock_return.type_union.is_array()
                && !docblock_return.type_union.is_list()
                && !docblock_return.type_union.has_iterable()
                && !docblock_return.type_union.has_mixed()))
    {
        Some(native_return.type_union.clone())
    } else {
        invocation.target.get_return_type().cloned()
    };

    let mut resulting_type = if let Some(return_type) = target_return_type {
        resolve_invocation_type(context, invocation, template_result, parameters, return_type)
    } else {
        get_mixed()
    };

    if let Some(function_like_metadata) = invocation.target.get_function_like_metadata()
        && function_like_metadata.flags.is_by_reference()
    {
        resulting_type.set_by_reference(true);
    }

    resulting_type
}
