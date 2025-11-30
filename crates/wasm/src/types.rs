use serde::Deserialize;
use serde::Serialize;

use mago_database::file::File;
use mago_reporting::AnnotationKind;
use mago_reporting::Issue;
use mago_reporting::Level;

/// The source tool that reported the issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueSource {
    Linter,
    Analyzer,
    Both,
}

/// WASM-safe issue representation that uses line/column positions instead of byte offsets.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WasmIssue {
    /// The source tool that reported the issue.
    pub source: IssueSource,
    /// The severity level: "error", "warning", "note", or "help".
    pub level: String,
    /// An optional code identifying the issue type (e.g., "lint:redundant-final-method-modifier").
    pub code: Option<String>,
    /// The main message describing the issue.
    pub message: String,
    /// Additional notes related to the issue.
    pub notes: Vec<String>,
    /// An optional help message suggesting possible solutions.
    pub help: Option<String>,
    /// An optional link to external resources for more information.
    pub link: Option<String>,
    /// Annotations highlighting specific code locations.
    pub annotations: Vec<WasmAnnotation>,
}

/// WASM-safe annotation with line/column positions instead of byte offsets.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WasmAnnotation {
    /// An optional message associated with the annotation.
    pub message: Option<String>,
    /// The kind of annotation: "primary" or "secondary".
    pub kind: String,
    /// The starting line number (1-based).
    pub start_line: u32,
    /// The starting column number (1-based).
    pub start_column: u32,
    /// The ending line number (1-based).
    pub end_line: u32,
    /// The ending column number (1-based).
    pub end_column: u32,
}

impl WasmIssue {
    /// Converts an `Issue` to a `WasmIssue`, resolving byte offsets to line/column positions.
    ///
    /// # Arguments
    ///
    /// * `issue` - The issue to convert.
    /// * `file` - The file containing the issue, used to resolve positions.
    /// * `source` - The source tool that reported the issue.
    pub fn from_issue(issue: &Issue, file: &File, source: IssueSource) -> Self {
        let annotations = issue
            .annotations
            .iter()
            .map(|ann| {
                let start_offset = ann.span.start.offset;
                let end_offset = ann.span.end.offset;

                let start_line = file.line_number(start_offset) + 1;
                let start_column = file.column_number(start_offset) + 1;
                let end_line = file.line_number(end_offset) + 1;
                let end_column = file.column_number(end_offset) + 1;

                WasmAnnotation {
                    message: ann.message.clone(),
                    kind: match ann.kind {
                        AnnotationKind::Primary => "primary".to_string(),
                        AnnotationKind::Secondary => "secondary".to_string(),
                    },
                    start_line,
                    start_column,
                    end_line,
                    end_column,
                }
            })
            .collect();

        WasmIssue {
            source,
            level: match issue.level {
                Level::Error => "error".to_string(),
                Level::Warning => "warning".to_string(),
                Level::Note => "note".to_string(),
                Level::Help => "help".to_string(),
            },
            code: issue.code.clone(),
            message: issue.message.clone(),
            notes: issue.notes.clone(),
            help: issue.help.clone(),
            link: issue.link.clone(),
            annotations,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WasmRuleInfo {
    pub code: String,
    pub name: String,
    pub description: String,
    pub category: String,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct WasmAnalyzerSettings {
    pub find_unused_expressions: bool,
    pub find_unused_definitions: bool,
    pub analyze_dead_code: bool,
    pub memoize_properties: bool,
    pub allow_possibly_undefined_array_keys: bool,
    pub check_throws: bool,
    pub perform_heuristic_checks: bool,
    pub strict_list_index_checks: bool,
    pub no_boolean_literal_comparison: bool,
    pub check_missing_type_hints: bool,
    pub check_closure_missing_type_hints: bool,
    pub check_arrow_function_missing_type_hints: bool,
    pub register_super_globals: bool,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct WasmLinterSettings {
    pub disabled_rules: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct WasmSettings {
    pub php_version: String,
    pub analyzer: WasmAnalyzerSettings,
    pub linter: WasmLinterSettings,
}
