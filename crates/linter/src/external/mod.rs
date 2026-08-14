//! Worker-backed custom linter rules.
//!
//! A pool is queried once for immutable rule metadata. Linting then sends one
//! coarse-grained request per file and extension, containing source text, a
//! stable flat syntax tree, comments, and resolved names. Node callbacks are
//! dispatched inside the worker rather than crossing IPC one node at a time.

use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;

use mago_database::file::File;
use mago_extension::WorkerError;
use mago_extension::WorkerPool;
use mago_names::ResolvedNames;
use mago_php_version::PHPVersion;
use mago_reporting::IssueCollection;
use mago_reporting::Level;
use mago_syntax::cst::NodeKind;
use mago_syntax::cst::Program;

use crate::rule::AnyRule;
use crate::settings::Settings;

pub use error::ExternalLintError;
use protocol::Registration;

mod error;
pub mod protocol;

const SLOW_FILE_THRESHOLD: Duration = Duration::from_millis(10);

#[derive(Debug, Default)]
struct ExternalLintTelemetry {
    files: AtomicU64,
    backend_checks: AtomicU64,
    inactive_backends: AtomicU64,
    unmatched_backends: AtomicU64,
    requests: AtomicU64,
    errors: AtomicU64,
    active_rules: AtomicU64,
    targets: AtomicU64,
    nodes: AtomicU64,
    names: AtomicU64,
    trivia: AtomicU64,
    issues: AtomicU64,
    request_bytes: AtomicU64,
    response_bytes: AtomicU64,
    encode_ns: AtomicU64,
    snapshot_ns: AtomicU64,
    serialization_ns: AtomicU64,
    ipc_ns: AtomicU64,
    decode_ns: AtomicU64,
    total_ns: AtomicU64,
}

/// Metadata advertised for one custom linter rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalRule {
    /// Identifier of the extension that owns this rule.
    pub extension: String,
    /// Globally unique issue code reported by the rule.
    pub code: String,
    /// Human-readable rule name.
    pub name: String,
    /// Short rule description.
    pub description: String,
    /// Severity assigned to issues reported by this rule.
    pub default_level: Level,
    /// Whether the rule runs when no `--only` filter is present.
    pub default_enabled: bool,
    /// CST node kinds subscribed to by the rule.
    pub targets: Vec<NodeKind>,
}

/// Metadata advertised by one extension worker pool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalExtension {
    pub identifier: String,
    pub name: String,
    pub version: String,
    pub rules: Vec<ExternalRule>,
}

pub(crate) trait LinterTransport: std::fmt::Debug + Send + Sync {
    fn broadcast(&self, payload: &[u8]) -> Result<Vec<Vec<u8>>, WorkerError>;
    fn request(&self, payload: Vec<u8>) -> Result<Vec<u8>, WorkerError>;
}

impl LinterTransport for WorkerPool {
    fn broadcast(&self, payload: &[u8]) -> Result<Vec<Vec<u8>>, WorkerError> {
        Self::broadcast(self, payload)
    }

    fn request(&self, payload: Vec<u8>) -> Result<Vec<u8>, WorkerError> {
        Self::request(self, payload)
    }
}

#[derive(Debug)]
struct Backend<T> {
    transport: Arc<T>,
    registration: Registration,
    default_plan: ActiveRulePlan,
}

#[derive(Debug)]
struct ActiveRulePlan {
    indices: Box<[u16]>,
    targets: [bool; u8::MAX as usize + 1],
}

impl ActiveRulePlan {
    fn build(rules: &[ExternalRule], enabled: impl Fn(&ExternalRule) -> bool) -> Result<Self, ExternalLintError> {
        let mut indices = Vec::new();
        let mut targets = [false; u8::MAX as usize + 1];
        for (index, rule) in rules.iter().enumerate().filter(|(_, rule)| enabled(rule)) {
            let index = u16::try_from(index).map_err(|_| {
                ExternalLintError::Protocol("worker registered more than 65,536 linter rules".to_string())
            })?;
            indices.push(index);
            for target in &rule.targets {
                targets[*target as usize] = true;
            }
        }

        Ok(Self { indices: indices.into_boxed_slice(), targets })
    }
}

/// A set of custom linter rules backed by one or more extension worker pools.
#[derive(Debug)]
pub struct ExternalLinter<T = WorkerPool> {
    backends: Box<[Backend<T>]>,
    extensions: Box<[ExternalExtension]>,
    rules: Box<[ExternalRule]>,
    trace_enabled: bool,
    telemetry: ExternalLintTelemetry,
    started_at: Option<Instant>,
}

impl ExternalLinter<WorkerPool> {
    /// Discovers and validates the rules exposed by each worker pool.
    ///
    /// Every process in a pool must advertise equivalent metadata. Extension
    /// identifiers and rule codes must also be unique across pools.
    ///
    /// # Errors
    ///
    /// Returns an error if a worker fails, sends malformed metadata, disagrees
    /// with its peers, or conflicts with another extension.
    pub fn initialize(
        pools: impl IntoIterator<Item = Arc<WorkerPool>>,
        php_version: PHPVersion,
    ) -> Result<Self, ExternalLintError> {
        let linter = Self::initialize_transports(pools, php_version)?;
        for backend in &linter.backends {
            if backend.registration.has_worker_reducer {
                backend.transport.enable_worker_reduction();
            }
        }

        Ok(linter)
    }
}

impl<T> ExternalLinter<T> {
    /// Returns metadata for every registered extension.
    #[must_use]
    pub fn extensions(&self) -> &[ExternalExtension] {
        &self.extensions
    }

    /// Returns metadata for every registered custom rule.
    #[must_use]
    pub fn rules(&self) -> &[ExternalRule] {
        &self.rules
    }
}

impl<T> ExternalLinter<T> {
    pub(crate) fn lint<'arena>(
        &self,
        file: &File,
        program: &Program<'arena>,
        resolved_names: &ResolvedNames<'arena>,
        only: Option<&[String]>,
    ) -> Result<IssueCollection, ExternalLintError>
    where
        T: LinterTransport,
    {
        let trace_start = self.trace_enabled.then(Instant::now);
        let mut file_encode_ns = 0u64;
        let mut file_ipc_ns = 0u64;
        let mut file_decode_ns = 0u64;
        let mut file_requests = 0u64;
        let mut file_issues = 0u64;
        let mut file_request_bytes = 0u64;
        let mut file_targets = 0u64;
        let mut file_nodes = 0u64;
        if self.trace_enabled {
            self.telemetry.files.fetch_add(1, Ordering::Relaxed);
        }
        let mut issues = IssueCollection::new();
        for backend in &self.backends {
            if self.trace_enabled {
                self.telemetry.backend_checks.fetch_add(1, Ordering::Relaxed);
            }
            let selected_plan = only
                .map(|codes| {
                    ActiveRulePlan::build(&backend.registration.rules, |rule| {
                        codes.iter().any(|code| code == &rule.code)
                    })
                })
                .transpose()?;
            let plan = selected_plan.as_ref().unwrap_or(&backend.default_plan);
            let active_rule_indices = &plan.indices;

            if active_rule_indices.is_empty() {
                if self.trace_enabled {
                    self.telemetry.inactive_backends.fetch_add(1, Ordering::Relaxed);
                }
                continue;
            }

            let encode_start = self.trace_enabled.then(Instant::now);
            let request = protocol::encode_lint_request(
                file,
                program,
                resolved_names,
                active_rule_indices,
                &plan.targets,
                self.trace_enabled,
            );
            if let Some(start) = encode_start {
                let elapsed = duration_nanos(start.elapsed());
                file_encode_ns = file_encode_ns.saturating_add(elapsed);
                self.telemetry.encode_ns.fetch_add(elapsed, Ordering::Relaxed);
            }
            let Some(request) = request.inspect_err(|_| {
                if self.trace_enabled {
                    self.telemetry.errors.fetch_add(1, Ordering::Relaxed);
                }
            })?
            else {
                if self.trace_enabled {
                    self.telemetry.unmatched_backends.fetch_add(1, Ordering::Relaxed);
                }
                continue;
            };
            if self.trace_enabled {
                self.telemetry.requests.fetch_add(1, Ordering::Relaxed);
                self.telemetry.active_rules.fetch_add(active_rule_indices.len() as u64, Ordering::Relaxed);
                self.telemetry.targets.fetch_add(request.targets as u64, Ordering::Relaxed);
                self.telemetry.nodes.fetch_add(request.nodes as u64, Ordering::Relaxed);
                self.telemetry.names.fetch_add(request.names as u64, Ordering::Relaxed);
                self.telemetry.trivia.fetch_add(request.trivia as u64, Ordering::Relaxed);
                self.telemetry.request_bytes.fetch_add(request.payload.len() as u64, Ordering::Relaxed);
                self.telemetry.snapshot_ns.fetch_add(duration_nanos(request.snapshot_duration), Ordering::Relaxed);
                self.telemetry
                    .serialization_ns
                    .fetch_add(duration_nanos(request.serialization_duration), Ordering::Relaxed);
                file_requests += 1;
                file_request_bytes = file_request_bytes.saturating_add(request.payload.len() as u64);
                file_targets = file_targets.saturating_add(request.targets as u64);
                file_nodes = file_nodes.saturating_add(request.nodes as u64);
            }
            let ipc_start = self.trace_enabled.then(Instant::now);
            let response = backend.transport.request(request.payload).inspect_err(|_| {
                if self.trace_enabled {
                    self.telemetry.errors.fetch_add(1, Ordering::Relaxed);
                }
            })?;
            if let Some(start) = ipc_start {
                let elapsed = duration_nanos(start.elapsed());
                file_ipc_ns = file_ipc_ns.saturating_add(elapsed);
                self.telemetry.ipc_ns.fetch_add(elapsed, Ordering::Relaxed);
                self.telemetry.response_bytes.fetch_add(response.len() as u64, Ordering::Relaxed);
            }
            let decode_start = self.trace_enabled.then(Instant::now);
            let backend_issues =
                protocol::decode_lint_response(&response, file, &backend.registration.rules, active_rule_indices)
                    .inspect_err(|_| {
                        if self.trace_enabled {
                            self.telemetry.errors.fetch_add(1, Ordering::Relaxed);
                        }
                    })?;
            if let Some(start) = decode_start {
                let elapsed = duration_nanos(start.elapsed());
                file_decode_ns = file_decode_ns.saturating_add(elapsed);
                self.telemetry.decode_ns.fetch_add(elapsed, Ordering::Relaxed);
                self.telemetry.issues.fetch_add(backend_issues.len() as u64, Ordering::Relaxed);
                file_issues = file_issues.saturating_add(backend_issues.len() as u64);
            }
            issues.extend(backend_issues);
        }

        if let Some(start) = trace_start {
            let elapsed = start.elapsed();
            self.telemetry.total_ns.fetch_add(duration_nanos(elapsed), Ordering::Relaxed);
            if elapsed >= SLOW_FILE_THRESHOLD {
                tracing::trace!(
                    file = %mago_bytes::BytesDisplay(&file.name),
                    elapsed = ?elapsed,
                    requests = file_requests,
                    issues = file_issues,
                    request_bytes = file_request_bytes,
                    targets = file_targets,
                    nodes = file_nodes,
                    encode_ms = nanos_millis(file_encode_ns),
                    ipc_ms = nanos_millis(file_ipc_ns),
                    decode_ms = nanos_millis(file_decode_ns),
                    "Slow external linter file completed."
                );
            }
        }

        Ok(issues)
    }

    fn initialize_transports(
        transports: impl IntoIterator<Item = Arc<T>>,
        php_version: PHPVersion,
    ) -> Result<Self, ExternalLintError>
    where
        T: LinterTransport,
    {
        let trace_enabled = tracing::enabled!(tracing::Level::TRACE);
        let started_at = trace_enabled.then(Instant::now);
        tracing::trace!(php_version = %php_version, "Initializing external linter registrations.");
        let describe = protocol::encode_describe_request(php_version);
        let mut backends = Vec::new();
        let mut extensions = Vec::new();
        let mut rules = Vec::new();
        let mut extension_identifiers = HashSet::new();
        let native_settings = Settings { php_version, ..Settings::default() };
        let mut rule_codes = AnyRule::get_all_for(&native_settings, None, true)
            .into_iter()
            .map(|(rule, _)| rule.code().to_string())
            .collect::<HashSet<_>>();

        for (backend_index, transport) in transports.into_iter().enumerate() {
            let backend_start = trace_enabled.then(Instant::now);
            tracing::trace!(
                backend = backend_index,
                request_bytes = describe.len(),
                "Describing external linter backend."
            );
            let responses = transport.broadcast(&describe)?;
            let response_bytes = responses.iter().map(Vec::len).sum::<usize>();
            let mut decoded = responses.iter().map(|response| protocol::decode_registration(response));
            let Some(first) = decoded.next() else {
                return Err(ExternalLintError::Protocol("worker pool returned no registration responses".to_string()));
            };
            let registration = first?;
            for response in decoded {
                if response? != registration {
                    return Err(ExternalLintError::InconsistentRegistration);
                }
            }

            for extension in &registration.extensions {
                if !extension_identifiers.insert(extension.identifier.to_ascii_lowercase()) {
                    return Err(ExternalLintError::DuplicateExtension(extension.identifier.clone()));
                }
            }

            for rule in &registration.rules {
                if !rule_codes.insert(rule.code.clone()) {
                    return Err(ExternalLintError::DuplicateRule(rule.code.clone()));
                }
            }

            extensions.extend(registration.extensions.iter().cloned());
            rules.extend(registration.rules.iter().cloned());
            if let Some(start) = backend_start {
                tracing::trace!(
                    backend = backend_index,
                    workers = responses.len(),
                    response_bytes,
                    extensions = registration.extensions.len(),
                    rules = registration.rules.len(),
                    elapsed = ?start.elapsed(),
                    "External linter backend registered."
                );
            }
            let default_plan = ActiveRulePlan::build(&registration.rules, |rule| rule.default_enabled)?;
            backends.push(Backend { transport, registration, default_plan });
        }

        let linter = Self {
            backends: backends.into_boxed_slice(),
            extensions: extensions.into_boxed_slice(),
            rules: rules.into_boxed_slice(),
            trace_enabled,
            telemetry: ExternalLintTelemetry::default(),
            started_at,
        };
        if let Some(start) = started_at {
            tracing::trace!(
                backends = linter.backends.len(),
                extensions = linter.extensions.len(),
                rules = linter.rules.len(),
                elapsed = ?start.elapsed(),
                "External linter initialized."
            );
        }

        Ok(linter)
    }
}

impl<T> Drop for ExternalLinter<T> {
    fn drop(&mut self) {
        if !self.trace_enabled {
            return;
        }

        let files = self.telemetry.files.load(Ordering::Relaxed);
        let requests = self.telemetry.requests.load(Ordering::Relaxed);
        tracing::trace!(
            files,
            backend_checks = self.telemetry.backend_checks.load(Ordering::Relaxed),
            inactive_backends = self.telemetry.inactive_backends.load(Ordering::Relaxed),
            unmatched_backends = self.telemetry.unmatched_backends.load(Ordering::Relaxed),
            requests,
            errors = self.telemetry.errors.load(Ordering::Relaxed),
            issues = self.telemetry.issues.load(Ordering::Relaxed),
            "External linter dispatch summary."
        );
        tracing::trace!(
            active_rules = self.telemetry.active_rules.load(Ordering::Relaxed),
            targets = self.telemetry.targets.load(Ordering::Relaxed),
            nodes = self.telemetry.nodes.load(Ordering::Relaxed),
            resolved_names = self.telemetry.names.load(Ordering::Relaxed),
            trivia = self.telemetry.trivia.load(Ordering::Relaxed),
            request_bytes = self.telemetry.request_bytes.load(Ordering::Relaxed),
            response_bytes = self.telemetry.response_bytes.load(Ordering::Relaxed),
            "External linter snapshot summary."
        );
        tracing::trace!(
            encode_ms = nanos_millis(self.telemetry.encode_ns.load(Ordering::Relaxed)),
            snapshot_ms = nanos_millis(self.telemetry.snapshot_ns.load(Ordering::Relaxed)),
            serialization_ms = nanos_millis(self.telemetry.serialization_ns.load(Ordering::Relaxed)),
            ipc_ms = nanos_millis(self.telemetry.ipc_ns.load(Ordering::Relaxed)),
            decode_ms = nanos_millis(self.telemetry.decode_ns.load(Ordering::Relaxed)),
            total_worker_cpu_ms = nanos_millis(self.telemetry.total_ns.load(Ordering::Relaxed)),
            average_file_micros = average_micros(self.telemetry.total_ns.load(Ordering::Relaxed), files),
            average_request_micros = average_micros(self.telemetry.ipc_ns.load(Ordering::Relaxed), requests),
            lifetime = ?self.started_at.map(|start| start.elapsed()).unwrap_or_default(),
            "External linter timing summary."
        );
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
    use std::borrow::Cow;
    use std::sync::Mutex;

    use mago_allocator::LocalArena;
    use mago_database::file::File;
    use mago_names::resolver::NameResolver;
    use mago_reporting::Annotation;
    use mago_reporting::Issue;
    use mago_span::Span;
    use mago_syntax::parser::parse_file;
    use mago_text_edit::Safety;
    use mago_text_edit::TextEdit;

    use crate::Linter;
    use crate::registry::RuleRegistry;
    use crate::settings::Settings;

    use super::protocol::testing;
    use super::*;

    #[derive(Debug)]
    struct MockTransport {
        registration: Vec<u8>,
        response: Vec<u8>,
        request: Mutex<Option<testing::DecodedRequest>>,
        workers: usize,
    }

    impl LinterTransport for MockTransport {
        fn broadcast(&self, _payload: &[u8]) -> Result<Vec<Vec<u8>>, mago_extension::WorkerError> {
            Ok(vec![self.registration.clone(); self.workers])
        }

        fn request(&self, payload: Vec<u8>) -> Result<Vec<u8>, mago_extension::WorkerError> {
            *self.request.lock().unwrap() = Some(testing::decode_lint_request(&payload).unwrap());
            Ok(self.response.clone())
        }
    }

    #[test]
    fn batches_matching_subtrees_and_decodes_native_issues() {
        let source = b"<?php\n\n/** docs */\nuse Vendor\\Package as P;\nP\\run();\n";
        let arena = LocalArena::new();
        let file = File::ephemeral(Cow::Borrowed(b"src/test.php"), Cow::Borrowed(source));
        let program = parse_file(&arena, &file);
        let resolved_names = NameResolver::new(&arena).resolve(program);
        let call_start = source.windows(b"P\\run()".len()).position(|window| window == b"P\\run()").unwrap() as u32;
        let target_span = Span::new(
            file.id,
            mago_span::Position::new(call_start),
            mago_span::Position::new(call_start + b"P\\run()".len() as u32),
        );
        let issue = Issue::warning("Do not call this function")
            .with_code("acme/no-run")
            .with_annotation(Annotation::primary(target_span).with_message("Call occurs here"))
            .with_edit(
                file.id,
                TextEdit::replace(call_start..call_start + 1, "run_safely").with_safety(Safety::PotentiallyUnsafe),
            );
        let transport = Arc::new(MockTransport {
            registration: testing::describe_response(
                "acme/tools",
                "Acme Tools",
                "1.0.0",
                &[("acme/no-run", "No run", "Disallows this call.", Level::Warning, true, &[NodeKind::FunctionCall])],
            ),
            response: testing::lint_response(&[(0, issue)]),
            request: Mutex::new(None),
            workers: 3,
        });
        let external = ExternalLinter::initialize_transports([Arc::clone(&transport)], PHPVersion::PHP85)
            .expect("registration should succeed");

        let issues = external.lint(&file, program, &resolved_names, None).expect("external lint should succeed");
        assert_eq!(issues.len(), 1);
        let issue = issues.iter().next().expect("the external issue should exist");
        assert_eq!(issue.code.as_deref(), Some("acme/no-run"));
        let edits = issue.edits.get(&file.id).expect("the external issue should retain its edit batch");
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].range.start, call_start);
        assert_eq!(edits[0].range.end, call_start + 1);
        assert_eq!(edits[0].new_text, b"run_safely");
        assert_eq!(edits[0].safety, Safety::PotentiallyUnsafe);

        let request = transport.request.lock().unwrap();
        let request = request.as_ref().expect("one request should be captured");
        assert_eq!(request.file_name, b"src/test.php");
        assert_eq!(request.source, source);
        assert_eq!(request.active_rules, [0]);
        assert_eq!(request.targets.len(), 1);
        assert_eq!(request.nodes[request.targets[0] as usize].kind, "FunctionCall");
        assert!(!request.nodes.iter().any(|node| node.kind == "Program"));
        assert!(request.nodes.iter().any(|node| node.kind == "FunctionCall" && node.parent.is_none()));
        assert!(request.nodes.iter().any(|node| !node.children.is_empty()));
        assert!(request.names.iter().any(|(_, _, name, imported)| name == b"Vendor\\Package\\run" && *imported));
        assert!(request.trivia.iter().any(|(kind, _, _)| kind == "DocBlockComment"));
    }

    #[test]
    fn skips_worker_when_a_file_has_no_matching_nodes() {
        let source = b"<?php\nconst ANSWER = 42;\n";
        let arena = LocalArena::new();
        let file = File::ephemeral(Cow::Borrowed(b"src/test.php"), Cow::Borrowed(source));
        let program = parse_file(&arena, &file);
        let resolved_names = NameResolver::new(&arena).resolve(program);
        let transport = Arc::new(MockTransport {
            registration: testing::describe_response(
                "acme/tools",
                "Acme Tools",
                "1.0.0",
                &[("acme/no-run", "No run", "Disallows this call.", Level::Warning, true, &[NodeKind::FunctionCall])],
            ),
            response: testing::lint_response(&[]),
            request: Mutex::new(None),
            workers: 1,
        });
        let external = ExternalLinter::initialize_transports([Arc::clone(&transport)], PHPVersion::PHP85)
            .expect("registration should succeed");

        let issues = external.lint(&file, program, &resolved_names, None).expect("external lint should succeed");

        assert!(issues.is_empty());
        assert!(transport.request.lock().unwrap().is_none());
    }

    #[test]
    fn deduplicates_nested_matching_subtrees() {
        let source = b"<?php\nouter(inner());\n";
        let arena = LocalArena::new();
        let file = File::ephemeral(Cow::Borrowed(b"src/test.php"), Cow::Borrowed(source));
        let program = parse_file(&arena, &file);
        let resolved_names = NameResolver::new(&arena).resolve(program);
        let transport = Arc::new(MockTransport {
            registration: testing::describe_response(
                "acme/tools",
                "Acme Tools",
                "1.0.0",
                &[("acme/no-call", "No calls", "Disallows calls.", Level::Warning, true, &[NodeKind::FunctionCall])],
            ),
            response: testing::lint_response(&[]),
            request: Mutex::new(None),
            workers: 1,
        });
        let external = ExternalLinter::initialize_transports([Arc::clone(&transport)], PHPVersion::PHP85)
            .expect("registration should succeed");

        external.lint(&file, program, &resolved_names, None).expect("external lint should succeed");

        let request = transport.request.lock().unwrap();
        let request = request.as_ref().expect("one request should be captured");
        assert_eq!(request.targets.len(), 2);
        assert_eq!(request.nodes.iter().filter(|node| node.kind == "FunctionCall").count(), 2);
        assert_ne!(request.targets[0], request.targets[1]);
    }

    #[test]
    fn external_issues_obey_lint_expect_pragmas() {
        let source = b"<?php\n// @mago-expect lint:acme/no-foo\nfoo();\n";
        let arena = LocalArena::new();
        let file = File::ephemeral(Cow::Borrowed(b"test.php"), Cow::Borrowed(source));
        let program = parse_file(&arena, &file);
        let resolved_names = NameResolver::new(&arena).resolve(program);
        let call_start = source.windows(b"foo()".len()).position(|window| window == b"foo()").unwrap() as u32;
        let issue =
            Issue::warning("Do not call foo").with_code("acme/no-foo").with_annotation(Annotation::primary(Span::new(
                file.id,
                mago_span::Position::new(call_start),
                mago_span::Position::new(call_start + b"foo()".len() as u32),
            )));
        let transport = Arc::new(MockTransport {
            registration: testing::describe_response(
                "acme/tools",
                "Acme Tools",
                "1.0.0",
                &[("acme/no-foo", "No foo", "Disallows foo.", Level::Warning, true, &[NodeKind::FunctionCall])],
            ),
            response: testing::lint_response(&[(0, issue)]),
            request: Mutex::new(None),
            workers: 1,
        });
        let external =
            ExternalLinter::initialize_transports([transport], PHPVersion::PHP85).expect("registration should succeed");
        let settings = Settings::default();
        let registry = Arc::new(RuleRegistry::build(&settings, Some(&["acme/no-foo".to_string()]), false));
        let linter = Linter::from_registry(&arena, registry, settings.php_version);

        let issues = linter
            .lint_internal(&file, program, &resolved_names, Some(&external))
            .expect("external lint should succeed");
        assert!(issues.is_empty(), "expected pragma should suppress the external issue: {issues:#?}");
    }

    #[test]
    fn rejects_inconsistent_pool_registrations() {
        #[derive(Debug)]
        struct InconsistentTransport;

        impl LinterTransport for InconsistentTransport {
            fn broadcast(&self, _payload: &[u8]) -> Result<Vec<Vec<u8>>, mago_extension::WorkerError> {
                Ok(vec![
                    testing::describe_response("acme/a", "A", "1", &[]),
                    testing::describe_response("acme/b", "B", "1", &[]),
                ])
            }

            fn request(&self, _payload: Vec<u8>) -> Result<Vec<u8>, mago_extension::WorkerError> {
                unreachable!()
            }
        }

        let result = ExternalLinter::initialize_transports([Arc::new(InconsistentTransport)], PHPVersion::PHP85);
        assert!(matches!(result, Err(ExternalLintError::InconsistentRegistration)));
    }

    #[test]
    fn rejects_case_insensitive_duplicate_extension_identifiers() {
        let first = Arc::new(MockTransport {
            registration: testing::describe_response("Acme/Tools", "Acme", "1", &[]),
            response: testing::lint_response(&[]),
            request: Mutex::new(None),
            workers: 1,
        });
        let second = Arc::new(MockTransport {
            registration: testing::describe_response("acme/tools", "Acme", "2", &[]),
            response: testing::lint_response(&[]),
            request: Mutex::new(None),
            workers: 1,
        });

        let result = ExternalLinter::initialize_transports([first, second], PHPVersion::PHP85);

        assert!(matches!(result, Err(ExternalLintError::DuplicateExtension(identifier)) if identifier == "acme/tools"));
    }

    #[test]
    fn rejects_issue_codes_owned_by_native_rules() {
        let transport = Arc::new(MockTransport {
            registration: testing::describe_response(
                "acme/tools",
                "Acme",
                "1",
                &[(
                    "array-style",
                    "Conflicting rule",
                    "Attempts to shadow a native rule.",
                    Level::Warning,
                    true,
                    &[NodeKind::Array],
                )],
            ),
            response: testing::lint_response(&[]),
            request: Mutex::new(None),
            workers: 1,
        });

        let result = ExternalLinter::initialize_transports([transport], PHPVersion::PHP85);

        assert!(matches!(result, Err(ExternalLintError::DuplicateRule(code)) if code == "array-style"));
    }

    #[test]
    fn registers_multiple_extensions_from_one_worker_pool() {
        let function_targets = [NodeKind::FunctionCall];
        let interface_targets = [NodeKind::Interface];
        let iteration_rules = [(
            "acme/prefer-array-any",
            "Prefer array_any",
            "Prefers the native array_any function.",
            Level::Help,
            true,
            function_targets.as_slice(),
        )];
        let architecture_rules = [(
            "acme/no-interface",
            "No interfaces",
            "Disallows interface declarations.",
            Level::Warning,
            false,
            interface_targets.as_slice(),
        )];
        let transport = Arc::new(MockTransport {
            registration: testing::describe_extensions_response(&[
                ("acme/iteration", "Iteration", "1.0.0", &iteration_rules),
                ("acme/architecture", "Architecture", "2.0.0", &architecture_rules),
            ]),
            response: testing::lint_response(&[]),
            request: Mutex::new(None),
            workers: 2,
        });

        let external =
            ExternalLinter::initialize_transports([transport], PHPVersion::PHP85).expect("registration should succeed");

        assert_eq!(external.extensions().len(), 2);
        assert_eq!(external.extensions()[0].identifier, "acme/iteration");
        assert_eq!(external.extensions()[1].identifier, "acme/architecture");
        assert_eq!(external.rules().len(), 2);
        assert_eq!(external.rules()[0].extension, "acme/iteration");
        assert_eq!(external.rules()[1].extension, "acme/architecture");
    }
}
