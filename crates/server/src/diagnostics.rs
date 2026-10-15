//! Track changed diagnostics without keeping old analyses alive.

use std::sync::Arc;

use foldhash::HashMap;
use foldhash::HashSet;
use mago_database::Database;
use mago_database::DatabaseReader;
use mago_database::file::FileId;
use mago_reporting::CompiledIgnoreSet;
use mago_reporting::Issue;

use crate::Server;

mod presentation;
mod sources;

pub use presentation::DiagnosticTags;
pub use presentation::code_spans;
pub use presentation::diagnostic_message;
pub use presentation::diagnostic_tags;
pub use presentation::message_lines;
pub use presentation::primary_annotation;
pub use presentation::related_annotations;
use sources::{CachedSource, Source, SourceKey};

#[cfg(test)]
mod tests;

/// Keep source fingerprints and dependencies, rather than copies of issues.
#[derive(Default)]
pub struct DiagnosticCache {
    sources: HashMap<SourceKey, CachedSource>,
}

/// Issues for changed files. Empty files must also reach the caller so it can clear old output.
///
/// The batch borrows issue collections through shared owners. Dropping it releases those owners.
pub struct DiagnosticUpdate {
    pub files: HashSet<FileId>,
    sources: Vec<(SourceKey, Source)>,
}

impl DiagnosticCache {
    /// Return only changed targets, including files whose issues disappeared.
    pub fn refresh(&mut self, server: &Server, changed: &[FileId]) -> DiagnosticUpdate {
        let sources = server
            .shared_analyzer_issues()
            .filter(|(_, _, issues)| !issues.is_empty())
            .map(|(file, codebase, issues)| (SourceKey::Analyzer(file, codebase), Source::Analyzer(Arc::clone(issues))))
            .chain(
                server
                    .lint_issue_sources()
                    .filter(|(_, analysis)| !analysis.lint_issues.is_empty())
                    .map(|(file, analysis)| (SourceKey::Linter(file), Source::Linter(Arc::clone(analysis)))),
            );
        let files = self.update(sources, changed);
        self.select(files)
    }

    fn select(&self, files: HashSet<FileId>) -> DiagnosticUpdate {
        let mut sources: Vec<_> = self
            .sources
            .iter()
            .filter(|(_, source)| source.targets.iter().any(|target| files.contains(target)))
            .filter_map(|(key, source)| Some((*key, source.source.upgrade()?)))
            .collect();
        // Hash table growth must not change issue order.
        sources.sort_unstable_by_key(|(key, _)| *key);
        DiagnosticUpdate { files, sources }
    }
}

impl DiagnosticUpdate {
    /// Apply analyzer ignores and hide analyzer cascades while a file has parser errors.
    /// Linter issues keep their own rule settings and ignore policy.
    pub fn issues<'a>(
        &'a self,
        database: &'a Database<'_>,
        ignores: &'a CompiledIgnoreSet,
    ) -> impl Iterator<Item = &'a Issue> {
        let visible = |key: SourceKey, issue: &Issue| {
            !matches!(key, SourceKey::Analyzer(..))
                || !ignores.is_ignored(issue, |file| {
                    database.get_ref(&file).ok().map(|file| String::from_utf8_lossy(&file.name).into_owned())
                })
        };
        // Keep recovery data for queries, but hide a file's analyzer diagnostics while parser errors are visible.
        let parse_errors: HashSet<_> = self
            .sources
            .iter()
            .flat_map(|(key, source)| {
                source.issues().iter().filter(move |issue| is_parse_error(issue) && visible(*key, issue))
            })
            .filter_map(|issue| primary_annotation(issue).map(|primary| primary.span.file_id))
            .collect();
        self.sources
            .iter()
            .flat_map(|(key, source)| source.issues().iter().map(move |issue| (*key, issue)))
            .filter(move |(key, issue)| {
                primary_annotation(issue).is_some_and(|primary| {
                    self.files.contains(&primary.span.file_id)
                        && (!matches!(key, SourceKey::Analyzer(..))
                            || is_parse_error(issue)
                            || !parse_errors.contains(&primary.span.file_id))
                }) && visible(*key, issue)
            })
            .map(|(_, issue)| issue)
    }
}

fn is_parse_error(issue: &Issue) -> bool {
    matches!(issue.code.as_deref(), Some("parse" | "syntax"))
}
