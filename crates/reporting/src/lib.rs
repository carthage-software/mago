//! Issue reporting and formatting for Mago.
//!
//! This crate provides functionality for reporting code issues identified by the linter and analyzer.
//! It includes support for multiple output formats, baseline filtering, and rich terminal output.
//!
//! # Core Types
//!
//! - [`Issue`]: Represents a single code issue with severity level, annotations, and optional fixes
//! - [`IssueCollection`]: A collection of issues with filtering and sorting capabilities
//! - [`reporter::Reporter`]: Handles formatting and outputting issues in various formats
//! - [`baseline::Baseline`]: Manages baseline files to filter out known issues

use std::cmp::Ordering;
use std::iter::Once;
use std::str::FromStr;

use foldhash::HashMap;
use foldhash::HashMapExt;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use strum::Display;
use strum::VariantNames;

use mago_database::file::FileId;
use mago_span::Span;
use mago_text_edit::TextEdit;

/// Represents an entry in the analyzer's `ignore` configuration.
///
/// Can be either a plain code string (ignored everywhere) or a scoped entry
/// that only ignores a code in specific paths.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum IgnoreEntry {
    /// Ignore a code everywhere: `"code1"`
    Code(String),
    /// Ignore a code in specific paths: `{ code = "code2", in = "path/" }`
    Scoped {
        code: String,
        #[serde(rename = "in", deserialize_with = "one_or_many")]
        paths: Vec<String>,
    },
}

fn one_or_many<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum OneOrMany {
        One(String),
        Many(Vec<String>),
    }

    match OneOrMany::deserialize(deserializer)? {
        OneOrMany::One(s) => Ok(vec![s]),
        OneOrMany::Many(v) => Ok(v),
    }
}

mod formatter;
mod internal;

pub mod baseline;
pub mod color;
pub mod error;
pub mod output;
pub mod reporter;

pub use color::ColorChoice;
pub use formatter::ReportingFormat;
pub use output::ReportingTarget;

/// Represents the kind of annotation associated with an issue.
#[derive(Debug, PartialEq, Eq, Ord, Copy, Clone, Hash, PartialOrd, Deserialize, Serialize)]
pub enum AnnotationKind {
    /// A primary annotation, typically highlighting the main source of the issue.
    Primary,
    /// A secondary annotation, providing additional context or related information.
    Secondary,
}

/// An annotation associated with an issue, providing additional context or highlighting specific code spans.
#[derive(Debug, PartialEq, Eq, Ord, Clone, Hash, PartialOrd, Deserialize, Serialize)]
pub struct Annotation {
    /// An optional message associated with the annotation.
    pub message: Option<String>,
    /// The kind of annotation.
    pub kind: AnnotationKind,
    /// The code span that the annotation refers to.
    pub span: Span,
}

/// Represents the severity level of an issue.
#[derive(
    Debug, PartialEq, Eq, Ord, Copy, Clone, Hash, PartialOrd, Deserialize, Serialize, Display, VariantNames, JsonSchema,
)]
#[strum(serialize_all = "lowercase")]
pub enum Level {
    /// A note, providing additional information or context.
    #[serde(alias = "note")]
    Note,
    /// A help message, suggesting possible solutions or further actions.
    #[serde(alias = "help")]
    Help,
    /// A warning, indicating a potential problem that may need attention.
    #[serde(alias = "warning", alias = "warn")]
    Warning,
    /// An error, indicating a problem that prevents the code from functioning correctly.
    #[serde(alias = "error", alias = "err")]
    Error,
}

impl FromStr for Level {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "note" => Ok(Self::Note),
            "help" => Ok(Self::Help),
            "warning" => Ok(Self::Warning),
            "error" => Ok(Self::Error),
            _ => Err(()),
        }
    }
}

type IssueEdits = Vec<TextEdit>;
type IssueEditBatches = Vec<(Option<String>, IssueEdits)>;

/// Represents an issue identified in the code.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Issue {
    /// The severity level of the issue.
    pub level: Level,
    /// An optional code associated with the issue.
    pub code: Option<String>,
    /// The main message describing the issue.
    pub message: String,
    /// Additional notes related to the issue.
    pub notes: Vec<String>,
    /// An optional help message suggesting possible solutions or further actions.
    pub help: Option<String>,
    /// An optional link to external resources for more information about the issue.
    pub link: Option<String>,
    /// Annotations associated with the issue, providing additional context or highlighting specific code spans.
    pub annotations: Vec<Annotation>,
    /// Text edits that can be applied to fix the issue, grouped by file.
    pub edits: HashMap<FileId, IssueEdits>,
}

/// A collection of issues.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct IssueCollection {
    issues: Vec<Issue>,
}

impl AnnotationKind {
    /// Returns `true` if this annotation kind is primary.
    #[inline]
    #[must_use]
    pub const fn is_primary(&self) -> bool {
        matches!(self, AnnotationKind::Primary)
    }

    /// Returns `true` if this annotation kind is secondary.
    #[inline]
    #[must_use]
    pub const fn is_secondary(&self) -> bool {
        matches!(self, AnnotationKind::Secondary)
    }
}

impl Annotation {
    /// Creates a new annotation with the given kind and span.
    ///
    /// # Examples
    ///
    /// ```
    /// use mago_reporting::{Annotation, AnnotationKind};
    /// use mago_database::file::FileId;
    /// use mago_span::Span;
    /// use mago_span::Position;
    ///
    /// let file = FileId::zero();
    /// let start = Position::new(0);
    /// let end = Position::new(5);
    /// let span = Span::new(file, start, end);
    /// let annotation = Annotation::new(AnnotationKind::Primary, span);
    /// ```
    #[must_use]
    pub fn new(kind: AnnotationKind, span: Span) -> Self {
        Self { message: None, kind, span }
    }

    /// Creates a new primary annotation with the given span.
    ///
    /// # Examples
    ///
    /// ```
    /// use mago_reporting::{Annotation, AnnotationKind};
    /// use mago_database::file::FileId;
    /// use mago_span::Span;
    /// use mago_span::Position;
    ///
    /// let file = FileId::zero();
    /// let start = Position::new(0);
    /// let end = Position::new(5);
    /// let span = Span::new(file, start, end);
    /// let annotation = Annotation::primary(span);
    /// ```
    #[must_use]
    pub fn primary(span: Span) -> Self {
        Self::new(AnnotationKind::Primary, span)
    }

    /// Creates a new secondary annotation with the given span.
    ///
    /// # Examples
    ///
    /// ```
    /// use mago_reporting::{Annotation, AnnotationKind};
    /// use mago_database::file::FileId;
    /// use mago_span::Span;
    /// use mago_span::Position;
    ///
    /// let file = FileId::zero();
    /// let start = Position::new(0);
    /// let end = Position::new(5);
    /// let span = Span::new(file, start, end);
    /// let annotation = Annotation::secondary(span);
    /// ```
    #[must_use]
    pub fn secondary(span: Span) -> Self {
        Self::new(AnnotationKind::Secondary, span)
    }

    /// Sets the message of this annotation.
    ///
    /// # Examples
    ///
    /// ```
    /// use mago_reporting::{Annotation, AnnotationKind};
    /// use mago_database::file::FileId;
    /// use mago_span::Span;
    /// use mago_span::Position;
    ///
    /// let file = FileId::zero();
    /// let start = Position::new(0);
    /// let end = Position::new(5);
    /// let span = Span::new(file, start, end);
    /// let annotation = Annotation::primary(span).with_message("This is a primary annotation");
    /// ```
    #[must_use]
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());

        self
    }

    /// Returns `true` if this annotation is a primary annotation.
    #[must_use]
    pub fn is_primary(&self) -> bool {
        self.kind == AnnotationKind::Primary
    }
}

impl Level {
    /// Downgrades the level to the next lower severity.
    ///
    /// This function maps levels to their less severe counterparts:
    ///
    /// - `Error` becomes `Warning`
    /// - `Warning` becomes `Help`
    /// - `Help` becomes `Note`
    /// - `Note` remains as `Note`
    ///
    /// # Examples
    ///
    /// ```
    /// use mago_reporting::Level;
    ///
    /// let level = Level::Error;
    /// assert_eq!(level.downgrade(), Level::Warning);
    ///
    /// let level = Level::Warning;
    /// assert_eq!(level.downgrade(), Level::Help);
    ///
    /// let level = Level::Help;
    /// assert_eq!(level.downgrade(), Level::Note);
    ///
    /// let level = Level::Note;
    /// assert_eq!(level.downgrade(), Level::Note);
    /// ```
    #[must_use]
    pub fn downgrade(&self) -> Self {
        match self {
            Level::Error => Level::Warning,
            Level::Warning => Level::Help,
            Level::Help | Level::Note => Level::Note,
        }
    }
}

impl Issue {
    /// Creates a new issue with the given level and message.
    ///
    /// # Examples
    ///
    /// ```
    /// use mago_reporting::{Issue, Level};
    ///
    /// let issue = Issue::new(Level::Error, "This is an error");
    /// ```
    pub fn new(level: Level, message: impl Into<String>) -> Self {
        Self {
            level,
            code: None,
            message: message.into(),
            annotations: Vec::new(),
            notes: Vec::new(),
            help: None,
            link: None,
            edits: HashMap::default(),
        }
    }

    /// Creates a new error issue with the given message.
    ///
    /// # Examples
    ///
    /// ```
    /// use mago_reporting::Issue;
    ///
    /// let issue = Issue::error("This is an error");
    /// ```
    pub fn error(message: impl Into<String>) -> Self {
        Self::new(Level::Error, message)
    }

    /// Creates a new warning issue with the given message.
    ///
    /// # Examples
    ///
    /// ```
    /// use mago_reporting::Issue;
    ///
    /// let issue = Issue::warning("This is a warning");
    /// ```
    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(Level::Warning, message)
    }

    /// Creates a new help issue with the given message.
    ///
    /// # Examples
    ///
    /// ```
    /// use mago_reporting::Issue;
    ///
    /// let issue = Issue::help("This is a help message");
    /// ```
    pub fn help(message: impl Into<String>) -> Self {
        Self::new(Level::Help, message)
    }

    /// Creates a new note issue with the given message.
    ///
    /// # Examples
    ///
    /// ```
    /// use mago_reporting::Issue;
    ///
    /// let issue = Issue::note("This is a note");
    /// ```
    pub fn note(message: impl Into<String>) -> Self {
        Self::new(Level::Note, message)
    }

    /// Adds a code to this issue.
    ///
    /// # Examples
    ///
    /// ```
    /// use mago_reporting::{Issue, Level};
    ///
    /// let issue = Issue::error("This is an error").with_code("E0001");
    /// ```
    #[must_use]
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());

        self
    }

    /// Add an annotation to this issue.
    ///
    /// # Examples
    ///
    /// ```
    /// use mago_reporting::{Issue, Annotation, AnnotationKind};
    /// use mago_database::file::FileId;
    /// use mago_span::Span;
    /// use mago_span::Position;
    ///
    /// let file = FileId::zero();
    /// let start = Position::new(0);
    /// let end = Position::new(5);
    /// let span = Span::new(file, start, end);
    ///
    /// let issue = Issue::error("This is an error").with_annotation(Annotation::primary(span));
    /// ```
    #[must_use]
    pub fn with_annotation(mut self, annotation: Annotation) -> Self {
        self.annotations.push(annotation);

        self
    }

    #[must_use]
    pub fn with_annotations(mut self, annotation: impl IntoIterator<Item = Annotation>) -> Self {
        self.annotations.extend(annotation);

        self
    }

    /// Returns the deterministic primary annotation for this issue.
    ///
    /// If multiple primary annotations exist, the one with the smallest span is returned.
    #[must_use]
    pub fn primary_annotation(&self) -> Option<&Annotation> {
        self.annotations.iter().filter(|annotation| annotation.is_primary()).min_by_key(|annotation| annotation.span)
    }

    /// Returns the deterministic primary span for this issue.
    #[must_use]
    pub fn primary_span(&self) -> Option<Span> {
        self.primary_annotation().map(|annotation| annotation.span)
    }

    /// Add a note to this issue.
    ///
    /// # Examples
    ///
    /// ```
    /// use mago_reporting::Issue;
    ///
    /// let issue = Issue::error("This is an error").with_note("This is a note");
    /// ```
    #[must_use]
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());

        self
    }

    /// Add a help message to this issue.
    ///
    /// This is useful for providing additional context to the user on how to resolve the issue.
    ///
    /// # Examples
    ///
    /// ```
    /// use mago_reporting::Issue;
    ///
    /// let issue = Issue::error("This is an error").with_help("This is a help message");
    /// ```
    #[must_use]
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());

        self
    }

    /// Add a link to this issue.
    ///
    /// # Examples
    ///
    /// ```
    /// use mago_reporting::Issue;
    ///
    /// let issue = Issue::error("This is an error").with_link("https://example.com");
    /// ```
    #[must_use]
    pub fn with_link(mut self, link: impl Into<String>) -> Self {
        self.link = Some(link.into());

        self
    }

    /// Add a single edit to this issue.
    #[must_use]
    pub fn with_edit(mut self, file_id: FileId, edit: TextEdit) -> Self {
        self.edits.entry(file_id).or_default().push(edit);

        self
    }

    /// Add multiple edits to this issue.
    #[must_use]
    pub fn with_file_edits(mut self, file_id: FileId, edits: IssueEdits) -> Self {
        if !edits.is_empty() {
            self.edits.entry(file_id).or_default().extend(edits);
        }

        self
    }

    /// Take the edits from this issue.
    #[must_use]
    pub fn take_edits(&mut self) -> HashMap<FileId, IssueEdits> {
        std::mem::replace(&mut self.edits, HashMap::with_capacity(0))
    }
}

impl IssueCollection {
    #[must_use]
    pub fn new() -> Self {
        Self { issues: Vec::new() }
    }

    pub fn from(issues: impl IntoIterator<Item = Issue>) -> Self {
        Self { issues: issues.into_iter().collect() }
    }

    pub fn push(&mut self, issue: Issue) {
        self.issues.push(issue);
    }

    pub fn extend(&mut self, issues: impl IntoIterator<Item = Issue>) {
        self.issues.extend(issues);
    }

    pub fn reserve(&mut self, additional: usize) {
        self.issues.reserve(additional);
    }

    pub fn shrink_to_fit(&mut self) {
        self.issues.shrink_to_fit();
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.issues.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.issues.len()
    }

    /// Filters the issues in the collection to only include those with a severity level
    /// lower than or equal to the given level.
    #[must_use]
    pub fn with_maximum_level(self, level: Level) -> Self {
        Self { issues: self.issues.into_iter().filter(|issue| issue.level <= level).collect() }
    }

    /// Filters the issues in the collection to only include those with a severity level
    ///  higher than or equal to the given level.
    #[must_use]
    pub fn with_minimum_level(self, level: Level) -> Self {
        Self { issues: self.issues.into_iter().filter(|issue| issue.level >= level).collect() }
    }

    /// Returns `true` if the collection contains any issues with a severity level
    ///  higher than or equal to the given level.
    #[must_use]
    pub fn has_minimum_level(&self, level: Level) -> bool {
        self.issues.iter().any(|issue| issue.level >= level)
    }

    /// Returns the number of issues in the collection with the given severity level.
    #[must_use]
    pub fn get_level_count(&self, level: Level) -> usize {
        self.issues.iter().filter(|issue| issue.level == level).count()
    }

    /// Returns the highest severity level of the issues in the collection.
    #[must_use]
    pub fn get_highest_level(&self) -> Option<Level> {
        self.issues.iter().map(|issue| issue.level).max()
    }

    /// Returns the lowest severity level of the issues in the collection.
    #[must_use]
    pub fn get_lowest_level(&self) -> Option<Level> {
        self.issues.iter().map(|issue| issue.level).min()
    }

    pub fn filter_out_ignored<F>(&mut self, ignore: &[IgnoreEntry], resolve_file_name: F)
    where
        F: Fn(FileId) -> Option<String>,
    {
        if ignore.is_empty() {
            return;
        }

        self.issues.retain(|issue| {
            let Some(code) = &issue.code else {
                return true;
            };

            for entry in ignore {
                match entry {
                    IgnoreEntry::Code(ignored) if ignored == code => return false,
                    IgnoreEntry::Scoped { code: ignored, paths } if ignored == code => {
                        let file_name = issue.primary_span().and_then(|span| resolve_file_name(span.file_id));

                        if let Some(name) = file_name
                            && is_path_match(&name, paths)
                        {
                            return false;
                        }
                    }
                    _ => {}
                }
            }

            true
        });
    }

    pub fn filter_retain_codes(&mut self, retain_codes: &[String]) {
        self.issues.retain(|issue| if let Some(code) = &issue.code { retain_codes.contains(code) } else { false });
    }

    pub fn take_edits(&mut self) -> impl Iterator<Item = (FileId, IssueEdits)> + '_ {
        self.issues.iter_mut().flat_map(|issue| issue.take_edits().into_iter())
    }

    /// Filters the issues in the collection to only include those that have associated edits.
    #[must_use]
    pub fn with_edits(self) -> Self {
        Self { issues: self.issues.into_iter().filter(|issue| !issue.edits.is_empty()).collect() }
    }

    /// Sorts the issues in the collection.
    ///
    /// The issues are sorted by severity level in descending order,
    /// then by code in ascending order, and finally by the primary annotation span.
    #[must_use]
    pub fn sorted(self) -> Self {
        let mut issues = self.issues;

        issues.sort_by(|a, b| match a.level.cmp(&b.level) {
            Ordering::Greater => Ordering::Greater,
            Ordering::Less => Ordering::Less,
            Ordering::Equal => match a.code.as_deref().cmp(&b.code.as_deref()) {
                Ordering::Less => Ordering::Less,
                Ordering::Greater => Ordering::Greater,
                Ordering::Equal => {
                    let a_span = a.primary_span();
                    let b_span = b.primary_span();

                    match (a_span, b_span) {
                        (Some(a_span), Some(b_span)) => a_span.cmp(&b_span),
                        (Some(_), None) => Ordering::Less,
                        (None, Some(_)) => Ordering::Greater,
                        (None, None) => Ordering::Equal,
                    }
                }
            },
        });

        Self { issues }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Issue> {
        self.issues.iter()
    }

    /// Converts the collection into a map of edit batches grouped by file.
    ///
    /// Each batch contains all edits from a single issue along with the rule code.
    /// All edits from an issue must be applied together as a batch to maintain code validity.
    ///
    /// Returns `HashMap<FileId, Vec<(Option<String>, IssueEdits)>>` where each tuple
    /// is (rule_code, edits_for_that_issue).
    #[must_use]
    pub fn to_edit_batches(self) -> HashMap<FileId, IssueEditBatches> {
        let mut result: HashMap<FileId, Vec<(Option<String>, IssueEdits)>> = HashMap::default();
        for issue in self.issues.into_iter().filter(|issue| !issue.edits.is_empty()) {
            let code = issue.code;
            for (file_id, edit_list) in issue.edits {
                result.entry(file_id).or_default().push((code.clone(), edit_list));
            }
        }

        result
    }
}

impl IntoIterator for IssueCollection {
    type Item = Issue;

    type IntoIter = std::vec::IntoIter<Issue>;

    fn into_iter(self) -> Self::IntoIter {
        self.issues.into_iter()
    }
}

impl<'a> IntoIterator for &'a IssueCollection {
    type Item = &'a Issue;

    type IntoIter = std::slice::Iter<'a, Issue>;

    fn into_iter(self) -> Self::IntoIter {
        self.issues.iter()
    }
}

impl Default for IssueCollection {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoIterator for Issue {
    type Item = Issue;
    type IntoIter = Once<Issue>;

    fn into_iter(self) -> Self::IntoIter {
        std::iter::once(self)
    }
}

impl FromIterator<Issue> for IssueCollection {
    fn from_iter<T: IntoIterator<Item = Issue>>(iter: T) -> Self {
        Self { issues: iter.into_iter().collect() }
    }
}

fn is_path_match(file_name: &str, patterns: &[String]) -> bool {
    patterns.iter().any(|pattern| {
        if pattern.ends_with('/') {
            file_name.starts_with(pattern.as_str())
        } else {
            let dir_prefix = format!("{pattern}/");
            file_name.starts_with(&dir_prefix) || file_name == pattern
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_highest_collection_level() {
        let mut collection = IssueCollection::from(vec![]);
        assert_eq!(collection.get_highest_level(), None);

        collection.push(Issue::note("note"));
        assert_eq!(collection.get_highest_level(), Some(Level::Note));

        collection.push(Issue::help("help"));
        assert_eq!(collection.get_highest_level(), Some(Level::Help));

        collection.push(Issue::warning("warning"));
        assert_eq!(collection.get_highest_level(), Some(Level::Warning));

        collection.push(Issue::error("error"));
        assert_eq!(collection.get_highest_level(), Some(Level::Error));
    }

    #[test]
    pub fn test_level_downgrade() {
        assert_eq!(Level::Error.downgrade(), Level::Warning);
        assert_eq!(Level::Warning.downgrade(), Level::Help);
        assert_eq!(Level::Help.downgrade(), Level::Note);
        assert_eq!(Level::Note.downgrade(), Level::Note);
    }

    #[test]
    pub fn test_issue_collection_with_maximum_level() {
        let mut collection = IssueCollection::from(vec![
            Issue::error("error"),
            Issue::warning("warning"),
            Issue::help("help"),
            Issue::note("note"),
        ]);

        collection = collection.with_maximum_level(Level::Warning);
        assert_eq!(collection.len(), 3);
        assert_eq!(
            collection.iter().map(|issue| issue.level).collect::<Vec<_>>(),
            vec![Level::Warning, Level::Help, Level::Note]
        );
    }

    #[test]
    pub fn test_issue_collection_with_minimum_level() {
        let mut collection = IssueCollection::from(vec![
            Issue::error("error"),
            Issue::warning("warning"),
            Issue::help("help"),
            Issue::note("note"),
        ]);

        collection = collection.with_minimum_level(Level::Warning);
        assert_eq!(collection.len(), 2);
        assert_eq!(collection.iter().map(|issue| issue.level).collect::<Vec<_>>(), vec![Level::Error, Level::Warning,]);
    }

    #[test]
    pub fn test_issue_collection_has_minimum_level() {
        let mut collection = IssueCollection::from(vec![]);

        assert!(!collection.has_minimum_level(Level::Error));
        assert!(!collection.has_minimum_level(Level::Warning));
        assert!(!collection.has_minimum_level(Level::Help));
        assert!(!collection.has_minimum_level(Level::Note));

        collection.push(Issue::note("note"));

        assert!(!collection.has_minimum_level(Level::Error));
        assert!(!collection.has_minimum_level(Level::Warning));
        assert!(!collection.has_minimum_level(Level::Help));
        assert!(collection.has_minimum_level(Level::Note));

        collection.push(Issue::help("help"));

        assert!(!collection.has_minimum_level(Level::Error));
        assert!(!collection.has_minimum_level(Level::Warning));
        assert!(collection.has_minimum_level(Level::Help));
        assert!(collection.has_minimum_level(Level::Note));

        collection.push(Issue::warning("warning"));

        assert!(!collection.has_minimum_level(Level::Error));
        assert!(collection.has_minimum_level(Level::Warning));
        assert!(collection.has_minimum_level(Level::Help));
        assert!(collection.has_minimum_level(Level::Note));

        collection.push(Issue::error("error"));

        assert!(collection.has_minimum_level(Level::Error));
        assert!(collection.has_minimum_level(Level::Warning));
        assert!(collection.has_minimum_level(Level::Help));
        assert!(collection.has_minimum_level(Level::Note));
    }

    #[test]
    pub fn test_issue_collection_level_count() {
        let mut collection = IssueCollection::from(vec![]);

        assert_eq!(collection.get_level_count(Level::Error), 0);
        assert_eq!(collection.get_level_count(Level::Warning), 0);
        assert_eq!(collection.get_level_count(Level::Help), 0);
        assert_eq!(collection.get_level_count(Level::Note), 0);

        collection.push(Issue::error("error"));

        assert_eq!(collection.get_level_count(Level::Error), 1);
        assert_eq!(collection.get_level_count(Level::Warning), 0);
        assert_eq!(collection.get_level_count(Level::Help), 0);
        assert_eq!(collection.get_level_count(Level::Note), 0);

        collection.push(Issue::warning("warning"));

        assert_eq!(collection.get_level_count(Level::Error), 1);
        assert_eq!(collection.get_level_count(Level::Warning), 1);
        assert_eq!(collection.get_level_count(Level::Help), 0);
        assert_eq!(collection.get_level_count(Level::Note), 0);

        collection.push(Issue::help("help"));

        assert_eq!(collection.get_level_count(Level::Error), 1);
        assert_eq!(collection.get_level_count(Level::Warning), 1);
        assert_eq!(collection.get_level_count(Level::Help), 1);
        assert_eq!(collection.get_level_count(Level::Note), 0);

        collection.push(Issue::note("note"));

        assert_eq!(collection.get_level_count(Level::Error), 1);
        assert_eq!(collection.get_level_count(Level::Warning), 1);
        assert_eq!(collection.get_level_count(Level::Help), 1);
        assert_eq!(collection.get_level_count(Level::Note), 1);
    }

    #[test]
    pub fn test_primary_span_is_deterministic() {
        let file = FileId::zero();
        let span_later = Span::new(file, 20_u32.into(), 25_u32.into());
        let span_earlier = Span::new(file, 5_u32.into(), 10_u32.into());

        let issue = Issue::error("x")
            .with_annotation(Annotation::primary(span_later))
            .with_annotation(Annotation::primary(span_earlier));

        assert_eq!(issue.primary_span(), Some(span_earlier));
    }
}
