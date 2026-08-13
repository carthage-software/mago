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
use mago_database::DatabaseReader;
use mago_database::GlobSettings;
use mago_database::file::File;
use mago_database::file::FileId;
use mago_database::file::FileType;
use mago_extension::WorkerCommand;
use mago_extension::WorkerPool;
use mago_extension::WorkerPoolOptions;
use mago_formatter::settings::FormatSettings;
use mago_guard::settings::Settings as GuardSettings;
use mago_linter::settings::Settings as LinterSettings;
use mago_orchestrator::Orchestrator;
use mago_orchestrator::OrchestratorConfiguration;
use mago_orchestrator::service::incremental_analysis::IncrementalAnalysisService;
use mago_php_version::PHPVersion;
use mago_reporting::IssueCollection;
use mago_syntax::settings::ParserSettings;
use mago_text_edit::Safety;
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

fn make_database(changed_file: Option<usize>, external_sources: &[(Vec<u8>, Vec<u8>)]) -> (Database<'static>, FileId) {
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
        let extension_usage = if index == 0 {
            "\nfunction extension_consumer(ExtensionProvided $provided): int {\n    return $provided->answer() + extension_answer(0) + EXTENSION_ANSWER;\n}\n"
        } else {
            ""
        };
        let inheritance = if index == 0 { " extends ExtensionProvided" } else { "" };
        let contents = format!(
            "<?php\n\ndeclare(strict_types=1);\n\nfinal class LifecycleClass{index}{inheritance} {{\n    public function value(): int {{ return {value}; }}\n}}\n\nfunction lifecycle_function_{index}(int $value): int {{ return $value + {value}; }}\n{extension_usage}{generator}"
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

    for (name, contents) in external_sources {
        database.add(File::new(Cow::Owned(name.clone()), FileType::External, None, Cow::Owned(contents.clone())));
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
    let final_edits = final_issue
        .edits
        .get(&final_issue.annotations[0].span.file_id)
        .expect("the project-wide edit should resolve its named source file");
    assert_eq!(final_edits.len(), 1);
    assert_eq!(final_edits[0].range.start, final_issue.annotations[0].span.start.offset);
    assert_eq!(final_edits[0].range.end, final_issue.annotations[0].span.end.offset);
    assert!(final_edits[0].new_text.is_empty());
    assert_eq!(final_edits[0].safety, Safety::Unsafe);

    for issue in issues.iter().filter(|issue| issue.code.as_deref().is_some_and(|code| code.ends_with("/after-file"))) {
        assert_eq!(issue.edits.len(), 1);
        let (file_id, edits) = issue.edits.iter().next().expect("an after-file issue should contain one edit batch");
        assert_eq!(*file_id, issue.annotations[0].span.file_id);
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].range.start, 0);
        assert_eq!(edits[0].range.end, 5);
        assert_eq!(edits[0].new_text, b"<?php");
        assert_eq!(edits[0].safety, Safety::PotentiallyUnsafe);
    }
}

fn assert_initial_audit(entries: &[AuditEntry]) {
    assert_eq!(entries.iter().filter(|entry| entry.1 == "initialize").count(), PLUGINS.len() * 3);
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
    for plugin in PLUGINS {
        let initialization_workers = entries
            .iter()
            .filter(|entry| entry.0 == plugin && entry.1 == "initialize")
            .map(|entry| entry.3)
            .collect::<HashSet<_>>();
        assert_eq!(initialization_workers.len(), 3, "initialization must run on every worker");
    }
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
    let initialization_sources = analyzer
        .initialization_files()
        .into_iter()
        .map(|file| (file.name.into_owned(), file.contents.into_owned()))
        .collect::<Vec<_>>();
    assert_eq!(initialization_sources.len(), 1);
    let mut registry = PluginRegistry::with_library_providers();
    registry.set_external_analyzer(Arc::new(ExternalAnalyzerHandle::ready(analyzer)));

    let (database, _) = make_database(None, &initialization_sources);
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
    let unexpected = initial
        .issues
        .iter()
        .filter(|issue| issue.code.as_deref().is_none_or(|code| !code.starts_with("lifecycle-")))
        .map(|issue| (issue.code.as_deref(), issue.message.as_str()))
        .collect::<Vec<_>>();
    assert!(unexpected.is_empty(), "unexpected analyzer issues: {unexpected:#?}");
    assert_issue_cardinality(&initial.issues);
    let external_annotation = initial
        .issues
        .iter()
        .find(|issue| issue.code.as_deref() == Some("lifecycle-one/after"))
        .expect("the final lifecycle issue should exist")
        .annotations[1]
        .span;
    assert!(
        service.database().get_ref(&external_annotation.file_id).is_ok_and(|file| file.file_type.is_external()),
        "reporting must be able to resolve annotations into external stubs"
    );

    let initial_audit = read_audit(&audit_log);
    assert_eq!(initial_audit.len(), (PLUGINS.len() * 3) + 2 + (FILE_COUNT * 2) + 2);
    assert_initial_audit(&initial_audit);

    let (updated_database, changed_file) = make_database(Some(5), &initialization_sources);
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

    let first_changed_file = changed_file;
    let (second_updated_database, second_changed_file) = make_database(Some(6), &initialization_sources);
    service.update_database(second_updated_database.read_only());
    let second_incremental = service
        .analyze_incremental(Some(&[first_changed_file, second_changed_file]))
        .expect("a second incremental lifecycle analysis should rebuild extension metadata from source state");
    assert_issue_cardinality(&second_incremental.issues);

    let second_audit = read_audit(&audit_log);
    let second_incremental_audit = &second_audit[audit.len()..];
    assert_eq!(second_incremental_audit.iter().filter(|entry| entry.1 == "before").count(), PLUGINS.len());
    assert_eq!(second_incremental_audit.iter().filter(|entry| entry.1 == "after-file").count(), PLUGINS.len() * 2);
    assert_eq!(second_incremental_audit.iter().filter(|entry| entry.1 == "after").count(), PLUGINS.len());
    let analyzed_files = second_incremental_audit
        .iter()
        .filter(|entry| entry.1 == "after-file")
        .filter_map(|entry| entry.2.as_deref())
        .collect::<HashSet<_>>();
    assert_eq!(analyzed_files, HashSet::from(["src/file5.php", "src/file6.php"]));
}

#[test]
fn orchestrator_loads_initialization_stubs_into_the_source_database() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"));
    if !php_sdk_is_available(repository) {
        return;
    }

    let temporary = tempfile::tempdir().expect("temporary lifecycle directory should be created");
    let audit_log = temporary.path().join("audit.jsonl");
    std::fs::write(&audit_log, []).expect("lifecycle audit log should be initialized");
    std::fs::write(temporary.path().join("host.php"), "<?php\nfinal class HostClass {}\n")
        .expect("host source should be created");

    let command = WorkerCommand::new("php")
        .with_argument(repository.join("composer/tests/Sdk/Fixtures/analyzer-lifecycle-worker.php"))
        .with_current_directory(repository)
        .with_environment("MAGO_LIFECYCLE_AUDIT_LOG", &audit_log);
    let pool = WorkerPool::spawn(command, NonZeroUsize::MIN, WorkerPoolOptions::default())
        .expect("PHP lifecycle worker pool should start");
    let analyzer = ExternalAnalyzer::initialize([Arc::new(pool)], PHPVersion::PHP85, &[], false)
        .expect("PHP lifecycle analyzer should initialize");
    let orchestrator = Orchestrator::new(OrchestratorConfiguration {
        php_version: PHPVersion::PHP85,
        paths: vec![],
        includes: vec![],
        patches: vec![],
        excludes: vec![],
        extensions: vec!["php"],
        glob: GlobSettings::default(),
        parser_settings: ParserSettings::default(),
        analyzer_settings: Settings::new(PHPVersion::PHP85),
        linter_settings: LinterSettings::default(),
        guard_settings: GuardSettings::default(),
        formatter_settings: FormatSettings::default(),
        disable_default_analyzer_plugins: false,
        analyzer_plugins: vec![],
        use_progress_bars: false,
        use_colors: false,
    });

    orchestrator.set_external_analyzer(analyzer);

    let database = orchestrator
        .load_database(temporary.path(), true, None, None)
        .expect("database loading should include initialization stubs");
    let external = database.files().filter(|file| file.file_type.is_external()).collect::<Vec<_>>();
    assert_eq!(external.len(), 1);
    assert!(external[0].path.is_none());
    assert!(external[0].name.starts_with(b"@mago-extension/"));
    assert!(
        external[0]
            .contents
            .windows(b"class ExtensionProvided".len())
            .any(|bytes| { bytes == b"class ExtensionProvided" })
    );
}
