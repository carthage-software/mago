//! # Mago WASM Bindings
//!
//! This crate provides WebAssembly bindings for Mago's linting, static analysis,
//! and formatting functionality, designed to work in browser environments.

use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

use wasm_bindgen::prelude::*;

use mago_analyzer::plugin::available_plugins;
use mago_analyzer::plugin::create_registry_with_plugins;
use mago_analyzer::settings::Settings as AnalyzerSettings;
use mago_atom::ascii_lowercase_atom;
use mago_atom::atom;
use mago_database::DatabaseReader;
use mago_database::ReadDatabase;
use mago_database::file::File;
use mago_formatter::Formatter;
use mago_formatter::settings::FormatSettings;
use mago_linter::registry::RuleRegistry;
use mago_linter::rule::AnyRule;
use mago_linter::settings::Settings as LinterSettings;
use mago_orchestrator::service::analysis::AnalysisService;
use mago_orchestrator::service::lint::LintMode;
use mago_orchestrator::service::lint::LintService;
use mago_php_version::PHPVersion;
use mago_prelude::Prelude;
use mago_syntax::parser::parse_file;
use mago_syntax::settings::ParserSettings;

use crate::types::IssueSource;
use crate::types::WasmIssue;
use crate::types::WasmPluginInfo;
use crate::types::WasmRuleInfo;
use crate::types::WasmSettings;

mod types;

/// Embedded prelude containing PHP built-in symbols.
const PRELUDE_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/prelude.bin"));

/// Parses a PHP version string into a `PHPVersion`.
///
/// Accepts formats like "8.4", "8.3", "8.2", etc.
/// Returns `PHPVersion::default()` if parsing fails.
fn parse_php_version(version: &str) -> PHPVersion {
    match version {
        "8.6" => PHPVersion::PHP86,
        "8.5" => PHPVersion::PHP85,
        "8.4" => PHPVersion::PHP84,
        "8.3" => PHPVersion::PHP83,
        "8.2" => PHPVersion::PHP82,
        "8.1" => PHPVersion::PHP81,
        "8.0" => PHPVersion::PHP80,
        _ => PHPVersion::default(),
    }
}

/// Load and decode the embedded prelude.
fn load_prelude() -> Prelude {
    Prelude::decode(PRELUDE_BYTES).expect("Failed to decode embedded prelude")
}

fn issue_key(issue: &WasmIssue) -> Option<(String, u32, u32)> {
    let code = issue.code.as_ref()?;
    let ann = issue.annotations.first()?;
    Some((code.clone(), ann.start_line, ann.start_column))
}

#[wasm_bindgen]
pub fn run(code: String, settings_js: JsValue) -> Result<JsValue, JsValue> {
    let settings: WasmSettings = serde_wasm_bindgen::from_value(settings_js).unwrap_or_default();

    let version = parse_php_version(&settings.php_version);
    let file = File::ephemeral(Cow::Borrowed("code.php"), Cow::Owned(code));
    let file_id = file.id;

    let disabled_rules: HashSet<String> = settings.linter.disabled_rules.into_iter().collect();

    let linter_settings = LinterSettings { php_version: version, ..Default::default() };
    let database = ReadDatabase::empty();
    let service = LintService::new(database, linter_settings, ParserSettings::default(), false);
    let linter_issues = service.lint_file(&file, LintMode::Full, None);

    let mut issue_map: HashMap<(String, u32, u32), WasmIssue> = HashMap::new();
    let mut issues_without_key: Vec<WasmIssue> = Vec::new();

    for issue in &linter_issues {
        if let Some(code) = &issue.code
            && disabled_rules.contains(code)
        {
            continue;
        }

        let wasm_issue = WasmIssue::from_issue(issue, &file, IssueSource::Linter);
        if let Some(key) = issue_key(&wasm_issue) {
            issue_map.insert(key, wasm_issue);
        } else {
            issues_without_key.push(wasm_issue);
        }
    }

    let mut prelude = load_prelude();
    prelude.database.add(file);

    let s = &settings.analyzer;
    let analyzer_settings = AnalyzerSettings {
        version,
        find_unused_expressions: s.find_unused_expressions,
        find_unused_definitions: s.find_unused_definitions,
        analyze_dead_code: s.analyze_dead_code,
        memoize_properties: s.memoize_properties,
        allow_possibly_undefined_array_keys: s.allow_possibly_undefined_array_keys,
        check_throws: s.check_throws,
        unchecked_exceptions: s.unchecked_exceptions.iter().map(|e| atom(e.as_str())).collect(),
        unchecked_exception_classes: s.unchecked_exception_classes.iter().map(|e| atom(e.as_str())).collect(),
        check_missing_override: s.check_missing_override,
        find_unused_parameters: s.find_unused_parameters,
        strict_list_index_checks: s.strict_list_index_checks,
        no_boolean_literal_comparison: s.no_boolean_literal_comparison,
        check_missing_type_hints: s.check_missing_type_hints,
        check_closure_missing_type_hints: s.check_closure_missing_type_hints,
        check_arrow_function_missing_type_hints: s.check_arrow_function_missing_type_hints,
        register_super_globals: s.register_super_globals,
        trust_existence_checks: s.trust_existence_checks,
        class_initializers: s.class_initializers.iter().map(|s| ascii_lowercase_atom(s.as_str())).collect(),
        check_property_initialization: s.check_property_initialization,
        ..Default::default()
    };

    let plugin_registry = Arc::new(create_registry_with_plugins(&s.plugins, s.disable_default_plugins));
    let analysis_service = AnalysisService::new(
        prelude.database.read_only(),
        prelude.metadata,
        prelude.symbol_references,
        analyzer_settings,
        ParserSettings::default(),
        false,
        plugin_registry,
    );

    let analyzer_issues = analysis_service.oneshot(file_id);

    let file = prelude
        .database
        .get_ref(&file_id)
        .expect("File should exist in prelude database after being added prior to analysis");

    for issue in &analyzer_issues {
        let wasm_issue = WasmIssue::from_issue(issue, file, IssueSource::Analyzer);
        if let Some(key) = issue_key(&wasm_issue) {
            if let Some(existing) = issue_map.get_mut(&key) {
                existing.source = IssueSource::Both;
            } else {
                issue_map.insert(key, wasm_issue);
            }
        } else {
            issues_without_key.push(wasm_issue);
        }
    }

    let mut all_issues: Vec<WasmIssue> = issue_map.into_values().collect();
    all_issues.extend(issues_without_key);

    serde_wasm_bindgen::to_value(&all_issues).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Lints PHP code and returns issues as a JavaScript array.
///
/// This runs only the linter (style, best practices, etc.) without static analysis.
///
/// # Arguments
///
/// * `code` - The PHP source code to lint.
/// * `php_version` - The PHP version to use for analysis (e.g., "8.4").
///
/// # Returns
///
/// A JavaScript array of linter issue objects.
#[wasm_bindgen]
pub fn lint(code: String, php_version: &str) -> Result<JsValue, JsValue> {
    let version = parse_php_version(php_version);
    let file = File::ephemeral(Cow::Borrowed("code.php"), Cow::Owned(code));
    let settings = LinterSettings { php_version: version, ..Default::default() };

    let database = ReadDatabase::empty();
    let service = LintService::new(database, settings, ParserSettings::default(), false);
    let issues = service.lint_file(&file, LintMode::Full, None);

    let wasm_issues: Vec<WasmIssue> =
        issues.iter().map(|i| WasmIssue::from_issue(i, &file, IssueSource::Linter)).collect();

    serde_wasm_bindgen::to_value(&wasm_issues).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Runs static analysis on PHP code and returns issues as a JavaScript array.
///
/// This runs only the static analyzer (type errors, undefined variables, etc.)
/// without linter rules. It includes the PHP prelude for accurate analysis.
///
/// # Arguments
///
/// * `code` - The PHP source code to analyze.
/// * `php_version` - The PHP version to use for analysis (e.g., "8.4").
///
/// # Returns
///
/// A JavaScript array of analyzer issue objects.
#[wasm_bindgen]
pub fn analyze(code: String, php_version: &str) -> Result<JsValue, JsValue> {
    let version = parse_php_version(php_version);
    let file = File::ephemeral(Cow::Borrowed("code.php"), Cow::Owned(code));
    let file_id = file.id;

    // Load prelude and use orchestrator's analyze_file
    let mut prelude = load_prelude();
    prelude.database.add(file);

    let settings = AnalyzerSettings {
        version,
        find_unused_expressions: true,
        find_unused_definitions: true,
        analyze_dead_code: true,
        check_throws: true,
        check_missing_override: false,
        find_unused_parameters: false,
        check_missing_type_hints: true,
        check_closure_missing_type_hints: true,
        check_arrow_function_missing_type_hints: true,
        register_super_globals: true,
        trust_existence_checks: true,
        ..Default::default()
    };

    // Use default plugin configuration (stdlib enabled by default)
    let plugin_registry = Arc::new(create_registry_with_plugins(&[], false));
    let service = AnalysisService::new(
        prelude.database.read_only(),
        prelude.metadata,
        prelude.symbol_references,
        settings,
        ParserSettings::default(),
        false,
        plugin_registry,
    );

    let issues = service.oneshot(file_id);

    let file = prelude
        .database
        .get_ref(&file_id)
        .expect("File should exist in prelude database after being added prior to analysis");

    let wasm_issues: Vec<WasmIssue> =
        issues.iter().map(|i| WasmIssue::from_issue(i, file, IssueSource::Analyzer)).collect();

    serde_wasm_bindgen::to_value(&wasm_issues).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Formats PHP code and returns the formatted result.
///
/// # Arguments
///
/// * `code` - The PHP source code to format.
/// * `php_version` - The PHP version to use for formatting (e.g., "8.4").
///
/// # Returns
///
/// The formatted PHP code as a string, or an error if parsing fails.
#[wasm_bindgen]
pub fn format(code: String, php_version: &str) -> Result<String, JsValue> {
    let version = parse_php_version(php_version);
    let file = File::ephemeral(Cow::Borrowed("code.php"), Cow::Owned(code));

    let arena = bumpalo::Bump::new();
    let program = parse_file(&arena, &file);
    if let Some(e) = program.errors.first() {
        return Err(JsValue::from_str(&format!("Parse error: {e}")));
    }

    let formatter = Formatter::new(&arena, version, FormatSettings::default());
    Ok(formatter.format(&file, program).to_string())
}

/// Returns metadata for all available linter rules.
///
/// # Returns
///
/// A JavaScript array of rule metadata objects.
#[wasm_bindgen(js_name = getRules)]
pub fn get_rules() -> Result<JsValue, JsValue> {
    let settings = LinterSettings::default();
    let registry = RuleRegistry::build(&settings, None, true);

    let rules: Vec<WasmRuleInfo> = registry
        .rules()
        .iter()
        .map(|rule| {
            let meta = AnyRule::meta(rule);
            WasmRuleInfo {
                code: meta.code.to_string(),
                name: meta.name.to_string(),
                description: meta.description.to_string(),
                category: meta.category.as_str().to_string(),
            }
        })
        .collect();

    serde_wasm_bindgen::to_value(&rules).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Returns metadata for all available analyzer plugins.
///
/// # Returns
///
/// A JavaScript array of plugin metadata objects.
#[wasm_bindgen(js_name = getPlugins)]
pub fn get_plugins() -> Result<JsValue, JsValue> {
    let plugins: Vec<WasmPluginInfo> = available_plugins()
        .into_iter()
        .map(|meta| WasmPluginInfo {
            id: meta.id.to_string(),
            name: meta.name.to_string(),
            description: meta.description.to_string(),
            aliases: meta.aliases.iter().map(|s| (*s).to_string()).collect(),
            default_enabled: meta.default_enabled,
        })
        .collect();

    serde_wasm_bindgen::to_value(&plugins).map_err(|e| JsValue::from_str(&e.to_string()))
}
