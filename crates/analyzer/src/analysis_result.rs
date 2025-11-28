use std::time::Duration;

use mago_codex::reference::SymbolReferences;
use mago_reporting::IssueCollection;

#[derive(Clone, Debug)]
pub struct AnalysisResult {
    pub issues: IssueCollection,
    pub symbol_references: SymbolReferences,
    #[cfg(not(target_arch = "wasm32"))]
    pub time_in_analysis: Duration,
}

impl AnalysisResult {
    pub fn new(symbol_references: SymbolReferences) -> Self {
        Self {
            issues: IssueCollection::default(),
            symbol_references,
            #[cfg(not(target_arch = "wasm32"))]
            time_in_analysis: Duration::default(),
        }
    }

    pub fn extend(&mut self, other: Self) {
        self.issues.extend(other.issues);
        self.symbol_references.extend(other.symbol_references);
    }
}
