use std::borrow::Cow;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use mago_database::DatabaseReader;
use mago_database::ReadDatabase;
use mago_database::error::DatabaseError;
use mago_database::file::FileId;
use mago_database::file::FileType;
use mago_fixer::FixPlan;
use mago_span::Span;

use crate::Annotation;
use crate::AnnotationKind;
use crate::Issue;
use crate::IssueCollection;
use crate::Level;

/// Expanded representation of a file id.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ExpandedFileId {
    pub name: Cow<'static, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
    pub size: u32,
    pub file_type: FileType,
}

/// Expanded representation of a position within a file.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ExpandedPosition {
    pub offset: u32,
    pub line: u32,
}

/// Expanded representation of a span, including start and end positions.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ExpandedSpan {
    pub file_id: ExpandedFileId,
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
    pub suggestions: Vec<(ExpandedFileId, FixPlan)>,
}

/// A collection of expanded issues.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ExpandedIssueCollection {
    issues: Vec<ExpandedIssue>,
}

pub trait Expandable<T> {
    fn expand(&self, database: &ReadDatabase) -> Result<T, DatabaseError>;
}

impl Expandable<ExpandedFileId> for FileId {
    fn expand(&self, database: &ReadDatabase) -> Result<ExpandedFileId, DatabaseError> {
        let file = database.get(self)?;

        Ok(ExpandedFileId {
            name: file.name.clone(),
            path: file.path.clone(),
            size: file.size,
            file_type: file.file_type,
        })
    }
}

impl Expandable<ExpandedSpan> for Span {
    fn expand(&self, database: &ReadDatabase) -> Result<ExpandedSpan, DatabaseError> {
        let file = database.get(&self.file_id)?;

        Ok(ExpandedSpan {
            file_id: self.file_id.expand(database)?,
            start: ExpandedPosition { offset: self.start.offset, line: file.line_number(self.start.offset) },
            end: ExpandedPosition { offset: self.end.offset, line: file.line_number(self.end.offset) },
        })
    }
}

impl Expandable<ExpandedAnnotation> for Annotation {
    fn expand(&self, database: &ReadDatabase) -> Result<ExpandedAnnotation, DatabaseError> {
        Ok(ExpandedAnnotation { message: self.message.clone(), kind: self.kind, span: self.span.expand(database)? })
    }
}

impl Expandable<ExpandedIssue> for Issue {
    fn expand(&self, database: &ReadDatabase) -> Result<ExpandedIssue, DatabaseError> {
        let mut annotations = Vec::new();
        for annotation in &self.annotations {
            annotations.push(annotation.expand(database)?);
        }

        let mut suggestions = Vec::new();
        for (file_id, fix) in &self.suggestions {
            suggestions.push((file_id.expand(database)?, fix.clone()));
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
    fn expand(&self, database: &ReadDatabase) -> Result<ExpandedIssueCollection, DatabaseError> {
        let mut expanded_issues = Vec::new();
        for issue in self.issues.iter() {
            expanded_issues.push(issue.expand(database)?);
        }

        Ok(ExpandedIssueCollection { issues: expanded_issues })
    }
}
