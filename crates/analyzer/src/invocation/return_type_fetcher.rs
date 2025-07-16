use ahash::HashMap;

use mago_codex::data_flow::graph::GraphKind;
use mago_codex::data_flow::node::DataFlowNode;
use mago_codex::ttype::get_mixed_any;
use mago_codex::ttype::template::TemplateResult;
use mago_codex::ttype::union::TUnion;
use mago_interner::StringIdentifier;
use mago_span::Span;

use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::invocation::Invocation;
use crate::invocation::InvocationTarget;
use crate::invocation::resolver::resolve_invocation_type;
use crate::invocation::special_function_like_handler::handle_special_functions;

pub fn fetch_invocation_return_type(
    context: &mut Context<'_>,
    artifacts: &mut AnalysisArtifacts,
    invocation: &Invocation<'_>,
    template_result: &TemplateResult,
    parameters: &HashMap<StringIdentifier, TUnion>,
) -> TUnion {
    if let Some(return_type) = handle_special_functions(context, artifacts, invocation) {
        return add_dataflow(artifacts, invocation.span, &invocation.target, return_type);
    }

    let Some(return_type) = invocation.target.get_return_type().cloned() else {
        return add_dataflow(artifacts, invocation.span, &invocation.target, get_mixed_any());
    };

    let function_return_type = resolve_invocation_type(context, invocation, template_result, parameters, return_type);

    add_dataflow(artifacts, invocation.span, &invocation.target, function_return_type)
}

fn add_dataflow(
    artifacts: &mut AnalysisArtifacts,
    call_span: Span,
    call_target: &InvocationTarget<'_>,
    return_type: TUnion,
) -> TUnion {
    let InvocationTarget::FunctionLike { identifier, metadata, .. } = call_target else {
        return return_type;
    };

    let data_flow_graph = &mut artifacts.data_flow_graph;

    let mut return_type = return_type;
    let function_call_node = DataFlowNode::get_for_method_return(
        *identifier,
        if data_flow_graph.kind == GraphKind::FunctionBody {
            Some(call_span)
        } else if let Some(return_signature) = metadata.get_return_type_metadata() {
            Some(return_signature.span)
        } else {
            metadata.get_name_span()
        },
        if metadata.specialize_call { Some(call_span) } else { None },
    );

    data_flow_graph.add_node(function_call_node.clone());

    return_type.parent_nodes.push(function_call_node);
    return_type
}
