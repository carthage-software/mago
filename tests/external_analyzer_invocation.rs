#![allow(clippy::expect_used, clippy::missing_panics_doc)]

use std::borrow::Cow;
use std::num::NonZeroUsize;
use std::path::Path;
use std::sync::Arc;

use mago_analyzer::external::ExternalAnalyzer;
use mago_analyzer::external::ExternalAnalyzerHandle;
use mago_analyzer::plugin::PluginRegistry;
use mago_analyzer::settings::Settings;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::reference::SymbolReferences;
use mago_database::Database;
use mago_database::file::File;
use mago_database::file::FileType;
use mago_extension::WorkerCommand;
use mago_extension::WorkerPool;
use mago_extension::WorkerPoolOptions;
use mago_orchestrator::service::incremental_analysis::IncrementalAnalysisService;
use mago_php_version::PHPVersion;
use mago_syntax::settings::ParserSettings;

mod common;

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

interface ExternalInterface {}

final class ExternalResult
{
    public function path(): string
    {
        return '';
    }
}

final class ExternalService {}

final class ExternalStaticService {}

final class ExternalAssertions
{
    public static function isString(mixed $value): bool
    {
        return true;
    }
}

final class Attribute
{
    public const int TARGET_METHOD = 4;

    public function __construct(public int $flags = 0) {}
}

#[Attribute(Attribute::TARGET_METHOD)]
final class FrameworkInitializer {}

class FrameworkTestCase
{
    protected ExternalResult $framework;

    #[FrameworkInitializer]
    protected function setUp(): void
    {
        $this->framework = new ExternalResult();
    }

    public function getFramework(): ExternalResult
    {
        return $this->framework;
    }
}

final class ManagedTestCase extends FrameworkTestCase
{
    public ExternalResult $application;

    private ExternalResult $native;

    private ExternalResult $lifecycle;

    public function __construct()
    {
        $this->native = new ExternalResult();
    }

    public function getApplication(): ExternalResult
    {
        return $this->application;
    }

    public function getNative(): ExternalResult
    {
        return $this->native;
    }

    #[FrameworkInitializer]
    protected function setUp(): void
    {
        parent::setUp();
        $this->lifecycle = new ExternalResult();
    }

    public function getLifecycle(): ExternalResult
    {
        return $this->lifecycle;
    }
}

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

function external_assert_string(mixed $value): void {}

function collect(array $value): mixed
{
    return null;
}

/** @param Builder<User> $_builder */
function take_user_builder(Builder $_builder): void {}

function take_user(User $_user): void {}

function take_string(string $_value): void {}

function take_int(int $_value): void {}

function take_external_result(ExternalResult $_result): void {}

function take_intersection(BaseModel&Marker $_model): void {}

/** @param array<int, string> $_values */
function take_string_map(array $_values): void {}

/** @param list<Builder<string>> $_builders */
function take_string_builders(array $_builders): void {}

take_string(external_function());
/** @var mixed $asserted */
$asserted = 'asserted';
external_assert_string($asserted);
take_string($asserted);
/** @var mixed $conditional */
$conditional = 'conditional';
if (ExternalAssertions::isString($conditional)) {
    take_string($conditional);
}
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
take_string((new ExternalService())->resolve('instance-provider'));
take_string(ExternalStaticService::resolve('static-provider'));
User::callLate();
(new ManagedTestCase())->getApplication();
(new ManagedTestCase())->getNative();
(new ManagedTestCase())->getFramework();
(new ManagedTestCase())->getLifecycle();

function use_external_interface(ExternalInterface $service): void
{
    take_external_result($service->fetch());
}

function get_external_path(?ExternalInterface $service): ?string
{
    return $service?->fetch()->path();
}

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

const MISSING_METHOD_SOURCE: &str = r"<?php

declare(strict_types=1);

interface DeclinedContract {}

function test(DeclinedContract $contract): void
{
    $contract->missing();
}
";

const DECLINED_PROPERTY_INITIALIZATION_SOURCE: &str = r"<?php

declare(strict_types=1);

final class ExternalResult {}

class FrameworkTestCase {}

final class ManagedTestCase extends FrameworkTestCase
{
    public ExternalResult $application;

    public function __construct() {}

    public function getApplication(): ExternalResult
    {
        return $this->application;
    }
}

(new ManagedTestCase())->getApplication();
";

const ISSUE_FILTER_SOURCE: &str = r"<?php

declare(strict_types=1);

// issue-filter-batch-proof
function known_function(): void {}

filtered_missing();
retained_missing();
";

#[test]
fn external_providers_receive_complete_invocation_context() -> Result<(), Box<dyn std::error::Error>> {
    let observation = analyze_with_fixture(SOURCE, false, false, false)?;
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
            "class-initializer-frameworktestcase",
            "class-initializer-managedtestcase",
            "closure-this-return",
            "closure-this-signature",
            "complete-composite",
            "dynamic-instance",
            "dynamic-signature",
            "dynamic-signature-return",
            "dynamic-static",
            "function",
            "function-assertion",
            "function-signature",
            "function-signature-return",
            "generic",
            "instance",
            "intersection",
            "late-static",
            "method-assertion",
            "method-signature",
            "method-signature-return",
            "missing-class-return",
            "missing-class-signature",
            "missing-interface-return",
            "missing-interface-return",
            "missing-interface-signature",
            "missing-interface-signature",
            "missing-static-return",
            "missing-static-signature",
            "property-id-read",
            "property-id-write",
            "property-initialization",
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
    let observation = analyze_with_fixture(NO_METHOD_PROVIDER_SOURCE, true, false, false)?;
    assert!(observation.issues.is_empty(), "native method analysis should remain unchanged: {:#?}", observation.issues);
    assert!(observation.invocations.is_empty(), "a host without method providers must not receive method requests");

    Ok(())
}

#[test]
fn declined_dynamic_methods_preserve_non_documented_method_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
    let observation = analyze_with_fixture(UNDOCUMENTED_METHOD_SOURCE, true, false, false)?;
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

#[test]
fn declined_missing_methods_preserve_non_existent_method_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
    let observation = analyze_with_fixture(MISSING_METHOD_SOURCE, false, false, false)?;
    assert_eq!(
        observation.issues,
        [(
            Some("non-existent-method".to_owned()),
            "Method `missing` does not exist on type `DeclinedContract`.".to_owned(),
        )]
    );
    assert!(observation.invocations.is_empty(), "a declining provider must not establish the missing method");

    Ok(())
}

#[test]
fn declined_property_initialization_preserves_uninitialized_diagnostic() -> Result<(), Box<dyn std::error::Error>> {
    let observation = analyze_with_fixture(DECLINED_PROPERTY_INITIALIZATION_SOURCE, false, true, false)?;
    assert_eq!(observation.issues.len(), 1, "a declining provider must preserve the native diagnostic");
    assert_eq!(observation.issues[0].0.as_deref(), Some("uninitialized-property"));
    assert!(observation.invocations.is_empty(), "a declining provider must not claim initialization");

    Ok(())
}

#[test]
fn external_issue_filters_batch_and_remove_selected_native_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
    let observation = analyze_with_fixture(ISSUE_FILTER_SOURCE, true, false, true)?;
    assert_eq!(observation.issues.len(), 1, "only the explicitly filtered native issue should be suppressed");
    assert_eq!(observation.issues[0].0.as_deref(), Some("non-existent-function"));
    assert!(observation.issues[0].1.contains("`retained_missing`"));
    assert_eq!(observation.invocations.len(), 2, "both issues from the file should reach the worker batch");
    assert!(observation.invocations.iter().all(|entry| entry.starts_with("issue-filter-")));

    Ok(())
}

fn analyze_with_fixture(
    source: &str,
    function_only: bool,
    decline_property_initialization: bool,
    register_issue_filter: bool,
) -> Result<AnalysisObservation, Box<dyn std::error::Error>> {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"));
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
    if decline_property_initialization {
        command = command.with_environment("MAGO_INVOCATION_DECLINE_PROPERTY_INITIALIZATION", "1");
    }
    if register_issue_filter {
        command = command.with_environment("MAGO_INVOCATION_ISSUE_FILTER", "1");
    }
    let pool = WorkerPool::spawn(command, NonZeroUsize::MIN, WorkerPoolOptions::default())?;
    let external = ExternalAnalyzer::initialize([Arc::new(pool)], PHPVersion::PHP85, &[], false)?;
    let mut registry = PluginRegistry::with_library_providers();
    registry.set_external_analyzer(Arc::new(ExternalAnalyzerHandle::ready(external)));

    let configuration = common::database_configuration("/invocation-proof", vec![Cow::Borrowed(b"src")]);
    let mut database = Database::new(configuration);
    database.add(File::new(
        Cow::Borrowed(b"src/invocation.php"),
        FileType::Host,
        None,
        Cow::Owned(source.as_bytes().to_vec()),
    ));

    let mut settings = Settings::new(PHPVersion::PHP85);
    settings.find_unused_parameters = false;
    settings.check_property_initialization = true;
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
