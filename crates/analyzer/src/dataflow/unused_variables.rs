use std::collections::VecDeque;

use ahash::HashSet;

use mago_codex::data_flow::graph::DataFlowGraph;
use mago_codex::data_flow::node::DataFlowNode;
use mago_codex::data_flow::node::DataFlowNodeId;
use mago_codex::data_flow::node::DataFlowNodeKind;

/// Identifies unused variable source nodes in the data flow graph.
///
/// Performs a backward BFS traversal from all sink nodes to mark nodes
/// that contribute to a potential usage (reaching a sink). Variable source
/// nodes not visited during this backward traversal are considered unused.
///
/// # Arguments
///
/// * `graph` - The DataFlowGraph containing nodes and edges.
///
/// # Returns
///
/// A tuple containing two vectors:
///
/// 1. `unused_pure_nodes`: Nodes corresponding to unused variables declared as 'pure',
///    likely indicating completely dead code.
/// 2. `unused_impure_nodes`: Nodes corresponding to unused variables that are not 'pure',
///    the variable assignment might have side effects, but the value is unused.
#[inline]
pub fn check_variables_used(graph: &DataFlowGraph) -> (Vec<&DataFlowNode>, Vec<&DataFlowNode>) {
    let mut queue: VecDeque<&DataFlowNodeId> = VecDeque::new();
    let mut used_node_ids: HashSet<&DataFlowNodeId> = HashSet::default();

    for sink_id in graph.sinks.keys() {
        if used_node_ids.insert(sink_id) {
            queue.push_back(sink_id);
        }
    }

    while let Some(current_id) = queue.pop_front() {
        if let Some(predecessors) = graph.backward_edges.get(current_id) {
            for pred_id in predecessors {
                if used_node_ids.insert(pred_id) {
                    queue.push_back(pred_id);
                }
            }
        }
    }

    let mut unused_pure_nodes = Vec::new();
    let mut unused_impure_nodes = Vec::new();

    for (source_id, source_node) in graph.sources.iter() {
        if !used_node_ids.contains(source_id) {
            if let DataFlowNodeKind::VariableUseSource { pure, .. } = source_node.kind {
                if pure {
                    unused_pure_nodes.push(source_node);
                } else {
                    unused_impure_nodes.push(source_node);
                }
            } else {
                unused_impure_nodes.push(source_node);
            }
        }
    }

    (unused_pure_nodes, unused_impure_nodes)
}
