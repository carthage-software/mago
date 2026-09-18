//! `get_code_actions`: quickfixes from analyzer/linter autofixes.
//!
//! For each issue with edits whose primary annotation overlaps the requested
//! byte range (in the requested file), emit one quickfix carrying the issue's
//! edits.

use foldhash::HashMap;

use mago_database::file::FileId;
use mago_reporting::Annotation;
use mago_reporting::AnnotationKind;
use mago_reporting::Issue;
use mago_reporting::Level;

use crate::Server;
use crate::domain::CodeActionItem;
use crate::domain::DiagnosticData;
use crate::domain::Range;
use crate::domain::Severity;
use crate::domain::TextReplacement;

impl Server {
    /// Quickfix code actions for issues anchored within `[start, end]` of
    /// `file_id`.
    #[must_use]
    pub fn get_code_actions(&self, file_id: FileId, start: u32, end: u32) -> Vec<CodeActionItem> {
        let mut actions = Vec::new();

        if let Some(issues) = self.last_issues() {
            for issue in issues.iter() {
                if let Some(action) = action_for(issue, file_id, start, end) {
                    actions.push(action);
                }
            }
        }

        for analysis in self.analyses() {
            for issue in analysis.lint_issues.iter() {
                if let Some(action) = action_for(issue, file_id, start, end) {
                    actions.push(action);
                }
            }
        }

        actions
    }
}

/// Build a code action for `issue` if it has edits and its primary annotation
/// overlaps `[start, end]` in `file_id`.
fn action_for(issue: &Issue, file_id: FileId, start: u32, end: u32) -> Option<CodeActionItem> {
    if issue.edits.is_empty() || !overlaps(issue, file_id, start, end) {
        return None;
    }

    let mut edits: HashMap<FileId, Vec<TextReplacement>> = HashMap::default();
    for (edit_file_id, file_edits) in &issue.edits {
        let replacements: Vec<TextReplacement> = file_edits
            .iter()
            .map(|edit| TextReplacement {
                range: Range::new(edit.range.start, edit.range.end),
                new_text: String::from_utf8_lossy(&edit.new_text).into_owned(),
            })
            .collect();

        if !replacements.is_empty() {
            edits.entry(*edit_file_id).or_default().extend(replacements);
        }
    }

    if edits.is_empty() {
        return None;
    }

    let title = issue.help.clone().unwrap_or_else(|| format!("Apply mago fix: {}", issue.message));
    let diagnostic = primary_annotation(issue).map(|primary| DiagnosticData {
        file: primary.span.file_id,
        range: Range::new(primary.span.start.offset, primary.span.end.offset),
        severity: level_to_severity(issue.level),
        code: issue.code.clone(),
        message: issue.message.clone(),
    });

    Some(CodeActionItem { title, edits, diagnostic })
}

/// Whether `issue`'s primary annotation is in `file_id` and overlaps `[start, end]`.
fn overlaps(issue: &Issue, file_id: FileId, start: u32, end: u32) -> bool {
    let Some(primary) = primary_annotation(issue) else {
        return false;
    };

    primary.span.file_id == file_id && primary.span.start.offset <= end && start <= primary.span.end.offset
}

fn primary_annotation(issue: &Issue) -> Option<&Annotation> {
    issue.annotations.iter().find(|a| matches!(a.kind, AnnotationKind::Primary)).or_else(|| issue.annotations.first())
}

const fn level_to_severity(level: Level) -> Severity {
    match level {
        Level::Error => Severity::Error,
        Level::Warning => Severity::Warning,
        Level::Help => Severity::Hint,
        Level::Note => Severity::Information,
    }
}
