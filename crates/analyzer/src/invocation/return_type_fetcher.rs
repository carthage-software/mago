use mago_atom::AtomMap;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::template::TemplateResult;
use mago_codex::ttype::union::TUnion;

use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::invocation::Invocation;
use crate::invocation::resolver::resolve_invocation_type;
use crate::invocation::special_function_like_handler::handle_special_functions;

pub fn fetch_invocation_return_type<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    invocation: &Invocation<'ctx, 'ast, 'arena>,
    template_result: &TemplateResult,
    parameters: &AtomMap<TUnion>,
) -> TUnion {
    if let Some(return_type) = handle_special_functions(context, block_context, artifacts, invocation) {
        return return_type;
    }

    let mut resulting_type = if let Some(return_type) = invocation.target.get_return_type().cloned() {
        resolve_invocation_type(context, invocation, template_result, parameters, return_type)
    } else {
        get_mixed()
    };

    if let Some(function_like_metadata) = invocation.target.get_function_like_metadata()
        && function_like_metadata.flags.is_by_reference()
    {
        resulting_type.by_reference = true;
    }

    resulting_type
}
