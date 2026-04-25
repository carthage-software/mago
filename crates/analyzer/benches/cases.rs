//! Per-category analyzer benchmarks driven by the `tests/cases/` corpus.

use std::borrow::Cow;
use std::path::Path;
use std::path::PathBuf;
use std::sync::LazyLock;

use bumpalo::Bump;
use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;

use foldhash::HashSet;

use mago_analyzer::Analyzer;
use mago_analyzer::analysis_result::AnalysisResult;
use mago_analyzer::plugin::PluginRegistry;
use mago_analyzer::settings::Settings;
use mago_atom::AtomSet;
use mago_codex::populator::populate_codebase;
use mago_codex::scanner::scan_program;
use mago_database::DatabaseReader;
use mago_database::file::File;
use mago_names::resolver::NameResolver;
use mago_prelude::Prelude;
use mago_syntax::parser::parse_file;

static PRELUDE: LazyLock<Prelude> = LazyLock::new(Prelude::build);
static PLUGIN_REGISTRY: LazyLock<PluginRegistry> = LazyLock::new(PluginRegistry::with_library_providers);

fn analyze_file(path: &Path, content: &str) {
    let Prelude { mut database, mut metadata, mut symbol_references } = PRELUDE.clone();

    let file = File::ephemeral(
        Cow::Owned(path.file_name().unwrap().to_string_lossy().into_owned()),
        Cow::Owned(content.to_string()),
    );
    let file_id = database.add(file);
    let source_file = database.get_ref(&file_id).expect("file just added must exist");

    let arena = Bump::new();
    let program = parse_file(&arena, source_file);
    if program.has_errors() {
        panic!("Parse failed for {}: {:?}", path.display(), program.errors);
    }

    let resolver = NameResolver::new(&arena);
    let resolved_names = resolver.resolve(program);

    metadata.extend(scan_program(&arena, source_file, program, &resolved_names));
    populate_codebase(&mut metadata, &mut symbol_references, AtomSet::default(), HashSet::default());

    let settings = Settings {
        find_unused_expressions: true,
        find_unused_definitions: true,
        check_throws: true,
        allow_possibly_undefined_array_keys: false,
        strict_list_index_checks: true,
        check_property_initialization: true,
        ..Default::default()
    };

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

criterion_group!(benches, bench_cases);
criterion_main!(benches);
