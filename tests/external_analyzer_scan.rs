#![allow(clippy::expect_used, clippy::missing_panics_doc)]

use std::borrow::Cow;
use std::collections::BTreeMap;
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
use mago_syntax::settings::ParserSettings;

type AuditEntry = (u32, BTreeMap<String, Vec<String>>);

fn php_sdk_is_available(repository: &Path) -> bool {
    repository.join("vendor/autoload.php").is_file()
        && Command::new("php").arg("--version").output().is_ok_and(|output| output.status.success())
}

fn database(first: &str, ignored: usize) -> (Database<'static>, FileId, FileId) {
    let configuration = DatabaseConfiguration {
        workspace: Cow::Owned(Path::new("/scan-proof").to_path_buf()),
        paths: vec![Cow::Borrowed(b".")],
        includes: vec![],
        patches: vec![],
        excludes: vec![],
        extensions: vec![Cow::Borrowed(b"php")],
        glob: GlobSettings::default(),
    };
    let mut database = Database::new(configuration);
    let first = File::new(
        Cow::Borrowed(b"database/migrations/001.php"),
        FileType::Host,
        None,
        Cow::Owned(format!("<?php\n\n$value = \"{first}\";\n").into_bytes()),
    );
    let first_id = first.id;
    database.add(first);
    database.add(File::new(
        Cow::Borrowed(b"database/migrations/nested/002.php"),
        FileType::Host,
        None,
        Cow::Borrowed(b"<?php\n\n$value = 'two';\n"),
    ));
    let ignored = File::new(
        Cow::Borrowed(b"src/ignored.php"),
        FileType::Host,
        None,
        Cow::Owned(format!("<?php\n\nconst IGNORED = {ignored};\n").into_bytes()),
    );
    let ignored_id = ignored.id;
    database.add(ignored);
    database.add(File::new(
        Cow::Borrowed(b"database/migrations/003.php"),
        FileType::Vendored,
        None,
        Cow::Borrowed(b"<?php\n\n$value = 'vendor';\n"),
    ));

    (database, first_id, ignored_id)
}

fn audit(path: &Path) -> Vec<AuditEntry> {
    std::fs::read_to_string(path)
        .expect("scan audit should be readable")
        .lines()
        .map(|line| serde_json::from_str(line).expect("scan audit should contain JSON records"))
        .collect()
}

#[test]
fn filtered_first_parse_snapshots_refresh_every_worker_incrementally() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"));
    if !php_sdk_is_available(repository) {
        return;
    }

    let temporary = tempfile::tempdir().expect("temporary scan directory should be created");
    let audit_log = temporary.path().join("audit.jsonl");
    std::fs::write(&audit_log, []).expect("scan audit should be initialized");
    let command = WorkerCommand::new("php")
        .with_argument(repository.join("composer/tests/Sdk/Fixtures/analyzer-scan-worker.php"))
        .with_current_directory(repository)
        .with_environment("MAGO_SCAN_AUDIT_LOG", &audit_log);
    let pool = WorkerPool::spawn(command, NonZeroUsize::new(3).expect("three workers"), WorkerPoolOptions::default())
        .expect("scan worker pool should start");
    let analyzer = ExternalAnalyzer::initialize([Arc::new(pool)], PHPVersion::PHP85, &[], false)
        .expect("scan analyzer should initialize");
    let mut registry = PluginRegistry::with_library_providers();
    registry.set_external_analyzer(Arc::new(ExternalAnalyzerHandle::ready(analyzer)));

    let (initial, migration_id, ignored_id) = database("one\\n", 1);
    let mut settings = Settings::new(PHPVersion::PHP85);
    settings.find_unused_definitions = false;
    settings.find_unused_expressions = false;
    let mut service = IncrementalAnalysisService::new(
        initial.read_only(),
        CodebaseMetadata::new(),
        SymbolReferences::new(),
        settings,
        ParserSettings::default(),
        Arc::new(registry),
    );

    service.analyze().expect("initial analysis should scan selected files");
    let initial_audit = audit(&audit_log);
    assert_eq!(initial_audit.len(), 3);
    for (_, files) in &initial_audit {
        assert_eq!(files.len(), 2);
        assert_eq!(files["database/migrations/001.php"], ["one\n"]);
        assert_eq!(files["database/migrations/nested/002.php"], ["two"]);
    }

    let (changed, _, _) = database("changed\\t", 1);
    service.update_database(changed.read_only());
    service.analyze_incremental(Some(&[migration_id])).expect("a selected source change should refresh every worker");
    let changed_audit = audit(&audit_log);
    assert_eq!(changed_audit.len(), 6);
    for (_, files) in &changed_audit[3..] {
        assert_eq!(files["database/migrations/001.php"], ["changed\t"]);
    }

    let (ignored, _, _) = database("changed\\t", 2);
    service.update_database(ignored.read_only());
    service.analyze_incremental(Some(&[ignored_id])).expect("an unrelated source change should remain analyzable");
    assert_eq!(audit(&audit_log).len(), 6, "unmatched changes must not cross the worker boundary");
}
