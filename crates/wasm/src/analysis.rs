//! # Analysis Core
//!
//! This module contains the primary logic for running Mago's analysis pipeline
//! in a WASM context using the Orchestrator services.

use std::borrow::Cow;
use std::sync::LazyLock;

use mago_analyzer::analysis_result::AnalysisResult;
use mago_orchestrator::service::analysis::AnalysisService;
use mago_orchestrator::service::format::FileFormatStatus;
use mago_orchestrator::service::format::FormatService;
use serde::Serialize;

use mago_codex::reference::SymbolReferences;
use mago_database::DatabaseReader;
use mago_database::ReadDatabase;
use mago_database::file::File;
use mago_orchestrator::service::lint::LintMode;
use mago_prelude::Prelude;
use mago_reporting::Issue;
use mago_reporting::IssueCollection;

use crate::settings::WasmSettings;

static STATIC_PRELUDE: LazyLock<Prelude> = LazyLock::new(Prelude::build);

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

/// Runs the complete analysis pipeline on a string of PHP code using the Orchestrator.
pub fn analyze_code(code: String, settings: WasmSettings) -> WasmAnalysisResults {
    let Prelude { database: _, metadata, symbol_references } = LazyLock::force(&STATIC_PRELUDE).clone();

    // Create orchestrator configuration from WASM settings
    let config = settings.to_orchestrator_config();

    // Run linting (includes parsing and semantics)
    let lint_file = File::ephemeral(Cow::Borrowed("code.php"), Cow::Owned(code.clone()));
    let lint_database = ReadDatabase::single(lint_file);
    let lint_service = mago_orchestrator::service::lint::LintService::new(
        lint_database,
        config.linter_settings.clone(),
        false, // no progress bars in WASM
    );

    let lint_issues = lint_service.lint(LintMode::Full).unwrap_or_else(|_| IssueCollection::new());

    // For WASM, we'll put all lint issues together
    let parse_error = None;
    let semantic_issues = IssueCollection::new();
    let linter_issues = lint_issues;

    // Run analysis
    let analysis_file = File::ephemeral(Cow::Borrowed("code.php"), Cow::Owned(code.clone()));
    let analysis_database = ReadDatabase::single(analysis_file);
    let analysis_service = AnalysisService::new(
        analysis_database,
        metadata,
        symbol_references,
        config.analyzer_settings,
        false, // no progress bars in WASM
    );

    let analysis_result = analysis_service.run().unwrap_or_else(|_| AnalysisResult::new(SymbolReferences::new()));

    let analyzer_issues = analysis_result.issues;
    let symbol_references = analysis_result.symbol_references;

    // Run formatting
    let format_file = File::ephemeral(Cow::Borrowed("code.php"), Cow::Owned(code.clone()));
    let format_file_id = format_file.id;
    let format_database = ReadDatabase::single(format_file);

    // Get the file from the database for formatting
    let file = format_database.get(&format_file_id).expect("File should exist in database");
    let format_service = FormatService::new(
        format_database,
        config.php_version,
        config.formatter_settings,
        false, // no progress bars in WASM
    );

    let formatted_code = format_service.format_file(&file).ok().and_then(|status| match status {
        FileFormatStatus::Changed(content) => Some(content),
        FileFormatStatus::Unchanged => Some(code),
        FileFormatStatus::FailedToParse(_) => None,
    });

    WasmAnalysisResults {
        parse_error,
        semantic_issues,
        linter_issues,
        analyzer_issues,
        symbol_references,
        formatted_code,
    }
}
