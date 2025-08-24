//! # Analysis Core
//!
//! This module contains the primary logic for running Mago's analysis pipeline
//! in a WASM context. It defines the structure for the analysis results and
//! orchestrates the execution of the parser, semantic checker, analyzer, linter,
//! and formatter.

use std::borrow::Cow;

use bumpalo::Bump;
use serde::Serialize;

use mago_analyzer::Analyzer;
use mago_codex::reference::SymbolReferences;
use mago_database::file::File;
use mago_formatter::Formatter;
use mago_linter::Linter;
use mago_names::resolver::NameResolver;
use mago_reporting::Issue;
use mago_reporting::IssueCollection;
use mago_semantics::SemanticsChecker;
use mago_syntax::parser::parse_file;

use crate::settings::WasmSettings;

/// Represents the result of a full analysis pass.
///
/// This struct is serialized to a JavaScript object and returned to the caller.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WasmAnalysisResults {
    pub parse_error: Option<Issue>,
    pub semantic_issues: IssueCollection,
    pub linter_issues: IssueCollection,
    pub analyzer_issues: IssueCollection,
    pub symbol_references: SymbolReferences,
    pub formatted_code: Option<String>,
}

/// Runs the complete analysis pipeline on a string of PHP code.
pub fn analyze_code(code: String, settings: WasmSettings) -> WasmAnalysisResults {
    let arena = Bump::new();
    let source_file = File::ephemeral(Cow::Borrowed("code.php"), Cow::Owned(code));

    let (program, parse_error) = parse_file(&arena, &source_file);
    let resolved_names = NameResolver::new(&arena).resolve(program);

    let semantic_issues = SemanticsChecker::new(settings.php_version).check(&source_file, program, &resolved_names);

    let mut codebase = mago_codex::scanner::scan_program(&arena, &source_file, program, &resolved_names);
    let mut symbol_references = SymbolReferences::new();

    mago_codex::populator::populate_codebase(
        &mut codebase,
        &mut symbol_references,
        Default::default(),
        Default::default(),
    );

    let analyzer_settings = settings.analyzer.to_analyzer_settings(settings.php_version);
    let analyzer = Analyzer::new(&arena, &source_file, &resolved_names, &codebase, analyzer_settings);
    let mut analyzer_analysis_result = mago_analyzer::analysis_result::AnalysisResult::new(Default::default());
    analyzer.analyze(program, &mut analyzer_analysis_result).unwrap();
    let analyzer_issues = analyzer_analysis_result.issues;

    symbol_references.extend(analyzer_analysis_result.symbol_references);

    let linter_settings = settings.linter.to_linter_settings(settings.php_version);
    let linter = Linter::new(&arena, linter_settings, None);
    let linter_issues = linter.lint(&source_file, program, &resolved_names);

    let formatted_code = if parse_error.is_none() {
        let formatter = Formatter::new(&arena, settings.php_version, settings.formatter);

        Some(formatter.format(&source_file, program).to_string())
    } else {
        None
    };

    WasmAnalysisResults {
        parse_error: parse_error.as_ref().map(|e| e.into()),
        semantic_issues,
        linter_issues,
        analyzer_issues,
        symbol_references,
        formatted_code,
    }
}
