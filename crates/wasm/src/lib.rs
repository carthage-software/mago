//! # Mago WASM Bindings
//!
//! This crate provides [wasm-bindgen] exports that wrap Mago’s internal
//! functionality (formatter, parser, linter, etc.) so they can be called
//! from JavaScript in a WebAssembly environment.

use std::borrow::Cow;

use bumpalo::Bump;
use wasm_bindgen::prelude::*;

use mago_formatter::Formatter;
use mago_formatter::settings::FormatSettings;
use mago_php_version::PHPVersion;

mod analysis;
mod rules;
mod settings;

/// Runs the full analysis pipeline (parse, semantics, lint, analyze, format).
///
/// Takes a string of PHP code and a settings object, returning a comprehensive
/// analysis result.
#[wasm_bindgen(js_name = run)]
pub fn run(code: String, settings: JsValue) -> Result<JsValue, JsValue> {
    let settings: settings::WasmSettings = if !settings.is_undefined() && !settings.is_null() {
        serde_wasm_bindgen::from_value(settings)?
    } else {
        settings::WasmSettings::default()
    };

    let results = analysis::analyze_code(code, settings);

    Ok(serde_wasm_bindgen::to_value(&results)?)
}

/// Returns metadata for all available linter rules.
///
/// This allows a UI to dynamically display available rules and their descriptions.
#[wasm_bindgen(js_name = getRules)]
pub fn get_rules(linter_settings: JsValue) -> Result<JsValue, JsValue> {
    let settings = if !linter_settings.is_undefined() && !linter_settings.is_null() {
        serde_wasm_bindgen::from_value(linter_settings)?
    } else {
        mago_linter::settings::Settings::default()
    };

    let rules = rules::get_available_rules(settings);
    Ok(serde_wasm_bindgen::to_value(&rules)?)
}

/// Runs only the formatter on the given code.
///
/// This is a lightweight function for callers who only need to format code
/// without performing a full analysis.
#[wasm_bindgen(js_name = formatCode)]
pub fn format_code(code: String, php_version: JsValue, settings: JsValue) -> Result<String, JsValue> {
    let php_version: PHPVersion = if !php_version.is_undefined() && !php_version.is_null() {
        serde_wasm_bindgen::from_value(php_version)?
    } else {
        PHPVersion::default()
    };

    let settings: FormatSettings = if !settings.is_undefined() && !settings.is_null() {
        serde_wasm_bindgen::from_value(settings)?
    } else {
        FormatSettings::default()
    };

    let arena = Bump::new();
    let formatter = Formatter::new(&arena, php_version, settings);

    formatter
        .format_code(Cow::Borrowed("code.php"), Cow::Owned(code))
        .map(|s| s.to_string())
        .map_err(|err| JsValue::from_str(&err.to_string()))
}
