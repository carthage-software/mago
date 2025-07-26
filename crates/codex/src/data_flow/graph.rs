use std::collections::VecDeque;

use ahash::HashMap;
use ahash::HashSet;

use mago_interner::StringIdentifier;
use mago_span::Position;
use mago_span::Span;

use crate::data_flow::node::DataFlowNode;
use crate::data_flow::node::DataFlowNodeId;
use crate::data_flow::node::DataFlowNodeKind;
use crate::data_flow::node::VariableSourceKind;
use crate::data_flow::path::DataFlowPath;
use crate::data_flow::path::PathKind;
use crate::identifier::function_like::FunctionLikeIdentifier;
use crate::misc::VariableIdentifier;
use crate::ttype::union::TUnion;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphKind {
    FunctionBody,
    WholeProgram,
}

#[derive(Debug, Clone)]
pub struct DataFlowGraph {
    pub kind: GraphKind,
    pub vertices: HashMap<DataFlowNodeId, DataFlowNode>,
    pub forward_edges: HashMap<DataFlowNodeId, HashMap<DataFlowNodeId, DataFlowPath>>,
    pub backward_edges: HashMap<DataFlowNodeId, HashSet<DataFlowNodeId>>,
    pub sources: HashMap<DataFlowNodeId, DataFlowNode>,
    pub sinks: HashMap<DataFlowNodeId, DataFlowNode>,
    pub mixed_source_counts: HashMap<DataFlowNodeId, HashSet<String>>,
    pub specializations: HashMap<DataFlowNodeId, HashSet<Position>>,
    pub specialized_calls: HashMap<Position, HashSet<DataFlowNodeId>>,
}

impl DataFlowGraph {
    pub fn new(kind: GraphKind) -> Self {
        Self {
            kind,
            vertices: HashMap::default(),
            forward_edges: HashMap::default(),
            backward_edges: HashMap::default(),
            sources: HashMap::default(),
            sinks: HashMap::default(),
            mixed_source_counts: HashMap::default(),
            specializations: HashMap::default(),
            specialized_calls: HashMap::default(),
        }
    }

    /// Adds a node to the graph, placing it in the appropriate collection
    /// (vertices, sources, or sinks) based on its kind.
    /// Also handles tracking specializations in WholeProgram graphs.
    pub fn add_node(&mut self, node: DataFlowNode) {
        let node_id = node.id.clone();

        match &node.kind {
            DataFlowNodeKind::Vertex { is_specialized, .. } => {
                if let GraphKind::WholeProgram = self.kind
                    && *is_specialized
                {
                    let (unspecialized_id, specialization_key) = node.id.unspecialize();
                    self.specializations.entry(unspecialized_id.clone()).or_default().insert(specialization_key);
                    self.specialized_calls.entry(specialization_key).or_default().insert(unspecialized_id);
                }
                self.vertices.insert(node_id, node);
            }
            DataFlowNodeKind::VariableUseSource { .. }
            | DataFlowNodeKind::DataSource { .. }
            | DataFlowNodeKind::ForLoopInit { .. } => {
                self.sources.insert(node_id, node);
            }
            DataFlowNodeKind::VariableUseSink { .. } => {
                self.sinks.insert(node_id, node);
            }
        }
    }

    /// Adds a directed edge (path) between two nodes in the graph.
    pub fn add_path(&mut self, from: &DataFlowNode, to: &DataFlowNode, path_kind: PathKind) {
        let from_id = &from.id;
        let to_id = &to.id;

        // Avoid self-loops
        if from_id == to_id {
            return;
        }

        // Add backward edge only for FunctionBody graphs (used for origin tracing)
        if self.kind == GraphKind::FunctionBody {
            self.backward_edges.entry(to_id.clone()).or_default().insert(from_id.clone());
        }

        // Add forward edge using entry API
        self.forward_edges.entry(from_id.clone()).or_default().insert(to_id.clone(), DataFlowPath { kind: path_kind });
    }

    /// Merges another DataFlowGraph into this one.
    ///
    /// # Panics
    ///
    /// If the two graphs have different kinds, this function will panic.
    pub fn add_graph(&mut self, graph: DataFlowGraph) {
        if self.kind != graph.kind {
            panic!("cannot merge data flow graphs of different kinds");
        }

        // Merge forward edges
        for (key, edges_to_add) in graph.forward_edges {
            self.forward_edges.entry(key).or_default().extend(edges_to_add);
        }

        // Merge context specific fields
        if self.kind == GraphKind::FunctionBody {
            // Merge backward edges
            for (key, edges_to_add) in graph.backward_edges {
                self.backward_edges.entry(key).or_default().extend(edges_to_add);
            }
            // Merge mixed source counts
            for (key, counts_to_add) in graph.mixed_source_counts {
                self.mixed_source_counts.entry(key).or_default().extend(counts_to_add);
            }
        } else {
            // Merge specializations (WholeProgram only)
            for (key, specializations_to_add) in graph.specializations {
                self.specializations.entry(key).or_default().extend(specializations_to_add);
            }

            // Merge specialized_calls (WholeProgram only)
            for (key, calls_to_add) in graph.specialized_calls {
                self.specialized_calls.entry(key).or_default().extend(calls_to_add);
            }
        }

        self.vertices.extend(graph.vertices);
        self.sources.extend(graph.sources);
        self.sinks.extend(graph.sinks);
    }

    /// Performs a backward Breadth-First Search (BFS) from a starting node
    /// to find all reachable origin nodes (sources or vertices without incoming edges within the traversal scope).
    /// See: https://en.wikipedia.org/wiki/Breadth-first_search
    ///
    /// # Arguments
    ///
    /// * `start_node_id` - The ID of the node to start the backward traversal from.
    /// * `ignore_paths` - A slice of `PathKind` values. If the path from a parent to a child
    ///   matches any of these kinds, that parent path is not followed.
    /// * `var_ids_only` - If true, only returns origin nodes that represent variables or parameters.
    ///
    /// # Returns
    ///
    /// A vector of `DataFlowNodeId` representing the identified origin nodes.
    pub fn get_origin_node_ids(
        &self,
        start_node_id: &DataFlowNodeId,
        ignore_paths: &[PathKind],
        var_ids_only: bool,
    ) -> Vec<DataFlowNodeId> {
        let mut origins = Vec::new();
        let mut queue = VecDeque::new();
        let mut visited = HashSet::default();

        // Check if the starting node itself exists in the graph (vertices or sources)
        if self.vertices.contains_key(start_node_id) || self.sources.contains_key(start_node_id) {
            if visited.insert(start_node_id.clone()) {
                queue.push_back(start_node_id);
            }
        } else {
            // Start node doesn't exist in the graph context we care about
            return origins;
        }

        while let Some(current_id) = queue.pop_front() {
            let mut has_valid_parents = false;

            if let Some(parents) = self.backward_edges.get(current_id) {
                for parent_id in parents {
                    let should_ignore = self
                        .forward_edges
                        .get(parent_id)
                        .and_then(|edges| edges.get(current_id))
                        .is_some_and(|path| ignore_paths.contains(&path.kind));

                    if should_ignore {
                        continue;
                    }

                    if self.vertices.contains_key(parent_id) || self.sources.contains_key(parent_id) {
                        has_valid_parents = true;
                        if visited.insert(parent_id.clone()) {
                            // Avoid cycles and redundant work
                            queue.push_back(parent_id);
                        }
                    }
                }
            }

            if !has_valid_parents
                && (!var_ids_only || matches!(current_id, DataFlowNodeId::Var(..) | DataFlowNodeId::Parameter(..)))
            {
                origins.push(current_id.clone());
            }
        }

        origins
    }

    /// Returns an iterator over all nodes in the graph (sources, vertices, and sinks).
    pub fn get_all_nodes(&self) -> impl Iterator<Item = &DataFlowNode> {
        self.sources.values().chain(self.vertices.values()).chain(self.sinks.values())
    }

    /// Helper function to get a node reference from any of the graph's collections.
    #[inline]
    pub fn get_node(&self, id: &DataFlowNodeId) -> Option<&DataFlowNode> {
        // Use or_else for potentially slightly cleaner chaining, though performance difference is likely negligible.
        self.vertices.get(id).or_else(|| self.sources.get(id)).or_else(|| self.sinks.get(id))
    }

    /// Records that a 'mixed' type originated from certain source nodes.
    /// Finds the origins of the assignment node and updates counts for specific source types.
    pub fn add_mixed_data(&mut self, assignment_node: &DataFlowNode, span: Span) {
        let origin_node_ids = self.get_origin_node_ids(&assignment_node.id, &[], false);
        let span_string = span.to_string();

        for origin_node_id in origin_node_ids {
            // Check if the origin is a call-related node
            if matches!(origin_node_id, DataFlowNodeId::CallTo(..) | DataFlowNodeId::SpecializedCallTo(..)) {
                self.mixed_source_counts.entry(origin_node_id).or_default().insert(span_string.clone());
            }
        }
    }

    /// Traces back from a TUnion's parent nodes to find originating function calls.
    pub fn get_source_functions(&self, expr_type: &TUnion, ignore_paths: &[PathKind]) -> Vec<FunctionLikeIdentifier> {
        let mut source_functions = Vec::new();
        let mut origin_node_ids = Vec::new();
        // Collect origin node IDs from parent nodes
        for parent_node in &expr_type.parent_nodes {
            // Use the optimized origin tracing
            let parent_origin_node_ids = self.get_origin_node_ids(&parent_node.id, ignore_paths, false);

            origin_node_ids.extend(parent_origin_node_ids);
        }

        let mut visited_origins = HashSet::default();
        for origin_node_id in &origin_node_ids {
            // Only process each unique origin once
            if !visited_origins.insert(origin_node_id) {
                continue;
            }

            if let DataFlowNodeId::CallTo(id) | DataFlowNodeId::SpecializedCallTo(id, ..) = origin_node_id {
                // Verify it's a Vertex node before adding (optional safety check)
                if let Some(origin_node) = self.get_node(origin_node_id)
                    && matches!(origin_node.kind, DataFlowNodeKind::Vertex { .. })
                {
                    source_functions.push(*id);
                }
            }
        }

        source_functions
    }

    /// Traces back from a TUnion's parent nodes to find originating property accesses.
    pub fn get_source_properties(&self, expr_type: &TUnion) -> Vec<(StringIdentifier, StringIdentifier)> {
        let mut source_properties = Vec::new();
        let mut origin_node_ids = Vec::new();
        // Collect origin node IDs from parent nodes
        for parent_node in &expr_type.parent_nodes {
            // Use the optimized origin tracing
            let parent_origin_node_ids = self.get_origin_node_ids(&parent_node.id, &[], false);

            origin_node_ids.extend(parent_origin_node_ids);
        }

        let mut visited_origins = HashSet::default();

        for origin_node_id in &origin_node_ids {
            // Only process each unique origin once
            if !visited_origins.insert(origin_node_id) {
                continue;
            }

            if let DataFlowNodeId::Property(a, b) | DataFlowNodeId::SpecializedProperty(a, b, ..) = origin_node_id {
                source_properties.push((*a, *b));
            }
        }

        source_properties
    }

    /// Checks if a TUnion originates from a function parameter source node.
    pub fn is_from_param(&self, variable_type: &TUnion) -> bool {
        let mut visited_origins = HashSet::default();
        // Iterate through parent nodes of the TUnion
        let mut origin_node_ids = Vec::new();
        for parent_node in &variable_type.parent_nodes {
            // Use the optimized origin tracing, ignore_paths is empty slice
            let parent_origin_node_ids = self.get_origin_node_ids(&parent_node.id, &[], false);

            origin_node_ids.extend(parent_origin_node_ids);
        }

        for origin_node_id in &origin_node_ids {
            // Check each unique origin only once
            if visited_origins.insert(origin_node_id)
                && let Some(node) = self.get_node(origin_node_id)
                && let DataFlowNodeKind::VariableUseSource { kind, .. } = &node.kind
                && matches!(kind, VariableSourceKind::PrivateParameter | VariableSourceKind::NonPrivateParameter)
            {
                return true; // Found a parameter source, no need to check further
            }
        }

        false // No parameter source found after checking all origins
    }

    /// Finds all assignment/initialization spans for a given variable.
    ///
    /// This function iterates through the graph's source nodes to find all points
    /// where the specified variable is assigned a value. This includes direct assignments
    /// and initializations in `for` loops.
    ///
    /// # Arguments
    ///
    /// * `variable_to_find` - The `VariableIdentifier` of the variable to search for.
    ///
    /// # Returns
    ///
    /// A `Vec<Span>` containing the spans of all assignment locations for the variable.
    /// The vector is sorted by the start offset of the spans and duplicates are removed.
    pub fn get_variable_assignment_spans(&self, variable_to_find: VariableIdentifier) -> Vec<Span> {
        let mut assignment_spans_set = HashSet::default();

        for node in self.sources.values() {
            if let DataFlowNodeId::Var(label, assignment_site_span) = &node.id
                && *label == variable_to_find
                && matches!(node.kind, DataFlowNodeKind::VariableUseSource { .. })
            {
                assignment_spans_set.insert(*assignment_site_span);
            }
        }

        let mut sorted_spans: Vec<Span> = assignment_spans_set.into_iter().collect();
        sorted_spans.sort_by_key(|s| s.start.offset);

        sorted_spans
    }

    /// Finds the most recent assignment span for a given variable.
    ///
    /// This function iterates through the graph's source nodes to find the most recent
    /// assignment span for the specified variable. It returns the span with the highest
    /// start offset.
    ///
    /// # Arguments
    ///
    /// * `variable_to_find` - The `VariableIdentifier` of the variable to search for.
    ///
    /// # Returns
    ///
    /// An `Option<Span>` containing the span of the most recent assignment location for the variable.
    pub fn get_variable_most_recent_assignment_span(&self, variable_to_find: VariableIdentifier) -> Option<Span> {
        let mut assignment_spans_set = HashSet::default();

        for node in self.sources.values() {
            if let DataFlowNodeId::Var(label, assignment_site_span) = &node.id
                && *label == variable_to_find
                && matches!(node.kind, DataFlowNodeKind::VariableUseSource { .. })
            {
                assignment_spans_set.insert(*assignment_site_span);
            }
        }

        assignment_spans_set.into_iter().max_by_key(|s| s.start.offset)
    }
}
