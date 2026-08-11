//! Worker-backed custom linter rules.
//!
//! A pool is queried once for immutable rule metadata. Linting then sends one
//! coarse-grained request per file and extension, containing source text, a
//! stable flat syntax tree, comments, and resolved names. Node callbacks are
//! dispatched inside the worker rather than crossing IPC one node at a time.

use std::collections::HashSet;
use std::sync::Arc;

use mago_database::file::File;
use mago_extension::WorkerError;
use mago_extension::WorkerPool;
use mago_names::ResolvedNames;
use mago_php_version::PHPVersion;
use mago_reporting::IssueCollection;
use mago_reporting::Level;
use mago_syntax::cst::NodeKind;
use mago_syntax::cst::Program;

pub use error::ExternalLintError;
use protocol::Registration;

mod error;
pub mod protocol;

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
    /// Severity used unless extension configuration overrides it.
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
}

/// A set of custom linter rules backed by one or more extension worker pools.
#[derive(Debug)]
pub struct ExternalLinter<T = WorkerPool> {
    php_version: PHPVersion,
    backends: Box<[Backend<T>]>,
    extensions: Box<[ExternalExtension]>,
    rules: Box<[ExternalRule]>,
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
        Self::initialize_transports(pools, php_version)
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
        let mut issues = IssueCollection::new();
        for backend in &self.backends {
            let mut active_rules = Vec::new();
            let mut target_kinds = [false; u8::MAX as usize + 1];
            for rule in &backend.registration.rules {
                if only.map_or(rule.default_enabled, |codes| codes.iter().any(|code| code == &rule.code)) {
                    active_rules.push(rule.code.as_str());
                    for target in &rule.targets {
                        target_kinds[*target as usize] = true;
                    }
                }
            }

            if active_rules.is_empty() {
                continue;
            }

            let Some(payload) = protocol::encode_lint_request(
                self.php_version,
                file,
                program,
                resolved_names,
                &active_rules,
                &target_kinds,
            )?
            else {
                continue;
            };
            let response = backend.transport.request(payload)?;
            issues.extend(protocol::decode_lint_response(&response, file, &active_rules)?);
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
        let describe = protocol::encode_describe_request(php_version);
        let mut backends = Vec::new();
        let mut extensions = Vec::new();
        let mut rules = Vec::new();
        let mut extension_identifiers = HashSet::new();
        let mut rule_codes = HashSet::new();

        for transport in transports {
            let responses = transport.broadcast(&describe)?;
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

            if !extension_identifiers.insert(registration.identifier.clone()) {
                return Err(ExternalLintError::DuplicateExtension(registration.identifier));
            }

            for rule in &registration.rules {
                if !rule_codes.insert(rule.code.clone()) {
                    return Err(ExternalLintError::DuplicateRule(rule.code.clone()));
                }
            }

            extensions.push(ExternalExtension {
                identifier: registration.identifier.clone(),
                name: registration.name.clone(),
                version: registration.version.clone(),
                rules: registration.rules.clone(),
            });

            rules.extend(registration.rules.iter().cloned());
            backends.push(Backend { transport, registration });
        }

        Ok(Self {
            php_version,
            backends: backends.into_boxed_slice(),
            extensions: extensions.into_boxed_slice(),
            rules: rules.into_boxed_slice(),
        })
    }
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
            .with_annotation(Annotation::primary(target_span).with_message("Call occurs here"));
        let transport = Arc::new(MockTransport {
            registration: testing::describe_response(
                "acme/tools",
                "Acme Tools",
                "1.0.0",
                &[("acme/no-run", "No run", "Disallows this call.", Level::Warning, true, &[NodeKind::FunctionCall])],
            ),
            response: testing::lint_response(&[issue]),
            request: Mutex::new(None),
            workers: 3,
        });
        let external = ExternalLinter::initialize_transports([Arc::clone(&transport)], PHPVersion::PHP85)
            .expect("registration should succeed");

        let issues = external.lint(&file, program, &resolved_names, None).expect("external lint should succeed");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues.iter().next().and_then(|issue| issue.code.as_deref()), Some("acme/no-run"));

        let request = transport.request.lock().unwrap();
        let request = request.as_ref().expect("one request should be captured");
        assert_eq!(request.file_name, b"src/test.php");
        assert_eq!(request.source, source);
        assert_eq!(request.active_rules, ["acme/no-run"]);
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
            response: testing::lint_response(&[issue]),
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
}
