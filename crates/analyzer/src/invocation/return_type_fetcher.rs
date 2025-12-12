use mago_atom::AtomMap;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::template::TemplateResult;
use mago_codex::ttype::union::TUnion;

use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::invocation::Invocation;
use crate::invocation::resolver::resolve_invocation_type;

pub fn fetch_invocation_return_type<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    invocation: &Invocation<'ctx, 'ast, 'arena>,
    template_result: &TemplateResult,
    parameters: &AtomMap<TUnion>,
) -> TUnion {
    // Try to get a custom return type from plugins
    if let Some(identifier) = invocation.target.get_function_like_identifier()
        && let Some(result) = context.plugin_registry.get_function_like_return_type(
            context.codebase,
            block_context,
            artifacts,
            identifier,
            invocation,
        )
    {
        for reported_issue in result.issues {
            context.collector.report_with_code(reported_issue.code, reported_issue.issue);
        }

        if let Some(ty) = result.return_type {
            return ty;
        }
    }

    let mut resulting_type = if let Some(return_type) = invocation.target.get_return_type().cloned() {
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
