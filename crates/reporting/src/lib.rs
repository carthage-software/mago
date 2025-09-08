use std::cmp::Ordering;
use std::collections::hash_map::Entry;
use std::iter::Once;
use std::str::FromStr;

use ahash::HashMap;
use serde::Deserialize;
use serde::Serialize;
use strum::Display;
use strum::VariantNames;

use mago_database::file::FileId;
use mago_fixer::FixPlan;
use mago_span::Span;

mod internal;

pub mod error;
pub mod reporter;

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
#[derive(Debug, PartialEq, Eq, Ord, Copy, Clone, Hash, PartialOrd, Deserialize, Serialize, Display, VariantNames)]
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

/// Represents an issue identified in the code.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
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
    /// Modification suggestions that can be applied to fix the issue.
    pub suggestions: Vec<(FileId, FixPlan)>,
}

/// A collection of issues.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IssueCollection {
    issues: Vec<Issue>,
}

impl AnnotationKind {
    /// Returns `true` if this annotation kind is primary.
    #[inline]
    pub const fn is_primary(&self) -> bool {
        matches!(self, AnnotationKind::Primary)
    }

    /// Returns `true` if this annotation kind is secondary.
    #[inline]
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
            suggestions: Vec::new(),
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

    /// Add a code modification suggestion to this issue.
    #[must_use]
    pub fn with_suggestion(mut self, file_id: FileId, plan: FixPlan) -> Self {
        self.suggestions.push((file_id, plan));

        self
    }

    /// Take the code modification suggestion from this issue.
    #[must_use]
    pub fn take_suggestions(&mut self) -> Vec<(FileId, FixPlan)> {
        self.suggestions.drain(..).collect()
    }
}

impl IssueCollection {
    pub fn new() -> Self {
        Self { issues: Vec::new() }
    }

    pub fn from(issues: impl IntoIterator<Item = Issue>) -> Self {
        Self { issues: issues.into_iter().collect() }
    }

    pub fn push(&mut self, issue: Issue) {
        if self.issues.contains(&issue) {
            return; // Avoid duplicates
        }

        self.issues.push(issue);
    }

    pub fn extend(&mut self, issues: impl IntoIterator<Item = Issue>) {
        self.issues.extend(issues);
    }

    pub fn shrink_to_fit(&mut self) {
        self.issues.shrink_to_fit();
    }

    pub fn is_empty(&self) -> bool {
        self.issues.is_empty()
    }

    pub fn len(&self) -> usize {
        self.issues.len()
    }

    /// Filters the issues in the collection to only include those with a severity level
    /// lower than or equal to the given level.
    pub fn with_maximum_level(self, level: Level) -> Self {
        Self { issues: self.issues.into_iter().filter(|issue| issue.level <= level).collect() }
    }

    /// Filters the issues in the collection to only include those with a severity level
    ///  higher than or equal to the given level.
    pub fn with_minimum_level(self, level: Level) -> Self {
        Self { issues: self.issues.into_iter().filter(|issue| issue.level >= level).collect() }
    }

    /// Returns `true` if the collection contains any issues with a severity level
    ///  higher than or equal to the given level.
    pub fn has_minimum_level(&self, level: Level) -> bool {
        self.issues.iter().any(|issue| issue.level >= level)
    }

    /// Returns the number of issues in the collection with the given severity level.
    pub fn get_level_count(&self, level: Level) -> usize {
        self.issues.iter().filter(|issue| issue.level == level).count()
    }

    /// Returns the highest severity level of the issues in the collection.
    pub fn get_highest_level(&self) -> Option<Level> {
        self.issues.iter().map(|issue| issue.level).max()
    }

    pub fn filter_out_ignored(&mut self, ignore: &[String]) {
        self.issues.retain(|issue| if let Some(code) = &issue.code { !ignore.contains(code) } else { true });
    }

    pub fn take_suggestions(&mut self) -> impl Iterator<Item = (FileId, FixPlan)> + '_ {
        self.issues.iter_mut().flat_map(|issue| issue.take_suggestions())
    }

    pub fn only_fixable(self) -> impl Iterator<Item = Issue> {
        self.issues.into_iter().filter(|issue| !issue.suggestions.is_empty())
    }

    /// Sorts the issues in the collection.
    ///
    /// The issues are sorted by severity level in descending order,
    /// then by code in ascending order, and finally by the primary annotation span.
    pub fn sorted(self) -> Self {
        let mut issues = self.issues;

        issues.sort_by(|a, b| match a.level.cmp(&b.level) {
            Ordering::Greater => Ordering::Greater,
            Ordering::Less => Ordering::Less,
            Ordering::Equal => match a.code.as_deref().cmp(&b.code.as_deref()) {
                Ordering::Less => Ordering::Less,
                Ordering::Greater => Ordering::Greater,
                Ordering::Equal => {
                    let a_span = a
                        .annotations
                        .iter()
                        .find(|annotation| annotation.is_primary())
                        .map(|annotation| annotation.span);

                    let b_span = b
                        .annotations
                        .iter()
                        .find(|annotation| annotation.is_primary())
                        .map(|annotation| annotation.span);

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

    pub fn to_fix_plans(self) -> HashMap<FileId, FixPlan> {
        let mut plans: HashMap<FileId, FixPlan> = HashMap::default();
        for issue in self.issues.into_iter().filter(|issue| !issue.suggestions.is_empty()) {
            for suggestion in issue.suggestions.into_iter() {
                match plans.entry(suggestion.0) {
                    Entry::Occupied(mut occupied_entry) => {
                        occupied_entry.get_mut().merge(suggestion.1);
                    }
                    Entry::Vacant(vacant_entry) => {
                        vacant_entry.insert(suggestion.1);
                    }
                }
            }
        }

        plans
    }
}

impl IntoIterator for IssueCollection {
    type Item = Issue;

    type IntoIter = std::vec::IntoIter<Issue>;

    fn into_iter(self) -> Self::IntoIter {
        self.issues.into_iter()
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
}
