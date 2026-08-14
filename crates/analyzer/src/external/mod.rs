//! Worker-backed analyzer plugins.

use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::thread::JoinHandle;
use std::time::Duration;
use std::time::Instant;

use mago_codex::metadata::CodebaseMetadata;
use mago_codex::reference::SymbolReferences;
use mago_codex::ttype::union::TUnion;
use mago_database::file::File;
use mago_database::file::FileId;
use mago_database::file::FileType;
use mago_extension::Frame;
use mago_extension::WorkerError;
use mago_extension::WorkerPool;
use mago_extension::WorkerRequestHandler;
use mago_php_version::PHPVersion;
use mago_reporting::IssueCollection;
use mago_word::WordMap;
use mago_word::ascii_lowercase_word;
use mago_word::concat_word;
use mago_word::starts_with_ignore_case;

use crate::artifacts::AnalysisArtifacts;
use crate::invocation::Invocation;
use crate::plugin::available_plugins;

pub use error::ExternalAnalyzerError;
pub use lifecycle::AFTER_FILE_ANALYSIS_BATCH_SIZE;
pub use lifecycle::FileAnalysisSnapshot;
use protocol::Registration;

mod error;
mod lifecycle;
mod metadata;
pub mod protocol;

const SLOW_PROVIDER_THRESHOLD: Duration = Duration::from_millis(5);
const SLOW_LIFECYCLE_THRESHOLD: Duration = Duration::from_millis(5);
static NEXT_ANALYSIS_GENERATION: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Default)]
pub struct BeforeAnalysisResult {
    pub issues: IssueCollection,
    pub references: SymbolReferences,
}

/// Immutable request context shared by every external hook in one analysis run.
///
/// A new session is created for every frozen codebase generation. Keeping the
/// generation on the request, instead of on the worker pool, makes PHP-side
/// metadata caches safe when a pool is reused by watch mode or by concurrent
/// analysis services.
#[derive(Debug)]
pub struct ExternalAnalysisSession {
    generation: u64,
    sources: foldhash::HashMap<FileId, ExternalSource>,
}

#[derive(Debug)]
struct ExternalSource {
    name: Arc<[u8]>,
    size: u32,
}

impl ExternalAnalysisSession {
    #[must_use]
    pub fn from_files(files: impl IntoIterator<Item = Arc<File>>) -> Self {
        let generation = NEXT_ANALYSIS_GENERATION.fetch_add(1, Ordering::Relaxed);
        let sources = files
            .into_iter()
            .map(|file| (file.id, ExternalSource { name: Arc::from(file.name.as_ref()), size: file.size }))
            .collect();

        Self { generation, sources }
    }

    #[inline]
    #[must_use]
    pub(crate) const fn generation(&self) -> u64 {
        self.generation
    }

    #[inline]
    #[must_use]
    pub(crate) fn source_name(&self, file_id: FileId) -> Option<&[u8]> {
        self.sources.get(&file_id).map(|source| source.name.as_ref())
    }

    fn source(&self, name: &[u8]) -> Option<(FileId, u32)> {
        self.sources
            .iter()
            .find_map(|(file_id, source)| (source.name.as_ref() == name).then_some((*file_id, source.size)))
    }
}

#[derive(Debug, Default)]
struct ExternalAnalyzerTelemetry {
    function_lookups: AtomicU64,
    method_lookups: AtomicU64,
    backend_checks: AtomicU64,
    candidate_providers: AtomicU64,
    matched_providers: AtomicU64,
    unmatched_lookups: AtomicU64,
    requests: AtomicU64,
    provided_types: AtomicU64,
    declined_requests: AtomicU64,
    errors: AtomicU64,
    snapshotted_types: AtomicU64,
    arguments: AtomicU64,
    typed_arguments: AtomicU64,
    request_bytes: AtomicU64,
    response_bytes: AtomicU64,
    nested_requests: AtomicU64,
    nested_errors: AtomicU64,
    nested_request_bytes: AtomicU64,
    nested_response_bytes: AtomicU64,
    comparisons: AtomicU64,
    metadata_queries: AtomicU64,
    analysis_queries: AtomicU64,
    symbol_reference_queries: AtomicU64,
    before_analysis_requests: AtomicU64,
    after_file_analysis_requests: AtomicU64,
    after_file_analysis_files: AtomicU64,
    after_analysis_requests: AtomicU64,
    lifecycle_plugins: AtomicU64,
    lifecycle_issues: AtomicU64,
    lifecycle_errors: AtomicU64,
    lifecycle_request_bytes: AtomicU64,
    lifecycle_response_bytes: AtomicU64,
    matching_ns: AtomicU64,
    encode_ns: AtomicU64,
    type_snapshot_ns: AtomicU64,
    ipc_ns: AtomicU64,
    comparison_ns: AtomicU64,
    metadata_query_ns: AtomicU64,
    analysis_query_ns: AtomicU64,
    symbol_reference_query_ns: AtomicU64,
    lifecycle_encode_ns: AtomicU64,
    lifecycle_ipc_ns: AtomicU64,
    lifecycle_decode_ns: AtomicU64,
    lifecycle_ns: AtomicU64,
    nested_ns: AtomicU64,
    decode_ns: AtomicU64,
    lookup_ns: AtomicU64,
}

impl ExternalAnalyzerTelemetry {
    fn record_nested_request(
        &self,
        request_bytes: usize,
        elapsed: Duration,
        result: &Result<(protocol::NestedRequestKind, Vec<u8>), ExternalAnalyzerError>,
    ) {
        self.nested_requests.fetch_add(1, Ordering::Relaxed);
        self.nested_request_bytes.fetch_add(request_bytes as u64, Ordering::Relaxed);
        self.nested_ns.fetch_add(duration_nanos(elapsed), Ordering::Relaxed);

        let Ok((kind, response)) = result else {
            self.nested_errors.fetch_add(1, Ordering::Relaxed);
            self.errors.fetch_add(1, Ordering::Relaxed);
            return;
        };

        self.nested_response_bytes.fetch_add(response.len() as u64, Ordering::Relaxed);
        match kind {
            protocol::NestedRequestKind::TypeComparison => {
                self.comparisons.fetch_add(1, Ordering::Relaxed);
                self.comparison_ns.fetch_add(duration_nanos(elapsed), Ordering::Relaxed);
            }
            protocol::NestedRequestKind::CodebaseQuery => {
                self.metadata_queries.fetch_add(1, Ordering::Relaxed);
                self.metadata_query_ns.fetch_add(duration_nanos(elapsed), Ordering::Relaxed);
            }
            protocol::NestedRequestKind::AnalysisQuery => {
                self.analysis_queries.fetch_add(1, Ordering::Relaxed);
                self.analysis_query_ns.fetch_add(duration_nanos(elapsed), Ordering::Relaxed);
            }
            protocol::NestedRequestKind::SymbolReferenceQuery => {
                self.symbol_reference_queries.fetch_add(1, Ordering::Relaxed);
                self.symbol_reference_query_ns.fetch_add(duration_nanos(elapsed), Ordering::Relaxed);
            }
        }
    }
}

#[derive(Clone, Copy)]
enum LifecyclePhase {
    Before,
    AfterFile,
    AfterFileBatch,
    After,
}

impl LifecyclePhase {
    const fn request_kind(self) -> u16 {
        match self {
            Self::Before => lifecycle::BEFORE_ANALYSIS_REQUEST,
            Self::AfterFile => lifecycle::AFTER_FILE_ANALYSIS_REQUEST,
            Self::AfterFileBatch => lifecycle::AFTER_FILE_ANALYSIS_BATCH_REQUEST,
            Self::After => lifecycle::AFTER_ANALYSIS_REQUEST,
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::Before => "before-analysis",
            Self::AfterFile | Self::AfterFileBatch => "after-file-analysis",
            Self::After => "after-analysis",
        }
    }
}

struct LookupTrace<'telemetry> {
    telemetry: &'telemetry ExternalAnalyzerTelemetry,
    started_at: Instant,
}

impl Drop for LookupTrace<'_> {
    fn drop(&mut self) {
        self.telemetry.lookup_ns.fetch_add(duration_nanos(self.started_at.elapsed()), Ordering::Relaxed);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalExtension {
    pub identifier: String,
    pub name: String,
    pub version: String,
    pub plugins: Vec<ExternalPlugin>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalPlugin {
    pub index: u16,
    pub extension: String,
    pub identifier: String,
    pub name: String,
    pub description: String,
    pub aliases: Vec<String>,
    pub default_enabled: bool,
    pub initialization: bool,
    pub before_analysis: bool,
    pub after_file_analysis: bool,
    pub after_analysis: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExternalStub {
    name: Vec<u8>,
    contents: Vec<u8>,
}

impl ExternalStub {
    fn new(extension: &str, plugin: &str, filename: &[u8], contents: Vec<u8>) -> Self {
        let mut name = Vec::with_capacity(18 + extension.len() + plugin.len() + filename.len());
        name.extend_from_slice(b"@mago-extension/");
        append_path_component(&mut name, extension.as_bytes());
        name.push(b'/');
        append_path_component(&mut name, plugin.as_bytes());
        name.push(b'/');
        name.extend_from_slice(filename);
        Self { name, contents }
    }

    fn to_file(&self) -> File {
        File::new(Cow::Owned(self.name.clone()), FileType::External, None, Cow::Owned(self.contents.clone()))
    }
}

impl ExternalPlugin {
    fn matches(&self, name: &str) -> bool {
        self.identifier.eq_ignore_ascii_case(name) || self.aliases.iter().any(|alias| alias.eq_ignore_ascii_case(name))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum FunctionTarget {
    Exact(Vec<u8>),
    Prefix(Vec<u8>),
    Namespace(Vec<u8>),
}

impl FunctionTarget {
    fn matches(&self, name: &[u8]) -> bool {
        match self {
            Self::Exact(target) => name.eq_ignore_ascii_case(target),
            Self::Prefix(prefix) | Self::Namespace(prefix) => starts_with_ignore_case(name, prefix),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MethodTarget {
    class: Vec<u8>,
    method: Vec<u8>,
}

impl MethodTarget {
    fn matches(&self, class: &[u8], method: &[u8]) -> bool {
        pattern_matches(&self.class, class) && pattern_matches(&self.method, method)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FunctionProvider {
    plugin: String,
    index: u16,
    targets: Vec<FunctionTarget>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MethodProvider {
    plugin: String,
    index: u16,
    targets: Vec<MethodTarget>,
}

enum ProviderIndices {
    One(u16),
    Multiple(Vec<u16>),
}

impl ProviderIndices {
    #[inline]
    fn from_iter(mut indices: impl Iterator<Item = u16>) -> Option<Self> {
        let first = indices.next()?;
        let Some(second) = indices.next() else {
            return Some(Self::One(first));
        };

        let mut matched = Vec::with_capacity(4);
        matched.extend([first, second]);
        matched.extend(indices);
        Some(Self::Multiple(matched))
    }

    #[inline]
    fn as_slice(&self) -> &[u16] {
        match self {
            Self::One(index) => std::slice::from_ref(index),
            Self::Multiple(indices) => indices,
        }
    }

    fn insert_sorted(&mut self, index: u16) {
        match self {
            Self::One(existing) if *existing == index => {}
            Self::One(existing) => {
                let (first, second) = if *existing < index { (*existing, index) } else { (index, *existing) };
                *self = Self::Multiple(vec![first, second]);
            }
            Self::Multiple(indices) => {
                if let Err(position) = indices.binary_search(&index) {
                    indices.insert(position, index);
                }
            }
        }
    }

    fn from_exact_and_wildcards(exact: &[u16], wildcards: impl IntoIterator<Item = u16>) -> Option<ProviderIndices> {
        let mut indices = Self::from_iter(exact.iter().copied());
        for index in wildcards {
            match &mut indices {
                Some(indices) => indices.insert_sorted(index),
                None => indices = Some(Self::One(index)),
            }
        }

        indices
    }
}

/// Request transport used by worker-backed analyzer providers.
pub trait AnalyzerTransport: std::fmt::Debug + Send + Sync {
    /// Sends initialization data to every worker process.
    ///
    /// # Errors
    ///
    /// Returns an error if a worker cannot process the request.
    fn broadcast(&self, payload: &[u8]) -> Result<Vec<Vec<u8>>, WorkerError>;

    /// Sends one provider request to an available worker.
    ///
    /// # Errors
    ///
    /// Returns an error if no worker can process the request.
    fn request(&self, payload: Vec<u8>) -> Result<Vec<u8>, WorkerError>;

    /// Sends one provider request while servicing nested analyzer queries.
    ///
    /// # Errors
    ///
    /// Returns an error if the worker or nested query handler fails.
    fn request_with_handler<H>(&self, payload: Vec<u8>, handler: &mut H) -> Result<Vec<u8>, WorkerError>
    where
        H: WorkerRequestHandler;
}

impl AnalyzerTransport for WorkerPool {
    fn broadcast(&self, payload: &[u8]) -> Result<Vec<Vec<u8>>, WorkerError> {
        Self::broadcast(self, payload)
    }

    fn request(&self, payload: Vec<u8>) -> Result<Vec<u8>, WorkerError> {
        Self::request(self, payload)
    }

    fn request_with_handler<H>(&self, payload: Vec<u8>, handler: &mut H) -> Result<Vec<u8>, WorkerError>
    where
        H: WorkerRequestHandler,
    {
        Self::request_with_handler(self, payload, handler)
    }
}

#[derive(Debug)]
struct Backend<T> {
    transport: Arc<T>,
    registration: Registration,
    function_exact: WordMap<Vec<u16>>,
    function_wildcard: Vec<(u16, Vec<FunctionTarget>)>,
    method_exact: WordMap<Vec<u16>>,
    method_wildcard: Vec<(u16, Vec<MethodTarget>)>,
}

impl<T> Backend<T> {
    fn new(transport: Arc<T>, registration: Registration) -> Self {
        let mut function_exact = WordMap::default();
        let mut function_wildcard = Vec::new();
        for provider in &registration.function_providers {
            let mut wildcard_targets = Vec::new();
            for target in &provider.targets {
                match target {
                    FunctionTarget::Exact(name) => {
                        let indices = function_exact.entry(ascii_lowercase_word(name)).or_insert_with(Vec::new);
                        if indices.last() != Some(&provider.index) {
                            indices.push(provider.index);
                        }
                    }
                    FunctionTarget::Prefix(_) | FunctionTarget::Namespace(_) => {
                        wildcard_targets.push(target.clone());
                    }
                }
            }
            if !wildcard_targets.is_empty() {
                function_wildcard.push((provider.index, wildcard_targets));
            }
        }

        let mut method_exact = WordMap::default();
        let mut method_wildcard = Vec::new();
        for provider in &registration.method_providers {
            let mut wildcard_targets = Vec::new();
            for target in &provider.targets {
                if target.class.contains(&b'*') || target.method.contains(&b'*') {
                    wildcard_targets.push(target.clone());
                } else {
                    let class = ascii_lowercase_word(&target.class);
                    let method = ascii_lowercase_word(&target.method);
                    let indices = method_exact.entry(concat_word!(class, b"::", method)).or_insert_with(Vec::new);
                    if indices.last() != Some(&provider.index) {
                        indices.push(provider.index);
                    }
                }
            }
            if !wildcard_targets.is_empty() {
                method_wildcard.push((provider.index, wildcard_targets));
            }
        }

        Self { transport, registration, function_exact, function_wildcard, method_exact, method_wildcard }
    }

    fn matching_function_providers(&self, function: &[u8]) -> (Option<ProviderIndices>, usize) {
        let function = ascii_lowercase_word(function);
        let exact = self.function_exact.get(&function).map_or(&[][..], Vec::as_slice);
        let candidates = exact.len() + self.function_wildcard.len();
        let wildcards = self
            .function_wildcard
            .iter()
            .filter(|(_, targets)| targets.iter().any(|target| target.matches(function.as_bytes())))
            .map(|(index, _)| *index);

        (ProviderIndices::from_exact_and_wildcards(exact, wildcards), candidates)
    }

    fn matching_method_providers(&self, class: &[u8], method: &[u8]) -> (Option<ProviderIndices>, usize) {
        let key = concat_word!(ascii_lowercase_word(class), b"::", ascii_lowercase_word(method));
        let exact = self.method_exact.get(&key).map_or(&[][..], Vec::as_slice);
        let candidates = exact.len() + self.method_wildcard.len();
        let wildcards = self
            .method_wildcard
            .iter()
            .filter(|(_, targets)| targets.iter().any(|target| target.matches(class, method)))
            .map(|(index, _)| *index);

        (ProviderIndices::from_exact_and_wildcards(exact, wildcards), candidates)
    }
}

#[derive(Debug)]
pub struct ExternalAnalyzer<T = WorkerPool> {
    backends: Box<[Backend<T>]>,
    extensions: Box<[ExternalExtension]>,
    plugins: Box<[ExternalPlugin]>,
    initialization_stubs: Box<[ExternalStub]>,
    trace_enabled: bool,
    telemetry: ExternalAnalyzerTelemetry,
    started_at: Option<Instant>,
}

/// An analyzer initialized concurrently with the codebase pipeline.
#[derive(Debug)]
pub struct ExternalAnalyzerHandle {
    analyzer: OnceLock<Result<ExternalAnalyzer, String>>,
    initializer: Mutex<Option<JoinHandle<Result<ExternalAnalyzer, ExternalAnalyzerError>>>>,
    trace_enabled: bool,
    started_at: Option<Instant>,
    prepare_calls: AtomicU64,
    initialization_wait_ns: AtomicU64,
}

impl ExternalAnalyzerHandle {
    /// Wraps an analyzer that has already completed initialization.
    #[must_use]
    pub fn ready(analyzer: ExternalAnalyzer) -> Self {
        let trace_enabled = tracing::enabled!(tracing::Level::TRACE);
        let cell = OnceLock::new();
        let _result = cell.set(Ok(analyzer));
        tracing::trace!("Created ready external analyzer handle.");
        Self {
            analyzer: cell,
            initializer: Mutex::new(None),
            trace_enabled,
            started_at: trace_enabled.then(Instant::now),
            prepare_calls: AtomicU64::new(0),
            initialization_wait_ns: AtomicU64::new(0),
        }
    }

    /// Wraps an analyzer initialization thread without waiting for it.
    #[must_use]
    pub fn pending(initializer: JoinHandle<Result<ExternalAnalyzer, ExternalAnalyzerError>>) -> Self {
        let trace_enabled = tracing::enabled!(tracing::Level::TRACE);
        tracing::trace!("Created pending external analyzer handle.");
        Self {
            analyzer: OnceLock::new(),
            initializer: Mutex::new(Some(initializer)),
            trace_enabled,
            started_at: trace_enabled.then(Instant::now),
            prepare_calls: AtomicU64::new(0),
            initialization_wait_ns: AtomicU64::new(0),
        }
    }

    pub(crate) fn prepare(&self) -> Result<(), String> {
        if self.trace_enabled {
            self.prepare_calls.fetch_add(1, Ordering::Relaxed);
        }
        self.get().map(|_| ())
    }

    pub(crate) fn initialization_files(&self) -> Result<Vec<File>, String> {
        Ok(self.get()?.initialization_stubs.iter().map(ExternalStub::to_file).collect())
    }

    pub(crate) fn get_function_return_type(
        &self,
        function: &[u8],
        invocation: &Invocation<'_, '_, '_>,
        artifacts: &AnalysisArtifacts,
        source_file: &File,
        codebase: &CodebaseMetadata,
        session: &ExternalAnalysisSession,
    ) -> Result<Option<TUnion>, String> {
        self.get()?
            .get_function_return_type(function, invocation, artifacts, source_file, codebase, session)
            .map_err(|error| error.to_string())
    }

    pub(crate) fn get_method_return_type(
        &self,
        class: &[u8],
        method: &[u8],
        invocation: &Invocation<'_, '_, '_>,
        artifacts: &AnalysisArtifacts,
        source_file: &File,
        codebase: &CodebaseMetadata,
        session: &ExternalAnalysisSession,
    ) -> Result<Option<TUnion>, String> {
        self.get()?
            .get_method_return_type(class, method, invocation, artifacts, source_file, codebase, session)
            .map_err(|error| error.to_string())
    }

    pub(crate) fn has_after_file_analysis_hooks(&self) -> Result<bool, String> {
        Ok(self.get()?.backends.iter().any(|backend| !backend.registration.after_file_analysis_plugins.is_empty()))
    }

    pub(crate) fn has_function_return_type_providers(&self) -> Result<bool, String> {
        Ok(self.get()?.backends.iter().any(|backend| !backend.registration.function_providers.is_empty()))
    }

    pub(crate) fn has_method_return_type_providers(&self) -> Result<bool, String> {
        Ok(self.get()?.backends.iter().any(|backend| !backend.registration.method_providers.is_empty()))
    }

    pub(crate) fn has_after_analysis_hooks(&self) -> Result<bool, String> {
        Ok(self.get()?.backends.iter().any(|backend| !backend.registration.after_analysis_plugins.is_empty()))
    }

    pub(crate) fn run_before_analysis_hooks(
        &self,
        codebase: &CodebaseMetadata,
        session: &ExternalAnalysisSession,
    ) -> Result<BeforeAnalysisResult, String> {
        self.get()?.run_before_analysis_hooks(codebase, session).map_err(|error| error.to_string())
    }

    pub(crate) fn run_after_file_analysis_hooks(
        &self,
        file: &File,
        artifacts: &AnalysisArtifacts,
        codebase: &CodebaseMetadata,
        session: &ExternalAnalysisSession,
    ) -> Result<IssueCollection, String> {
        self.get()?.run_after_file_analysis_hooks(file, artifacts, codebase, session).map_err(|error| error.to_string())
    }

    pub(crate) fn run_after_file_analysis_batch_hooks(
        &self,
        files: &[Arc<FileAnalysisSnapshot>],
        codebase: &CodebaseMetadata,
        session: &ExternalAnalysisSession,
    ) -> Result<IssueCollection, String> {
        self.get()?.run_after_file_analysis_batch_hooks(files, codebase, session).map_err(|error| error.to_string())
    }

    pub(crate) fn run_after_analysis_hooks(
        &self,
        result: &crate::analysis_result::AnalysisResult,
        files: &[Arc<FileAnalysisSnapshot>],
        codebase: &CodebaseMetadata,
        session: &ExternalAnalysisSession,
    ) -> Result<IssueCollection, String> {
        self.get()?.run_after_analysis_hooks(result, files, codebase, session).map_err(|error| error.to_string())
    }

    fn get(&self) -> Result<&ExternalAnalyzer, String> {
        self.analyzer
            .get_or_init(|| {
                let wait_start = self.trace_enabled.then(Instant::now);
                tracing::trace!("Waiting for external analyzer initialization thread.");
                let initializer = self
                    .initializer
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .take()
                    .ok_or_else(|| "external analyzer initialization thread is unavailable".to_string())?;
                let result = initializer
                    .join()
                    .map_err(|_| "external analyzer initialization thread panicked".to_string())?
                    .map_err(|error| error.to_string());

                if let Some(start) = wait_start {
                    self.initialization_wait_ns.fetch_add(duration_nanos(start.elapsed()), Ordering::Relaxed);
                    tracing::trace!(
                        elapsed = ?start.elapsed(),
                        success = result.is_ok(),
                        "External analyzer initialization thread joined."
                    );
                }

                result
            })
            .as_ref()
            .map_err(Clone::clone)
    }
}

impl Drop for ExternalAnalyzerHandle {
    fn drop(&mut self) {
        if self.analyzer.get().is_none() {
            let initializer = self.initializer.get_mut().unwrap_or_else(std::sync::PoisonError::into_inner).take();
            if let Some(initializer) = initializer {
                tracing::trace!("Joining unused external analyzer initialization thread during shutdown.");
                let _result = initializer.join();
            }
        }

        if self.trace_enabled {
            tracing::trace!(
                initialized = self.analyzer.get().is_some(),
                prepare_calls = self.prepare_calls.load(Ordering::Relaxed),
                initialization_wait = ?Duration::from_nanos(self.initialization_wait_ns.load(Ordering::Relaxed)),
                lifetime = ?self.started_at.map(|start| start.elapsed()).unwrap_or_default(),
                "External analyzer handle dropped."
            );
        }
    }
}

impl ExternalAnalyzer<WorkerPool> {
    /// Discovers and validates the analyzer plugins exposed by worker pools.
    ///
    /// # Errors
    ///
    /// Returns an error when a worker fails, sends malformed metadata, disagrees
    /// with another process in its pool, or advertises duplicate identifiers.
    pub fn initialize(
        pools: impl IntoIterator<Item = Arc<WorkerPool>>,
        php_version: PHPVersion,
        enabled_plugins: &[String],
        disable_defaults: bool,
    ) -> Result<Self, ExternalAnalyzerError> {
        let analyzer = Self::initialize_transports(pools, php_version, enabled_plugins, disable_defaults)?;
        for backend in &analyzer.backends {
            if backend.registration.has_worker_reducer {
                backend.transport.enable_worker_reduction();
            }
        }

        Ok(analyzer)
    }
}

impl<T> ExternalAnalyzer<T> {
    #[must_use]
    pub fn extensions(&self) -> &[ExternalExtension] {
        &self.extensions
    }

    #[must_use]
    pub fn plugins(&self) -> &[ExternalPlugin] {
        &self.plugins
    }

    #[must_use]
    pub fn initialization_files(&self) -> Vec<File> {
        self.initialization_stubs.iter().map(ExternalStub::to_file).collect()
    }
}

impl<T> ExternalAnalyzer<T>
where
    T: AnalyzerTransport,
{
    fn dispatch_lifecycle_request<H>(
        &self,
        backend: &Backend<T>,
        phase: LifecyclePhase,
        plugins: &[u16],
        logical_callbacks: usize,
        request: Vec<u8>,
        handler: &mut H,
        session: &ExternalAnalysisSession,
        default_file: Option<&File>,
        codebase: &CodebaseMetadata,
        started_at: Option<Instant>,
    ) -> Result<lifecycle::LifecycleEffects, ExternalAnalyzerError>
    where
        H: WorkerRequestHandler,
    {
        if self.trace_enabled {
            match phase {
                LifecyclePhase::Before => {
                    self.telemetry.before_analysis_requests.fetch_add(1, Ordering::Relaxed);
                }
                LifecyclePhase::AfterFile | LifecyclePhase::AfterFileBatch => {
                    self.telemetry.after_file_analysis_requests.fetch_add(1, Ordering::Relaxed);
                }
                LifecyclePhase::After => {
                    self.telemetry.after_analysis_requests.fetch_add(1, Ordering::Relaxed);
                }
            }

            self.telemetry.lifecycle_plugins.fetch_add(logical_callbacks as u64, Ordering::Relaxed);
            self.telemetry.lifecycle_request_bytes.fetch_add(request.len() as u64, Ordering::Relaxed);
        }

        let ipc_start = self.trace_enabled.then(Instant::now);
        let response = backend.transport.request_with_handler(request, handler).inspect_err(|_| {
            if self.trace_enabled {
                self.telemetry.lifecycle_errors.fetch_add(1, Ordering::Relaxed);
                self.telemetry.errors.fetch_add(1, Ordering::Relaxed);
            }
        })?;

        if let Some(start) = ipc_start {
            self.telemetry.lifecycle_ipc_ns.fetch_add(duration_nanos(start.elapsed()), Ordering::Relaxed);
            self.telemetry.lifecycle_response_bytes.fetch_add(response.len() as u64, Ordering::Relaxed);
        }

        let decode_start = self.trace_enabled.then(Instant::now);
        let issues = lifecycle::decode_lifecycle_response(
            &response,
            phase.request_kind(),
            plugins,
            &backend.registration.plugins,
            session,
            default_file,
            codebase,
        )
        .inspect_err(|_| {
            if self.trace_enabled {
                self.telemetry.lifecycle_errors.fetch_add(1, Ordering::Relaxed);
                self.telemetry.errors.fetch_add(1, Ordering::Relaxed);
            }
        })?;

        if let Some(start) = decode_start {
            self.telemetry.lifecycle_decode_ns.fetch_add(duration_nanos(start.elapsed()), Ordering::Relaxed);
            self.telemetry.lifecycle_issues.fetch_add(issues.issues.len() as u64, Ordering::Relaxed);
        }

        if let Some(start) = started_at {
            let elapsed = start.elapsed();
            self.telemetry.lifecycle_ns.fetch_add(duration_nanos(elapsed), Ordering::Relaxed);
            if elapsed >= SLOW_LIFECYCLE_THRESHOLD {
                let file = default_file
                    .map_or_else(|| "<project>".into(), |file| String::from_utf8_lossy(&file.name).into_owned());
                tracing::trace!(
                    phase = phase.name(),
                    plugins = plugins.len(),
                    file,
                    response_bytes = response.len(),
                    issues = issues.issues.len(),
                    elapsed = ?elapsed,
                    "Slow external analyzer lifecycle request completed."
                );
            }
        }

        Ok(issues)
    }

    fn run_before_analysis_hooks(
        &self,
        codebase: &CodebaseMetadata,
        session: &ExternalAnalysisSession,
    ) -> Result<BeforeAnalysisResult, ExternalAnalyzerError> {
        let mut result = BeforeAnalysisResult::default();
        for backend in &self.backends {
            let plugins = &backend.registration.before_analysis_plugins;
            if plugins.is_empty() {
                continue;
            }

            let lifecycle_start = self.trace_enabled.then(Instant::now);
            let encode_start = self.trace_enabled.then(Instant::now);
            let request =
                lifecycle::encode_before_analysis_request(session.generation(), plugins).inspect_err(|_| {
                    if self.trace_enabled {
                        self.telemetry.lifecycle_errors.fetch_add(1, Ordering::Relaxed);
                        self.telemetry.errors.fetch_add(1, Ordering::Relaxed);
                    }
                })?;

            if let Some(start) = encode_start {
                self.telemetry.lifecycle_encode_ns.fetch_add(duration_nanos(start.elapsed()), Ordering::Relaxed);
            }

            let nested_telemetry = &self.telemetry;
            let trace_enabled = self.trace_enabled;
            let mut handler = |frame: &Frame| {
                let nested_start = trace_enabled.then(Instant::now);
                let result = protocol::handle_nested_request(&frame.payload, codebase, session, |_| None);
                if let Some(start) = nested_start {
                    nested_telemetry.record_nested_request(frame.payload.len(), start.elapsed(), &result);
                }

                result.map(|(_, response)| response).map_err(|error| error.to_string().into_bytes())
            };

            let effects = self.dispatch_lifecycle_request(
                backend,
                LifecyclePhase::Before,
                plugins,
                plugins.len(),
                request,
                &mut handler,
                session,
                None,
                codebase,
                lifecycle_start,
            )?;
            result.issues.extend(effects.issues);
            result.references.extend(effects.references);
        }

        Ok(result)
    }

    fn run_after_file_analysis_hooks(
        &self,
        file: &File,
        artifacts: &AnalysisArtifacts,
        codebase: &CodebaseMetadata,
        session: &ExternalAnalysisSession,
    ) -> Result<IssueCollection, ExternalAnalyzerError> {
        let mut issues = IssueCollection::new();
        let store = lifecycle::AnalysisStore::File { file, artifacts };
        for backend in &self.backends {
            let plugins = &backend.registration.after_file_analysis_plugins;
            if plugins.is_empty() {
                continue;
            }

            let lifecycle_start = self.trace_enabled.then(Instant::now);
            let encode_start = self.trace_enabled.then(Instant::now);
            let request = lifecycle::encode_after_file_analysis_request(session.generation(), plugins, file, artifacts)
                .inspect_err(|_| {
                    if self.trace_enabled {
                        self.telemetry.lifecycle_errors.fetch_add(1, Ordering::Relaxed);
                        self.telemetry.errors.fetch_add(1, Ordering::Relaxed);
                    }
                })?;
            if let Some(start) = encode_start {
                self.telemetry.lifecycle_encode_ns.fetch_add(duration_nanos(start.elapsed()), Ordering::Relaxed);
            }

            let nested_telemetry = &self.telemetry;
            let trace_enabled = self.trace_enabled;
            let mut handler = |frame: &Frame| {
                let nested_start = trace_enabled.then(Instant::now);
                let result =
                    if protocol::message_kind(&frame.payload).map_err(|error| error.to_string().into_bytes())? == 8 {
                        lifecycle::handle_analysis_query(&frame.payload, session, &store)
                            .map(|response| (protocol::NestedRequestKind::AnalysisQuery, response))
                    } else {
                        protocol::handle_nested_request(&frame.payload, codebase, session, |_| None)
                    };
                if let Some(start) = nested_start {
                    nested_telemetry.record_nested_request(frame.payload.len(), start.elapsed(), &result);
                }

                result.map(|(_, response)| response).map_err(|error| error.to_string().into_bytes())
            };

            let effects = self.dispatch_lifecycle_request(
                backend,
                LifecyclePhase::AfterFile,
                plugins,
                plugins.len(),
                request,
                &mut handler,
                session,
                Some(file),
                codebase,
                lifecycle_start,
            )?;
            issues.extend(effects.issues);
        }

        Ok(issues)
    }

    fn run_after_file_analysis_batch_hooks(
        &self,
        files: &[Arc<FileAnalysisSnapshot>],
        codebase: &CodebaseMetadata,
        session: &ExternalAnalysisSession,
    ) -> Result<IssueCollection, ExternalAnalyzerError> {
        if files.is_empty() {
            return Ok(IssueCollection::new());
        }

        let mut issues = IssueCollection::new();
        let store = lifecycle::AnalysisStore::Project(files);
        for backend in &self.backends {
            let plugins = &backend.registration.after_file_analysis_plugins;
            if plugins.is_empty() {
                continue;
            }

            let lifecycle_start = self.trace_enabled.then(Instant::now);
            let encode_start = self.trace_enabled.then(Instant::now);
            let request = lifecycle::encode_after_file_analysis_batch_request(session.generation(), plugins, files)
                .inspect_err(|_| {
                    if self.trace_enabled {
                        self.telemetry.lifecycle_errors.fetch_add(1, Ordering::Relaxed);
                        self.telemetry.errors.fetch_add(1, Ordering::Relaxed);
                    }
                })?;
            if let Some(start) = encode_start {
                self.telemetry.lifecycle_encode_ns.fetch_add(duration_nanos(start.elapsed()), Ordering::Relaxed);
                self.telemetry.after_file_analysis_files.fetch_add(files.len() as u64, Ordering::Relaxed);
            }

            let nested_telemetry = &self.telemetry;
            let trace_enabled = self.trace_enabled;
            let mut handler = |frame: &Frame| {
                let nested_start = trace_enabled.then(Instant::now);
                let result =
                    if protocol::message_kind(&frame.payload).map_err(|error| error.to_string().into_bytes())? == 8 {
                        lifecycle::handle_analysis_query(&frame.payload, session, &store)
                            .map(|response| (protocol::NestedRequestKind::AnalysisQuery, response))
                    } else {
                        protocol::handle_nested_request(&frame.payload, codebase, session, |_| None)
                    };
                if let Some(start) = nested_start {
                    nested_telemetry.record_nested_request(frame.payload.len(), start.elapsed(), &result);
                }

                result.map(|(_, response)| response).map_err(|error| error.to_string().into_bytes())
            };

            let effects = self.dispatch_lifecycle_request(
                backend,
                LifecyclePhase::AfterFileBatch,
                plugins,
                plugins.len().saturating_mul(files.len()),
                request,
                &mut handler,
                session,
                None,
                codebase,
                lifecycle_start,
            )?;
            issues.extend(effects.issues);
        }

        Ok(issues)
    }

    fn run_after_analysis_hooks(
        &self,
        analysis_result: &crate::analysis_result::AnalysisResult,
        files: &[Arc<FileAnalysisSnapshot>],
        codebase: &CodebaseMetadata,
        session: &ExternalAnalysisSession,
    ) -> Result<IssueCollection, ExternalAnalyzerError> {
        let mut issues = IssueCollection::new();
        let store = lifecycle::AnalysisStore::Project(files);
        let reference_store = lifecycle::SymbolReferenceStore::new(&analysis_result.symbol_references);
        for backend in &self.backends {
            let plugins = &backend.registration.after_analysis_plugins;
            if plugins.is_empty() {
                continue;
            }

            let lifecycle_start = self.trace_enabled.then(Instant::now);
            let encode_start = self.trace_enabled.then(Instant::now);
            let request =
                lifecycle::encode_after_analysis_request(session.generation(), plugins, analysis_result, files)
                    .inspect_err(|_| {
                        if self.trace_enabled {
                            self.telemetry.lifecycle_errors.fetch_add(1, Ordering::Relaxed);
                            self.telemetry.errors.fetch_add(1, Ordering::Relaxed);
                        }
                    })?;
            if let Some(start) = encode_start {
                self.telemetry.lifecycle_encode_ns.fetch_add(duration_nanos(start.elapsed()), Ordering::Relaxed);
            }

            let nested_telemetry = &self.telemetry;
            let trace_enabled = self.trace_enabled;
            let mut handler = |frame: &Frame| {
                let nested_start = trace_enabled.then(Instant::now);
                let kind = protocol::message_kind(&frame.payload).map_err(|error| error.to_string().into_bytes())?;
                let result = if kind == 8 {
                    lifecycle::handle_analysis_query(&frame.payload, session, &store)
                        .map(|response| (protocol::NestedRequestKind::AnalysisQuery, response))
                } else if lifecycle::is_symbol_reference_query(&frame.payload)
                    .map_err(|error| error.to_string().into_bytes())?
                {
                    lifecycle::handle_symbol_reference_query(&frame.payload, session, codebase, &reference_store)
                        .map(|response| (protocol::NestedRequestKind::SymbolReferenceQuery, response))
                } else {
                    protocol::handle_nested_request(&frame.payload, codebase, session, |_| None)
                };
                if let Some(start) = nested_start {
                    nested_telemetry.record_nested_request(frame.payload.len(), start.elapsed(), &result);
                }

                result.map(|(_, response)| response).map_err(|error| error.to_string().into_bytes())
            };

            let effects = self.dispatch_lifecycle_request(
                backend,
                LifecyclePhase::After,
                plugins,
                plugins.len(),
                request,
                &mut handler,
                session,
                None,
                codebase,
                lifecycle_start,
            )?;
            issues.extend(effects.issues);
        }

        Ok(issues)
    }

    pub(crate) fn get_function_return_type(
        &self,
        function: &[u8],
        invocation: &Invocation<'_, '_, '_>,
        artifacts: &AnalysisArtifacts,
        source_file: &File,
        codebase: &CodebaseMetadata,
        session: &ExternalAnalysisSession,
    ) -> Result<Option<TUnion>, ExternalAnalyzerError> {
        let _lookup_trace =
            self.trace_enabled.then(|| LookupTrace { telemetry: &self.telemetry, started_at: Instant::now() });
        if self.trace_enabled {
            self.telemetry.function_lookups.fetch_add(1, Ordering::Relaxed);
        }

        let mut dispatched = false;
        for backend in &self.backends {
            let matching_start = self.trace_enabled.then(Instant::now);
            let (indices, candidates) = backend.matching_function_providers(function);
            if self.trace_enabled {
                self.telemetry.backend_checks.fetch_add(1, Ordering::Relaxed);
                self.telemetry.candidate_providers.fetch_add(candidates as u64, Ordering::Relaxed);
            }

            if let Some(start) = matching_start {
                self.telemetry.matching_ns.fetch_add(duration_nanos(start.elapsed()), Ordering::Relaxed);
            }

            let Some(indices) = indices else {
                continue;
            };
            dispatched = true;
            let provider_start = self.trace_enabled.then(Instant::now);
            if self.trace_enabled {
                self.telemetry.requests.fetch_add(1, Ordering::Relaxed);
                self.telemetry.matched_providers.fetch_add(indices.as_slice().len() as u64, Ordering::Relaxed);
            }

            let encode_start = self.trace_enabled.then(Instant::now);
            let request = protocol::encode_function_return_type_request(
                indices.as_slice(),
                function,
                invocation,
                artifacts,
                source_file,
                session.generation(),
                self.trace_enabled,
            )
            .inspect_err(|_| {
                if self.trace_enabled {
                    self.telemetry.errors.fetch_add(1, Ordering::Relaxed);
                }
            })?;

            if let Some(start) = encode_start {
                self.telemetry.encode_ns.fetch_add(duration_nanos(start.elapsed()), Ordering::Relaxed);
                self.telemetry.snapshotted_types.fetch_add(request.types.len() as u64, Ordering::Relaxed);
                self.telemetry.arguments.fetch_add(request.arguments as u64, Ordering::Relaxed);
                self.telemetry.typed_arguments.fetch_add(request.typed_arguments as u64, Ordering::Relaxed);
                self.telemetry
                    .type_snapshot_ns
                    .fetch_add(duration_nanos(request.type_snapshot_duration), Ordering::Relaxed);
                self.telemetry.request_bytes.fetch_add(request.payload.len() as u64, Ordering::Relaxed);
            }

            let nested_telemetry = &self.telemetry;
            let trace_enabled = self.trace_enabled;
            let mut handler = |frame: &Frame| {
                let nested_start = trace_enabled.then(Instant::now);

                let result = protocol::handle_nested_request(&frame.payload, codebase, session, |handle| {
                    request.types.get(handle).copied()
                });
                if let Some(start) = nested_start {
                    nested_telemetry.record_nested_request(frame.payload.len(), start.elapsed(), &result);
                }

                result.map(|(_, response)| response).map_err(|error| error.to_string().into_bytes())
            };

            let ipc_start = self.trace_enabled.then(Instant::now);
            let response = backend.transport.request_with_handler(request.payload, &mut handler).inspect_err(|_| {
                if self.trace_enabled {
                    self.telemetry.errors.fetch_add(1, Ordering::Relaxed);
                }
            })?;

            if let Some(start) = ipc_start {
                self.telemetry.ipc_ns.fetch_add(duration_nanos(start.elapsed()), Ordering::Relaxed);
                self.telemetry.response_bytes.fetch_add(response.len() as u64, Ordering::Relaxed);
            }

            let decode_start = self.trace_enabled.then(Instant::now);
            let result = protocol::decode_return_type_response(&response, |handle| request.types.get(handle).copied())
                .inspect_err(|_| {
                    if self.trace_enabled {
                        self.telemetry.errors.fetch_add(1, Ordering::Relaxed);
                    }
                })?;

            if let Some(start) = decode_start {
                self.telemetry.decode_ns.fetch_add(duration_nanos(start.elapsed()), Ordering::Relaxed);
            }

            if let Some(start) = provider_start
                && start.elapsed() >= SLOW_PROVIDER_THRESHOLD
            {
                tracing::trace!(
                    function = %mago_bytes::BytesDisplay(function),
                    providers = indices.as_slice().len(),
                    arguments = request.arguments,
                    typed_arguments = request.typed_arguments,
                    argument_types = request.types.len(),
                    response_bytes = response.len(),
                    elapsed = ?start.elapsed(),
                    provided = result.is_some(),
                    "Slow external function return-type provider request completed."
                );
            }

            if let Some(result) = result {
                if self.trace_enabled {
                    self.telemetry.provided_types.fetch_add(1, Ordering::Relaxed);
                }
                return Ok(Some(result));
            }

            if self.trace_enabled {
                self.telemetry.declined_requests.fetch_add(1, Ordering::Relaxed);
            }
        }

        if self.trace_enabled && !dispatched {
            self.telemetry.unmatched_lookups.fetch_add(1, Ordering::Relaxed);
        }

        Ok(None)
    }

    pub(crate) fn get_method_return_type(
        &self,
        class: &[u8],
        method: &[u8],
        invocation: &Invocation<'_, '_, '_>,
        artifacts: &AnalysisArtifacts,
        source_file: &File,
        codebase: &CodebaseMetadata,
        session: &ExternalAnalysisSession,
    ) -> Result<Option<TUnion>, ExternalAnalyzerError> {
        let _lookup_trace =
            self.trace_enabled.then(|| LookupTrace { telemetry: &self.telemetry, started_at: Instant::now() });
        if self.trace_enabled {
            self.telemetry.method_lookups.fetch_add(1, Ordering::Relaxed);
        }

        let mut dispatched = false;
        for backend in &self.backends {
            let matching_start = self.trace_enabled.then(Instant::now);
            let (indices, candidates) = backend.matching_method_providers(class, method);
            if self.trace_enabled {
                self.telemetry.backend_checks.fetch_add(1, Ordering::Relaxed);
                self.telemetry.candidate_providers.fetch_add(candidates as u64, Ordering::Relaxed);
            }

            if let Some(start) = matching_start {
                self.telemetry.matching_ns.fetch_add(duration_nanos(start.elapsed()), Ordering::Relaxed);
            }

            let Some(indices) = indices else {
                continue;
            };

            dispatched = true;
            let provider_start = self.trace_enabled.then(Instant::now);
            if self.trace_enabled {
                self.telemetry.requests.fetch_add(1, Ordering::Relaxed);
                self.telemetry.matched_providers.fetch_add(indices.as_slice().len() as u64, Ordering::Relaxed);
            }

            let encode_start = self.trace_enabled.then(Instant::now);
            let request = protocol::encode_method_return_type_request(
                indices.as_slice(),
                class,
                method,
                invocation,
                artifacts,
                source_file,
                session.generation(),
                self.trace_enabled,
            )
            .inspect_err(|_| {
                if self.trace_enabled {
                    self.telemetry.errors.fetch_add(1, Ordering::Relaxed);
                }
            })?;

            if let Some(start) = encode_start {
                self.telemetry.encode_ns.fetch_add(duration_nanos(start.elapsed()), Ordering::Relaxed);
                self.telemetry.snapshotted_types.fetch_add(request.types.len() as u64, Ordering::Relaxed);
                self.telemetry.arguments.fetch_add(request.arguments as u64, Ordering::Relaxed);
                self.telemetry.typed_arguments.fetch_add(request.typed_arguments as u64, Ordering::Relaxed);
                self.telemetry
                    .type_snapshot_ns
                    .fetch_add(duration_nanos(request.type_snapshot_duration), Ordering::Relaxed);
                self.telemetry.request_bytes.fetch_add(request.payload.len() as u64, Ordering::Relaxed);
            }

            let nested_telemetry = &self.telemetry;
            let trace_enabled = self.trace_enabled;
            let mut handler = |frame: &Frame| {
                let nested_start = trace_enabled.then(Instant::now);

                let result = protocol::handle_nested_request(&frame.payload, codebase, session, |handle| {
                    request.types.get(handle).copied()
                });
                if let Some(start) = nested_start {
                    nested_telemetry.record_nested_request(frame.payload.len(), start.elapsed(), &result);
                }

                result.map(|(_, response)| response).map_err(|error| error.to_string().into_bytes())
            };

            let ipc_start = self.trace_enabled.then(Instant::now);
            let response = backend.transport.request_with_handler(request.payload, &mut handler).inspect_err(|_| {
                if self.trace_enabled {
                    self.telemetry.errors.fetch_add(1, Ordering::Relaxed);
                }
            })?;
            if let Some(start) = ipc_start {
                self.telemetry.ipc_ns.fetch_add(duration_nanos(start.elapsed()), Ordering::Relaxed);
                self.telemetry.response_bytes.fetch_add(response.len() as u64, Ordering::Relaxed);
            }

            let decode_start = self.trace_enabled.then(Instant::now);
            let result = protocol::decode_return_type_response(&response, |handle| request.types.get(handle).copied())
                .inspect_err(|_| {
                    if self.trace_enabled {
                        self.telemetry.errors.fetch_add(1, Ordering::Relaxed);
                    }
                })?;

            if let Some(start) = decode_start {
                self.telemetry.decode_ns.fetch_add(duration_nanos(start.elapsed()), Ordering::Relaxed);
            }

            if let Some(start) = provider_start
                && start.elapsed() >= SLOW_PROVIDER_THRESHOLD
            {
                tracing::trace!(
                    class = %mago_bytes::BytesDisplay(class),
                    method = %mago_bytes::BytesDisplay(method),
                    providers = indices.as_slice().len(),
                    arguments = request.arguments,
                    typed_arguments = request.typed_arguments,
                    argument_types = request.types.len(),
                    response_bytes = response.len(),
                    elapsed = ?start.elapsed(),
                    provided = result.is_some(),
                    "Slow external method return-type provider request completed."
                );
            }

            if let Some(result) = result {
                if self.trace_enabled {
                    self.telemetry.provided_types.fetch_add(1, Ordering::Relaxed);
                }

                return Ok(Some(result));
            }

            if self.trace_enabled {
                self.telemetry.declined_requests.fetch_add(1, Ordering::Relaxed);
            }
        }

        if self.trace_enabled && !dispatched {
            self.telemetry.unmatched_lookups.fetch_add(1, Ordering::Relaxed);
        }

        Ok(None)
    }

    fn initialize_transports(
        transports: impl IntoIterator<Item = Arc<T>>,
        php_version: PHPVersion,
        enabled_plugins: &[String],
        disable_defaults: bool,
    ) -> Result<Self, ExternalAnalyzerError> {
        let trace_enabled = tracing::enabled!(tracing::Level::TRACE);
        let started_at = trace_enabled.then(Instant::now);
        tracing::trace!(
            php_version = %php_version,
            explicitly_enabled_plugins = enabled_plugins.len(),
            disable_defaults,
            "Initializing external analyzer registrations."
        );

        let describe = protocol::encode_describe_request(php_version);
        let mut backends = Vec::new();
        let mut extensions = Vec::new();
        let mut plugins = Vec::new();
        let mut initialization_stubs = Vec::new();
        let mut extension_identifiers = HashSet::new();
        let mut plugin_selectors = HashMap::new();
        for plugin in available_plugins() {
            for selector in std::iter::once(plugin.id).chain(plugin.aliases.iter().copied()) {
                let selector = selector.to_ascii_lowercase();
                if let Some(first) = plugin_selectors.insert(selector.clone(), plugin.id.to_string()) {
                    return Err(ExternalAnalyzerError::DuplicatePluginSelector {
                        selector,
                        first,
                        second: plugin.id.to_string(),
                    });
                }
            }
        }

        for (backend_index, transport) in transports.into_iter().enumerate() {
            let backend_start = trace_enabled.then(Instant::now);
            tracing::trace!(
                backend = backend_index,
                request_bytes = describe.len(),
                "Describing external analyzer backend."
            );

            let responses = transport.broadcast(&describe)?;
            let response_bytes = responses.iter().map(Vec::len).sum::<usize>();
            let mut decoded = responses.iter().map(|response| protocol::decode_registration(response));
            let Some(first) = decoded.next() else {
                return Err(error::protocol("worker pool returned no analyzer registration responses"));
            };

            let mut registration = first?;
            for response in decoded {
                if response? != registration {
                    return Err(ExternalAnalyzerError::InconsistentRegistration);
                }
            }

            for extension in &registration.extensions {
                if !extension_identifiers.insert(extension.identifier.to_ascii_lowercase()) {
                    return Err(ExternalAnalyzerError::DuplicateExtension(extension.identifier.clone()));
                }
            }

            for plugin in &registration.plugins {
                for selector in
                    std::iter::once(plugin.identifier.as_str()).chain(plugin.aliases.iter().map(String::as_str))
                {
                    let selector = selector.to_ascii_lowercase();
                    if let Some(first) = plugin_selectors.insert(selector.clone(), plugin.identifier.clone()) {
                        return Err(ExternalAnalyzerError::DuplicatePluginSelector {
                            selector,
                            first,
                            second: plugin.identifier.clone(),
                        });
                    }
                }
            }

            let enabled = registration
                .plugins
                .iter()
                .filter(|plugin| {
                    enabled_plugins.iter().any(|name| plugin.matches(name))
                        || (!disable_defaults && plugin.default_enabled)
                })
                .map(|plugin| plugin.identifier.as_str())
                .collect::<HashSet<_>>();

            let advertised_function_providers = registration.function_providers.len();
            let advertised_method_providers = registration.method_providers.len();
            let enabled_indices = registration
                .plugins
                .iter()
                .filter(|plugin| enabled.contains(plugin.identifier.as_str()))
                .map(|plugin| plugin.index)
                .collect::<HashSet<_>>();
            registration.function_providers.retain(|provider| enabled.contains(provider.plugin.as_str()));
            registration.method_providers.retain(|provider| enabled.contains(provider.plugin.as_str()));
            registration.initialization_plugins.retain(|index| enabled_indices.contains(index));
            registration.before_analysis_plugins.retain(|index| enabled_indices.contains(index));
            registration.after_file_analysis_plugins.retain(|index| enabled_indices.contains(index));
            registration.after_analysis_plugins.retain(|index| enabled_indices.contains(index));

            if !registration.initialization_plugins.is_empty() {
                let initialize = protocol::encode_initialization_request(&registration.initialization_plugins)?;
                let responses = transport.broadcast(&initialize)?;
                let mut decoded = responses.iter().map(|response| {
                    protocol::decode_initialization_response(response, &registration.initialization_plugins)
                });

                let Some(first) = decoded.next() else {
                    return Err(error::protocol("worker pool returned no analyzer initialization responses"));
                };

                let initialized = first?;
                for response in decoded {
                    if response? != initialized {
                        return Err(ExternalAnalyzerError::InconsistentInitialization);
                    }
                }

                for stub in initialized {
                    let plugin = registration.plugins.get(usize::from(stub.plugin)).ok_or_else(|| {
                        error::protocol(format!("initialization references unknown plugin index {}", stub.plugin))
                    })?;

                    initialization_stubs.push(ExternalStub::new(
                        &plugin.extension,
                        &plugin.identifier,
                        &stub.filename,
                        stub.contents,
                    ));
                }
            }

            extensions.extend(registration.extensions.iter().cloned());
            plugins.extend(registration.plugins.iter().cloned());
            if let Some(start) = backend_start {
                tracing::trace!(
                    backend = backend_index,
                    workers = responses.len(),
                    response_bytes,
                    extensions = registration.extensions.len(),
                    plugins = registration.plugins.len(),
                    initialization_plugins = registration.initialization_plugins.len(),
                    enabled_plugins = enabled.len(),
                    function_providers = registration.function_providers.len(),
                    disabled_function_providers = advertised_function_providers - registration.function_providers.len(),
                    method_providers = registration.method_providers.len(),
                    disabled_method_providers = advertised_method_providers - registration.method_providers.len(),
                    before_analysis_plugins = registration.before_analysis_plugins.len(),
                    after_file_analysis_plugins = registration.after_file_analysis_plugins.len(),
                    after_analysis_plugins = registration.after_analysis_plugins.len(),
                    elapsed = ?start.elapsed(),
                    "External analyzer backend registered."
                );
            }

            backends.push(Backend::new(transport, registration));
        }

        let analyzer = Self {
            backends: backends.into_boxed_slice(),
            extensions: extensions.into_boxed_slice(),
            plugins: plugins.into_boxed_slice(),
            initialization_stubs: initialization_stubs.into_boxed_slice(),
            trace_enabled,
            telemetry: ExternalAnalyzerTelemetry::default(),
            started_at,
        };

        if let Some(start) = started_at {
            let function_providers =
                analyzer.backends.iter().map(|backend| backend.registration.function_providers.len()).sum::<usize>();
            let method_providers =
                analyzer.backends.iter().map(|backend| backend.registration.method_providers.len()).sum::<usize>();
            let before_analysis_plugins = analyzer
                .backends
                .iter()
                .map(|backend| backend.registration.before_analysis_plugins.len())
                .sum::<usize>();
            let after_file_analysis_plugins = analyzer
                .backends
                .iter()
                .map(|backend| backend.registration.after_file_analysis_plugins.len())
                .sum::<usize>();
            let after_analysis_plugins = analyzer
                .backends
                .iter()
                .map(|backend| backend.registration.after_analysis_plugins.len())
                .sum::<usize>();
            tracing::trace!(
                backends = analyzer.backends.len(),
                extensions = analyzer.extensions.len(),
                plugins = analyzer.plugins.len(),
                function_providers,
                method_providers,
                before_analysis_plugins,
                after_file_analysis_plugins,
                after_analysis_plugins,
                elapsed = ?start.elapsed(),
                "External analyzer initialized."
            );
        }

        Ok(analyzer)
    }
}

impl<T> Drop for ExternalAnalyzer<T> {
    fn drop(&mut self) {
        if !self.trace_enabled {
            return;
        }

        let function_lookups = self.telemetry.function_lookups.load(Ordering::Relaxed);
        let method_lookups = self.telemetry.method_lookups.load(Ordering::Relaxed);
        let lookups = function_lookups.saturating_add(method_lookups);
        let requests = self.telemetry.requests.load(Ordering::Relaxed);
        let nested_requests = self.telemetry.nested_requests.load(Ordering::Relaxed);
        let comparisons = self.telemetry.comparisons.load(Ordering::Relaxed);
        let metadata_queries = self.telemetry.metadata_queries.load(Ordering::Relaxed);
        let analysis_queries = self.telemetry.analysis_queries.load(Ordering::Relaxed);
        let symbol_reference_queries = self.telemetry.symbol_reference_queries.load(Ordering::Relaxed);
        let lifecycle_requests = self
            .telemetry
            .before_analysis_requests
            .load(Ordering::Relaxed)
            .saturating_add(self.telemetry.after_file_analysis_requests.load(Ordering::Relaxed))
            .saturating_add(self.telemetry.after_analysis_requests.load(Ordering::Relaxed));
        tracing::trace!(
            function_lookups,
            method_lookups,
            backend_checks = self.telemetry.backend_checks.load(Ordering::Relaxed),
            candidate_providers = self.telemetry.candidate_providers.load(Ordering::Relaxed),
            matched_providers = self.telemetry.matched_providers.load(Ordering::Relaxed),
            unmatched_lookups = self.telemetry.unmatched_lookups.load(Ordering::Relaxed),
            "External analyzer matching summary."
        );
        tracing::trace!(
            requests,
            provided_types = self.telemetry.provided_types.load(Ordering::Relaxed),
            declined_requests = self.telemetry.declined_requests.load(Ordering::Relaxed),
            errors = self.telemetry.errors.load(Ordering::Relaxed),
            arguments = self.telemetry.arguments.load(Ordering::Relaxed),
            typed_arguments = self.telemetry.typed_arguments.load(Ordering::Relaxed),
            snapshotted_types = self.telemetry.snapshotted_types.load(Ordering::Relaxed),
            request_bytes = self.telemetry.request_bytes.load(Ordering::Relaxed),
            response_bytes = self.telemetry.response_bytes.load(Ordering::Relaxed),
            "External analyzer provider summary."
        );

        tracing::trace!(
            nested_requests,
            nested_errors = self.telemetry.nested_errors.load(Ordering::Relaxed),
            nested_request_bytes = self.telemetry.nested_request_bytes.load(Ordering::Relaxed),
            nested_response_bytes = self.telemetry.nested_response_bytes.load(Ordering::Relaxed),
            comparisons,
            metadata_queries,
            analysis_queries,
            symbol_reference_queries,
            "External analyzer nested-query summary."
        );

        tracing::trace!(
            before_analysis_requests = self.telemetry.before_analysis_requests.load(Ordering::Relaxed),
            after_file_analysis_requests = self.telemetry.after_file_analysis_requests.load(Ordering::Relaxed),
            after_file_analysis_files = self.telemetry.after_file_analysis_files.load(Ordering::Relaxed),
            after_analysis_requests = self.telemetry.after_analysis_requests.load(Ordering::Relaxed),
            lifecycle_plugins = self.telemetry.lifecycle_plugins.load(Ordering::Relaxed),
            lifecycle_issues = self.telemetry.lifecycle_issues.load(Ordering::Relaxed),
            lifecycle_errors = self.telemetry.lifecycle_errors.load(Ordering::Relaxed),
            lifecycle_request_bytes = self.telemetry.lifecycle_request_bytes.load(Ordering::Relaxed),
            lifecycle_response_bytes = self.telemetry.lifecycle_response_bytes.load(Ordering::Relaxed),
            "External analyzer lifecycle summary."
        );

        tracing::trace!(
            matching_ms = nanos_millis(self.telemetry.matching_ns.load(Ordering::Relaxed)),
            encode_ms = nanos_millis(self.telemetry.encode_ns.load(Ordering::Relaxed)),
            type_snapshot_ms = nanos_millis(self.telemetry.type_snapshot_ns.load(Ordering::Relaxed)),
            ipc_ms = nanos_millis(self.telemetry.ipc_ns.load(Ordering::Relaxed)),
            comparison_ms = nanos_millis(self.telemetry.comparison_ns.load(Ordering::Relaxed)),
            metadata_query_ms = nanos_millis(self.telemetry.metadata_query_ns.load(Ordering::Relaxed)),
            analysis_query_ms = nanos_millis(self.telemetry.analysis_query_ns.load(Ordering::Relaxed)),
            symbol_reference_query_ms = nanos_millis(
                self.telemetry.symbol_reference_query_ns.load(Ordering::Relaxed),
            ),
            nested_query_ms = nanos_millis(self.telemetry.nested_ns.load(Ordering::Relaxed)),
            decode_ms = nanos_millis(self.telemetry.decode_ns.load(Ordering::Relaxed)),
            total_worker_cpu_ms = nanos_millis(self.telemetry.lookup_ns.load(Ordering::Relaxed)),
            average_lookup_micros = average_micros(self.telemetry.lookup_ns.load(Ordering::Relaxed), lookups),
            average_request_micros = average_micros(self.telemetry.ipc_ns.load(Ordering::Relaxed), requests),
            average_comparison_micros = average_micros(
                self.telemetry.comparison_ns.load(Ordering::Relaxed),
                comparisons,
            ),
            average_metadata_query_micros = average_micros(
                self.telemetry.metadata_query_ns.load(Ordering::Relaxed),
                metadata_queries,
            ),
            average_analysis_query_micros = average_micros(
                self.telemetry.analysis_query_ns.load(Ordering::Relaxed),
                analysis_queries,
            ),
            average_symbol_reference_query_micros = average_micros(
                self.telemetry.symbol_reference_query_ns.load(Ordering::Relaxed),
                symbol_reference_queries,
            ),
            average_nested_query_micros = average_micros(
                self.telemetry.nested_ns.load(Ordering::Relaxed),
                nested_requests,
            ),
            lifetime = ?self.started_at.map(|start| start.elapsed()).unwrap_or_default(),
            "External analyzer timing summary."
        );

        tracing::trace!(
            lifecycle_encode_ms = nanos_millis(self.telemetry.lifecycle_encode_ns.load(Ordering::Relaxed)),
            lifecycle_ipc_ms = nanos_millis(self.telemetry.lifecycle_ipc_ns.load(Ordering::Relaxed)),
            lifecycle_decode_ms = nanos_millis(self.telemetry.lifecycle_decode_ns.load(Ordering::Relaxed)),
            lifecycle_worker_cpu_ms = nanos_millis(self.telemetry.lifecycle_ns.load(Ordering::Relaxed)),
            average_lifecycle_request_micros =
                average_micros(self.telemetry.lifecycle_ns.load(Ordering::Relaxed), lifecycle_requests,),
            "External analyzer lifecycle timing summary."
        );
    }
}

fn pattern_matches(pattern: &[u8], value: &[u8]) -> bool {
    if pattern == b"*" {
        return true;
    }

    if let Some(prefix) = pattern.strip_suffix(b"*") {
        starts_with_ignore_case(value, prefix)
    } else {
        value.eq_ignore_ascii_case(pattern)
    }
}

fn append_path_component(target: &mut Vec<u8>, component: &[u8]) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";

    for byte in component {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.') {
            target.push(*byte);
        } else {
            target.push(b'%');
            target.push(HEX[usize::from(byte >> 4)]);
            target.push(HEX[usize::from(byte & 0x0f)]);
        }
    }
}

fn duration_nanos(duration: Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}

#[allow(clippy::cast_precision_loss, clippy::float_arithmetic)]
fn nanos_millis(nanos: u64) -> f64 {
    nanos as f64 / 1_000_000.0
}

fn average_micros(nanos: u64, count: u64) -> u64 {
    nanos.checked_div(count).unwrap_or(0) / 1_000
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_in_result, clippy::unwrap_used)]
mod tests {
    use std::sync::Mutex;

    use mago_codex::ttype::TType;
    use mago_codex::ttype::atomic::TAtomic;
    use mago_codex::ttype::atomic::array::TArray;
    use mago_extension::WorkerError;

    use super::*;

    #[derive(Debug)]
    struct TestTransport {
        requests: Mutex<Vec<Vec<u8>>>,
    }

    #[derive(Debug)]
    struct InitializationTransport {
        broadcasts: Mutex<usize>,
        initialization_responses: Vec<Vec<u8>>,
    }

    #[derive(Debug)]
    struct RegistrationTransport(Vec<u8>);

    impl AnalyzerTransport for TestTransport {
        fn broadcast(&self, _payload: &[u8]) -> Result<Vec<Vec<u8>>, WorkerError> {
            Ok(vec![protocol::testing::registration_response()])
        }

        fn request(&self, payload: Vec<u8>) -> Result<Vec<u8>, WorkerError> {
            self.requests.lock().unwrap().push(payload);
            Ok(protocol::testing::named_object_response("Demo\\Service"))
        }

        fn request_with_handler<H>(&self, payload: Vec<u8>, _handler: &mut H) -> Result<Vec<u8>, WorkerError>
        where
            H: WorkerRequestHandler,
        {
            self.request(payload)
        }
    }

    impl AnalyzerTransport for InitializationTransport {
        fn broadcast(&self, _payload: &[u8]) -> Result<Vec<Vec<u8>>, WorkerError> {
            let mut broadcasts = self.broadcasts.lock().unwrap();
            let responses = if *broadcasts == 0 {
                vec![protocol::testing::registration_response_with_initialization()]
            } else {
                self.initialization_responses.clone()
            };

            *broadcasts += 1;
            Ok(responses)
        }

        fn request(&self, _payload: Vec<u8>) -> Result<Vec<u8>, WorkerError> {
            unreachable!("initialization tests do not issue routed requests")
        }

        fn request_with_handler<H>(&self, _payload: Vec<u8>, _handler: &mut H) -> Result<Vec<u8>, WorkerError>
        where
            H: WorkerRequestHandler,
        {
            unreachable!("initialization tests do not issue routed requests")
        }
    }

    impl AnalyzerTransport for RegistrationTransport {
        fn broadcast(&self, _payload: &[u8]) -> Result<Vec<Vec<u8>>, WorkerError> {
            Ok(vec![self.0.clone()])
        }

        fn request(&self, _payload: Vec<u8>) -> Result<Vec<u8>, WorkerError> {
            unreachable!("registration tests do not issue routed requests")
        }

        fn request_with_handler<H>(&self, _payload: Vec<u8>, _handler: &mut H) -> Result<Vec<u8>, WorkerError>
        where
            H: WorkerRequestHandler,
        {
            unreachable!("registration tests do not issue routed requests")
        }
    }

    #[test]
    fn registration_is_decoded_and_filtered() {
        let transport = Arc::new(TestTransport { requests: Mutex::new(Vec::new()) });
        let analyzer = ExternalAnalyzer::initialize_transports([transport], PHPVersion::PHP85, &[], false)
            .expect("registration should succeed");

        assert_eq!(analyzer.extensions().len(), 1);
        assert_eq!(analyzer.plugins()[0].identifier, "demo");
        assert_eq!(analyzer.backends[0].registration.function_providers.len(), 1);
    }

    #[test]
    fn exact_and_wildcard_provider_indices_preserve_registration_order() {
        let indices =
            ProviderIndices::from_exact_and_wildcards(&[1, 3], [0, 3, 4]).expect("at least one provider should match");

        assert_eq!(indices.as_slice(), &[0, 1, 3, 4]);
        assert!(ProviderIndices::from_exact_and_wildcards(&[], []).is_none());
    }

    #[test]
    fn default_plugins_can_be_disabled_and_reenabled_by_alias() {
        let disabled_transport = Arc::new(TestTransport { requests: Mutex::new(Vec::new()) });
        let disabled = ExternalAnalyzer::initialize_transports([disabled_transport], PHPVersion::PHP85, &[], true)
            .expect("registration should succeed");
        assert!(disabled.backends[0].registration.function_providers.is_empty());

        let enabled_transport = Arc::new(TestTransport { requests: Mutex::new(Vec::new()) });
        let enabled = ExternalAnalyzer::initialize_transports(
            [enabled_transport],
            PHPVersion::PHP85,
            &["EXAMPLE".to_string()],
            true,
        )
        .expect("registration should succeed");
        assert_eq!(enabled.backends[0].registration.function_providers.len(), 1);
    }

    #[test]
    fn plugin_selectors_cannot_shadow_native_plugins() {
        let transport = Arc::new(RegistrationTransport(protocol::testing::registration_response_with_plugin(
            "demo/extension",
            "demo",
            &["StD"],
        )));

        let result = ExternalAnalyzer::initialize_transports([transport], PHPVersion::PHP85, &[], false);

        assert!(matches!(
            result,
            Err(ExternalAnalyzerError::DuplicatePluginSelector { selector, first, second })
                if selector == "std" && first == "stdlib" && second == "demo"
        ));
    }

    #[test]
    fn plugin_aliases_must_be_unique_across_extension_hosts() {
        let first = Arc::new(RegistrationTransport(protocol::testing::registration_response_with_plugin(
            "demo/first",
            "first",
            &["shared"],
        )));
        let second = Arc::new(RegistrationTransport(protocol::testing::registration_response_with_plugin(
            "demo/second",
            "second",
            &["SHARED"],
        )));

        let result = ExternalAnalyzer::initialize_transports([first, second], PHPVersion::PHP85, &[], false);

        assert!(matches!(
            result,
            Err(ExternalAnalyzerError::DuplicatePluginSelector { selector, first, second })
                if selector == "shared" && first == "first" && second == "second"
        ));
    }

    #[test]
    fn initialization_stubs_are_scoped_and_exposed_as_external_files() {
        let response = protocol::testing::initialization_response(b"framework.php", b"<?php class FrameworkStub {}");
        let transport = Arc::new(InitializationTransport {
            broadcasts: Mutex::new(0),
            initialization_responses: vec![response.clone(), response],
        });

        let analyzer = ExternalAnalyzer::initialize_transports([Arc::clone(&transport)], PHPVersion::PHP85, &[], false)
            .expect("initialization should succeed");

        let files = analyzer.initialization_files();
        assert_eq!(*transport.broadcasts.lock().unwrap(), 2);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].name.as_ref(), b"@mago-extension/demo%2Fextension/demo/framework.php");
        assert_eq!(files[0].contents.as_ref(), b"<?php class FrameworkStub {}");
        assert!(files[0].file_type.is_external());
        assert!(files[0].path.is_none());
    }

    #[test]
    fn initialization_requires_identical_stubs_from_every_worker() {
        let transport = Arc::new(InitializationTransport {
            broadcasts: Mutex::new(0),
            initialization_responses: vec![
                protocol::testing::initialization_response(b"framework.php", b"<?php class First {}"),
                protocol::testing::initialization_response(b"framework.php", b"<?php class Second {}"),
            ],
        });

        let result = ExternalAnalyzer::initialize_transports([transport], PHPVersion::PHP85, &[], false);

        assert!(matches!(result, Err(ExternalAnalyzerError::InconsistentInitialization)));
    }

    #[test]
    fn disabled_plugins_are_not_initialized() {
        let transport =
            Arc::new(InitializationTransport { broadcasts: Mutex::new(0), initialization_responses: Vec::new() });
        let analyzer = ExternalAnalyzer::initialize_transports([Arc::clone(&transport)], PHPVersion::PHP85, &[], true)
            .expect("registration should succeed");

        assert_eq!(*transport.broadcasts.lock().unwrap(), 1);
        assert!(analyzer.initialization_files().is_empty());
    }

    #[test]
    fn decodes_constructed_and_lossless_reference_types() {
        let named =
            protocol::decode_return_type_response(&protocol::testing::named_object_response("Demo\\Service"), |_| None)
                .expect("response should decode")
                .expect("response should be handled");
        assert_eq!(
            named.get_single_named_object().expect("type should be a named object").get_name().as_bytes(),
            b"Demo\\Service"
        );

        let original = mago_codex::ttype::get_literal_string(mago_word::word(b"hello"));
        let referenced = protocol::decode_return_type_response(&protocol::testing::reference_response(0), |slot| {
            (slot == 0).then_some(&original)
        })
        .expect("response should decode")
        .expect("response should be handled");
        assert_eq!(referenced.get_id(), original.get_id());

        let non_negative =
            protocol::decode_return_type_response(&protocol::testing::non_negative_int_response(), |_| None)
                .expect("response should decode")
                .expect("response should be handled");
        assert!(non_negative.get_single_int().expect("type should be an integer").is_non_negative());

        let non_empty =
            protocol::decode_return_type_response(&protocol::testing::non_empty_string_response(), |_| None)
                .expect("response should decode")
                .expect("response should be handled");
        assert!(non_empty.is_non_empty_string());

        let complete = protocol::decode_return_type_response(
            &protocol::testing::complete_non_empty_string_list_response(),
            |_| None,
        )
        .expect("complete response should decode")
        .expect("complete response should be handled");
        let TAtomic::Array(TArray::List(list)) = complete.get_single() else {
            panic!("complete type should be a list");
        };

        assert!(list.non_empty);
        assert!(list.element_type.is_string());
    }
}
