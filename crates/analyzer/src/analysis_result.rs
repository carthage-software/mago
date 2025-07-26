use std::collections::BTreeMap;
use std::time::Duration;

use ahash::HashMap;
use ahash::HashSet;

use mago_codex::data_flow::graph::DataFlowGraph;
use mago_codex::data_flow::graph::GraphKind;
use mago_codex::data_flow::node::DataFlowNodeId;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::reference::SymbolReferences;
use mago_interner::ThreadedInterner;
use mago_reporting::Issue;
use mago_source::SourceIdentifier;

#[derive(Clone, Debug)]
pub struct AnalysisResult {
    pub emitted_issues: HashMap<SourceIdentifier, Vec<Issue>>,
    pub emitted_definition_issues: HashMap<SourceIdentifier, Vec<Issue>>,
    pub mixed_source_counts: HashMap<DataFlowNodeId, HashSet<String>>,
    pub program_dataflow_graph: DataFlowGraph,
    pub symbol_references: SymbolReferences,
    pub issue_counts: HashMap<String, usize>,
    pub time_in_analysis: Duration,
    pub functions_to_migrate: HashMap<FunctionLikeIdentifier, bool>,
    pub has_invalid_hack_files: bool,
    pub changed_during_analysis_files: HashSet<SourceIdentifier>,
}

impl AnalysisResult {
    pub fn new(graph_kind: GraphKind, symbol_references: SymbolReferences) -> Self {
        Self {
            emitted_issues: HashMap::default(),
            emitted_definition_issues: HashMap::default(),
            mixed_source_counts: HashMap::default(),
            program_dataflow_graph: DataFlowGraph::new(graph_kind),
            issue_counts: HashMap::default(),
            symbol_references,
            time_in_analysis: Duration::default(),
            functions_to_migrate: HashMap::default(),
            has_invalid_hack_files: false,
            changed_during_analysis_files: HashSet::default(),
        }
    }

    pub fn extend(&mut self, other: Self) {
        for (file_path, issues) in other.emitted_issues {
            self.emitted_issues.entry(file_path).or_default().extend(issues);
        }

        for (id, c) in other.mixed_source_counts {
            self.mixed_source_counts.entry(id).or_default().extend(c);
        }

        self.program_dataflow_graph.add_graph(other.program_dataflow_graph);
        self.symbol_references.extend(other.symbol_references);

        for (kind, count) in other.issue_counts {
            *self.issue_counts.entry(kind).or_insert(0) += count;
        }

        self.functions_to_migrate.extend(other.functions_to_migrate);
        self.changed_during_analysis_files.extend(other.changed_during_analysis_files);
        self.has_invalid_hack_files = self.has_invalid_hack_files || other.has_invalid_hack_files;
    }

    pub fn get_all_issues(&self, interner: &ThreadedInterner) -> BTreeMap<String, Vec<&Issue>> {
        let mut issues = self
            .emitted_issues
            .iter()
            .filter(|(_, v)| !v.is_empty())
            .map(|(k, v)| (interner.lookup(&k.0).to_string(), v.iter().collect::<Vec<_>>()))
            .collect::<BTreeMap<_, _>>();

        for (file_path, file_definition_issues) in &self.emitted_definition_issues {
            let file_path = interner.lookup(&file_path.0).to_string();

            if let Some(file_issues) = issues.get_mut(&file_path) {
                file_issues.extend(file_definition_issues);
            } else {
                let file_issues: Vec<_> = file_definition_issues.iter().collect::<Vec<_>>();
                if file_issues.is_empty() {
                    continue;
                }

                issues.insert(file_path, file_issues);
            }
        }

        issues
    }
}
