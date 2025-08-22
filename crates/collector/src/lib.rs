use ahash::HashMap;
use bumpalo::Bump;
use bumpalo::collections::CollectIn;
use bumpalo::collections::Vec;

use mago_database::file::File;
use mago_fixer::FixPlan;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::IssueCollection;
use mago_span::Span;
use mago_syntax::ast::Program;

use crate::pragma::Pragma;
use crate::pragma::PragmaKind;
use crate::walk::attach_pragma_scopes;

pub mod pragma;

mod walk;

/// A stateful collector for diagnostics (`Issue`s) within a specific category (e.g., "lint", "analysis").
///
/// It is responsible for:
///
/// - Collecting issues reported by various tools.
/// - Filtering issues based on configuration or suppression pragmas (`@mago-ignore`, `@mago-expect`).
/// - Reporting unused or unfulfilled pragmas.
#[derive(Debug)]
pub struct Collector<'ctx, 'arena> {
    /// The arena used for allocation of issues and pragmas.
    arena: &'arena Bump,
    /// The source file from which this collector was created.
    file: &'ctx File,
    /// All pragmas that have not yet been applied to a node.
    pragmas: Vec<'arena, Pragma<'arena>>,
    /// The collection of issues that have been reported and not suppressed.
    issues: IssueCollection,
    /// A stack of issue collections for recording issues speculatively.
    recordings: Vec<'arena, IssueCollection>,
    /// A list of issue codes that should be silently ignored.
    disabled_codes: Vec<'arena, &'static str>,
    /// A map of legacy issue codes to their new, canonical counterparts.
    aliases: HashMap<&'static str, &'static str>,
    /// An optional URL template for generating links to issue documentation.
    link_template: Option<&'static str>,
}

impl<'ctx, 'arena> Collector<'ctx, 'arena> {
    /// Creates a new `Collector` from a slice of trivia.
    ///
    /// This is the primary constructor. It pre-parses the given trivia to find pragmas
    /// relevant to the specified category. This is useful when the full program AST is not
    /// needed or available.
    pub fn new<'ast>(
        arena: &'arena Bump,
        file: &'ctx File,
        program: &'ast Program<'arena>,
        category: &'static str,
    ) -> Self {
        let mut collector = Self {
            arena,
            file,
            pragmas: Pragma::extract(arena, file, program.trivia.as_slice(), Some(category)),
            issues: IssueCollection::new(),
            recordings: Vec::new_in(arena),
            disabled_codes: Vec::new_in(arena),
            aliases: HashMap::default(),
            link_template: None,
        };

        attach_pragma_scopes(&mut collector, program);

        collector
    }

    /// Sets the issue code aliases.
    ///
    /// This allows old issue codes used in pragmas to be mapped to their new,
    /// canonical counterparts. The map should be from `alias -> canonical_code`.
    #[inline]
    pub fn set_aliases<'c>(&mut self, aliases: impl IntoIterator<Item = &'c (&'static str, &'static str)>) {
        self.aliases = aliases.into_iter().copied().collect();
    }

    /// Sets the link template for generating documentation URLs for issues.
    ///
    /// The template should contain a `{code}` placeholder which will be replaced
    /// by the issue's code.
    #[inline]
    pub fn set_link_template(&mut self, template: &'static str) {
        self.link_template = Some(template);
    }

    /// Overwrites the list of disabled issue codes.
    #[inline]
    pub fn set_disabled_codes(&mut self, codes: impl IntoIterator<Item = &'static str>) {
        self.disabled_codes = codes.into_iter().collect_in(self.arena);
    }

    /// Adds new codes to the list of disabled issue codes.
    #[inline]
    pub fn add_disabled_codes(&mut self, codes: impl IntoIterator<Item = &'static str>) {
        self.disabled_codes.extend(codes);
    }

    /// Reports an issue without checking for suppression pragmas.
    ///
    /// This should be used for issues that must always be reported, such as internal errors
    /// or issues related to pragmas themselves.
    ///
    /// If a recording is active (see `start_recording`), the issue is added to the
    /// current recording. Otherwise, it is added to the main issue collection.
    #[inline]
    pub fn force_report(&mut self, mut issue: Issue) {
        issue.annotations.retain(|annotation| !annotation.span.file_id.is_zero());

        if let (Some(template), Some(code)) = (self.link_template, issue.code.as_deref()) {
            let link = template.replace("{code}", code);

            issue = issue.with_link(link);
        }

        if let Some(recording) = self.recordings.last_mut() {
            recording.push(issue);
        } else {
            self.issues.push(issue);
        }
    }

    /// Reports an issue, returning `true` if it was added or `false` if it was suppressed.
    #[inline]
    pub fn report(&mut self, issue: Issue) -> bool {
        let primary_span = issue.annotations.iter().find(|ann| ann.kind.is_primary()).map(|ann| ann.span);

        if let Some(code) = issue.code.as_deref() {
            if self.disabled_codes.contains(&code) {
                // This code is disabled, do not report it.
                return false;
            }
        } else if cfg!(debug_assertions) {
            let mut missing_code_issue = Issue::error("Internal: Diagnostic is missing a code.")
                .with_code("missing-code")
                .with_note("This diagnostic was reported without a unique code, which is required by the collector.")
                .with_help("Please report this issue to the Mago team.")
                .with_link("https://github.com/carthage-software/mago");

            if let Some(span) = primary_span {
                missing_code_issue = missing_code_issue.with_annotation(
                    Annotation::primary(span).with_message("This diagnostic was reported without a unique code."),
                );
            }

            self.force_report(missing_code_issue);

            return false;
        }

        if let Some(span) = primary_span
            && let Some(code) = &issue.code
            && !self.is_recording()
        {
            if self.is_ignored(span, code) {
                return false;
            }

            if self.is_expected(span, code) {
                return false;
            }
        }

        self.force_report(issue);
        true
    }

    /// Reports an issue with a specific code, returning `true` if it was added.
    ///
    /// This is a convenience method that is equivalent to `report(issue.with_code(code))`.
    #[inline]
    pub fn report_with_code(&mut self, code: impl Into<String>, issue: Issue) -> bool {
        self.report(issue.with_code(code))
    }

    /// Extends the collector with issues from an issue iterator.
    ///
    /// Each issue from the provided iterator is passed through the `report` method,
    /// which means they will be subject to the same suppression and filtering logic
    /// as individually reported issues.
    #[inline]
    pub fn extend(&mut self, issues: impl IntoIterator<Item = Issue>) {
        for issue in issues.into_iter() {
            self.report(issue);
        }
    }

    /// Reports an issue with a suggested fix, returning `true` if it was added.
    ///
    /// This is a convenience method that builds a `FixPlan` from the provided closure
    /// and attaches it to the issue before calling `report`.
    #[inline]
    pub fn propose<F>(&mut self, mut issue: Issue, f: F) -> bool
    where
        F: FnOnce(&mut FixPlan),
    {
        let mut plan = FixPlan::new();
        f(&mut plan);
        if !plan.is_empty() {
            issue = issue.with_suggestion(self.file.id, plan);
        }

        self.report(issue)
    }

    /// Records all issues generated by a callback without modifying the collector's state.
    ///
    /// This method allows you to run a closure that reports issues and capture them
    /// in a `IssueCollection` without consuming pragmas or permanently adding the issues
    /// to the main collector. This is useful for speculative analysis.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let issues = collector.record(|c| {
    ///     c.report(Issue::error("speculative error"));
    /// });
    ///
    /// // `issues` contains the speculative error, but the main collector is unchanged.
    /// ```
    #[inline]
    pub fn record<F, T>(&mut self, f: F) -> (T, IssueCollection)
    where
        F: FnOnce(&mut Self) -> T,
    {
        self.start_recording();
        let result = f(self);
        let recorded_issues = self.finish_recording().unwrap_or_default();

        // Return the captured issues.
        (result, recorded_issues)
    }

    /// Starts a new recording session for speculative analysis.
    ///
    /// Any issues reported after this call will be captured in a separate collection
    /// instead of the main one. This is useful for temporarily capturing diagnostics
    /// without affecting the final report. Recordings can be nested.
    ///
    /// Each call to `start_recording` should be paired with a call to `stop_recording`.
    #[inline]
    pub fn start_recording(&mut self) {
        self.recordings.push(IssueCollection::new());
    }

    /// Checks if a recording session is currently active.
    ///
    /// Returns `true` if there is at least one recording in progress.
    /// This is useful to determine if you can safely call `stop_recording`.
    #[inline]
    pub fn is_recording(&self) -> bool {
        !self.recordings.is_empty()
    }

    /// Finish the current recording session and returns the captured issues.
    ///
    /// Returns `None` if no recording session is active.
    #[inline]
    pub fn finish_recording(&mut self) -> Option<IssueCollection> {
        self.recordings.pop()
    }

    /// Finalizes the collection process and returns an iterator over all generated issues.
    ///
    /// This method consumes the collector and performs final checks, generating new issues for:
    ///
    /// - Unfulfilled `@mago-expect` pragmas.
    /// - Unused pragmas of any kind.
    #[inline]
    pub fn finish(mut self) -> IssueCollection {
        let mut issues = self.issues;

        for pragma in self.pragmas.drain(..) {
            if pragma.used {
                continue;
            }

            match pragma.kind {
                PragmaKind::Ignore => {
                    issues.push(
                        Issue::note("This pragma was not used and may be removed.")
                            .with_code("unused-pragma")
                            .with_annotation(
                                Annotation::primary(pragma.span)
                                    .with_message("This ignore pragma does not match any reported issue."),
                            )
                            .with_annotation(
                                Annotation::secondary(pragma.trivia_span).with_message("...within this comment."),
                            ),
                    );
                }
                PragmaKind::Expect => {
                    issues.push(
                        Issue::warning("This pragma was not used and may be removed.")
                            .with_code("unfulfilled-expect")
                            .with_annotation(
                                Annotation::primary(pragma.span).with_message("This expect pragma was not fulfilled."),
                            )
                            .with_annotation(
                                Annotation::secondary(pragma.trivia_span).with_message("...within this comment."),
                            ),
                    );
                }
            }
        }

        issues
    }

    /// Checks if an issue is suppressed by an `@mago-ignore` pragma.
    ///
    /// Finds the nearest applicable pragma and marks it as used.
    #[inline]
    fn is_ignored(&mut self, issue_span: Span, issue_code: &str) -> bool {
        if let Some(pragma) = self.find_best_applicable_pragma_mut(issue_span, PragmaKind::Ignore, issue_code) {
            pragma.used = true;
            return true;
        }
        false
    }

    /// Checks if an issue is suppressed by an `@mago-expect` pragma.
    ///
    /// Finds the nearest applicable pragma and marks it as used.
    #[inline]
    fn is_expected(&mut self, issue_span: Span, issue_code: &str) -> bool {
        if let Some(pragma) = self.find_best_applicable_pragma_mut(issue_span, PragmaKind::Expect, issue_code) {
            pragma.used = true;
            return true;
        }

        false
    }

    /// Finds the *nearest* pragma that applies to a given issue and returns a mutable reference to it.
    ///
    /// This method does **not** consume the pragma, allowing a single scoped pragma to be used
    /// multiple times. It determines applicability and proximity to find the single best match.
    #[inline]
    fn find_best_applicable_pragma_mut(
        &mut self,
        issue_span: Span,
        kind: PragmaKind,
        issue_code: &str,
    ) -> Option<&mut Pragma<'arena>> {
        let issue_start_line = self.file.line_number(issue_span.start.offset);

        let mut best_match_index = None;

        for (i, pragma) in self.pragmas.iter().enumerate() {
            let resolved_pragma_code = self.aliases.get(pragma.code).copied().unwrap_or(pragma.code);

            if pragma.kind != kind || resolved_pragma_code != issue_code {
                continue;
            }

            let is_applicable = if let Some(scope_span) = pragma.scope_span {
                scope_span.contains(&issue_span) || issue_span.contains(&scope_span)
            } else if pragma.used {
                false
            } else if pragma.trivia_span.contains(&issue_span) || issue_span.contains(&pragma.trivia_span) {
                // The issue is inside the same comment as the pragma!
                true
            } else if pragma.own_line {
                pragma.start_line < issue_start_line
            } else {
                self.file.line_number(pragma.span.start.offset) == issue_start_line
            };

            if !is_applicable {
                continue;
            }

            if let Some(current_best_index) = best_match_index {
                let current_best: &Pragma<'_> = &self.pragmas[current_best_index];
                if !current_best.own_line && pragma.own_line {
                    // Current best is inline, new one is docblock. Keep current.
                } else if current_best.own_line && !pragma.own_line {
                    // Current best is docblock, new one is inline. New one is better.
                    best_match_index = Some(i);
                } else if pragma.start_line > current_best.start_line {
                    // Both are same type, the one on a later line is better.
                    best_match_index = Some(i);
                }
            } else {
                best_match_index = Some(i);
            }
        }

        best_match_index.map(|i| &mut self.pragmas[i])
    }
}
