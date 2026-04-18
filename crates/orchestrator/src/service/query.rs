//! Service for running a compiled [`mago_pattern`] query across a database of files.

use std::path::Path;
use std::sync::Arc;

use foldhash::HashMap;
use foldhash::HashMapExt;

use bumpalo::Bump;
use mago_database::ReadDatabase;
use mago_database::file::File;
use mago_database::file::FileId;
use mago_pattern::CompiledPattern;
use mago_pattern::MagoIndex;
use mago_pattern::Match;
use mago_pattern::Rewrite;
use mago_syntax::parser::parse_file_with_settings;
use mago_syntax::settings::ParserSettings;

use crate::error::OrchestratorError;
use crate::service::pipeline::StatelessParallelPipeline;
use crate::service::pipeline::StatelessReducer;

/// Per-file outcome of running a query.
#[derive(Debug, Clone)]
pub struct FileQueryStatus {
    /// Every match produced in this file.
    pub matches: Vec<Match>,
    /// Rewritten file contents, if the pattern produced any rewrites.
    ///
    /// When `None` the file has no rewrites (either the pattern doesn't rewrite or no
    /// matches fired). When `Some`, the string is the file's full contents with every
    /// rewrite applied back-to-front.
    pub rewritten: Option<String>,
}

impl FileQueryStatus {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.matches.is_empty() && self.rewritten.is_none()
    }
}

/// Aggregate result of running a query across every file in the database.
#[derive(Debug, Default)]
pub struct QueryResult {
    /// Per-file outcomes, keyed by file id. Files that produced neither matches nor
    /// rewrites are omitted.
    pub files: HashMap<FileId, FileQueryStatus>,
}

impl QueryResult {
    #[must_use]
    pub fn new() -> Self {
        Self { files: HashMap::new() }
    }

    /// Returns `true` when no file produced a match.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Returns the total number of matches produced across every file.
    #[must_use]
    pub fn total_matches(&self) -> usize {
        self.files.values().map(|s| s.matches.len()).sum()
    }

    /// Iterates over per-file status entries.
    pub fn iter(&self) -> impl Iterator<Item = (&FileId, &FileQueryStatus)> {
        self.files.iter()
    }

    /// Iterates only over files that produced rewrites. Yields
    /// `(file_id, rewritten_contents)` pairs.
    pub fn rewritten_files(&self) -> impl Iterator<Item = (&FileId, &String)> {
        self.files.iter().filter_map(|(id, status)| status.rewritten.as_ref().map(|s| (id, s)))
    }

    /// Number of files that produced at least one rewrite.
    #[must_use]
    pub fn rewritten_files_count(&self) -> usize {
        self.files.values().filter(|s| s.rewritten.is_some()).count()
    }
}

/// Service that runs a compiled query pattern across a database of files in parallel.
pub struct QueryService {
    database: ReadDatabase,
    pattern: Arc<CompiledPattern>,
    parser_settings: ParserSettings,
    use_progress_bars: bool,
}

impl QueryService {
    #[must_use]
    pub fn new(
        database: ReadDatabase,
        pattern: Arc<CompiledPattern>,
        parser_settings: ParserSettings,
        use_progress_bars: bool,
    ) -> Self {
        Self { database, pattern, parser_settings, use_progress_bars }
    }

    /// Runs the pattern against every host file in the database in parallel.
    pub fn run(self) -> Result<QueryResult, OrchestratorError> {
        let context = QueryContext { pattern: self.pattern, parser_settings: self.parser_settings };

        let pipeline = StatelessParallelPipeline::new(
            "🔎 Querying",
            self.database,
            context,
            Box::new(QueryReducer),
            self.use_progress_bars,
        );

        pipeline.run(query_one)
    }

    /// Runs the pattern against a specific subset of files identified by [`FileId`].
    pub fn run_on_files<Iter>(self, file_ids: Iter) -> Result<QueryResult, OrchestratorError>
    where
        Iter: IntoIterator<Item = FileId>,
    {
        let context = QueryContext { pattern: self.pattern, parser_settings: self.parser_settings };

        let pipeline = StatelessParallelPipeline::new(
            "🔎 Querying",
            self.database,
            context,
            Box::new(QueryReducer),
            self.use_progress_bars,
        );

        pipeline.run_on_files(file_ids, query_one)
    }
}

/// Runs the pattern against a single file; shared by [`QueryService::run`] and
/// [`QueryService::run_on_files`].
fn query_one(context: QueryContext, arena: &Bump, file: Arc<File>) -> Result<QueryResult, OrchestratorError> {
    let program = parse_file_with_settings(arena, &file, context.parser_settings);
    let source = arena.alloc_str(file.contents.as_ref());
    let index = MagoIndex::new(program, source);
    let path = file.path.as_deref().unwrap_or_else(|| Path::new(file.name.as_ref()));
    let matches = mago_pattern::query(context.pattern.as_ref(), &index, path)
        .map_err(|err| OrchestratorError::General(format!("{}: {err}", file.name)))?;

    let mut files = HashMap::with_capacity(1);
    if !matches.is_empty() {
        let rewritten = apply_rewrites(source, &matches);
        files.insert(file.id, FileQueryStatus { matches, rewritten });
    }

    Ok(QueryResult { files })
}

/// Applies every rewrite in `matches` to `source`, back-to-front so earlier ranges stay
/// valid. Returns `None` when there are no rewrites to apply.
fn apply_rewrites(source: &str, matches: &[Match]) -> Option<String> {
    let mut all: Vec<Rewrite> = matches.iter().flat_map(|m| m.rewrites.iter().cloned()).collect();
    if all.is_empty() {
        return None;
    }

    all.sort_by_key(|r| std::cmp::Reverse(r.range.start));
    let mut out = source.to_string();
    for r in &all {
        let start = r.range.start.min(out.len());
        let end = r.range.end.min(out.len());
        if start > end {
            continue;
        }

        out.replace_range(start..end, &r.replacement);
    }

    Some(out)
}

/// Shared read-only context handed to every parallel query task.
#[derive(Clone)]
struct QueryContext {
    pattern: Arc<CompiledPattern>,
    parser_settings: ParserSettings,
}

/// Reducer that merges per-file `QueryResult`s into a single aggregate.
#[derive(Debug)]
struct QueryReducer;

impl StatelessReducer<QueryResult, QueryResult> for QueryReducer {
    fn reduce(&self, results: Vec<QueryResult>) -> Result<QueryResult, OrchestratorError> {
        let mut files = HashMap::with_capacity(results.len());
        for result in results {
            files.extend(result.files);
        }

        Ok(QueryResult { files })
    }
}
