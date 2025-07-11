use ahash::HashMap;

use mago_codex::data_flow::graph::GraphKind;
use mago_codex::data_flow::node::DataFlowNode;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::ttype::get_literal_int;
use mago_codex::ttype::get_mixed_any;
use mago_codex::ttype::template::TemplateResult;
use mago_codex::ttype::union::TUnion;
use mago_interner::StringIdentifier;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::invocation::Invocation;
use crate::invocation::InvocationArgumentsSource;
use crate::invocation::InvocationTarget;
use crate::invocation::resolver::resolve_invocation_type;

pub fn fetch_invocation_return_type(
    context: &mut Context<'_>,
    artifacts: &mut AnalysisArtifacts,
    invocation: &Invocation<'_>,
    template_result: &TemplateResult,
    parameters: &HashMap<StringIdentifier, TUnion>,
) -> TUnion {
    if let Some(FunctionLikeIdentifier::Function(name)) = invocation.target.get_function_like_identifier()
        && let Some(return_type) = handle_special_functions(context, artifacts, name, invocation.arguments_source)
    {
        return add_dataflow(artifacts, invocation.span, &invocation.target, return_type);
    }

    let Some(return_type) = invocation.target.get_return_type().cloned() else {
        return add_dataflow(artifacts, invocation.span, &invocation.target, get_mixed_any());
    };

    let function_return_type = resolve_invocation_type(context, invocation, template_result, parameters, return_type);

    add_dataflow(artifacts, invocation.span, &invocation.target, function_return_type)
}

fn handle_special_functions(
    context: &mut Context<'_>,
    artifacts: &AnalysisArtifacts,
    function_name: &StringIdentifier,
    call_arguments: InvocationArgumentsSource<'_>,
) -> Option<TUnion> {
    fn get_argument<'argument>(
        context: &Context<'_>,
        call_arguments: InvocationArgumentsSource<'argument>,
        index: usize,
        names: Vec<&'static str>,
    ) -> Option<&'argument Expression> {
        match call_arguments {
            InvocationArgumentsSource::ArgumentList(argument_list) => {
                if let Some(Argument::Positional(argument)) = argument_list.arguments.get(index) {
                    return Some(&argument.value);
                }

                for argument in argument_list.arguments.iter() {
                    let Argument::Named(named_argument) = argument else {
                        continue;
                    };

                    let name = context.interner.lookup(&named_argument.name.value);
                    if names.contains(&name) {
                        return Some(&named_argument.value);
                    }
                }

                None
            }
            InvocationArgumentsSource::PipeInput(pipe) => {
                if index == 0 {
                    Some(&pipe.input)
                } else {
                    None
                }
            }
            InvocationArgumentsSource::LanguageConstructExpressions(_) => None,
            InvocationArgumentsSource::None(_) => None,
        }
    }

    let lowercase_name = context.interner.lookup(function_name).to_ascii_lowercase();
    match lowercase_name.as_ref() {
        "strlen" => {
            let string_argument = get_argument(context, call_arguments, 0, vec!["string"])?;
            let string_argument_type = artifacts.get_expression_type(string_argument)?;
            let string_literal = string_argument_type.get_single_literal_string_value()?;

            Some(get_literal_int(string_literal.len() as i64))
        }
        _ => None,
    }
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
