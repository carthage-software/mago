#![allow(clippy::expect_used, clippy::missing_panics_doc)]

use std::borrow::Cow;
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
use mago_database::file::FileType;
use mago_extension::WorkerCommand;
use mago_extension::WorkerPool;
use mago_extension::WorkerPoolOptions;
use mago_orchestrator::service::incremental_analysis::IncrementalAnalysisService;
use mago_php_version::PHPVersion;
use mago_syntax::settings::ParserSettings;

const SOURCE: &str = r"<?php

declare(strict_types=1);

/** @template T */
class Builder
{
    /** @return T */
    public function first(): mixed
    {
        return null;
    }
}

/** @property string $magic */
class BaseModel
{
    public function __get(string $property): mixed
    {
        return null;
    }

    public function __set(string $property, mixed $value): void {}

    public static function query(): mixed
    {
        return null;
    }

    public function newQuery(): mixed
    {
        return null;
    }

    public static function echoArgument(string $value): mixed
    {
        return null;
    }

    public static function acceptString(array $value): mixed
    {
        return null;
    }

    public static function late(): mixed
    {
        return null;
    }

    public static function magicValues(): mixed
    {
        return null;
    }

    public static function callLate(): mixed
    {
        return static::late();
    }

    public function passthrough(): object
    {
        return $this;
    }
}

final class User extends BaseModel {}

class Relation
{
    public function __call(string $name, array $arguments): mixed
    {
        return null;
    }
}

class HasMany extends Relation {}

final class CustomRelation extends HasMany {}

interface Marker {}

class DynamicProxy
{
    public function __call(string $name, array $arguments): mixed
    {
        return null;
    }
}

class DynamicFacade
{
    public static function __callStatic(string $name, array $arguments): mixed
    {
        return null;
    }
}

class ClosureCommand
{
    public function comment(string $message): void {}
}

class Artisan
{
    public static function command(string $signature, Closure $callback): void {}
}

function external_function(): mixed
{
    return null;
}

function collect(array $value): mixed
{
    return null;
}

/** @param Builder<User> $_builder */
function take_user_builder(Builder $_builder): void {}

function take_user(User $_user): void {}

function take_string(string $_value): void {}

function take_int(int $_value): void {}

function take_intersection(BaseModel&Marker $_model): void {}

/** @param array<int, string> $_values */
function take_string_map(array $_values): void {}

/** @param list<Builder<string>> $_builders */
function take_string_builders(array $_builders): void {}

take_string(external_function());
take_string(collect('function'));
Artisan::command('inspire', function (): void {
    $this->comment('Be inspired!');
});
take_user_builder(User::query());
take_string(User::echoArgument('value'));
take_string(User::acceptString('method'));
take_string_builders(User::magicValues());
take_string((new DynamicProxy())->dynamic('instance'));
take_string((new DynamicProxy())->acceptString('dynamic-signature'));
take_string_map(DynamicFacade::dynamic('static'));
take_string((new CustomRelation())->where());
User::callLate();

$user = new User();
take_user_builder($user->newQuery());
take_string($user->name);
$user->name = 123;
take_string($user->name);
take_int($user->id);
take_user($user->self);
$user->secret = 'token';
// @mago-expect analysis:invalid-property-write
$user->id = 'forbidden';
// @mago-expect analysis:invalid-property-read
// @mago-expect analysis:no-value
echo $user->secret;

/** @var Builder<User> $builder */
$builder = new Builder();
take_user($builder->first());

/** @param BaseModel&Marker $model */
function inspect_intersection(BaseModel $model): void
{
    take_intersection($model->passthrough());
}
";

const NO_METHOD_PROVIDER_SOURCE: &str = r"<?php

declare(strict_types=1);

class BaseModel
{
    public static function query(): mixed
    {
        return null;
    }
}

final class User extends BaseModel {}

User::query();
";

const UNDOCUMENTED_METHOD_SOURCE: &str = r"<?php

declare(strict_types=1);

final class DynamicProxy
{
    /** @return static */
    public function __call(string $name, array $arguments): mixed
    {
        return $this;
    }
}

final class DynamicFacade
{
    /** @return static */
    public static function __callStatic(string $name, array $arguments): mixed
    {
        return new static();
    }
}

function take_proxy(DynamicProxy $_proxy): void {}

function take_facade(DynamicFacade $_facade): void {}

take_proxy((new DynamicProxy())->dynamic('instance'));
take_facade(DynamicFacade::dynamic('static'));
";

#[test]
fn external_providers_receive_complete_invocation_context() -> Result<(), Box<dyn std::error::Error>> {
    let observation = analyze_with_fixture(SOURCE, false)?;
    assert!(
        observation.issues.is_empty(),
        "complete external invocation context should preserve precise types: {:#?}",
        observation.issues
    );

    let mut invocations = observation.invocations;
    invocations.sort_unstable();
    assert_eq!(
        invocations,
        [
            "argument-handle",
            "closure-this-return",
            "closure-this-signature",
            "complete-composite",
            "dynamic-instance",
            "dynamic-signature",
            "dynamic-signature-return",
            "dynamic-static",
            "function",
            "function-signature",
            "function-signature-return",
            "generic",
            "instance",
            "intersection",
            "late-static",
            "method-signature",
            "method-signature-return",
            "property-id-read",
            "property-id-write",
            "property-name-read",
            "property-name-write",
            "property-secret-read",
            "property-secret-write",
            "property-self-read",
            "relation-subclass",
            "static"
        ]
    );

    Ok(())
}

#[test]
fn analyzer_without_method_providers_sends_no_method_request() -> Result<(), Box<dyn std::error::Error>> {
    let observation = analyze_with_fixture(NO_METHOD_PROVIDER_SOURCE, true)?;
    assert!(observation.issues.is_empty(), "native method analysis should remain unchanged: {:#?}", observation.issues);
    assert!(observation.invocations.is_empty(), "a host without method providers must not receive method requests");

    Ok(())
}

#[test]
fn declined_dynamic_methods_preserve_non_documented_method_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
    let observation = analyze_with_fixture(UNDOCUMENTED_METHOD_SOURCE, true)?;
    assert_eq!(
        observation.issues,
        [
            (
                Some("non-documented-method".to_owned()),
                "Ambiguous method call to `dynamic` on class `DynamicProxy`.".to_owned()
            ),
            (
                Some("non-documented-method".to_owned()),
                "Ambiguous method call to `dynamic` on class `DynamicFacade`.".to_owned()
            ),
        ]
    );
    assert!(observation.invocations.is_empty(), "a host without method providers must receive no dynamic requests");

    Ok(())
}

fn analyze_with_fixture(source: &str, function_only: bool) -> Result<AnalysisObservation, Box<dyn std::error::Error>> {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"));
    let sdk_available = repository.join("vendor/autoload.php").is_file()
        && Command::new("php").arg("--version").output().is_ok_and(|output| output.status.success());
    assert!(
        sdk_available || std::env::var_os("MAGO_REQUIRE_PHP_SDK_TESTS").is_none(),
        "PHP and vendor dependencies are required for the external invocation-context test"
    );
    if !sdk_available {
        return Ok(AnalysisObservation { issues: Vec::new(), invocations: Vec::new() });
    }

    let temporary = tempfile::tempdir()?;
    let audit = temporary.path().join("invocations.txt");
    std::fs::write(&audit, [])?;
    let mut command = WorkerCommand::new("php")
        .with_argument(repository.join("composer/tests/Sdk/Fixtures/analyzer-invocation-worker.php"))
        .with_current_directory(repository)
        .with_environment("MAGO_INVOCATION_AUDIT_LOG", &audit);
    if function_only {
        command = command.with_environment("MAGO_INVOCATION_FUNCTION_ONLY", "1");
    }
    let pool = WorkerPool::spawn(command, NonZeroUsize::MIN, WorkerPoolOptions::default())?;
    let external = ExternalAnalyzer::initialize([Arc::new(pool)], PHPVersion::PHP85, &[], false)?;
    let mut registry = PluginRegistry::with_library_providers();
    registry.set_external_analyzer(Arc::new(ExternalAnalyzerHandle::ready(external)));

    let configuration = DatabaseConfiguration {
        workspace: Cow::Owned(Path::new("/invocation-proof").to_path_buf()),
        paths: vec![Cow::Borrowed(b"src")],
        includes: vec![],
        patches: vec![],
        excludes: vec![],
        extensions: vec![Cow::Borrowed(b"php")],
        glob: GlobSettings::default(),
    };
    let mut database = Database::new(configuration);
    database.add(File::new(
        Cow::Borrowed(b"src/invocation.php"),
        FileType::Host,
        None,
        Cow::Owned(source.as_bytes().to_vec()),
    ));

    let mut settings = Settings::new(PHPVersion::PHP85);
    settings.find_unused_parameters = false;
    let mut service = IncrementalAnalysisService::new(
        database.read_only(),
        CodebaseMetadata::new(),
        SymbolReferences::new(),
        settings,
        ParserSettings::default(),
        Arc::new(registry),
    );
    let result = service.analyze()?;
    let issues = result.issues.iter().map(|issue| (issue.code.clone(), issue.message.clone())).collect::<Vec<_>>();
    let invocations = std::fs::read_to_string(audit)?.lines().map(str::to_owned).collect::<Vec<_>>();

    Ok(AnalysisObservation { issues, invocations })
}

struct AnalysisObservation {
    issues: Vec<(Option<String>, String)>,
    invocations: Vec<String>,
}
