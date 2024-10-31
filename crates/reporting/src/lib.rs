use std::iter::Once;

use codespan_reporting::diagnostic::Diagnostic;
use codespan_reporting::diagnostic::Label;
use codespan_reporting::diagnostic::LabelStyle;
use codespan_reporting::diagnostic::Severity;
use serde::Deserialize;
use serde::Serialize;

use fennec_fixer::FixPlan;
use fennec_source::SourceIdentifier;
use fennec_span::Span;

pub mod reporter;

/// Represents the kind of annotation associated with an issue.
#[derive(Debug, PartialEq, Eq, Ord, Copy, Clone, Hash, PartialOrd, Deserialize, Serialize)]
#[serde(tag = "type", content = "value")]
pub enum AnnotationKind {
    /// A primary annotation, typically highlighting the main source of the issue.
    Primary,
    /// A secondary annotation, providing additional context or related information.
    Secondary,
}

/// An annotation associated with an issue, providing additional context or highlighting specific code spans.
#[derive(Debug, PartialEq, Eq, Ord, Clone, Hash, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Annotation {
    /// An optional message associated with the annotation.
    pub message: Option<String>,
    /// The kind of annotation.
    pub kind: AnnotationKind,
    /// The code span that the annotation refers to.
    pub span: Span,
}

/// Represents the severity level of an issue.
#[derive(Debug, PartialEq, Eq, Ord, Copy, Clone, Hash, PartialOrd, Deserialize, Serialize)]
#[serde(tag = "type", content = "value")]
pub enum Level {
    /// A note, providing additional information or context.
    Note,
    /// A help message, suggesting possible solutions or further actions.
    Help,
    /// A warning, indicating a potential problem that may need attention.
    Warning,
    /// An error, indicating a problem that prevents the code from functioning correctly.
    Error,
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
    /// use fennec_reporting::issue::{Annotation, AnnotationKind};
    /// use fennec_span::Span;
    /// use fennec_span::Position;
    /// use fennec_source::SourceIdentifier;
    ///
    /// let source = SourceIdentifier::empty();
    /// let start = Position::new(source, 0, 0, 0);
    /// let end = Position::new(source, 0, 0, 5);
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
    /// use fennec_reporting::issue::{Annotation, AnnotationKind};
    /// use fennec_span::Span;
    /// use fennec_span::Position;
    /// use fennec_source::SourceIdentifier;
    ///
    /// let source = SourceIdentifier::empty();
    /// let start = Position::new(source, 0, 0, 0);
    /// let end = Position::new(source, 0, 0, 5);
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
    /// use fennec_reporting::issue::{Annotation, AnnotationKind};
    /// use fennec_span::Span;
    /// use fennec_span::Position;
    /// use fennec_source::SourceIdentifier;
    ///
    /// let source = SourceIdentifier::empty();
    /// let start = Position::new(source, 0, 0, 0);
    /// let end = Position::new(source, 0, 0, 5);
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
    /// use fennec_reporting::issue::{Annotation, AnnotationKind};
    /// use fennec_span::Span;
    /// use fennec_span::Position;
    /// use fennec_source::SourceIdentifier;
    ///
    /// let source = SourceIdentifier::empty();
    /// let start = Position::new(source, 0, 0, 0);
    /// let end = Position::new(source, 0, 0, 5);
    /// let span = Span::new(start, end);
    /// let annotation = Annotation::primary(span).with_message("This is a primary annotation");
    /// ```
    #[must_use]
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());

        self
    }
}

impl Issue {
    /// Creates a new issue with the given level and message.
    ///
    /// # Examples
    ///
    /// ```
    /// use fennec_reporting::issue::{Issue, Level};
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
    /// use fennec_reporting::issue::Issue;
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
    /// use fennec_reporting::issue::Issue;
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
    /// use fennec_reporting::issue::Issue;
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
    /// use fennec_reporting::issue::Issue;
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
    /// use fennec_reporting::issue::{Issue, Level};
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
    /// use fennec_reporting::issue::{Issue, Annotation, AnnotationKind};
    /// use fennec_span::Span;
    /// use fennec_span::Position;
    /// use fennec_source::SourceIdentifier;
    ///
    /// let source = SourceIdentifier::empty();
    /// let start = Position::new(source, 0, 0, 0);
    /// let end = Position::new(source, 0, 0, 5);
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
    /// use fennec_reporting::issue::Issue;
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
    /// use fennec_reporting::issue::Issue;
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
    /// use fennec_reporting::issue::Issue;
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
        self.issues.iter_mut().map(|issue| issue.take_suggestions()).flatten()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Issue> {
        self.issues.iter()
    }
}

impl IntoIterator for IssueCollection {
    type Item = Issue;

    type IntoIter = std::vec::IntoIter<Issue>;

    fn into_iter(self) -> Self::IntoIter {
        self.issues.into_iter()
    }
}

impl Into<LabelStyle> for AnnotationKind {
    fn into(self) -> LabelStyle {
        match self {
            AnnotationKind::Primary => LabelStyle::Primary,
            AnnotationKind::Secondary => LabelStyle::Secondary,
        }
    }
}

impl Into<Label<SourceIdentifier>> for Annotation {
    fn into(self) -> Label<SourceIdentifier> {
        let mut label = Label::new(self.kind.into(), self.span.start.source, self.span);

        if let Some(message) = self.message {
            label.message = message;
        }

        label
    }
}

impl Into<Severity> for Level {
    fn into(self) -> Severity {
        match self {
            Level::Note => Severity::Note,
            Level::Help => Severity::Help,
            Level::Warning => Severity::Warning,
            Level::Error => Severity::Error,
        }
    }
}

impl Into<Diagnostic<SourceIdentifier>> for Issue {
    fn into(self) -> Diagnostic<SourceIdentifier> {
        let mut diagnostic = Diagnostic::new(self.level.into()).with_message(self.message);

        if let Some(code) = self.code {
            diagnostic.code = Some(code);
        }

        for annotation in self.annotations {
            diagnostic.labels.push(annotation.into());
        }

        for note in self.notes {
            diagnostic.notes.push(note);
        }

        if let Some(help) = self.help {
            diagnostic.notes.push(format!("help: {}", help));
        }

        if let Some(link) = self.link {
            diagnostic.notes.push(format!("see: {}", link));
        }

        diagnostic
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
