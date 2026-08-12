#![allow(clippy::expect_used, clippy::indexing_slicing, clippy::missing_panics_doc, clippy::panic, clippy::unwrap_used)]

use std::borrow::Cow;
use std::collections::HashSet;
use std::num::NonZeroUsize;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use mago_analyzer::external::ExternalAnalyzer;
use mago_analyzer::external::ExternalAnalyzerHandle;
use mago_analyzer::plugin::PluginRegistry;
use mago_analyzer::settings::Settings;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::reference::SymbolReferences;
use mago_database::Database;
use mago_database::DatabaseConfiguration;
use mago_database::GlobSettings;
use mago_database::file::File;
use mago_database::file::FileId;
use mago_database::file::FileType;
use mago_extension::WorkerCommand;
use mago_extension::WorkerPool;
use mago_extension::WorkerPoolOptions;
use mago_orchestrator::service::incremental_analysis::IncrementalAnalysisService;
use mago_php_version::PHPVersion;
use mago_reporting::IssueCollection;
use mago_syntax::settings::ParserSettings;
use serde::Deserialize;

const FILE_COUNT: usize = 96;
const PLUGINS: [&str; 2] = ["lifecycle-one", "lifecycle-two"];

#[derive(Debug, Deserialize)]
struct AuditEntry(String, String, Option<String>, u32);

fn php_sdk_is_available(repository: &Path) -> bool {
    let available = repository.join("vendor/autoload.php").is_file()
        && Command::new("php").arg("--version").output().is_ok_and(|output| output.status.success());
    assert!(
        available || std::env::var_os("MAGO_REQUIRE_PHP_SDK_TESTS").is_none(),
        "PHP and vendor dependencies are required for the external analyzer lifecycle test"
    );
    available
}

fn make_database(changed_file: Option<usize>) -> (Database<'static>, FileId) {
    let configuration = DatabaseConfiguration {
        workspace: Cow::Owned(Path::new("/lifecycle-proof").to_path_buf()),
        paths: vec![Cow::Borrowed(b"src")],
        includes: vec![],
        patches: vec![],
        excludes: vec![],
        extensions: vec![Cow::Borrowed(b"php")],
        glob: GlobSettings::default(),
    };
    let mut database = Database::new(configuration);
    let mut changed_id = FileId::zero();
    for index in 0..FILE_COUNT {
        let value = if changed_file == Some(index) { index + 100 } else { index };
        let generator = if index == FILE_COUNT - 1 {
            format!("\nfunction lifecycle_generator(): iterable {{ yield {index} => 'value'; }}\n")
        } else {
            String::new()
        };
        let contents = format!(
            "<?php\n\ndeclare(strict_types=1);\n\nfinal class LifecycleClass{index} {{\n    public function value(): int {{ return {value}; }}\n}}\n\nfunction lifecycle_function_{index}(int $value): int {{ return $value + {value}; }}\n{generator}"
        );
        let file = File::new(
            Cow::Owned(format!("src/file{index}.php").into_bytes()),
            FileType::Host,
            None,
            Cow::Owned(contents.into_bytes()),
        );
        if index == changed_file.unwrap_or_default() {
            changed_id = file.id;
        }
        database.add(file);
    }

    (database, changed_id)
}

fn read_audit(path: &Path) -> Vec<AuditEntry> {
    std::fs::read_to_string(path)
        .expect("lifecycle audit log should be readable")
        .lines()
        .map(|line| serde_json::from_str(line).expect("lifecycle audit entry should be valid JSON"))
        .collect()
}

fn count_code(issues: &IssueCollection, code: &str) -> usize {
    issues.iter().filter(|issue| issue.code.as_deref() == Some(code)).count()
}

fn assert_issue_cardinality(issues: &IssueCollection) {
    for plugin in PLUGINS {
        assert_eq!(count_code(issues, &format!("{plugin}/before")), 1);
        assert_eq!(count_code(issues, &format!("{plugin}/after-file")), FILE_COUNT);
        assert_eq!(count_code(issues, &format!("{plugin}/after")), 1);
    }

    let final_issue = issues
        .iter()
        .find(|issue| issue.code.as_deref() == Some("lifecycle-one/after"))
        .expect("the final lifecycle issue should exist");
    assert_eq!(final_issue.annotations.len(), 2);
    assert_ne!(final_issue.annotations[0].span.file_id, final_issue.annotations[1].span.file_id);
}

fn assert_initial_audit(entries: &[AuditEntry]) {
    assert_eq!(entries.iter().filter(|entry| entry.1 == "before").count(), PLUGINS.len());
    assert_eq!(entries.iter().filter(|entry| entry.1 == "after-file").count(), FILE_COUNT * PLUGINS.len());
    assert_eq!(entries.iter().filter(|entry| entry.1 == "after").count(), PLUGINS.len());

    let callbacks = entries
        .iter()
        .filter(|entry| entry.1 == "after-file")
        .map(|entry| (entry.0.as_str(), entry.2.as_deref()))
        .collect::<HashSet<_>>();
    assert_eq!(callbacks.len(), FILE_COUNT * PLUGINS.len());

    let workers = entries.iter().filter(|entry| entry.1 == "after-file").map(|entry| entry.3).collect::<HashSet<_>>();
    assert!(workers.len() > 1, "per-file hooks should use more than one worker");
}

#[test]
fn external_analyzer_lifecycle_is_exact_across_workers_and_incremental_runs() {
    if std::env::var_os("MAGO_TRACE_PHP_SDK_TESTS").is_some() {
        let _subscriber = tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .with_test_writer()
            .try_init();
    }

    let repository = Path::new(env!("CARGO_MANIFEST_DIR"));
    if !php_sdk_is_available(repository) {
        return;
    }

    let temporary = tempfile::tempdir().expect("temporary lifecycle directory should be created");
    let audit_log = temporary.path().join("audit.jsonl");
    std::fs::write(&audit_log, []).expect("lifecycle audit log should be initialized");

    let command = WorkerCommand::new("php")
        .with_argument(repository.join("composer/tests/Sdk/Fixtures/analyzer-lifecycle-worker.php"))
        .with_current_directory(repository)
        .with_environment("MAGO_LIFECYCLE_AUDIT_LOG", &audit_log);
    let pool =
        WorkerPool::spawn(command, NonZeroUsize::new(3).expect("three is non-zero"), WorkerPoolOptions::default())
            .expect("PHP lifecycle worker pool should start");
    let analyzer = ExternalAnalyzer::initialize([Arc::new(pool)], PHPVersion::PHP85, &[], false)
        .expect("PHP lifecycle analyzer should initialize");
    let mut registry = PluginRegistry::with_library_providers();
    registry.set_external_analyzer(Arc::new(ExternalAnalyzerHandle::ready(analyzer)));

    let (database, _) = make_database(None);
    let mut settings = Settings::new(PHPVersion::PHP85);
    settings.find_unused_expressions = false;
    settings.find_unused_definitions = false;
    let mut service = IncrementalAnalysisService::new(
        database.read_only(),
        CodebaseMetadata::new(),
        SymbolReferences::new(),
        settings,
        ParserSettings::default(),
        Arc::new(registry),
    );

    let initial = service.analyze().expect("initial lifecycle analysis should succeed");
    assert_issue_cardinality(&initial.issues);
    let initial_audit = read_audit(&audit_log);
    assert_eq!(initial_audit.len(), 2 + (FILE_COUNT * 2) + 2);
    assert_initial_audit(&initial_audit);

    let (updated_database, changed_file) = make_database(Some(5));
    service.update_database(updated_database.read_only());
    let incremental =
        service.analyze_incremental(Some(&[changed_file])).expect("incremental lifecycle analysis should succeed");
    assert_issue_cardinality(&incremental.issues);

    let audit = read_audit(&audit_log);
    let incremental_audit = &audit[initial_audit.len()..];
    assert_eq!(incremental_audit.iter().filter(|entry| entry.1 == "before").count(), PLUGINS.len());
    assert_eq!(incremental_audit.iter().filter(|entry| entry.1 == "after-file").count(), PLUGINS.len());
    assert_eq!(incremental_audit.iter().filter(|entry| entry.1 == "after").count(), PLUGINS.len());
    assert!(
        incremental_audit
            .iter()
            .filter(|entry| entry.1 == "after-file")
            .all(|entry| entry.2.as_deref() == Some("src/file5.php"))
    );
}
