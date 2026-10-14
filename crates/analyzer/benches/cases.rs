//! Per-category analyzer benchmarks driven by the `tests/cases/` corpus.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::borrow::Cow;
use std::path::Path;
use std::path::PathBuf;
use std::sync::LazyLock;

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
use mago_codex::populator::populate_codebase;
use mago_codex::reference::SymbolReferences;
use mago_codex::scanner::scan_program;
use mago_database::DatabaseReader;
use mago_database::file::File;
use mago_names::resolver::NameResolver;
use mago_prelude::Prelude;
use mago_syntax::parser::parse_file;
use mago_word::WordSet;

static PRELUDE: LazyLock<Prelude> = LazyLock::new(Prelude::build);
static PLUGIN_REGISTRY: LazyLock<PluginRegistry> = LazyLock::new(PluginRegistry::with_library_providers);

fn analyze_file(path: &Path, content: &str) {
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

const BENCHMARKED_PREFIXES: &[&str] = &["issue_", "psl_", "sealed_class_", "trait_", "conditional_"];

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

criterion_group!(benches, bench_cases, bench_parameter_attribute_defaults);
criterion_main!(benches);
