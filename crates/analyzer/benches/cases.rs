//! Per-category analyzer benchmarks driven by the `tests/cases/` corpus.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::borrow::Cow;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::LazyLock;
use std::time::Duration;

use criterion::BatchSize;
use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;
use mago_allocator::LocalArena;

use foldhash::HashSet;

use mago_analyzer::Analyzer;
use mago_analyzer::analysis_result::AnalysisResult;
use mago_analyzer::plugin::PluginRegistry;
use mago_analyzer::settings::Settings;
use mago_codex::misc::GenericParent;
use mago_codex::populator::populate_codebase;
use mago_codex::reference::SymbolReferences;
use mago_codex::scanner::scan_program;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::generic::TGenericParameter;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::union::TUnion;
use mago_database::DatabaseReader;
use mago_database::file::File;
use mago_names::resolver::NameResolver;
use mago_prelude::Prelude;
use mago_span::Span;
use mago_syntax::parser::parse_file;
use mago_word::Word;
use mago_word::WordSet;
use mago_word::word;

static PRELUDE: LazyLock<Prelude> = LazyLock::new(Prelude::build);
static PLUGIN_REGISTRY: LazyLock<PluginRegistry> = LazyLock::new(PluginRegistry::with_library_providers);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct BenchmarkThrownGenericException {
    atomic_type: TAtomic,
    span: Span,
    object_names: Vec<Word>,
}

fn analyze_file(path: &Path, content: &str) {
    analyze_file_with_settings(path, content, false);
}

fn analyze_file_with_settings(path: &Path, content: &str, check_dead_catch: bool) {
    let Prelude { mut database, mut metadata, mut symbol_references } = PRELUDE.clone();

    let file = File::ephemeral(
        Cow::Owned(path.file_name().unwrap().to_string_lossy().into_owned().into_bytes()),
        Cow::Owned(content.as_bytes().to_vec()),
    );
    let file_id = database.add(file);
    let source_file = database.get_ref(&file_id).expect("file just added must exist");

    let arena = LocalArena::new();
    let program = parse_file(&arena, source_file);
    if program.has_errors() {
        panic!("Parse failed for {}: {:?}", path.display(), program.errors);
    }

    let resolver = NameResolver::new(&arena);
    let resolved_names = resolver.resolve(program);

    let settings = Settings {
        find_unused_expressions: true,
        find_unused_definitions: true,
        check_throws: true,
        check_dead_catch,
        allow_possibly_undefined_array_keys: false,
        strict_list_index_checks: true,
        check_property_initialization: true,
        ..Default::default()
    };

    metadata.extend(scan_program(&arena, source_file, program, &resolved_names, settings.version));
    populate_codebase(&mut metadata, &mut symbol_references, WordSet::default(), HashSet::default());

    let mut analysis_result = AnalysisResult::new(symbol_references);
    let analyzer = Analyzer::new(&arena, source_file, &resolved_names, &metadata, &PLUGIN_REGISTRY, settings);

    if let Err(e) = analyzer.analyze(program, &mut analysis_result) {
        panic!("Analysis failed for {}: {e}", path.display());
    }
}

const BENCHMARKED_PREFIXES: &[&str] = &["issue_", "psl_", "sealed_class_", "trait_", "conditional_", "too_wide_throws"];
const GENERIC_THROW_FIXTURES: &[&str] = &[
    "generic_iterator_aggregate_union_bindings",
    "generic_method_throw_dead_catch",
    "generic_throw_template_bound_chain",
    "generic_throw_unused_detection",
    "generic_throwing_iterator_foreach",
    "recursive_generic_throw_resolution",
    "too_wide_generic_throw",
    "unresolved_recursive_generic_throw",
];

fn discover_cases_by_prefix() -> Vec<(&'static str, Vec<(PathBuf, String)>)> {
    let cases_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("cases");
    let mut buckets: Vec<(&'static str, Vec<(PathBuf, String)>)> =
        BENCHMARKED_PREFIXES.iter().map(|p| (*p, Vec::new())).collect();

    let entries =
        std::fs::read_dir(&cases_dir).unwrap_or_else(|e| panic!("failed to read {}: {e}", cases_dir.display()));
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("php") {
            continue;
        }

        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else { continue };
        let Some(bucket) = buckets.iter_mut().find(|(p, _)| stem.starts_with(p)) else { continue };
        let content =
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
        bucket.1.push((path, content));
    }

    for (_, files) in &mut buckets {
        files.sort_by(|a, b| a.0.cmp(&b.0));
    }

    buckets
}

fn bench_cases(c: &mut Criterion) {
    LazyLock::force(&PRELUDE);
    LazyLock::force(&PLUGIN_REGISTRY);

    let mut group = c.benchmark_group("analyzer_case");
    for (prefix, files) in discover_cases_by_prefix() {
        if files.is_empty() {
            continue;
        }

        let bench_name = prefix.trim_end_matches('_');
        group.bench_with_input(BenchmarkId::from_parameter(bench_name), &files, |b, files| {
            b.iter(|| {
                for (p, c) in files {
                    analyze_file(p, c);
                }
            });
        });
    }

    group.finish();
}

fn bench_generic_throw_fixtures(c: &mut Criterion) {
    let cases_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("cases");
    let mut group = c.benchmark_group("analyzer_generic_throw_fixture");
    group.sample_size(10);
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(1));

    for fixture in GENERIC_THROW_FIXTURES {
        let path = cases_dir.join(format!("{fixture}.php"));
        let content =
            std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        let check_dead_catch = *fixture == "generic_method_throw_dead_catch";

        group.bench_function(*fixture, |b| {
            b.iter(|| analyze_file_with_settings(&path, &content, check_dead_catch));
        });
    }

    group.finish();
}

fn attribute_dependencies(many_parameters: bool) -> String {
    let mut parameters = vec![
        "string|array|null $kind = null".to_owned(),
        "?string $format = null".to_owned(),
        "mixed $nested = null".to_owned(),
        "array|string|null $choices = null".to_owned(),
        "mixed $fallback = null".to_owned(),
    ];
    if many_parameters {
        parameters.extend((0..54).map(|index| format!("mixed $parameter_{index} = null")));
    }

    format!(
        r"<?php
namespace Benchmark;
class Value {{ public function __construct({}) {{}} }}
#[\Attribute(\Attribute::TARGET_METHOD | \Attribute::IS_REPEATABLE)]
final class ExampleAttribute {{ public function __construct(mixed $value = null) {{}} }}
#[\Attribute(\Attribute::TARGET_PARAMETER)]
final class FlagsAttribute {{
    /**
     * @param (FILTER_VALIDATE_*)|(FILTER_SANITIZE_*)|null $filter
     * @param int-mask-of<(FILTER_FLAG_*)|FILTER_NULL_ON_FAILURE> $flags
     */
    public function __construct(?string $name = null, ?int $filter = null, int $flags = 0, array $options = []) {{}}
}}
#[\Attribute(\Attribute::TARGET_PARAMETER)]
final class SimpleFlagsAttribute {{
    public function __construct(?string $name = null, ?int $filter = null, int $flags = 0, array $options = []) {{}}
}}
",
        parameters.join(", ")
    )
}

fn bench_parameter_attribute_defaults(c: &mut Criterion) {
    let cases = [
        (
            "many_parameters_one_default_mask",
            true,
            "final class Controller { #[ExampleAttribute(value: new Value(kind: 'string'))] public function __invoke(#[FlagsAttribute] int $page = 1): void {} }".to_owned(),
        ),
        (
            "many_parameters_three_default_masks",
            true,
            "final class Controller { #[ExampleAttribute(value: new Value(kind: 'string'))] public function __invoke(#[FlagsAttribute] int $page = 1, #[FlagsAttribute] int $pageSize = 20, #[FlagsAttribute(name: 'status')] ?array $status = null): void {} }".to_owned(),
        ),
        (
            "many_parameters_explicit_mask",
            true,
            "final class Controller { #[ExampleAttribute(value: new Value(kind: 'string'))] public function __invoke(#[FlagsAttribute(flags: FILTER_FLAG_STRIP_LOW | FILTER_FLAG_STRIP_HIGH)] int $page = 1): void {} }".to_owned(),
        ),
        (
            "many_parameters_simple_default",
            true,
            "final class Controller { #[ExampleAttribute(value: new Value(kind: 'string'))] public function __invoke(#[SimpleFlagsAttribute] int $page = 1): void {} }".to_owned(),
        ),
        (
            "many_parameters_nested_value_and_three_default_masks",
            true,
            "final class Controller { #[ExampleAttribute(value: new Value(kind: 'array', nested: new Value(kind: 'string', choices: ['draft', 'approved', 'rejected'])))] public function __invoke(#[FlagsAttribute] int $page = 1, #[FlagsAttribute] int $pageSize = 20, #[FlagsAttribute(name: 'status')] ?array $status = null): void {} }".to_owned(),
        ),
        (
            "few_parameters_nested_value_and_three_default_masks",
            false,
            "final class Controller { #[ExampleAttribute(value: new Value(kind: 'array', nested: new Value(kind: 'string', choices: ['draft', 'approved', 'rejected'])))] public function __invoke(#[FlagsAttribute] int $page = 1, #[FlagsAttribute] int $pageSize = 20, #[FlagsAttribute(name: 'status')] ?array $status = null): void {} }".to_owned(),
        ),
    ];
    bench_nested_attribute_group(c, "analyzer_many_parameters", &cases);
}

fn bench_nested_attribute_group(c: &mut Criterion, group_name: &str, cases: &[(&str, bool, String)]) {
    let mut group = c.benchmark_group(group_name);
    group.sample_size(50);
    for (name, many_parameters, class_source) in cases {
        let Prelude { mut database, mut metadata, mut symbol_references } = PRELUDE.clone();
        let arena = LocalArena::new();
        let settings = Settings::default();
        let resolver = NameResolver::new(&arena);
        let dependencies =
            vec![(PathBuf::from("attribute_dependencies.php"), attribute_dependencies(*many_parameters))];
        for (path, content) in &dependencies {
            let dependency_id = database.add(File::ephemeral(
                path.as_os_str().as_encoded_bytes().to_vec().into(),
                content.as_bytes().to_vec().into(),
            ));
            let dependency_file = database.get_ref(&dependency_id).expect("dependency file must exist");
            let dependency_program = parse_file(&arena, dependency_file);
            assert!(!dependency_program.has_errors(), "dependency fixture must parse: {}", path.display());
            let dependency_names = resolver.resolve(dependency_program);
            metadata.extend(scan_program(
                &arena,
                dependency_file,
                dependency_program,
                &dependency_names,
                settings.version,
            ));
        }

        let source = format!("<?php namespace Benchmark; {class_source}");
        let file_id =
            database.add(File::ephemeral(b"nested_attributes.php".as_slice().into(), source.into_bytes().into()));
        let source_file = database.get_ref(&file_id).expect("controller file must exist");
        let program = parse_file(&arena, source_file);
        assert!(!program.has_errors(), "controller fixture must parse");
        let resolved_names = resolver.resolve(program);
        metadata.extend(scan_program(&arena, source_file, program, &resolved_names, settings.version));
        populate_codebase(&mut metadata, &mut symbol_references, WordSet::default(), HashSet::default());

        // Exclude dependency scanning and codebase population so these timings measure analysis only.
        let run_analysis = || {
            let analysis_arena = LocalArena::new();
            let analyzer = Analyzer::new(
                &analysis_arena,
                source_file,
                &resolved_names,
                &metadata,
                &PLUGIN_REGISTRY,
                settings.clone(),
            );
            let mut result = AnalysisResult::new(SymbolReferences::default());
            analyzer.analyze(program, &mut result).expect("controller analysis must succeed");
            result
        };
        let result = run_analysis();
        assert!(result.issues.is_empty(), "{name} must be error-free: {:?}", result.issues);
        group.bench_function(*name, |b| b.iter(|| std::hint::black_box(run_analysis())));
    }
    group.finish();
}

fn generic_exceptions(
    cardinality: usize,
    constraint_width: usize,
    distinct_parameters: bool,
) -> Vec<BenchmarkThrownGenericException> {
    let constraint = Arc::new(TUnion::from_vec(
        (0..constraint_width)
            .map(|index| TAtomic::Object(TObject::Named(TNamedObject::new(word(format!("Bound{index}").as_bytes())))))
            .collect(),
    ));
    let entity = GenericParent::FunctionLike((word(b"benchmark"), word(b"throws")));

    (0..cardinality)
        .map(|index| {
            let parameter_name =
                if distinct_parameters { word(format!("TFailure{index}").as_bytes()) } else { word(b"TFailure") };
            let atomic_type =
                TAtomic::GenericParameter(TGenericParameter::new(parameter_name, Arc::clone(&constraint), entity));

            BenchmarkThrownGenericException {
                atomic_type,
                span: Span::dummy(index as u32, index as u32 + 1),
                object_names: vec![word(b"Exception")],
            }
        })
        .collect()
}

fn merge_generic_exceptions_with_vec(
    mut destination: Vec<BenchmarkThrownGenericException>,
    source: &[BenchmarkThrownGenericException],
) -> Vec<BenchmarkThrownGenericException> {
    for thrown in source {
        if !destination.contains(thrown) {
            destination.push(thrown.clone());
        }
    }

    destination
}

fn merge_generic_exceptions_with_hashset(
    mut destination: HashSet<BenchmarkThrownGenericException>,
    source: &[BenchmarkThrownGenericException],
) -> HashSet<BenchmarkThrownGenericException> {
    destination.extend(source.iter().cloned());
    destination
}

fn generic_exception_vecs_are_equal(
    left: &[BenchmarkThrownGenericException],
    right: &[BenchmarkThrownGenericException],
) -> bool {
    left.len() == right.len() && left.iter().all(|exception| right.contains(exception))
}

fn bench_generic_exception_set_merges(c: &mut Criterion) {
    let mut group = c.benchmark_group("generic_exception_set_merge");
    group.sample_size(10);
    group.warm_up_time(Duration::from_millis(200));
    group.measurement_time(Duration::from_millis(500));

    for cardinality in [4, 32] {
        for constraint_width in [1, 8] {
            for distinct_parameters in [false, true] {
                let pattern = if distinct_parameters { "distinct" } else { "repeated" };
                let all = generic_exceptions(cardinality + cardinality / 2, constraint_width, distinct_parameters);
                let destination_vec = all[..cardinality].to_vec();
                let destination_set = destination_vec.iter().cloned().collect::<HashSet<_>>();
                let mut source = all[..cardinality / 2].to_vec();
                source.extend_from_slice(&all[cardinality..]);
                let merged_vec = merge_generic_exceptions_with_vec(destination_vec.clone(), &source);
                let merged_set = merge_generic_exceptions_with_hashset(destination_set.clone(), &source);
                assert_eq!(merged_vec.len(), merged_set.len());
                assert!(merged_vec.iter().all(|exception| merged_set.contains(exception)));

                let parameters = format!("n{cardinality}_w{constraint_width}_{pattern}");
                group.bench_function(BenchmarkId::new("vec", &parameters), |b| {
                    b.iter_batched(
                        || destination_vec.clone(),
                        |destination| {
                            std::hint::black_box(merge_generic_exceptions_with_vec(destination, &source));
                        },
                        BatchSize::SmallInput,
                    );
                });
                group.bench_function(BenchmarkId::new("hashset", parameters), |b| {
                    b.iter_batched(
                        || destination_set.clone(),
                        |destination| {
                            std::hint::black_box(merge_generic_exceptions_with_hashset(destination, &source));
                        },
                        BatchSize::SmallInput,
                    );
                });

                let reordered_vec = merged_vec.iter().rev().cloned().collect::<Vec<_>>();
                let reordered_set = reordered_vec.iter().cloned().collect::<HashSet<_>>();
                let equality_parameters = format!("n{cardinality}_w{constraint_width}_{pattern}");
                group.bench_function(BenchmarkId::new("vec_set_equality", &equality_parameters), |b| {
                    b.iter(|| std::hint::black_box(generic_exception_vecs_are_equal(&merged_vec, &reordered_vec)));
                });
                group.bench_function(BenchmarkId::new("hashset_equality", equality_parameters), |b| {
                    b.iter(|| std::hint::black_box(merged_set == reordered_set));
                });
            }
        }
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_cases,
    bench_generic_throw_fixtures,
    bench_parameter_attribute_defaults,
    bench_generic_exception_set_merges
);
criterion_main!(benches);
