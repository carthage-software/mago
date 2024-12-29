use std::path::PathBuf;

use mago_source::SourceCategory;
use serde::Deserialize;
use serde::Serialize;

use mago_fixer::FixPlan;
use mago_interner::ThreadedInterner;
use mago_source::error::SourceError;
use mago_source::SourceIdentifier;
use mago_source::SourceManager;
use mago_span::Position;
use mago_span::Span;

use crate::Annotation;
use crate::AnnotationKind;
use crate::Issue;
use crate::IssueCollection;
use crate::Level;

pub mod emitter;
pub mod writer;

/// Expanded representation of a source identifier.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ExpandedSourceIdentifier {
    pub identifier: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
    pub size: usize,
    pub category: SourceCategory,
}

/// Expanded representation of a position within a source file.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ExpandedPosition {
    pub source: ExpandedSourceIdentifier,
    pub offset: usize,
    pub line: usize,
}

/// Expanded representation of a span, including start and end positions.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ExpandedSpan {
    pub start: ExpandedPosition,
    pub end: ExpandedPosition,
}

/// Expanded annotation, enriched with resolved spans.
#[derive(Debug, PartialEq, Eq, Ord, Clone, Hash, PartialOrd, Deserialize, Serialize)]
pub struct ExpandedAnnotation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    pub kind: AnnotationKind,
    pub span: ExpandedSpan,
}

/// Expanded issue, containing detailed information for display or external reporting.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ExpandedIssue {
    pub level: Level,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    pub message: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub annotations: Vec<ExpandedAnnotation>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub suggestions: Vec<(ExpandedSourceIdentifier, FixPlan)>,
}

/// A collection of expanded issues.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ExpandedIssueCollection {
    issues: Vec<ExpandedIssue>,
}

pub trait Expandable<T> {
    fn expand(&self, manager: &SourceManager, interner: &ThreadedInterner) -> Result<T, SourceError>;
}

impl Expandable<ExpandedSourceIdentifier> for SourceIdentifier {
    fn expand(
        &self,
        manager: &SourceManager,
        interner: &ThreadedInterner,
    ) -> Result<ExpandedSourceIdentifier, SourceError> {
        let source = manager.load(self)?;

        Ok(ExpandedSourceIdentifier {
            identifier: interner.lookup(&source.identifier.0).to_string(),
            path: source.path.clone(),
            size: source.size,
            category: source.identifier.category(),
        })
    }
}

impl Expandable<ExpandedPosition> for Position {
    fn expand(&self, manager: &SourceManager, interner: &ThreadedInterner) -> Result<ExpandedPosition, SourceError> {
        let source = manager.load(&self.source)?;

        Ok(ExpandedPosition {
            source: self.source.expand(manager, interner)?,
            offset: self.offset,
            line: source.line_number(self.offset),
        })
    }
}

impl Expandable<ExpandedSpan> for Span {
    fn expand(&self, manager: &SourceManager, interner: &ThreadedInterner) -> Result<ExpandedSpan, SourceError> {
        Ok(ExpandedSpan { start: self.start.expand(manager, interner)?, end: self.end.expand(manager, interner)? })
    }
}

impl Expandable<ExpandedAnnotation> for Annotation {
    fn expand(&self, manager: &SourceManager, interner: &ThreadedInterner) -> Result<ExpandedAnnotation, SourceError> {
        Ok(ExpandedAnnotation {
            message: self.message.clone(),
            kind: self.kind,
            span: self.span.expand(manager, interner)?,
        })
    }
}

impl Expandable<ExpandedIssue> for Issue {
    fn expand(&self, manager: &SourceManager, interner: &ThreadedInterner) -> Result<ExpandedIssue, SourceError> {
        let mut annotations = Vec::new();
        for annotation in &self.annotations {
            annotations.push(annotation.expand(manager, interner)?);
        }

        let mut suggestions = Vec::new();
        for (source, fix) in &self.suggestions {
            suggestions.push((source.expand(manager, interner)?, fix.clone()));
        }

        Ok(ExpandedIssue {
            level: self.level,
            code: self.code.clone(),
            message: self.message.clone(),
            notes: self.notes.clone(),
            help: self.help.clone(),
            link: self.link.clone(),
            annotations,
            suggestions,
        })
    }
}

impl Expandable<ExpandedIssueCollection> for IssueCollection {
    fn expand(
        &self,
        manager: &SourceManager,
        interner: &ThreadedInterner,
    ) -> Result<ExpandedIssueCollection, SourceError> {
        let mut expanded_issues = Vec::new();
        for issue in self.issues.iter() {
            expanded_issues.push(issue.expand(manager, interner)?);
        }

        Ok(ExpandedIssueCollection { issues: expanded_issues })
    }
}
