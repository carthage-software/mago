#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;

use mago_codex::metadata::CodebaseMetadata;
use mago_codex::reference::SymbolReferences;
use mago_collector::DeferredPragmas;
use mago_reporting::IssueCollection;

use crate::code::IssueCode;
use crate::statement::class_like::unused_members::UnusedMemberSpans;
use crate::statement::class_like::unused_members::find_unused_member_spans;

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

/// Reconciles unused-member diagnostics against references discovered after file analysis.
#[derive(Debug)]
pub struct LateSymbolReferenceIssueReconciler {
    spans: UnusedMemberSpans,
}

impl LateSymbolReferenceIssueReconciler {
    #[must_use]
    pub fn new(codebase: &CodebaseMetadata, symbol_references: &SymbolReferences) -> Self {
        Self { spans: find_unused_member_spans(codebase, symbol_references) }
    }

    #[must_use]
    pub fn reconcile(&self, issues: IssueCollection) -> IssueCollection {
        IssueCollection::from(issues.into_iter().filter(|issue| {
            let Some(code) = issue.code.as_deref() else {
                return true;
            };

            let Some(primary) = issue.annotations.iter().find(|annotation| annotation.kind.is_primary()) else {
                return true;
            };

            match code {
                code if code == IssueCode::UnusedMethod.as_str() => self.spans.unused_methods.contains(&primary.span),
                code if code == IssueCode::UnusedProperty.as_str() => {
                    self.spans.unused_properties.contains(&primary.span)
                }
                code if code == IssueCode::WriteOnlyProperty.as_str() => {
                    !self.spans.read_properties.contains(&primary.span)
                }
                _ => true,
            }
        }))
    }
}
