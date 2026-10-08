use std::borrow::Cow;
use std::path::Path;

use foldhash::HashMap;
use serde::Serialize;
use serde::Serializer;
use serde::ser::SerializeSeq;
use serde::ser::SerializeStruct;

use mago_database::DatabaseReader;
use mago_database::ReadDatabase;
use mago_database::error::DatabaseError;
use mago_database::file::File;
use mago_database::file::FileId;
use mago_database::file::FileType;

use crate::Annotation;
use crate::Issue;

#[derive(Serialize)]
struct ExpandedFileId<'file> {
    name: Cow<'file, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<&'file Path>,
    size: u32,
    file_type: FileType,
}

struct FileDetails<'file> {
    file: &'file File,
    expanded: ExpandedFileId<'file>,
}

#[derive(Serialize)]
struct ExpandedPosition {
    offset: u32,
    line: u32,
}

#[derive(Serialize)]
struct ExpandedSpan<'file> {
    file_id: &'file ExpandedFileId<'file>,
    start: ExpandedPosition,
    end: ExpandedPosition,
}

#[derive(Serialize)]
struct ExpandedAnnotation<'annotation, 'file> {
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<&'annotation str>,
    kind: crate::AnnotationKind,
    span: ExpandedSpan<'file>,
}

pub(crate) struct ExpandedIssueCollection<'issue, 'file> {
    issues: Vec<&'issue Issue>,
    files: HashMap<FileId, FileDetails<'file>>,
}

impl<'issue, 'file> ExpandedIssueCollection<'issue, 'file> {
    pub(crate) fn new(issues: Vec<&'issue Issue>, database: &'file ReadDatabase) -> Result<Self, DatabaseError> {
        let mut files = HashMap::default();
        // Resolve files in expansion order before writing, preserving the first error and empty output on failure.
        for issue in &issues {
            for file_id in issue.annotations.iter().map(|annotation| &annotation.span.file_id).chain(issue.edits.keys())
            {
                if let std::collections::hash_map::Entry::Vacant(entry) = files.entry(*file_id) {
                    let file = database.get_ref(file_id)?;
                    entry.insert(FileDetails {
                        file,
                        expanded: ExpandedFileId {
                            name: String::from_utf8_lossy(&file.name),
                            path: file.path.as_deref(),
                            size: file.size,
                            file_type: file.file_type,
                        },
                    });
                }
            }
        }
        Ok(Self { issues, files })
    }
}

impl Serialize for ExpandedIssueCollection<'_, '_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut collection = serializer.serialize_struct("ExpandedIssueCollection", 1)?;
        collection.serialize_field("issues", &ExpandedIssues { issues: &self.issues, files: &self.files })?;
        collection.end()
    }
}

struct ExpandedIssues<'issue, 'file> {
    issues: &'issue [&'issue Issue],
    files: &'file HashMap<FileId, FileDetails<'file>>,
}

impl Serialize for ExpandedIssues<'_, '_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut issues = serializer.serialize_seq(Some(self.issues.len()))?;
        for issue in self.issues {
            issues.serialize_element(&ExpandedIssue { issue, files: self.files })?;
        }

        issues.end()
    }
}

struct ExpandedIssue<'issue, 'file> {
    issue: &'issue Issue,
    files: &'file HashMap<FileId, FileDetails<'file>>,
}

impl Serialize for ExpandedIssue<'_, '_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let issue = self.issue;
        let fields = 2
            + usize::from(issue.code.is_some())
            + usize::from(!issue.notes.is_empty())
            + usize::from(issue.help.is_some())
            + usize::from(issue.link.is_some())
            + usize::from(!issue.annotations.is_empty())
            + usize::from(!issue.edits.is_empty());
        let mut expanded = serializer.serialize_struct("ExpandedIssue", fields)?;
        expanded.serialize_field("level", &issue.level)?;
        if let Some(code) = &issue.code {
            expanded.serialize_field("code", code)?;
        }

        expanded.serialize_field("message", &issue.message)?;
        if !issue.notes.is_empty() {
            expanded.serialize_field("notes", &issue.notes)?;
        }

        if let Some(help) = &issue.help {
            expanded.serialize_field("help", help)?;
        }

        if let Some(link) = &issue.link {
            expanded.serialize_field("link", link)?;
        }

        if !issue.annotations.is_empty() {
            expanded.serialize_field(
                "annotations",
                &ExpandedAnnotations { annotations: &issue.annotations, files: self.files },
            )?;
        }

        if !issue.edits.is_empty() {
            expanded.serialize_field("edits", &ExpandedEdits { issue, files: self.files })?;
        }

        expanded.end()
    }
}

struct ExpandedAnnotations<'issue, 'file> {
    annotations: &'issue [Annotation],
    files: &'file HashMap<FileId, FileDetails<'file>>,
}

impl Serialize for ExpandedAnnotations<'_, '_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut annotations = serializer.serialize_seq(Some(self.annotations.len()))?;
        for annotation in self.annotations {
            let details = &self.files[&annotation.span.file_id];
            annotations.serialize_element(&ExpandedAnnotation {
                message: annotation.message.as_deref(),
                kind: annotation.kind,
                span: ExpandedSpan {
                    file_id: &details.expanded,
                    start: ExpandedPosition {
                        offset: annotation.span.start.offset,
                        line: details.file.line_number(annotation.span.start.offset),
                    },
                    end: ExpandedPosition {
                        offset: annotation.span.end.offset,
                        line: details.file.line_number(annotation.span.end.offset),
                    },
                },
            })?;
        }

        annotations.end()
    }
}

struct ExpandedEdits<'issue, 'file> {
    issue: &'issue Issue,
    files: &'file HashMap<FileId, FileDetails<'file>>,
}

impl Serialize for ExpandedEdits<'_, '_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut edits = serializer.serialize_seq(Some(self.issue.edits.len()))?;
        for (file_id, edit_list) in &self.issue.edits {
            edits.serialize_element(&(&self.files[file_id].expanded, edit_list))?;
        }

        edits.end()
    }
}
