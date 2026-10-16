#![allow(clippy::expect_used, clippy::missing_panics_doc)]

use std::borrow::Cow;
use std::num::NonZeroUsize;
use std::path::Path;
use std::sync::Arc;
use std::sync::LazyLock;

use mago_allocator::LocalArena;
use mago_analyzer::Analyzer;
use mago_analyzer::analysis_result::AnalysisResult;
use mago_analyzer::external::ExternalAnalysisSession;
use mago_analyzer::external::ExternalAnalyzer;
use mago_analyzer::external::ExternalAnalyzerHandle;
use mago_analyzer::plugin::PluginRegistry;
use mago_analyzer::settings::Settings;
use mago_codex::populator::populate_codebase;
use mago_codex::scanner::scan_program;
use mago_database::file::File;
use mago_database::file::FileType;
use mago_extension::WorkerCommand;
use mago_extension::WorkerPool;
use mago_extension::WorkerPoolOptions;
use mago_names::resolver::NameResolver;
use mago_php_version::PHPVersion;
use mago_prelude::Prelude;
use mago_syntax::parser::parse_file;
use mago_word::WordSet;

static PRELUDE: LazyLock<Prelude> = LazyLock::new(Prelude::build);

const SOURCE: &str = r"<?php

declare(strict_types=1);

class FallbackException {}

/** @throws FallbackException */
function external_throw(): void {}

/** @throws FallbackException */
function external_empty(): void {}

/** @throws FallbackException */
function external_fallback(): void {}

/** @throws FallbackException */
function external_disabled(): void {}

function external_caught(): void {}

function function_override(): void { external_throw(); }
function function_empty(): void { external_empty(); }
function function_fallback(): void { external_fallback(); }
function disabled_function_fallback(): void { external_disabled(); }
function function_caught(): void
{
    try {
        external_caught();
    } catch (\Throwable) {}
}

final class ThrowSubject
{
    /** @throws FallbackException */
    public function run(): void {}

    /** @throws FallbackException */
    public function disabledRun(): void {}
}

function method_override(): void { (new ThrowSubject())->run(); }
function disabled_method_fallback(): void { (new ThrowSubject())->disabledRun(); }
function expression_throw(): void { 1 + 1; }
function disabled_expression_fallback(): void { 2 + 2; }
";

#[test]
fn external_throw_type_providers_override_and_fall_back_to_native_contracts() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let available = repository.join("vendor/autoload.php").is_file()
        && std::process::Command::new("php").arg("--version").output().is_ok_and(|output| output.status.success());
    assert!(
        available || std::env::var_os("MAGO_REQUIRE_PHP_SDK_TESTS").is_none(),
        "PHP and vendor dependencies are required for the external throw-type provider test"
    );
    if !available {
        return;
    }

    let command = WorkerCommand::new("php")
        .with_argument(repository.join("composer/tests/Sdk/Fixtures/analyzer-throw-type-worker.php"))
        .with_current_directory(repository.clone());
    let pool = WorkerPool::spawn(command, NonZeroUsize::MIN, WorkerPoolOptions::default())
        .expect("failed to start the external analyzer worker pool");
    let external = ExternalAnalyzer::initialize([Arc::new(pool)], PHPVersion::PHP85, &[], false)
        .expect("failed to initialize the external analyzer");
    let mut registry = PluginRegistry::with_library_providers();
    registry.set_external_analyzer(Arc::new(ExternalAnalyzerHandle::ready(external)));
    registry.prepare_external_analyzer().expect("failed to prepare the external analyzer");

    let file = Arc::new(File::new(
        Cow::Borrowed(b"src/throw-types.php"),
        FileType::Host,
        None,
        Cow::Borrowed(SOURCE.as_bytes()),
    ));
    let arena = LocalArena::new();
    let program = parse_file(&arena, &file);
    assert!(!program.has_errors(), "the throw-type fixture should parse");
    let resolved_names = NameResolver::new(&arena).resolve(program);
    let mut codebase = PRELUDE.metadata.clone();
    codebase.extend(scan_program(&arena, &file, program, &resolved_names, PHPVersion::PHP85));
    let mut symbol_references = PRELUDE.symbol_references.clone();
    populate_codebase(&mut codebase, &mut symbol_references, WordSet::default(), foldhash::HashSet::default());

    let session = ExternalAnalysisSession::from_files([Arc::clone(&file)]);
    let mut settings = Settings::new(PHPVersion::PHP85);
    settings.find_unused_definitions = false;
    settings.find_unused_expressions = false;
    settings.find_unused_parameters = false;
    settings.check_throws = true;
    let analyzer = Analyzer::new(&arena, &file, &resolved_names, &codebase, &registry, settings)
        .with_external_analysis_session(&session);
    let mut result = AnalysisResult::new(symbol_references);
    analyzer.analyze(program, &mut result).expect("analysis with the external analyzer failed");

    let messages = result
        .issues
        .iter()
        .filter(|issue| issue.code.as_deref() == Some("unhandled-thrown-type"))
        .map(|issue| issue.message.to_string())
        .collect::<Vec<_>>();
    assert_eq!(messages.len(), 6, "unexpected unhandled-throw findings: {messages:#?}");
    assert!(
        messages.iter().any(|message| message.contains("RuntimeException") && message.contains("function_override"))
    );
    assert!(
        messages.iter().any(|message| message.contains("FallbackException") && message.contains("function_fallback"))
    );
    assert!(messages.iter().any(|message| message.contains("RuntimeException") && message.contains("method_override")));
    assert!(messages.iter().any(|message| message.contains("DomainException") && message.contains("expression_throw")));
    assert!(
        messages
            .iter()
            .any(|message| message.contains("FallbackException") && message.contains("disabled_function_fallback"))
    );
    assert!(
        messages
            .iter()
            .any(|message| message.contains("FallbackException") && message.contains("disabled_method_fallback"))
    );
    assert!(!messages.iter().any(|message| message.contains("function_empty")));
    assert!(!messages.iter().any(|message| message.contains("function_caught")));
    assert!(!messages.iter().any(|message| message.contains("disabled_expression_fallback")));
}
