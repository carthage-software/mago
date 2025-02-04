//! # Mago WASM Bindings
//!
//! This crate provides [wasm-bindgen] exports that wrap Mago’s internal
//! functionality (formatter, parser, linter, etc.) so they can be called
//! from JavaScript in a WebAssembly environment.
//!
//! ## Overview
//!
//! - **`mago_get_definitions`**: Returns metadata about all available
//!   plugins and rules in Mago.
//! - **`mago_analysis`**: Parses, lints, and optionally formats a given
//!   PHP snippet and returns structured results.
//! - **`mago_format`**: Formats a given PHP snippet according to the
//!   specified [FormatSettings].
//!
//! See each function’s documentation below for details on usage and
//! return values.

use wasm_bindgen::prelude::*;

use mago_formatter::settings::FormatSettings;
use mago_interner::ThreadedInterner;
use mago_linter::definition::PluginDefinition;
use mago_linter::definition::RuleDefinition;
use mago_linter::plugin::Plugin;
use mago_linter::settings::Settings;
use mago_parser::parse_source;
use mago_php_version::PHPVersion;
use mago_source::SourceCategory;
use mago_source::SourceManager;

use crate::analysis::AnalysisResults;

/// The `analysis` module contains data structures and logic
/// used to run combined parsing, linting, and formatting
/// operations within this WASM crate.
pub mod analysis;

/// Returns an array (serialized to JS via [`JsValue`]) of **all** plugin
/// definitions and their associated rules.
///
/// Each element in the returned array is a tuple:
/// `(PluginDefinition, Vec<RuleDefinition>)`.
///
/// # Errors
///
/// Returns a [`JsValue`] error if the serialization to
/// [`JsValue`] fails (unlikely).
///
/// # Example (JS Usage)
/// ```js
/// import init, { mago_get_definitions } from 'mago_wasm';
///
/// await init();
/// const defs = mago_get_definitions();
/// console.log(defs); // An array of plugin/rule definitions
/// ```
#[wasm_bindgen]
pub fn mago_get_definitions() -> Result<JsValue, JsValue> {
    let mut plugins: Vec<(PluginDefinition, Vec<RuleDefinition>)> = Vec::new();

    // Gather all plugin definitions and their rules
    mago_linter::foreach_plugin!(|p| {
        let plugin: Box<dyn Plugin> = Box::new(p);
        let definition = plugin.get_definition();
        let rules = plugin.get_rules().into_iter().map(|r| r.get_definition()).collect::<Vec<_>>();
        plugins.push((definition, rules));
    });

    // Serialize to JS-friendly output
    Ok(serde_wasm_bindgen::to_value(&plugins)?)
}

/// Performs a full “analysis” of the given `code` string:
/// 1. **Parse** the PHP code and detect any syntax errors.
/// 2. **Lint** the code using the provided linter settings.
/// 3. **Optionally** format the code if `format_settings` is provided.
///
/// Returns an [`AnalysisResults`] object (serialized to JS), which
/// contains any parse errors, semantic issues, linter issues, and
/// the formatted code (if no syntax errors were encountered).
///
/// # Arguments
///
/// * `code` - A string containing the PHP code to analyze.
/// * `format_settings` - A [`JsValue`] representing a
///   [`FormatSettings`](mago_formatter::settings::FormatSettings)
///   struct, or `null`/`undefined` to use the default settings.
/// * `linter_settings` - A [`JsValue`] representing a
///   [`Settings`](mago_linter::settings::Settings) struct, or
///   `null`/`undefined` to use the default settings (PHP 8.4).
///
/// # Errors
///
/// Returns a [`JsValue`] (string) error if deserialization of
/// the provided settings fails, or if parsing/analysis fails.
///
/// # Example (JS Usage)
/// ```js
/// import init, { mago_analysis } from 'mago_wasm';
///
/// await init();
/// const code = `<?php echo "Hello World"; ?>`;
/// const formatSettings = { indent_size: 2 };
/// const linterSettings = { php_version: "8.1" };
///
/// const analysis = mago_analysis(code, formatSettings, linterSettings);
/// console.log(analysis); // { parse_error: null, linter_issues: [...], formatted: "...", etc. }
/// ```
#[wasm_bindgen]
pub fn mago_analysis(code: String, format_settings: JsValue, linter_settings: JsValue) -> Result<JsValue, JsValue> {
    // Deserialize or use defaults
    let linter_settings = if !linter_settings.is_undefined() && !linter_settings.is_null() {
        serde_wasm_bindgen::from_value::<Settings>(linter_settings)?
    } else {
        Settings::new(PHPVersion::PHP84)
    };

    let format_settings = if !format_settings.is_undefined() && !format_settings.is_null() {
        serde_wasm_bindgen::from_value::<FormatSettings>(format_settings)?
    } else {
        FormatSettings::default()
    };

    // Run analysis
    let results = AnalysisResults::analyze(code, linter_settings, format_settings);

    // Return the analysis result as a JS object
    Ok(serde_wasm_bindgen::to_value(&results)?)
}

/// Formats the provided `code` string with the given
/// [`FormatSettings`], returning the resulting string.
///
/// # Arguments
///
/// * `code` - The PHP code to be formatted.
/// * `format_settings` - A [`JsValue`] representing
///   a [`FormatSettings`](mago_formatter::settings::FormatSettings)
///   struct, or `null`/`undefined` to use defaults.
///
/// # Errors
///
/// Returns a [`JsValue`] (string) error if the code is invalid
/// (i.e., parse error) or if deserialization of `format_settings` fails.
///
/// # Example (JS Usage)
/// ```js
/// import init, { mago_format } from 'mago_wasm';
///
/// await init();
/// const code = `<?php echo "Hello"; ?>`;
/// const fmtSet = { indent_size: 4, max_line_length: 100 };
///
/// try {
///   const formatted = mago_format(code, fmtSet);
///   console.log(formatted);
/// } catch (e) {
///   console.error("Formatting failed:", e);
/// }
/// ```
#[wasm_bindgen]
pub fn mago_format(code: String, format_settings: JsValue) -> Result<JsValue, JsValue> {
    // Deserialize or default
    let settings = if !format_settings.is_undefined() && !format_settings.is_null() {
        serde_wasm_bindgen::from_value::<FormatSettings>(format_settings)?
    } else {
        FormatSettings::default()
    };

    // Prepare interner and source manager
    let interner = ThreadedInterner::new();
    let manager = SourceManager::new(interner.clone());
    let source_id = manager.insert_content("code.php", code, SourceCategory::UserDefined);

    let source = manager.load(&source_id).map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Parse the code
    let (program, parse_error) = parse_source(&interner, &source);

    if let Some(err) = parse_error {
        return Err(JsValue::from_str(&err.to_string()));
    }

    // Format the parsed program
    let formatted = mago_formatter::format(&interner, &source, &program, settings);

    // Return the formatted string
    Ok(JsValue::from_str(&formatted))
}
