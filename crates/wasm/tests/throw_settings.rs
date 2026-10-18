#![allow(clippy::unwrap_used)]

use mago_php_version::PHPVersion;
use mago_wasm::WasmAnalyzerSettings;
use mago_wasm::build_analyzer_settings;

#[test]
fn overly_wide_throw_option_is_forwarded_from_playground_settings() {
    let settings: WasmAnalyzerSettings =
        serde_json::from_value(serde_json::json!({ "findOverlyWideThrowTypes": true })).unwrap();

    let analyzer_settings = build_analyzer_settings(&settings, PHPVersion::PHP85);

    assert!(analyzer_settings.find_overly_wide_throw_types);
    assert!(!analyzer_settings.check_throws);
}

#[test]
fn overly_wide_throw_option_defaults_to_disabled_in_playground_settings() {
    let settings: WasmAnalyzerSettings = serde_json::from_value(serde_json::json!({})).unwrap();

    let analyzer_settings = build_analyzer_settings(&settings, PHPVersion::PHP85);

    assert!(!analyzer_settings.find_overly_wide_throw_types);
}
