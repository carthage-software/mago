#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;

use mago_codex::reference::SymbolReferences;
use mago_collector::DeferredPragmas;
use mago_reporting::IssueCollection;

#[derive(Clone, Debug)]
pub struct AnalysisResult {
    pub issues: IssueCollection,
    pub symbol_references: SymbolReferences,
    deferred_pragmas: Vec<DeferredPragmas>,
    #[cfg(not(target_arch = "wasm32"))]
    pub time_in_analysis: Duration,
}

impl AnalysisResult {
    #[must_use]
    pub fn new(symbol_references: SymbolReferences) -> Self {
        Self {
            issues: IssueCollection::default(),
            symbol_references,
            deferred_pragmas: Vec::new(),
            #[cfg(not(target_arch = "wasm32"))]
            time_in_analysis: Duration::default(),
        }
    }

    pub fn extend(&mut self, other: Self) {
        self.issues.extend(other.issues);
        self.symbol_references.extend(other.symbol_references);
        self.deferred_pragmas.extend(other.deferred_pragmas);
    }

    pub(crate) fn add_deferred_pragmas(&mut self, pragmas: Option<DeferredPragmas>) {
        self.deferred_pragmas.extend(pragmas);
    }

    #[doc(hidden)]
    pub fn take_deferred_pragmas(&mut self) -> Vec<DeferredPragmas> {
        std::mem::take(&mut self.deferred_pragmas)
    }
}
