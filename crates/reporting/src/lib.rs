use std::collections::hash_map::Entry;
use std::iter::Once;

use ahash::HashMap;
use serde::Deserialize;
use serde::Serialize;
use strum::Display;

use mago_fixer::FixPlan;
use mago_source::SourceIdentifier;
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
#[derive(Debug, PartialEq, Eq, Ord, Copy, Clone, Hash, PartialOrd, Deserialize, Serialize, Display)]
pub enum Level {
    /// An error, indicating a problem that prevents the code from functioning correctly.
    Error,
    /// A warning, indicating a potential problem that may need attention.
    Warning,
    /// A help message, suggesting possible solutions or further actions.
    Help,
    /// A note, providing additional information or context.
    Note,
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
    pub suggestions: Vec<(SourceIdentifier, FixPlan)>,
}

/// A collection of issues.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IssueCollection {
    issues: Vec<Issue>,
}

impl Annotation {
    /// Creates a new annotation with the given kind and span.
    ///
    /// # Examples
    ///
    /// ```
    /// use mago_reporting::{Annotation, AnnotationKind};
    /// use mago_span::Span;
    /// use mago_span::Position;
    ///
    /// let start = Position::dummy(0);
    /// let end = Position::dummy(5);
    /// let span = Span::new(start, end);
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
    /// use mago_span::Span;
    /// use mago_span::Position;
    ///
    /// let start = Position::dummy(0);
    /// let end = Position::dummy(5);
    /// let span = Span::new(start, end);
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
    /// use mago_span::Span;
    /// use mago_span::Position;
    ///
    /// let start = Position::dummy(0);
    /// let end = Position::dummy(5);
    /// let span = Span::new(start, end);
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
    /// use mago_span::Span;
    /// use mago_span::Position;
    ///
    /// let start = Position::dummy(0);
    /// let end = Position::dummy(5);
    /// let span = Span::new(start, end);
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
    /// use mago::reporting::Level;
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
    /// use mago_span::Span;
    /// use mago_span::Position;
    ///
    /// let start = Position::dummy(0);
    /// let end = Position::dummy(5);
    /// let span = Span::new(start, end);
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
    pub fn with_suggestion(mut self, source: SourceIdentifier, plan: FixPlan) -> Self {
        self.suggestions.push((source, plan));

        self
    }

    /// Take the code modification suggestion from this issue.
    #[must_use]
    pub fn take_suggestions(&mut self) -> Vec<(SourceIdentifier, FixPlan)> {
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
        self.issues.push(issue);
    }

    pub fn extend(&mut self, issues: impl IntoIterator<Item = Issue>) {
        self.issues.extend(issues);
    }

    pub fn is_empty(&self) -> bool {
        self.issues.is_empty()
    }

    pub fn len(&self) -> usize {
        self.issues.len()
    }

    pub fn with_maximum_level(self, level: Level) -> Self {
        Self { issues: self.issues.into_iter().filter(|issue| issue.level <= level).collect() }
    }

    pub fn with_minimum_level(self, level: Level) -> Self {
        Self { issues: self.issues.into_iter().filter(|issue| issue.level >= level).collect() }
    }

    pub fn has_minimum_level(&self, level: Level) -> bool {
        self.issues.iter().any(|issue| issue.level >= level)
    }

    pub fn get_level_count(&self, level: Level) -> usize {
        self.issues.iter().filter(|issue| issue.level == level).count()
    }

    pub fn get_highest_level(&self) -> Option<Level> {
        self.issues.iter().map(|issue| issue.level).max()
    }

    pub fn with_code(self, code: impl Into<String>) -> IssueCollection {
        let code = code.into();

        Self { issues: self.issues.into_iter().map(|issue| issue.with_code(&code)).collect() }
    }

    pub fn take_suggestions(&mut self) -> impl Iterator<Item = (SourceIdentifier, FixPlan)> + '_ {
        self.issues.iter_mut().flat_map(|issue| issue.take_suggestions())
    }

    pub fn only_fixable(self) -> impl Iterator<Item = Issue> {
        self.issues.into_iter().filter(|issue| !issue.suggestions.is_empty())
    }

    pub fn iter(&self) -> impl Iterator<Item = &Issue> {
        self.issues.iter()
    }

    pub fn to_fix_plans(self) -> HashMap<SourceIdentifier, FixPlan> {
        let mut plans: HashMap<SourceIdentifier, FixPlan> = HashMap::default();
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
