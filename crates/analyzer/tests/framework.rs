use std::borrow::Cow;
use std::collections::BTreeMap;

use ahash::HashSet;

use bumpalo::Bump;
use mago_analyzer::Analyzer;
use mago_analyzer::analysis_result::AnalysisResult;
use mago_analyzer::settings::Settings;
use mago_atom::AtomSet;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::populator::populate_codebase;
use mago_codex::reference::SymbolReferences;
use mago_codex::scanner::scan_program;
use mago_database::file::File;
use mago_names::resolver::NameResolver;
use mago_syntax::parser::parse_file;

#[derive(Debug, Clone)]
pub struct TestCase<'a> {
    name: &'a str,
    content: &'a str,
}

impl<'a> TestCase<'a> {
    pub fn new(name: &'a str, content: &'a str) -> Self {
        Self { name, content }
    }

    pub fn run(self) {
        run_test_case_inner(self);
    }
}

fn run_test_case_inner(config: TestCase) {
    let arena = Bump::new();
    let source_file = File::ephemeral(Cow::Owned(config.name.to_string()), Cow::Owned(config.content.to_string()));

    let (program, parse_issues) = parse_file(&arena, &source_file);
    if parse_issues.is_some() {
        panic!("Test '{}' failed during parsing:\n{:#?}", config.name, parse_issues);
    }

    let resolver = NameResolver::new(&arena);
    let resolved_names = resolver.resolve(program);
    let mut codebase = scan_program(&arena, &source_file, program, &resolved_names);
    let mut symbol_references = SymbolReferences::new();

    populate_codebase(&mut codebase, &mut symbol_references, AtomSet::default(), HashSet::default());

    let mut analysis_result = AnalysisResult::new(symbol_references);
    let analyzer = Analyzer::new(
        &arena,
        &source_file,
        &resolved_names,
        &codebase,
        Settings {
            find_unused_expressions: true,
            check_throws: true,
            allow_possibly_undefined_array_keys: false,
            ..Default::default()
        },
    );

    let analysis_run_result = analyzer.analyze(program, &mut analysis_result);

    if let Err(err) = analysis_run_result {
        panic!("Test '{}': Expected analysis to succeed, but it failed with an error: {}", config.name, err);
    }

    verify_reported_issues(config.name, analysis_result, codebase);
}

fn verify_reported_issues(test_name: &str, mut analysis_result: AnalysisResult, mut codebase: CodebaseMetadata) {
    let mut actual_issues_collected = std::mem::take(&mut analysis_result.issues);

    actual_issues_collected.extend(codebase.take_issues(true));

    let mut actual_issue_counts: BTreeMap<String, usize> = BTreeMap::new();
    for actual_issue in actual_issues_collected.iter() {
        let Some(issue_code) = actual_issue.code.as_ref().cloned() else {
            panic!("Analyzer returned an issue with no code: {actual_issue:?}");
        };

        *actual_issue_counts.entry(issue_code).or_insert(0) += 1;
    }

    if !actual_issue_counts.is_empty() {
        let mut discrepancies = Vec::new();
        for (actual_kind, &actual_count) in &actual_issue_counts {
            discrepancies.push(format!("- Unexpected issue(s) `{}`: found {}.", actual_kind.as_str(), actual_count));
        }

        let mut panic_message = format!("Test '{test_name}' failed with issue discrepancies:\n");
        for d in discrepancies {
            panic_message.push_str(&format!("  {d}\n"));
        }

        panic!("{}", panic_message);
    }
}
