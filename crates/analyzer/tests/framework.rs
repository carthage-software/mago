#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::sync::LazyLock;

use foldhash::HashSet;

use bumpalo::Bump;
use mago_analyzer::Analyzer;
use mago_analyzer::analysis_result::AnalysisResult;
use mago_analyzer::plugin::PluginRegistry;
use mago_analyzer::settings::Settings;
use mago_atom::AtomSet;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::populator::populate_codebase;
use mago_codex::scanner::scan_program;
use mago_collector::Collector;
use mago_database::DatabaseReader;
use mago_database::file::File;
use mago_database::file::FileId;
use mago_database::file::FileType;
use mago_names::ResolvedNames;
use mago_names::resolver::NameResolver;
use mago_prelude::Prelude;
use mago_reporting::Issue;
use mago_syntax::ast::Program;
use mago_syntax::parser::parse_file;

static PRELUDE: LazyLock<Prelude> = LazyLock::new(Prelude::build);
static PLUGIN_REGISTRY: LazyLock<PluginRegistry> = LazyLock::new(PluginRegistry::with_library_providers);

/// Pragma categories used by the analyzer's own collector. Mirrored here so per-file
/// collectors built in this test framework agree on which `@mago-expect` / `@mago-ignore`
/// directives apply.
const COLLECTOR_CATEGORIES: &[&str] = &["analysis", "analyzer", "analyser"];

#[derive(Debug, Clone)]
pub struct TestCase<'src> {
    name: &'src str,
    content: &'src str,
    settings: Option<Settings>,
    vendor_file: Option<&'src str>,
    patch_file: Option<&'src str>,
}

impl<'src> TestCase<'src> {
    #[must_use]
    pub fn new(name: &'src str, content: &'src str) -> Self {
        Self { name, content, settings: None, vendor_file: None, patch_file: None }
    }

    #[must_use]
    pub fn settings(mut self, settings: Settings) -> Self {
        self.settings = Some(settings);
        self
    }

    /// Attach an auxiliary file scanned with `FileType::Vendored`. The vendor file is merged
    /// into the codebase before the user file, mirroring the production pipeline.
    #[must_use]
    pub fn with_vendor(mut self, content: &'src str) -> Self {
        self.vendor_file = Some(content);
        self
    }

    /// Attach an auxiliary file scanned with `FileType::Patch`. The patch file is merged after
    /// the user file, which is when `apply_patch` runs and `patch_diagnostics` are
    /// populated. Patch diagnostics whose primary span lies in this file are matched against
    /// `@mago-expect analysis:<code>` pragmas inside the patch.
    #[must_use]
    pub fn with_patch(mut self, content: &'src str) -> Self {
        self.patch_file = Some(content);
        self
    }

    pub fn run(self) {
        run_test_case_inner(self);
    }
}

#[must_use]
pub fn default_test_settings() -> Settings {
    Settings {
        find_unused_expressions: true,
        find_unused_definitions: true,
        check_throws: true,
        allow_possibly_undefined_array_keys: false,
        strict_list_index_checks: true,
        check_property_initialization: true,
        ..Default::default()
    }
}

#[must_use]
pub fn infection_like_settings() -> Settings {
    Settings {
        find_unused_expressions: true,
        find_unused_definitions: true,
        find_unused_parameters: true,
        check_throws: true,
        analyze_dead_code: true,
        memoize_properties: true,
        allow_possibly_undefined_array_keys: true,
        ..Default::default()
    }
}

#[must_use]
pub fn no_boolean_literal_comparison_settings() -> Settings {
    Settings { no_boolean_literal_comparison: true, ..Default::default() }
}

#[must_use]
pub fn php_90_settings() -> Settings {
    Settings::new(mago_php_version::PHPVersion::new(9, 0, 0))
}

#[must_use]
pub fn check_name_casing_settings() -> Settings {
    Settings { check_name_casing: true, ..Default::default() }
}

struct ParsedFile<'arena, 'ctx> {
    file: &'ctx File,
    program: &'arena Program<'arena>,
    #[allow(dead_code)]
    resolved: ResolvedNames<'arena>,
}

fn parse_aux<'arena, 'ctx>(arena: &'arena Bump, file: &'ctx File, label: &str) -> ParsedFile<'arena, 'ctx> {
    let program = parse_file(arena, file);
    assert!(!program.has_errors(), "{label} parse failed: {:?}", program.errors);
    let resolved = NameResolver::new(arena).resolve(program);
    ParsedFile { file, program, resolved }
}

fn run_test_case_inner(config: TestCase) {
    let Prelude { mut database, mut metadata, mut symbol_references } = PRELUDE.clone();

    // Register every file (vendor + user + patch) up front, then borrow them — the database
    // can't hand out refs while it's being mutated.
    let vendor_file_id = config.vendor_file.map(|content| {
        let name = format!("{}.vendor.php", config.name);
        database.add(File::new(Cow::Owned(name), FileType::Vendored, None, Cow::Owned(content.to_string())))
    });

    let user_file_id =
        database.add(File::ephemeral(Cow::Owned(config.name.to_string()), Cow::Owned(config.content.to_string())));

    let patch_file_id = config.patch_file.map(|content| {
        let name = format!("{}.patch.php", config.name);
        database.add(File::new(Cow::Owned(name), FileType::Patch, None, Cow::Owned(content.to_string())))
    });

    let user_file = database.get_ref(&user_file_id).expect("user file just added should exist");
    let vendor_file =
        vendor_file_id.as_ref().map(|id| database.get_ref(id).expect("vendor file just added should exist"));
    let patch_file = patch_file_id.as_ref().map(|id| database.get_ref(id).expect("patch file just added should exist"));

    let arena = Bump::new();

    let vendor_parsed = vendor_file.map(|f| parse_aux(&arena, f, "Vendor"));

    let user_program = parse_file(&arena, user_file);
    assert!(!user_program.has_errors(), "Parse failed: {:?}", user_program.errors);
    let user_resolved = NameResolver::new(&arena).resolve(user_program);

    let patch_parsed = patch_file.map(|f| parse_aux(&arena, f, "Patch"));

    // Vendor first — this matches the production merge order so any subsequent patch can
    // find a target to update.
    if let Some(p) = &vendor_parsed {
        metadata.extend(scan_program(&arena, p.file, p.program, &p.resolved));
    }
    metadata.extend(scan_program(&arena, user_file, user_program, &user_resolved));
    // Patch last so `apply_patch` runs against an already-populated codebase.
    if let Some(p) = &patch_parsed {
        metadata.apply_patches(scan_program(&arena, p.file, p.program, &p.resolved));
    }

    populate_codebase(&mut metadata, &mut symbol_references, AtomSet::default(), HashSet::default());

    let settings = config.settings.unwrap_or_else(default_test_settings);

    let mut analysis_result = AnalysisResult::new(symbol_references);
    let analyzer = Analyzer::new(&arena, user_file, &user_resolved, &metadata, &PLUGIN_REGISTRY, settings);

    let analysis_run_result = analyzer.analyze(user_program, &mut analysis_result);

    if let Err(err) = analysis_run_result {
        panic!("Test '{}': Expected analysis to succeed, but it failed with an error: {}", config.name, err);
    }

    verify_reported_issues(
        config.name,
        analysis_result,
        metadata,
        &arena,
        vendor_parsed.as_ref(),
        patch_parsed.as_ref(),
    );
}

/// Returns the `FileId` of the issue's primary annotation, if any.
fn issue_primary_file_id(issue: &Issue) -> Option<FileId> {
    issue
        .annotations
        .iter()
        .find(|ann| ann.kind.is_primary())
        .or_else(|| issue.annotations.first())
        .map(|ann| ann.span.file_id)
}

fn verify_reported_issues(
    test_name: &str,
    mut analysis_result: AnalysisResult,
    mut codebase: CodebaseMetadata,
    arena: &Bump,
    vendor_parsed: Option<&ParsedFile<'_, '_>>,
    patch_parsed: Option<&ParsedFile<'_, '_>>,
) {
    let mut actual_issues_collected = std::mem::take(&mut analysis_result.issues);

    // Build a per-file Collector for each aux file so that issues whose primary span lies
    // within an aux file (typically scan-time patch diagnostics) can be matched against
    // that file's `@mago-expect` / `@mago-ignore` pragmas. Issues whose span is in the
    // user file are skipped here — the analyzer already ran its own collector on those.
    let mut aux_collectors: Vec<(FileId, Collector<'_, '_>)> = Vec::new();
    for p in vendor_parsed.into_iter().chain(patch_parsed) {
        let collector = Collector::new(arena, p.file, p.program, COLLECTOR_CATEGORIES);
        aux_collectors.push((p.file.id, collector));
    }

    for issue in codebase.take_issues(true) {
        let primary_file = issue_primary_file_id(&issue);
        if let Some(file_id) = primary_file
            && let Some((_, collector)) = aux_collectors.iter_mut().find(|(id, _)| *id == file_id)
        {
            // Routing through `report` runs the issue past `@mago-expect` / `@mago-ignore`
            // pragma matching; suppressed issues are dropped, others end up in the
            // collector's internal list and surface via `finish()` below.
            collector.report(issue);
        } else {
            actual_issues_collected.push(issue);
        }
    }

    // Drain each aux collector — its remaining issues are the ones that weren't suppressed
    // by a pragma, plus any unfulfilled-`@mago-expect` warnings.
    for (_, collector) in aux_collectors {
        for issue in collector.finish() {
            actual_issues_collected.push(issue);
        }
    }

    let mut actual_issue_counts: BTreeMap<String, usize> = BTreeMap::new();
    for actual_issue in &actual_issues_collected {
        let Some(issue_code) = actual_issue.code.clone() else {
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
            let _ = writeln!(panic_message, "  {d}");
        }

        panic!("{}", panic_message);
    }
}
