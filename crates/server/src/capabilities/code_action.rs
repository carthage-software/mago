//! Quick fixes and source refactors for a selected range.

use foldhash::HashMap;

use mago_database::file::FileId;
use mago_reporting::Issue;
use mago_reporting::Level;

use crate::Server;
use crate::diagnostics::{diagnostic_message, primary_annotation};
use crate::domain::CodeActionItem;
use crate::domain::CodeActionKind;
use crate::domain::DiagnosticData;
use crate::domain::Range;
use crate::domain::Severity;
use crate::domain::TextReplacement;

impl Server {
    /// Actions for `[start, end]`, limited to the requested kinds when supplied.
    #[must_use]
    pub fn get_code_actions(
        &self,
        file_id: FileId,
        start: u32,
        end: u32,
        only: Option<&[&str]>,
    ) -> Vec<CodeActionItem> {
        if self.editable_file(file_id).is_none() {
            return Vec::new();
        }
        let mut actions: Vec<_> = if CodeActionKind::QuickFix.requested(only) {
            self.analyzer_issues()
                .chain(self.lint_issues())
                .flat_map(|issues| issues.iter())
                .filter_map(|issue| action_for(issue, file_id, start, end))
                .collect()
        } else {
            Vec::new()
        };
        if CodeActionKind::QuickFix.requested(only) || CodeActionKind::RefactorRewrite.requested(only) {
            actions.extend(super::refactor_import::import_classes(self, file_id, start, end, only).unwrap_or_default());
        }
        if CodeActionKind::QuickFix.requested(only) {
            actions.extend(super::refactor_implement::implement_methods(self, file_id, start, end));
            actions.extend(super::refactor_generate::generate_method(self, file_id, start, end));
        }
        if CodeActionKind::RefactorExtract.requested(only) {
            actions.extend(super::refactor_extract::extract_local(self, file_id, start, end));
            actions.extend(super::refactor_extract_method::extract_method(self, file_id, start, end));
            actions.extend(super::refactor_constant::extract_constant(self, file_id, start, end));
        }
        // Keep each action whole: dropping one target could turn a fix into a partial edit.
        actions.retain(|action| self.can_edit_files(action.edits.keys().copied()));
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

    let title = issue.help.as_ref().map_or_else(|| format!("Apply mago fix: {}", issue.message), ToString::to_string);
    let diagnostic = primary_annotation(issue).map(|primary| DiagnosticData {
        file: primary.span.file_id,
        range: Range::new(primary.span.start.offset, primary.span.end.offset),
        severity: level_to_severity(issue.level),
        code: issue.code.clone(),
        message: diagnostic_message(issue, primary),
    });

    Some(CodeActionItem { title, kind: CodeActionKind::QuickFix, edits, diagnostic })
}

/// Whether `issue`'s primary annotation is in `file_id` and overlaps `[start, end]`.
fn overlaps(issue: &Issue, file_id: FileId, start: u32, end: u32) -> bool {
    let Some(primary) = primary_annotation(issue) else {
        return false;
    };

    primary.span.file_id == file_id && primary.span.start.offset <= end && start <= primary.span.end.offset
}

const fn level_to_severity(level: Level) -> Severity {
    match level {
        Level::Error => Severity::Error,
        Level::Warning => Severity::Warning,
        Level::Help => Severity::Hint,
        Level::Note => Severity::Information,
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use mago_reporting::Annotation;
    use mago_span::{Position, Span};
    use mago_text_edit::TextEdit;

    use super::*;

    #[test]
    fn quick_fix_diagnostics_use_the_published_primary_and_message() {
        let file = FileId::new(b"source.php");
        let primary = Span::new(file, Position::new(10), Position::new(15));
        let issue = Issue::error("Syntax error")
            .with_code("parse")
            .with_annotation(Annotation::primary(Span::new(file, Position::new(30), Position::new(35))))
            .with_annotation(Annotation::primary(primary).with_message("Expected a closing parenthesis"))
            .with_note("The call starts here")
            .with_help("Close the argument list")
            .with_edit(file, TextEdit::insert(15, b")".to_vec()));
        let action = action_for(&issue, file, 10, 15).unwrap();
        let diagnostic = action.diagnostic.unwrap();
        assert_eq!(diagnostic.range, Range::new(10, 15));
        assert_eq!(
            diagnostic.message,
            "Expected a closing parenthesis\n = The call starts here\n = Help: Close the argument list"
        );
        assert!(action_for(&issue, file, 30, 35).is_none());
    }
}
