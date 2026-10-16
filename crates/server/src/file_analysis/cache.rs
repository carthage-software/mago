//! Owns per-file editor data and the settings used to build it.

use std::sync::Arc;

use foldhash::HashMap;
use mago_database::DatabaseReader;
use mago_database::file::File;
use mago_database::file::FileId;
use mago_database::file::FileType;
use mago_database::membership::PathExclusions;
use mago_syntax::settings::ParserSettings;
use rayon::prelude::*;
use xxhash_rust::xxh3::xxh3_64;

use super::FileAnalysis;
use crate::linter::LinterContext;

/// A workspace's parse, name, and lint results. All build paths use the same
/// settings; only changed host files need eager work to publish diagnostics.
pub(crate) struct FileAnalyses {
    entries: HashMap<FileId, CachedAnalysis>,
    linter: LinterContext,
    /// The analyzer reports syntax and semantic issues for the files it covers.
    /// The file pass owns those checks for the rest, even when lint rules are
    /// disabled, unless a file is excluded from both the analyzer and the linter.
    analyzer: bool,
    analyzer_excludes: PathExclusions,
}

struct CachedAnalysis {
    contents_hash: u64,
    analysis: Arc<FileAnalysis>,
}

impl FileAnalyses {
    pub(crate) fn new(linter: LinterContext, analyzer: bool, analyzer_excludes: PathExclusions) -> Self {
        Self { entries: HashMap::default(), linter, analyzer, analyzer_excludes }
    }

    pub(crate) fn parser_settings(&self) -> ParserSettings {
        self.linter.parser_settings
    }

    pub(crate) fn invalidate(&mut self, files: &[FileId]) {
        for file in files {
            self.entries.remove(file);
        }
    }

    /// Query paths accept any file type. Content checks keep direct database
    /// updates from serving stale data, even before an explicit refresh.
    pub(crate) fn get_or_build(&mut self, file: &File) -> Arc<FileAnalysis> {
        let contents_hash = xxh3_64(&file.contents);
        if let Some(cached) = self.entries.get(&file.id)
            && cached.contents_hash == contents_hash
        {
            return Arc::clone(&cached.analysis);
        }

        let entry = self.build(file, contents_hash);
        let analysis = Arc::clone(&entry.analysis);
        self.entries.insert(file.id, entry);
        analysis
    }

    /// Query-only data stays lazy. Diagnostic providers need fresh host results
    /// before the caller publishes issues.
    pub(crate) fn refresh_diagnostics(&mut self, database: &impl DatabaseReader, files: &[FileId]) {
        if !self.reports_any() {
            return;
        }
        for &id in files {
            let Ok(file) = database.get(&id) else {
                self.entries.remove(&id);
                continue;
            };
            if file.file_type == FileType::Host && self.reports(&file) {
                let entry = self.build(&file, xxh3_64(&file.contents));
                self.entries.insert(id, entry);
            }
        }
    }

    /// Whether any host file can have file-pass diagnostics.
    pub(crate) fn reports_any(&self) -> bool {
        self.linter.registry.is_some() || !self.analyzer || !self.analyzer_excludes.is_empty()
    }

    fn reports(&self, file: &File) -> bool {
        self.linter.registry_for(file).is_some() || self.with_semantics(file)
    }

    fn with_semantics(&self, file: &File) -> bool {
        let unanalyzed = self.analyzer_excludes.excludes_file(file);
        (!self.analyzer || unanalyzed) && !(unanalyzed && self.linter.excludes.excludes_file(file))
    }

    pub(crate) fn refresh(&mut self, database: &impl DatabaseReader, files: &[FileId]) {
        for &id in files {
            let Ok(file) = database.get(&id) else {
                self.entries.remove(&id);
                continue;
            };
            if file.file_type == FileType::Host {
                let entry = self.build(&file, xxh3_64(&file.contents));
                self.entries.insert(id, entry);
            }
        }
    }

    pub(crate) fn refresh_all(&mut self, database: &impl DatabaseReader) {
        let files: Vec<_> = database.files_with_type(FileType::Host).filter(|file| self.reports(file)).collect();
        let entries: Vec<_> =
            files.into_par_iter().map(|file| (file.id, self.build(&file, xxh3_64(&file.contents)))).collect();
        self.entries.extend(entries);
    }

    pub(crate) fn sources(&self) -> impl Iterator<Item = (FileId, &Arc<FileAnalysis>)> + '_ {
        self.entries.iter().map(|(&file, entry)| (file, &entry.analysis))
    }

    pub(crate) fn analyses(&self) -> impl Iterator<Item = &Arc<FileAnalysis>> + '_ {
        self.entries.values().map(|entry| &entry.analysis)
    }

    fn build(&self, file: &File, contents_hash: u64) -> CachedAnalysis {
        let analysis = super::build(file, &self.linter, self.with_semantics(file));
        CachedAnalysis { contents_hash, analysis: Arc::new(analysis) }
    }
}
