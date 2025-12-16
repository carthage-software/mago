use std::sync::Arc;

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
use crate::progress::ProgressBarTheme;
use crate::progress::create_progress_bar;
use crate::progress::remove_progress_bar;

use std::fmt::Debug;

/// A trait that defines the final "reduce" step of a parallel computation.
///
/// In a `MapReduce` pattern, after the "map" phase generates results for each input,
/// the `Reducer` is responsible for aggregating all intermediate results into a
/// single, final output value.
pub trait Reducer<T, R>: Debug {
    /// Aggregates intermediate results into a final result.
    ///
    /// # Arguments
    ///
    /// * `codebase`: The fully compiled and populated `CodebaseMetadata`.
    /// * `symbol_references`: The final set of `SymbolReferences`.
    /// * `results`: A vector containing the intermediate results from each parallel task.
    ///
    /// # Returns
    ///
    /// Returns a tuple of `(result, codebase, symbol_references)` where the codebase
    /// and `symbol_references` are returned after being used by the reducer.
    fn reduce(
        &self,
        codebase: CodebaseMetadata,
        symbol_references: SymbolReferences,
        results: Vec<T>,
    ) -> Result<(R, CodebaseMetadata, SymbolReferences), OrchestratorError>;
}

/// A trait that defines the final "reduce" step for a stateless parallel computation.
pub trait StatelessReducer<I, R>: Debug {
    /// Aggregates intermediate results from the parallel "map" phase into a final result.
    fn reduce(&self, results: Vec<I>) -> Result<R, OrchestratorError>;
}

/// A callback type invoked after the scanning phase completes.
pub type PostScanCallback = Box<dyn FnOnce(&mut CodebaseMetadata, &SymbolReferences) + Send>;

/// An orchestrator for a multi-phase, data-parallel computation pipeline.
///
/// This struct implements a two-phase MapReduce-like pattern for static analysis:
/// 1.  **Phase 1 (Compile):** A parallel "map" scans every file to produce partial
///     metadata, followed by a "reduce" that merges it into a single `CodebaseMetadata`.
/// 2.  **Phase 2 (Analyze):** A parallel "map" runs a user-provided analysis function
///     on each host file, using the final codebase from Phase 1 as input.
/// 3.  **Phase 3 (Finalize):** The user-provided [`Reducer`] aggregates the results
///     from the analysis phase into a final output.
pub struct ParallelPipeline<T, I, R> {
    task_name: &'static str,
    database: Arc<ReadDatabase>,
    codebase: CodebaseMetadata,
    symbol_references: SymbolReferences,
    shared_context: T,
    reducer: Box<dyn Reducer<I, R> + Send + Sync>,
    should_use_progress_bar: bool,
    after_scanning: Option<PostScanCallback>,
}

impl<T, I, R> std::fmt::Debug for ParallelPipeline<T, I, R>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParallelPipeline")
            .field("task_name", &self.task_name)
            .field("database", &self.database)
            .field("codebase", &self.codebase)
            .field("symbol_references", &self.symbol_references)
            .field("shared_context", &self.shared_context)
            .field("reducer", &"<reducer>")
            .field("should_use_progress_bar", &self.should_use_progress_bar)
            .field("after_scanning", &self.after_scanning.is_some())
            .finish()
    }
}

/// An orchestrator for a simple, single-phase data-parallel computation.
///
/// This struct is designed for tasks like formatting that can process each file
/// in isolation without needing a shared, global view of the entire codebase.
#[derive(Debug)]
pub struct StatelessParallelPipeline<T, I, R> {
    task_name: &'static str,
    database: Arc<ReadDatabase>,
    shared_context: T,
    reducer: Box<dyn StatelessReducer<I, R> + Send + Sync>,
    should_use_progress_bar: bool,
}

impl<T, I, R> ParallelPipeline<T, I, R>
where
    T: Clone + Send + Sync + 'static,
    I: Send + 'static,
    R: Send + 'static,
{
    /// Creates a new `ParallelPipeline`.
    pub fn new(
        task_name: &'static str,
        database: ReadDatabase,
        codebase: CodebaseMetadata,
        symbol_references: SymbolReferences,
        shared_context: T,
        reducer: Box<dyn Reducer<I, R> + Send + Sync>,
        should_use_progress_bar: bool,
    ) -> Self {
        Self {
            task_name,
            database: Arc::new(database),
            codebase,
            symbol_references,
            shared_context,
            reducer,
            should_use_progress_bar,
            after_scanning: None,
        }
    }

    /// Sets a callback to be invoked after the scanning phase completes.
    ///
    /// This callback receives mutable access to the codebase metadata and immutable
    /// access to symbol references. It's useful for operations like incremental analysis
    /// that need to mark safe symbols before the analysis phase begins.
    pub fn with_after_scanning(
        mut self,
        callback: impl FnOnce(&mut CodebaseMetadata, &SymbolReferences) + Send + 'static,
    ) -> Self {
        self.after_scanning = Some(Box::new(callback));
        self
    }

    /// Executes the full pipeline with a given map function.
    ///
    /// # Arguments
    ///
    /// * `map_function`: The core logic to be applied in parallel to each `Host` file
    ///   during the analysis phase. It receives the shared context, file data, and the
    ///   fully populated codebase, and returns an intermediate result.
    ///
    /// # Returns
    ///
    /// Returns a tuple of `(result, codebase, symbol_references)` where:
    /// - `result`: The aggregated result from the reducer
    /// - `codebase`: The final codebase metadata after all processing
    /// - `symbol_references`: The final symbol references
    pub fn run<F>(self, map_function: F) -> Result<(R, CodebaseMetadata, SymbolReferences), OrchestratorError>
    where
        F: Fn(T, &Bump, Arc<File>, Arc<CodebaseMetadata>) -> Result<I, OrchestratorError> + Send + Sync + 'static,
    {
        let source_files = self.database.files().filter(|f| f.file_type != FileType::Builtin).collect::<Vec<_>>();
        if source_files.is_empty() {
            tracing::info!("No source files found for analysis.");

            let (result, codebase, symbol_references) =
                self.reducer.reduce(self.codebase, self.symbol_references, Vec::new())?;
            return Ok((result, codebase, symbol_references));
        }

        let compiling_bar = if self.should_use_progress_bar {
            Some(create_progress_bar(source_files.len(), "📚 Compiling", ProgressBarTheme::Blue))
        } else {
            None
        };

        let partial_codebases: Result<Vec<CodebaseMetadata>, OrchestratorError> = source_files
            .into_par_iter()
            .map_init(Bump::new, |arena, file| -> Result<CodebaseMetadata, OrchestratorError> {
                let (program, parse_issues) = parse_file(arena, &file);
                if parse_issues.is_some() {
                    tracing::warn!("Parsing issues in '{}'. Codebase analysis may be incomplete.", file.name);
                }

                let resolver = NameResolver::new(arena);
                let resolved_names = resolver.resolve(program);

                let file_signature = signature_builder::build_file_signature(&file, program, &resolved_names);

                let mut metadata = scan_program(arena, &file, program, &resolved_names);
                metadata.set_file_signature(file.id, file_signature);

                arena.reset();
                if let Some(compiling_bar) = &compiling_bar {
                    compiling_bar.inc(1);
                }

                Ok(metadata)
            })
            .collect();

        let mut merged_codex = self.codebase;
        for partial in partial_codebases? {
            merged_codex.extend(partial);
        }

        let mut symbol_references = self.symbol_references;
        populate_codebase(&mut merged_codex, &mut symbol_references, AtomSet::default(), HashSet::default());

        // Invoke after_scanning callback if provided (for incremental analysis)
        if let Some(callback) = self.after_scanning {
            callback(&mut merged_codex, &symbol_references);
        }

        if let Some(compiling_bar) = compiling_bar {
            remove_progress_bar(compiling_bar);
        }

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
            return Ok((result, codebase, symbol_references));
        }

        let final_codebase = Arc::new(merged_codex);

        let main_task_bar = if self.should_use_progress_bar {
            Some(create_progress_bar(host_files.len(), self.task_name, ProgressBarTheme::Green))
        } else {
            None
        };

        let results: Vec<I> = host_files
            .into_par_iter()
            .map_init(Bump::new, |arena, file| {
                let context = self.shared_context.clone();
                let codebase = Arc::clone(&final_codebase);
                let result = map_function(context, arena, file, codebase);

                arena.reset();
                if let Some(main_task_bar) = &main_task_bar {
                    main_task_bar.inc(1);
                }

                result
            })
            .collect::<Result<Vec<I>, OrchestratorError>>()?;

        if let Some(main_task_bar) = main_task_bar {
            remove_progress_bar(main_task_bar);
        }

        let final_codebase = Arc::unwrap_or_clone(final_codebase);

        let (result, codebase, symbol_references) = self.reducer.reduce(final_codebase, symbol_references, results)?;
        Ok((result, codebase, symbol_references))
    }
}

impl<T, I, R> StatelessParallelPipeline<T, I, R>
where
    T: Clone + Send + Sync + 'static,
    I: Send + 'static,
    R: Send + 'static,
{
    pub fn new(
        task_name: &'static str,
        database: ReadDatabase,
        shared_context: T,
        reducer: Box<dyn StatelessReducer<I, R> + Send + Sync>,
        should_use_progress_bar: bool,
    ) -> Self {
        Self { task_name, database: Arc::new(database), shared_context, reducer, should_use_progress_bar }
    }

    /// Executes the pipeline with a given map function on all `Host` files.
    pub fn run<F>(&self, map_function: F) -> Result<R, OrchestratorError>
    where
        F: Fn(T, &Bump, Arc<File>) -> Result<I, OrchestratorError> + Send + Sync,
    {
        let host_files = self
            .database
            .files()
            .filter(|f| f.file_type == FileType::Host)
            .map(|f| self.database.get(&f.id))
            .collect::<Result<Vec<_>, _>>()?;

        if host_files.is_empty() {
            return self.reducer.reduce(Vec::new());
        }

        let results = if self.should_use_progress_bar {
            let progress_bar = create_progress_bar(host_files.len(), self.task_name, ProgressBarTheme::Magenta);

            let results: Vec<I> = host_files
                .into_par_iter()
                .map_init(Bump::new, |arena, file| {
                    let context = self.shared_context.clone();
                    let result = map_function(context, arena, file)?;

                    arena.reset();
                    progress_bar.inc(1);

                    Ok(result)
                })
                .collect::<Result<Vec<I>, OrchestratorError>>()?;

            remove_progress_bar(progress_bar);

            results
        } else {
            host_files
                .into_par_iter()
                .map_init(Bump::new, |arena, file| {
                    let context = self.shared_context.clone();
                    let result = map_function(context, arena, file)?;

                    arena.reset();
                    Ok(result)
                })
                .collect::<Result<Vec<I>, OrchestratorError>>()?
        };

        self.reducer.reduce(results)
    }

    /// Executes the pipeline with a given map function on specific files by ID.
    ///
    /// This method processes only the files with the given IDs, rather than all
    /// `Host` files in the database. This is useful for operations like formatting
    /// only staged files in git pre-commit hooks.
    ///
    /// # Arguments
    ///
    /// * `file_ids` - Iterator of file IDs to process
    /// * `map_function` - The function to apply to each file
    pub fn run_on_files<F, Iter>(&self, file_ids: Iter, map_function: F) -> Result<R, OrchestratorError>
    where
        F: Fn(T, &Bump, Arc<File>) -> Result<I, OrchestratorError> + Send + Sync,
        Iter: IntoIterator<Item = FileId>,
    {
        let files: Vec<_> = file_ids.into_iter().filter_map(|id| self.database.get(&id).ok()).collect();

        if files.is_empty() {
            return self.reducer.reduce(Vec::new());
        }

        let results = if self.should_use_progress_bar {
            let progress_bar = create_progress_bar(files.len(), self.task_name, ProgressBarTheme::Magenta);

            let results: Vec<I> = files
                .into_par_iter()
                .map_init(Bump::new, |arena, file| {
                    let context = self.shared_context.clone();
                    let result = map_function(context, arena, file)?;

                    arena.reset();
                    progress_bar.inc(1);

                    Ok(result)
                })
                .collect::<Result<Vec<I>, OrchestratorError>>()?;

            remove_progress_bar(progress_bar);

            results
        } else {
            files
                .into_par_iter()
                .map_init(Bump::new, |arena, file| {
                    let context = self.shared_context.clone();
                    let result = map_function(context, arena, file)?;

                    arena.reset();
                    Ok(result)
                })
                .collect::<Result<Vec<I>, OrchestratorError>>()?
        };

        self.reducer.reduce(results)
    }
}
