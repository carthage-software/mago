#![allow(clippy::too_many_arguments)]

use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

use bumpalo::Bump;
use foldhash::HashSet;
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
use mago_syntax::parser::parse_file_with_settings;
use mago_syntax::settings::ParserSettings;

use crate::error::OrchestratorError;
use crate::progress::ProgressBarTheme;
use crate::progress::create_progress_bar;
use crate::progress::remove_progress_bar;
#[cfg(not(target_arch = "wasm32"))]
use crate::service::telemetry::SlowestFiles;
#[cfg(not(target_arch = "wasm32"))]
use crate::service::telemetry::measure;

// No-op `measure!` stub for wasm so the pipeline body compiles without
// pulling in the telemetry module. On wasm `trace_enabled` is always
// `false`, so the body just runs and `$out` is never actually read.
#[cfg(target_arch = "wasm32")]
macro_rules! measure {
    ($trace_enabled:expr, $out:expr, $body:expr) => {{
        let _ = $trace_enabled;
        let _ = &mut $out;
        $body
    }};
}

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
    ) -> Result<R, OrchestratorError>;
}

/// A trait that defines the final "reduce" step for a stateless parallel computation.
pub trait StatelessReducer<I, R>: Debug {
    /// Aggregates intermediate results from the parallel "map" phase into a final result.
    fn reduce(&self, results: Vec<I>) -> Result<R, OrchestratorError>;
}

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
    parser_settings: ParserSettings,
    reducer: Box<dyn Reducer<I, R> + Send + Sync>,
    should_use_progress_bar: bool,
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
            .field("parser_settings", &self.parser_settings)
            .field("reducer", &"<reducer>")
            .field("should_use_progress_bar", &self.should_use_progress_bar)
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
        parser_settings: ParserSettings,
        reducer: Box<dyn Reducer<I, R> + Send + Sync>,
        should_use_progress_bar: bool,
    ) -> Self {
        Self {
            task_name,
            database: Arc::new(database),
            codebase,
            symbol_references,
            shared_context,
            parser_settings,
            reducer,
            should_use_progress_bar,
        }
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
    pub fn run<F>(self, map_function: F) -> Result<R, OrchestratorError>
    where
        F: Fn(T, &Bump, Arc<File>, Arc<CodebaseMetadata>) -> Result<I, OrchestratorError> + Send + Sync + 'static,
    {
        #[cfg(not(target_arch = "wasm32"))]
        let trace_enabled = tracing::enabled!(tracing::Level::TRACE);
        #[cfg(target_arch = "wasm32")]
        let trace_enabled = false;

        #[cfg(not(target_arch = "wasm32"))]
        let pipeline_start = trace_enabled.then(Instant::now);
        #[cfg(not(target_arch = "wasm32"))]
        let slowest_files = Arc::new(SlowestFiles::new());

        let mut source_discover_duration = Duration::ZERO;
        let source_files: Vec<_> = measure!(
            trace_enabled,
            source_discover_duration,
            self.database.files().filter(|f| f.file_type != FileType::Builtin).collect()
        );

        if source_files.is_empty() {
            tracing::info!("No source files found for analysis.");
            return self.reducer.reduce(self.codebase, self.symbol_references, Vec::new());
        }

        let compiling_bar = if self.should_use_progress_bar {
            Some(create_progress_bar(source_files.len(), "📚 Compiling", ProgressBarTheme::Blue))
        } else {
            None
        };

        let parser_settings = self.parser_settings;
        #[cfg(not(target_arch = "wasm32"))]
        let source_count = source_files.len();

        let mut compile_parallel_duration = Duration::ZERO;
        let partial_codebases: Vec<CodebaseMetadata> = measure!(
            trace_enabled,
            compile_parallel_duration,
            source_files
                .into_par_iter()
                .map_init(Bump::new, |arena, file| -> Result<CodebaseMetadata, OrchestratorError> {
                    let program = parse_file_with_settings(arena, &file, parser_settings);
                    if program.has_errors() {
                        tracing::warn!(
                            "Encountered {} parsing errors in file '{}'. Codebase analysis may be incomplete.",
                            program.errors.len(),
                            file.name
                        );
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
                .collect::<Result<Vec<_>, _>>()?
        );

        let mut merged_codex = self.codebase;
        let mut merge_duration = Duration::ZERO;
        measure!(trace_enabled, merge_duration, {
            for partial in partial_codebases {
                merged_codex.extend(partial);
            }
        });

        let mut symbol_references = self.symbol_references;
        let mut populate_duration = Duration::ZERO;
        measure!(trace_enabled, populate_duration, {
            populate_codebase(&mut merged_codex, &mut symbol_references, AtomSet::default(), HashSet::default());
        });

        if let Some(compiling_bar) = compiling_bar {
            remove_progress_bar(&compiling_bar);
        }

        let mut host_discover_duration = Duration::ZERO;
        let host_files = measure!(
            trace_enabled,
            host_discover_duration,
            self.database
                .files()
                .filter(|f| f.file_type == FileType::Host)
                .map(|f| self.database.get(&f.id))
                .collect::<Result<Vec<_>, _>>()?
        );

        if host_files.is_empty() {
            tracing::warn!("No host files found for analysis after compilation.");
            return self.reducer.reduce(merged_codex, symbol_references, Vec::new());
        }
        #[cfg(not(target_arch = "wasm32"))]
        let host_count = host_files.len();

        let final_codebase = Arc::new(merged_codex);

        let main_task_bar = if self.should_use_progress_bar {
            Some(create_progress_bar(host_files.len(), self.task_name, ProgressBarTheme::Green))
        } else {
            None
        };

        #[cfg(not(target_arch = "wasm32"))]
        let slowest_files_for_closure = Arc::clone(&slowest_files);

        let mut analyze_parallel_duration = Duration::ZERO;
        let results: Vec<I> = measure!(
            trace_enabled,
            analyze_parallel_duration,
            host_files
                .into_par_iter()
                .map_init(Bump::new, |arena, file| {
                    let context = self.shared_context.clone();
                    let codebase = Arc::clone(&final_codebase);

                    #[cfg(not(target_arch = "wasm32"))]
                    let file_for_record = trace_enabled.then(|| Arc::clone(&file));
                    #[cfg(not(target_arch = "wasm32"))]
                    let file_start = trace_enabled.then(Instant::now);

                    let result = map_function(context, arena, file, codebase);

                    #[cfg(not(target_arch = "wasm32"))]
                    if let (Some(start), Some(recorded_file)) = (file_start, file_for_record) {
                        slowest_files_for_closure.record(start.elapsed(), recorded_file);
                    }

                    arena.reset();
                    if let Some(main_task_bar) = &main_task_bar {
                        main_task_bar.inc(1);
                    }

                    result
                })
                .collect::<Result<Vec<I>, OrchestratorError>>()?
        );

        if let Some(main_task_bar) = main_task_bar {
            remove_progress_bar(&main_task_bar);
        }

        let final_codebase = Arc::unwrap_or_clone(final_codebase);

        let mut reduce_duration = Duration::ZERO;
        let result =
            measure!(trace_enabled, reduce_duration, self.reducer.reduce(final_codebase, symbol_references, results));

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(start) = pipeline_start {
            let compile_per_file_us = compile_parallel_duration.as_micros() as f64 / source_count as f64;
            let analyze_per_file_us = analyze_parallel_duration.as_micros() as f64 / host_count as f64;

            tracing::trace!("Discovered {source_count} source files in {source_discover_duration:?}.");
            tracing::trace!(
                "Compiled {source_count} source files in parallel in {compile_parallel_duration:?} (average {compile_per_file_us:.1} µs per file)."
            );
            tracing::trace!("Merged partial codebases in {merge_duration:?}.");
            tracing::trace!("Populated codebase metadata in {populate_duration:?}.");
            tracing::trace!("Discovered {host_count} host files in {host_discover_duration:?}.");
            tracing::trace!(
                "Analyzed {host_count} host files in parallel in {analyze_parallel_duration:?} (average {analyze_per_file_us:.1} µs per file)."
            );
            tracing::trace!("Reduced analysis results in {reduce_duration:?}.");
            tracing::trace!("Pipeline finished in {:?}.", start.elapsed());

            slowest_files.emit_slowest(20, "the analysis phase");
        }

        result
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
        #[cfg(not(target_arch = "wasm32"))]
        let trace_enabled = tracing::enabled!(tracing::Level::TRACE);
        #[cfg(target_arch = "wasm32")]
        let trace_enabled = false;

        #[cfg(not(target_arch = "wasm32"))]
        let pipeline_start = trace_enabled.then(Instant::now);
        #[cfg(not(target_arch = "wasm32"))]
        let slowest_files = Arc::new(SlowestFiles::new());

        let mut host_discover_duration = Duration::ZERO;
        let host_files: Vec<Arc<File>> = measure!(
            trace_enabled,
            host_discover_duration,
            self.database
                .files()
                .filter(|f| f.file_type == FileType::Host)
                .map(|f| self.database.get(&f.id))
                .collect::<Result<Vec<_>, _>>()?
        );

        if host_files.is_empty() {
            return self.reducer.reduce(Vec::new());
        }

        #[cfg(not(target_arch = "wasm32"))]
        let host_count = host_files.len();

        let progress_bar = self
            .should_use_progress_bar
            .then(|| create_progress_bar(host_files.len(), self.task_name, ProgressBarTheme::Magenta));

        #[cfg(not(target_arch = "wasm32"))]
        let slowest_files_for_closure = Arc::clone(&slowest_files);

        let mut map_duration = Duration::ZERO;
        let results: Vec<I> = measure!(
            trace_enabled,
            map_duration,
            host_files
                .into_par_iter()
                .map_init(Bump::new, |arena, file| {
                    let context = self.shared_context.clone();

                    #[cfg(not(target_arch = "wasm32"))]
                    let file_for_record = trace_enabled.then(|| Arc::clone(&file));
                    #[cfg(not(target_arch = "wasm32"))]
                    let file_start = trace_enabled.then(Instant::now);

                    let result = map_function(context, arena, file)?;

                    #[cfg(not(target_arch = "wasm32"))]
                    if let (Some(start), Some(recorded_file)) = (file_start, file_for_record) {
                        slowest_files_for_closure.record(start.elapsed(), recorded_file);
                    }

                    arena.reset();
                    if let Some(bar) = &progress_bar {
                        bar.inc(1);
                    }

                    Ok(result)
                })
                .collect::<Result<Vec<I>, OrchestratorError>>()?
        );

        if let Some(bar) = progress_bar {
            remove_progress_bar(&bar);
        }

        let mut reduce_duration = Duration::ZERO;
        let reduced = measure!(trace_enabled, reduce_duration, self.reducer.reduce(results));

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(start) = pipeline_start {
            let per_file_us = map_duration.as_micros() as f64 / host_count as f64;

            tracing::trace!("Discovered {host_count} host files in {host_discover_duration:?}.");
            tracing::trace!(
                "Processed {host_count} files in parallel in {map_duration:?} (average {per_file_us:.1} µs per file)."
            );
            tracing::trace!("Reduced results in {reduce_duration:?}.");
            tracing::trace!("Pipeline finished in {:?}.", start.elapsed());

            let phase_label = format!("the {} phase", self.task_name);
            slowest_files.emit_slowest(20, &phase_label);
        }

        reduced
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
        #[cfg(not(target_arch = "wasm32"))]
        let trace_enabled = tracing::enabled!(tracing::Level::TRACE);
        #[cfg(target_arch = "wasm32")]
        let trace_enabled = false;

        #[cfg(not(target_arch = "wasm32"))]
        let pipeline_start = trace_enabled.then(Instant::now);
        #[cfg(not(target_arch = "wasm32"))]
        let slowest_files = Arc::new(SlowestFiles::new());

        let mut lookup_duration = Duration::ZERO;
        let files: Vec<Arc<File>> = measure!(
            trace_enabled,
            lookup_duration,
            file_ids.into_iter().filter_map(|id| self.database.get(&id).ok()).collect()
        );

        if files.is_empty() {
            return self.reducer.reduce(Vec::new());
        }

        #[cfg(not(target_arch = "wasm32"))]
        let file_count = files.len();

        let progress_bar = self
            .should_use_progress_bar
            .then(|| create_progress_bar(files.len(), self.task_name, ProgressBarTheme::Magenta));

        #[cfg(not(target_arch = "wasm32"))]
        let slowest_files_for_closure = Arc::clone(&slowest_files);

        let mut map_duration = Duration::ZERO;
        let results: Vec<I> = measure!(
            trace_enabled,
            map_duration,
            files
                .into_par_iter()
                .map_init(Bump::new, |arena, file| {
                    let context = self.shared_context.clone();

                    #[cfg(not(target_arch = "wasm32"))]
                    let file_for_record = trace_enabled.then(|| Arc::clone(&file));
                    #[cfg(not(target_arch = "wasm32"))]
                    let file_start = trace_enabled.then(Instant::now);

                    let result = map_function(context, arena, file)?;

                    #[cfg(not(target_arch = "wasm32"))]
                    if let (Some(start), Some(recorded_file)) = (file_start, file_for_record) {
                        slowest_files_for_closure.record(start.elapsed(), recorded_file);
                    }

                    arena.reset();
                    if let Some(bar) = &progress_bar {
                        bar.inc(1);
                    }

                    Ok(result)
                })
                .collect::<Result<Vec<I>, OrchestratorError>>()?
        );

        if let Some(bar) = progress_bar {
            remove_progress_bar(&bar);
        }

        let mut reduce_duration = Duration::ZERO;
        let reduced = measure!(trace_enabled, reduce_duration, self.reducer.reduce(results));

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(start) = pipeline_start {
            let per_file_us = map_duration.as_micros() as f64 / file_count as f64;

            tracing::trace!("Resolved {file_count} files by id in {lookup_duration:?}.");
            tracing::trace!(
                "Processed {file_count} files in parallel in {map_duration:?} (average {per_file_us:.1} µs per file)."
            );
            tracing::trace!("Reduced results in {reduce_duration:?}.");
            tracing::trace!("Pipeline finished in {:?}.", start.elapsed());

            let phase_label = format!("the {} phase", self.task_name);
            slowest_files.emit_slowest(20, &phase_label);
        }

        reduced
    }
}
