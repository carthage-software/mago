//! Track issue owners and the files affected by their replacement.

use std::sync::Arc;
use std::sync::Weak;

use crate::FileAnalysis;
use foldhash::HashSet;
use mago_database::file::FileId;
use mago_reporting::IssueCollection;

use super::DiagnosticCache;
use super::primary_annotation;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(super) enum SourceKey {
    Analyzer(Option<FileId>, bool),
    Linter(FileId),
}

pub(super) enum Source {
    Analyzer(Arc<IssueCollection>),
    Linter(Arc<FileAnalysis>),
}

impl Source {
    pub(super) fn issues(&self) -> &IssueCollection {
        match self {
            Self::Analyzer(issues) => issues,
            Self::Linter(analysis) => &analysis.lint_issues,
        }
    }

    pub(super) fn downgrade(&self) -> WeakSource {
        match self {
            Self::Analyzer(issues) => WeakSource::Analyzer(Arc::downgrade(issues)),
            Self::Linter(analysis) => WeakSource::Linter(Arc::downgrade(analysis)),
        }
    }
}

/// Track diagnostics without keeping a replaced analysis alive.
pub(super) enum WeakSource {
    Analyzer(Weak<IssueCollection>),
    Linter(Weak<FileAnalysis>),
}

impl WeakSource {
    pub(super) fn upgrade(&self) -> Option<Source> {
        match self {
            Self::Analyzer(issues) => issues.upgrade().map(Source::Analyzer),
            Self::Linter(analysis) => analysis.upgrade().map(Source::Linter),
        }
    }

    fn same_issues(&self, other: &Source) -> bool {
        match (self, other) {
            (Self::Analyzer(a), Source::Analyzer(b)) if std::ptr::eq(a.as_ptr(), Arc::as_ptr(b)) => true,
            (Self::Linter(a), Source::Linter(b)) if std::ptr::eq(a.as_ptr(), Arc::as_ptr(b)) => true,
            _ => self.upgrade().is_some_and(|previous| previous.issues() == other.issues()),
        }
    }
}

pub(super) struct CachedSource {
    pub(super) source: WeakSource,
    pub(super) targets: Vec<FileId>,
    dependencies: Vec<FileId>,
}

impl CachedSource {
    fn new(source: Source) -> Self {
        let mut targets = Vec::new();
        let mut dependencies = Vec::new();
        for issue in source.issues() {
            if let Some(primary) = primary_annotation(issue) {
                targets.push(primary.span.file_id);
            }
            dependencies.extend(issue.annotations.iter().map(|annotation| annotation.span.file_id));
        }
        targets.sort_unstable();
        targets.dedup();
        dependencies.sort_unstable();
        dependencies.dedup();
        Self { source: source.downgrade(), targets, dependencies }
    }
}

impl DiagnosticCache {
    pub(super) fn update(
        &mut self,
        sources: impl Iterator<Item = (SourceKey, Source)>,
        changed: &[FileId],
    ) -> HashSet<FileId> {
        let changed: HashSet<_> = changed.iter().copied().collect();
        let mut dirty = changed.clone();
        let mut seen = HashSet::default();
        for (key, source) in sources {
            seen.insert(key);
            if let Some(cached) = self.sources.get_mut(&key) {
                if cached.source.same_issues(&source) {
                    cached.source = source.downgrade();
                    if cached.dependencies.iter().any(|file| changed.contains(file)) {
                        dirty.extend(&cached.targets);
                    }
                    continue;
                }
                dirty.extend(&cached.targets);
            }
            let cached = CachedSource::new(source);
            dirty.extend(&cached.targets);
            self.sources.insert(key, cached);
        }
        let previous_capacity = self.sources.capacity();
        self.sources.retain(|key, source| {
            if seen.contains(key) {
                true
            } else {
                dirty.extend(&source.targets);
                false
            }
        });
        if previous_capacity.max(self.sources.capacity()) > self.sources.len().saturating_mul(4).max(64) {
            self.sources.shrink_to_fit();
        }
        dirty
    }
}
