use std::sync::Arc;

use ahash::HashMap;
use ahash::HashSet;
use bumpalo::Bump;
use rayon::prelude::*;

use mago_atom::AtomSet;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::populator::populate_codebase;
use mago_codex::reference::SymbolReferences;
use mago_codex::scanner::scan_program;
use mago_codex::signature_builder;
use mago_database::DatabaseReader;
use mago_database::ReadDatabase;
use mago_database::file::File;
use mago_database::file::FileId;
use mago_database::file::FileType;
use mago_names::resolver::NameResolver;
use mago_syntax::parser::parse_file;

use crate::error::OrchestratorError;
use crate::service::pipeline::Reducer;

use std::fmt::Debug;

/// A callback type invoked after the scanning phase completes.
pub type PostScanCallback = Box<dyn FnOnce(&mut CodebaseMetadata, &SymbolReferences) + Send>;

/// File state tracked for incremental analysis
#[derive(Debug, Clone, Copy)]
pub struct FileState {
    /// Content hash of the file (`xxhash3_64`)
    pub content_hash: u64,
}

/// Incremental pipeline that tracks file changes and only re-scans changed files.
///
/// This pipeline is optimized for watch mode and LSP usage where files change
/// infrequently. It maintains a cache of file content hashes and their corresponding
/// metadata, allowing it to skip parsing and scanning of unchanged files entirely.
pub struct IncrementalParallelPipeline<T, I, R> {
    task_name: &'static str,
    database: Arc<ReadDatabase>,
    codebase: CodebaseMetadata,
    symbol_references: SymbolReferences,
    shared_context: T,
    reducer: Box<dyn Reducer<I, R> + Send + Sync>,
    after_scanning: Option<PostScanCallback>,
    previous_file_states: HashMap<FileId, FileState>,
}

impl<T, I, R> std::fmt::Debug for IncrementalParallelPipeline<T, I, R>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IncrementalParallelPipeline")
            .field("task_name", &self.task_name)
            .field("database", &self.database)
            .field("codebase", &self.codebase)
            .field("symbol_references", &self.symbol_references)
            .field("shared_context", &self.shared_context)
            .field("reducer", &"<reducer>")
            .field("after_scanning", &self.after_scanning.is_some())
            .field("previous_file_states", &format!("{} tracked files", self.previous_file_states.len()))
            .finish()
    }
}

impl<T, I, R> IncrementalParallelPipeline<T, I, R>
where
    T: Clone + Send + Sync + 'static,
    I: Send + 'static,
    R: Send + 'static,
{
    /// Creates a new `IncrementalParallelPipeline`.
    pub fn new(
        task_name: &'static str,
        database: ReadDatabase,
        codebase: CodebaseMetadata,
        symbol_references: SymbolReferences,
        shared_context: T,
        reducer: Box<dyn Reducer<I, R> + Send + Sync>,
        previous_file_states: HashMap<FileId, FileState>,
    ) -> Self {
        Self {
            task_name,
            database: Arc::new(database),
            codebase,
            symbol_references,
            shared_context,
            reducer,
            after_scanning: None,
            previous_file_states,
        }
    }

    /// Sets a callback to be invoked after the scanning phase completes.
    pub fn with_after_scanning(
        mut self,
        callback: impl FnOnce(&mut CodebaseMetadata, &SymbolReferences) + Send + 'static,
    ) -> Self {
        self.after_scanning = Some(Box::new(callback));
        self
    }

    /// Executes the incremental pipeline with a given map function.
    ///
    /// This method:
    /// 1. Computes content hashes for all files
    /// 2. Compares against previous hashes to detect changes
    /// 3. Only parses + scans changed files
    /// 4. Reuses cached metadata for unchanged files
    /// 5. Returns updated file states for next run
    pub fn run<F>(
        self,
        map_function: F,
    ) -> Result<(R, CodebaseMetadata, SymbolReferences, HashMap<FileId, FileState>), OrchestratorError>
    where
        F: Fn(T, &Bump, Arc<File>, Arc<CodebaseMetadata>) -> Result<I, OrchestratorError> + Send + Sync + 'static,
    {
        let source_files = self.database.files().filter(|f| f.file_type != FileType::Builtin).collect::<Vec<_>>();
        if source_files.is_empty() {
            tracing::info!("No source files found for analysis.");

            let (result, codebase, symbol_references) =
                self.reducer.reduce(self.codebase, self.symbol_references, Vec::new())?;
            return Ok((result, codebase, symbol_references, HashMap::default()));
        }

        // Phase 1: Detect changed files by comparing content hashes
        tracing::debug!("Computing content hashes for {} files", source_files.len());

        let file_hashes: HashMap<FileId, u64> = source_files
            .par_iter()
            .map(|file| {
                // Compute xxhash3_64 of file contents
                let hash = xxhash_rust::xxh3::xxh3_64(file.contents.as_bytes());
                (file.id, hash)
            })
            .collect();

        // Identify changed vs unchanged files
        let mut changed_files = Vec::new();
        let mut unchanged_files = Vec::new();

        for file in &source_files {
            let current_hash = file_hashes[&file.id];

            if let Some(prev_state) = self.previous_file_states.get(&file.id) {
                if prev_state.content_hash == current_hash {
                    // File unchanged - keep cached metadata
                    unchanged_files.push(file);
                    tracing::debug!("File unchanged (cached): {}", file.name);
                } else {
                    // File content changed - needs re-scanning
                    changed_files.push(file);
                    tracing::debug!("File changed (hash mismatch): {}", file.name);
                }
            } else {
                // New file - needs scanning
                changed_files.push(file);
                tracing::debug!("New file (not in cache): {}", file.name);
            }
        }

        tracing::debug!(
            "Incremental scan: {} changed/new, {} unchanged ({}% cached)",
            changed_files.len(),
            unchanged_files.len(),
            if source_files.is_empty() { 0 } else { (unchanged_files.len() * 100) / source_files.len() }
        );

        let partial_codebases: Vec<CodebaseMetadata> = changed_files
            .into_par_iter()
            .map_init(Bump::new, |arena, file| {
                let (program, parse_issues) = parse_file(arena, file);
                if parse_issues.is_some() {
                    tracing::warn!("Parsing issues in '{}'. Codebase analysis may be incomplete.", file.name);
                }

                let resolver = NameResolver::new(arena);
                let resolved_names = resolver.resolve(program);

                let mut metadata = scan_program(arena, file, program, &resolved_names);
                metadata.set_file_signature(
                    file.id,
                    signature_builder::build_file_signature(file, program, &resolved_names),
                );

                arena.reset();

                metadata
            })
            .collect();

        // Phase 3: Merge new metadata with cached metadata from unchanged files
        let mut merged_codex = self.codebase;

        // Add metadata from changed files
        for partial in partial_codebases {
            merged_codex.extend(partial);
        }

        // Note: We don't need to explicitly handle unchanged files here.
        // The metadata from the previous run is already in self.codebase,
        // and we only extended it with new metadata from changed files.
        // The unchanged files' metadata is already present in merged_codex.

        let mut symbol_references = self.symbol_references;
        populate_codebase(&mut merged_codex, &mut symbol_references, AtomSet::default(), HashSet::default());

        // Invoke after_scanning callback if provided (for incremental analysis)
        if let Some(callback) = self.after_scanning {
            callback(&mut merged_codex, &symbol_references);
        }

        // Phase 4: Build new file states cache for next run
        let new_file_states: HashMap<FileId, FileState> = source_files
            .iter()
            .map(|file| {
                let content_hash = file_hashes[&file.id];
                (file.id, FileState { content_hash })
            })
            .collect();

        // Phase 5: Run analysis on host files (same as regular pipeline)
        let host_files = self
            .database
            .files()
            .filter(|f| f.file_type == FileType::Host)
            .map(|f| self.database.get(&f.id))
            .collect::<Result<Vec<_>, _>>()?;

        if host_files.is_empty() {
            tracing::warn!("No host files found for analysis after compilation.");

            let (result, codebase, symbol_references) =
                self.reducer.reduce(merged_codex, symbol_references, Vec::new())?;
            return Ok((result, codebase, symbol_references, new_file_states));
        }

        let final_codebase = Arc::new(merged_codex);

        let results: Vec<I> = host_files
            .into_par_iter()
            .map_init(Bump::new, |arena, file| {
                let context = self.shared_context.clone();
                let codebase = Arc::clone(&final_codebase);
                let result = map_function(context, arena, file, codebase);

                arena.reset();

                result
            })
            .collect::<Result<Vec<I>, OrchestratorError>>()?;

        let final_codebase = Arc::unwrap_or_clone(final_codebase);

        let (result, codebase, symbol_references) = self.reducer.reduce(final_codebase, symbol_references, results)?;
        Ok((result, codebase, symbol_references, new_file_states))
    }
}
