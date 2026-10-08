#![allow(clippy::too_many_arguments)]

use std::cmp::Reverse;
use std::fmt::Debug;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

use mago_allocator::LocalArena;
use mago_php_version::PHPVersion;
use rayon::prelude::*;

use mago_codex::metadata::CodebaseMetadata;
use mago_codex::populator::populate_codebase_parallel;
use mago_codex::reference::SymbolReferences;
use mago_codex::scanner::scan_program;
use mago_codex::signature_builder;
use mago_word::WordSet;

use mago_database::DatabaseReader;
use mago_database::ReadDatabase;
use mago_database::file::File;
use mago_database::file::FileId;
use mago_database::file::FileType;

use mago_names::ResolvedNames;
use mago_names::resolver::NameResolver;
use mago_syntax::cst::Program;
use mago_syntax::parser::parse_file_borrowed_with_settings;
use mago_syntax::settings::ParserSettings;

use crate::error::OrchestratorError;
use crate::progress::ProgressBarTheme;
use crate::progress::create_progress_bar;
use crate::progress::remove_progress_bar;
#[cfg(not(target_arch = "wasm32"))]
use crate::service::telemetry::HangWatcher;
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
    php_version: PHPVersion,
    reducer: Box<dyn Reducer<I, R> + Send + Sync>,
    should_use_progress_bar: bool,
    build_file_signatures: bool,
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
            .field("php_version", &self.php_version)
            .field("reducer", &"<reducer>")
            .field("should_use_progress_bar", &self.should_use_progress_bar)
            .field("build_file_signatures", &self.build_file_signatures)
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
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        task_name: &'static str,
        database: ReadDatabase,
        codebase: CodebaseMetadata,
        symbol_references: SymbolReferences,
        shared_context: T,
        parser_settings: ParserSettings,
        php_version: PHPVersion,
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
            php_version,
            reducer,
            should_use_progress_bar,
            build_file_signatures: true,
        }
    }

    pub(super) fn with_file_signatures(mut self, enabled: bool) -> Self {
        self.build_file_signatures = enabled;
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
    pub fn run<F, B, C, S>(self, capture: C, before_map: B, map_function: F) -> Result<R, OrchestratorError>
    where
        F: Fn(T, &LocalArena, Arc<File>, Arc<CodebaseMetadata>) -> Result<I, OrchestratorError> + Send + Sync + 'static,
        B: FnOnce(&mut CodebaseMetadata, &mut SymbolReferences, Vec<S>) -> Result<Option<I>, OrchestratorError>,
        C: Fn(&Arc<File>, &Program<'_>, &ResolvedNames<'_>) -> Result<Option<S>, OrchestratorError> + Send + Sync,
        S: Send,
    {
        #[cfg(not(target_arch = "wasm32"))]
        let trace_enabled = tracing::enabled!(tracing::Level::TRACE);
        #[cfg(target_arch = "wasm32")]
        let trace_enabled = false;

        #[cfg(not(target_arch = "wasm32"))]
        let pipeline_start = trace_enabled.then(Instant::now);
        #[cfg(not(target_arch = "wasm32"))]
        let slowest_files = trace_enabled.then(|| Arc::new(SlowestFiles::new()));

        let mut source_discover_duration = Duration::ZERO;
        let mut source_files: Vec<_> = measure!(
            trace_enabled,
            source_discover_duration,
            self.database.files().filter(|f| f.file_type != FileType::Builtin).enumerate().collect()
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
        let php_version = self.php_version;
        let build_file_signatures = self.build_file_signatures;
        #[cfg(not(target_arch = "wasm32"))]
        let source_count = source_files.len();

        // Start large files early, then restore database order before merging.
        if rayon::current_num_threads() > 1 {
            source_files.sort_unstable_by_key(|(index, file)| (Reverse(file.size), *index));
        }

        let mut compile_parallel_duration = Duration::ZERO;
        let mut compiled: Vec<(usize, (CodebaseMetadata, Option<S>))> = measure!(
            trace_enabled,
            compile_parallel_duration,
            map_with_priority_queue(
                &source_files,
                LocalArena::new,
                |arena, (index, file)| -> Result<_, OrchestratorError> {
                    let program = parse_file_borrowed_with_settings(arena, file, parser_settings);
                    if program.has_errors() {
                        tracing::warn!(
                            "Encountered {} parsing errors in file '{}'. Codebase analysis may be incomplete.",
                            program.errors.len(),
                            mago_bytes::BytesDisplay(&file.name)
                        );
                    }

                    let resolver = NameResolver::new(arena);
                    let resolved_names = resolver.resolve(program);

                    let file_signature = build_file_signatures
                        .then(|| signature_builder::build_file_signature(program, &resolved_names));

                    let mut metadata = scan_program(arena, file, program, &resolved_names, php_version);
                    if let Some(file_signature) = file_signature {
                        metadata.set_file_signature(file.id, file_signature);
                    }

                    if file.file_type.is_patch() {
                        metadata.convert_partial_to_patch();
                    }

                    let record = capture(file, program, &resolved_names)?;
                    arena.reset();
                    if let Some(compiling_bar) = &compiling_bar {
                        compiling_bar.inc(1);
                    }

                    Ok((*index, (metadata, record)))
                }
            )?
        );
        compiled.sort_unstable_by_key(|(index, _)| *index);
        drop(source_files);

        let mut merged_codex = self.codebase;
        let mut captures = Vec::new();
        let mut safe_symbols = std::mem::take(&mut merged_codex.safe_symbols);
        let mut safe_symbol_members = std::mem::take(&mut merged_codex.safe_symbol_members);
        let mut replaced_classes = (!safe_symbol_members.is_empty()).then(WordSet::default);
        let mut merge_duration = Duration::ZERO;
        measure!(trace_enabled, merge_duration, {
            let partials = compiled
                .into_iter()
                .map(|(_, (partial, record))| {
                    captures.extend(record);
                    for name in partial.class_likes.keys().chain(partial.patch_class_likes.keys()) {
                        safe_symbols.remove(name);
                        if let Some(replaced_classes) = &mut replaced_classes {
                            replaced_classes.insert(*name);
                        }
                    }

                    for (scope, member) in partial.function_likes.keys().chain(partial.patch_function_likes.keys()) {
                        if member.is_empty() {
                            safe_symbols.remove(scope);
                        } else {
                            safe_symbol_members.remove(&(*scope, *member));
                        }
                    }

                    for name in partial.constants.keys().chain(partial.patch_constants.keys()) {
                        safe_symbols.remove(name);
                    }

                    partial
                })
                .collect();

            merged_codex.extend_many_parallel(partials);
            if let Some(replaced_classes) = replaced_classes {
                safe_symbol_members.retain(|(scope, _)| !replaced_classes.contains(scope));
            }

            merged_codex.apply_patches_pass();
        });

        let mut symbol_references = self.symbol_references;
        let mut populate_duration = Duration::ZERO;
        measure!(trace_enabled, populate_duration, {
            populate_codebase_parallel(&mut merged_codex, &mut symbol_references, safe_symbols, safe_symbol_members);
        });

        if let Some(compiling_bar) = compiling_bar {
            remove_progress_bar(&compiling_bar);
        }

        let mut host_discover_duration = Duration::ZERO;
        let mut host_files = measure!(
            trace_enabled,
            host_discover_duration,
            self.database.files().filter(|f| f.file_type == FileType::Host).enumerate().collect::<Vec<_>>()
        );

        let before_map_result = before_map(&mut merged_codex, &mut symbol_references, captures)?;

        if host_files.is_empty() {
            tracing::warn!("No host files found for analysis after compilation.");
            return self.reducer.reduce(merged_codex, symbol_references, before_map_result.into_iter().collect());
        }

        // Start large files early, then restore database order before reduction.
        host_files.sort_unstable_by_key(|(index, file)| (Reverse(file.size), *index));

        #[cfg(not(target_arch = "wasm32"))]
        let host_count = host_files.len();
        let final_codebase = Arc::new(merged_codex);
        let main_task_bar = if self.should_use_progress_bar {
            Some(create_progress_bar(host_files.len(), self.task_name, ProgressBarTheme::Green))
        } else {
            None
        };

        #[cfg(not(target_arch = "wasm32"))]
        let slowest_files_for_closure = slowest_files.as_ref().map(Arc::clone);

        #[cfg(not(target_arch = "wasm32"))]
        let hang_watcher = trace_enabled.then(|| HangWatcher::spawn(rayon::current_num_threads()));

        let mut analyze_parallel_duration = Duration::ZERO;
        let mut results: Vec<(usize, I)> = measure!(
            trace_enabled,
            analyze_parallel_duration,
            map_with_priority_queue(&host_files, LocalArena::new, |arena, (index, file)| {
                let file = Arc::clone(file);
                let context = self.shared_context.clone();
                let codebase = Arc::clone(&final_codebase);

                #[cfg(not(target_arch = "wasm32"))]
                let file_for_record = trace_enabled.then(|| Arc::clone(&file));
                #[cfg(not(target_arch = "wasm32"))]
                let file_start = trace_enabled.then(Instant::now);

                #[cfg(not(target_arch = "wasm32"))]
                let _hang_guard = hang_watcher.as_ref().map(|w| w.track(Arc::clone(&file)));

                let result = map_function(context, arena, file, codebase);

                #[cfg(not(target_arch = "wasm32"))]
                if let (Some(sink), Some(start), Some(recorded_file)) =
                    (slowest_files_for_closure.as_ref(), file_start, file_for_record)
                {
                    sink.record(start.elapsed(), recorded_file);
                }

                arena.reset();
                if let Some(main_task_bar) = &main_task_bar {
                    main_task_bar.inc(1);
                }

                result.map(|result| (*index, result))
            })?
        );

        results.sort_unstable_by_key(|(index, _)| *index);
        let mut results = results.into_iter().map(|(_, result)| result).collect::<Vec<_>>();

        if let Some(result) = before_map_result {
            results.insert(0, result);
        }

        #[cfg(not(target_arch = "wasm32"))]
        drop(hang_watcher);

        if let Some(main_task_bar) = main_task_bar {
            remove_progress_bar(&main_task_bar);
        }

        let final_codebase = Arc::unwrap_or_clone(final_codebase);

        let mut reduce_duration = Duration::ZERO;
        let result =
            measure!(trace_enabled, reduce_duration, self.reducer.reduce(final_codebase, symbol_references, results));

        #[cfg(not(target_arch = "wasm32"))]
        #[allow(clippy::float_arithmetic)]
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

            if let Some(slowest) = slowest_files.as_ref() {
                slowest.emit_slowest(20, "the analysis phase");
            }
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
        F: Fn(T, &LocalArena, Arc<File>) -> Result<I, OrchestratorError> + Send + Sync,
    {
        #[cfg(not(target_arch = "wasm32"))]
        let trace_enabled = tracing::enabled!(tracing::Level::TRACE);
        #[cfg(target_arch = "wasm32")]
        let trace_enabled = false;

        #[cfg(not(target_arch = "wasm32"))]
        let pipeline_start = trace_enabled.then(Instant::now);
        #[cfg(not(target_arch = "wasm32"))]
        let slowest_files = trace_enabled.then(|| Arc::new(SlowestFiles::new()));

        let mut host_discover_duration = Duration::ZERO;
        let host_files: Vec<Arc<File>> = measure!(
            trace_enabled,
            host_discover_duration,
            self.database.files().filter(|f| f.file_type == FileType::Host).collect()
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
        let slowest_files_for_closure = slowest_files.as_ref().map(Arc::clone);

        let mut map_duration = Duration::ZERO;
        let results: Vec<I> = measure!(
            trace_enabled,
            map_duration,
            host_files
                .into_par_iter()
                .map_init(LocalArena::new, |arena, file| {
                    let context = self.shared_context.clone();

                    #[cfg(not(target_arch = "wasm32"))]
                    let file_for_record = trace_enabled.then(|| Arc::clone(&file));
                    #[cfg(not(target_arch = "wasm32"))]
                    let file_start = trace_enabled.then(Instant::now);

                    let result = map_function(context, arena, file)?;

                    #[cfg(not(target_arch = "wasm32"))]
                    if let (Some(sink), Some(start), Some(recorded_file)) =
                        (slowest_files_for_closure.as_ref(), file_start, file_for_record)
                    {
                        sink.record(start.elapsed(), recorded_file);
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
        #[allow(clippy::float_arithmetic)]
        if let Some(start) = pipeline_start {
            let per_file_us = map_duration.as_micros() as f64 / host_count as f64;

            tracing::trace!("Discovered {host_count} host files in {host_discover_duration:?}.");
            tracing::trace!(
                "Processed {host_count} files in parallel in {map_duration:?} (average {per_file_us:.1} µs per file)."
            );
            tracing::trace!("Reduced results in {reduce_duration:?}.");
            tracing::trace!("Pipeline finished in {:?}.", start.elapsed());

            if let Some(slowest) = slowest_files.as_ref() {
                let phase_label = format!("the {} phase", self.task_name);
                slowest.emit_slowest(20, &phase_label);
            }
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
        F: Fn(T, &LocalArena, Arc<File>) -> Result<I, OrchestratorError> + Send + Sync,
        Iter: IntoIterator<Item = FileId>,
    {
        #[cfg(not(target_arch = "wasm32"))]
        let trace_enabled = tracing::enabled!(tracing::Level::TRACE);
        #[cfg(target_arch = "wasm32")]
        let trace_enabled = false;

        #[cfg(not(target_arch = "wasm32"))]
        let pipeline_start = trace_enabled.then(Instant::now);
        #[cfg(not(target_arch = "wasm32"))]
        let slowest_files = trace_enabled.then(|| Arc::new(SlowestFiles::new()));

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
        let slowest_files_for_closure = slowest_files.as_ref().map(Arc::clone);

        let mut map_duration = Duration::ZERO;
        let results: Vec<I> = measure!(
            trace_enabled,
            map_duration,
            files
                .into_par_iter()
                .map_init(LocalArena::new, |arena, file| {
                    let context = self.shared_context.clone();

                    #[cfg(not(target_arch = "wasm32"))]
                    let file_for_record = trace_enabled.then(|| Arc::clone(&file));
                    #[cfg(not(target_arch = "wasm32"))]
                    let file_start = trace_enabled.then(Instant::now);

                    let result = map_function(context, arena, file)?;

                    #[cfg(not(target_arch = "wasm32"))]
                    if let (Some(sink), Some(start), Some(recorded_file)) =
                        (slowest_files_for_closure.as_ref(), file_start, file_for_record)
                    {
                        sink.record(start.elapsed(), recorded_file);
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
        #[allow(clippy::float_arithmetic)]
        if let Some(start) = pipeline_start {
            let per_file_us = map_duration.as_micros() as f64 / file_count as f64;

            tracing::trace!("Resolved {file_count} files by id in {lookup_duration:?}.");
            tracing::trace!(
                "Processed {file_count} files in parallel in {map_duration:?} (average {per_file_us:.1} µs per file)."
            );
            tracing::trace!("Reduced results in {reduce_duration:?}.");
            tracing::trace!("Pipeline finished in {:?}.", start.elapsed());

            if let Some(slowest) = slowest_files.as_ref() {
                let phase_label = format!("the {} phase", self.task_name);
                slowest.emit_slowest(20, &phase_label);
            }
        }

        reduced
    }
}

/// Claims inputs in slice order, with one reusable state per worker.
fn map_with_priority_queue<T, S, I, E, F, C>(items: &[T], initialize: C, map: F) -> Result<Vec<I>, E>
where
    T: Sync,
    I: Send,
    E: Send,
    F: Fn(&mut S, &T) -> Result<I, E> + Sync,
    C: Fn() -> S + Sync,
{
    let next = AtomicUsize::new(0);
    let cancelled = AtomicBool::new(false);
    let worker_count = items.len().min(rayon::current_num_threads());
    let batches = (0..worker_count)
        .into_par_iter()
        .with_max_len(1)
        .map(|_| {
            let mut state = initialize();
            let mut results = Vec::new();
            while !cancelled.load(Ordering::Relaxed) {
                let index = next.fetch_add(1, Ordering::Relaxed);
                let Some(item) = items.get(index) else {
                    break;
                };

                match map(&mut state, item) {
                    Ok(result) => results.push(result),
                    Err(error) => {
                        cancelled.store(true, Ordering::Relaxed);
                        return Err(error);
                    }
                }
            }
            Ok(results)
        })
        .collect::<Result<Vec<_>, E>>()?;

    Ok(batches.into_iter().flatten().collect())
}

#[cfg(test)]
#[allow(clippy::panic_in_result_fn)]
mod tests {
    use std::borrow::Cow;
    use std::path::Path;
    use std::sync::Arc;
    use std::sync::Barrier;
    use std::sync::Mutex;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;

    use mago_codex::metadata::CodebaseMetadata;
    use mago_codex::reference::SymbolReferences;
    use mago_database::Database;
    use mago_database::DatabaseConfiguration;
    use mago_database::DatabaseReader;
    use mago_database::file::File;
    use mago_database::file::FileId;
    use mago_database::file::FileType;
    use mago_php_version::PHPVersion;
    use mago_syntax::settings::ParserSettings;

    use super::ParallelPipeline;
    use super::Reducer;
    use super::map_with_priority_queue;
    use crate::error::OrchestratorError;

    #[derive(Debug)]
    struct FileIds;

    impl Reducer<FileId, Vec<FileId>> for FileIds {
        fn reduce(
            &self,
            _codebase: CodebaseMetadata,
            _symbol_references: SymbolReferences,
            results: Vec<FileId>,
        ) -> Result<Vec<FileId>, OrchestratorError> {
            Ok(results)
        }
    }

    #[test]
    fn analysis_preserves_host_coverage_and_result_order() -> Result<(), Box<dyn std::error::Error>> {
        let configuration = DatabaseConfiguration::new(Path::new("."), vec![], vec![], vec![], vec![]);
        let mut database = Database::new(configuration);
        for index in 0..32 {
            database.add(File::ephemeral(
                Cow::Owned(format!("host-{index}.php").into_bytes()),
                Cow::Owned(format!("<?php /* {} */", "x".repeat(index * 100)).into_bytes()),
            ));
        }
        database.add(File::new(Cow::Borrowed(b"vendor.php"), FileType::Vendored, None, Cow::Borrowed(b"<?php")));

        let snapshot = database.read_only();
        let expected_captures = snapshot.files().map(|file| file.id).collect::<Vec<_>>();
        let before_map_id = FileId::new(b"before-map");
        let mut expected = vec![before_map_id];
        expected.extend(snapshot.files().filter(|file| file.file_type == FileType::Host).map(|file| file.id));

        for threads in [1, 4] {
            let pipeline = ParallelPipeline::new(
                "test",
                // database,
                database.read_only(),
                CodebaseMetadata::new(),
                SymbolReferences::new(),
                (),
                ParserSettings::default(),
                PHPVersion::new(8, 4, 0),
                Box::new(FileIds),
                false,
            );

            let results = rayon::ThreadPoolBuilder::new().num_threads(threads).build()?.install(|| {
                pipeline.run(
                    |file, _, _| Ok(Some(file.id)),
                    |codebase, _, captures| {
                        assert_eq!(captures, expected_captures, "Compilation keeps database order");
                        assert_eq!(codebase.file_signatures.len(), 33, "Public pipelines keep all file signatures");
                        Ok(Some(before_map_id))
                    },
                    |(), _, file, _| Ok(file.id),
                )
            })?;

            assert_eq!(results, expected);
        }

        Ok(())
    }

    #[test]
    fn workers_claim_largest_inputs_first_and_process_each_once() -> Result<(), Box<dyn std::error::Error>> {
        let items = (0..32).rev().collect::<Vec<_>>();
        let first_claims = Mutex::new(Vec::new());
        let first_batch = Barrier::new(4);
        let visits = [const { AtomicUsize::new(0) }; 32];
        let initialized = AtomicUsize::new(0);
        let mut results = rayon::ThreadPoolBuilder::new().num_threads(4).build()?.install(|| {
            map_with_priority_queue(
                &items,
                || {
                    initialized.fetch_add(1, Ordering::Relaxed);
                    true
                },
                |first, value| {
                    if *first {
                        first_claims.lock().map_err(|_| "first-claim lock poisoned")?.push(*value);
                        first_batch.wait();
                        *first = false;
                    }
                    visits[*value].fetch_add(1, Ordering::Relaxed);
                    Ok::<_, &'static str>(*value)
                },
            )
        })?;

        let mut first_claims = first_claims.into_inner().map_err(|_| "first-claim lock poisoned")?;
        first_claims.sort_unstable();
        assert_eq!(first_claims, [28, 29, 30, 31]);
        assert_eq!(initialized.load(Ordering::Relaxed), 4);
        assert!(visits.iter().all(|count| count.load(Ordering::Relaxed) == 1));
        results.sort_unstable();
        assert_eq!(results, (0..32).collect::<Vec<_>>());
        Ok(())
    }

    #[test]
    fn empty_queue_does_not_initialize_workers() -> Result<(), std::convert::Infallible> {
        let initialized = AtomicUsize::new(0);
        let result = map_with_priority_queue(
            &[] as &[usize],
            || {
                initialized.fetch_add(1, Ordering::Relaxed);
            },
            |(), value| Ok::<_, std::convert::Infallible>(*value),
        );
        assert!(result?.is_empty());
        assert_eq!(initialized.load(Ordering::Relaxed), 0);
        Ok(())
    }

    #[test]
    fn queue_errors_drop_completed_results() -> Result<(), Box<dyn std::error::Error>> {
        struct ResultValue(Arc<AtomicUsize>);

        impl Drop for ResultValue {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::Relaxed);
            }
        }

        for threads in [1, 4] {
            let created = AtomicUsize::new(0);
            let dropped = Arc::new(AtomicUsize::new(0));
            let items = (0..64).collect::<Vec<_>>();
            let result = rayon::ThreadPoolBuilder::new().num_threads(threads).build()?.install(|| {
                map_with_priority_queue(
                    &items,
                    || (),
                    |(), value| {
                        if *value == 5 {
                            return Err("stop");
                        }
                        created.fetch_add(1, Ordering::Relaxed);
                        Ok(ResultValue(Arc::clone(&dropped)))
                    },
                )
            });
            assert!(matches!(result, Err("stop")));
            assert_eq!(created.load(Ordering::Relaxed), dropped.load(Ordering::Relaxed));
            if threads == 1 {
                assert_eq!(created.load(Ordering::Relaxed), 5);
            }
        }
        Ok(())
    }
}
