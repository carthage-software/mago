use serde_json::json;

use super::harness::Harness;
use crate::language_server::ServerConfig;

#[tokio::test]
async fn disabled_analyzer_keeps_declarations_current_without_type_diagnostics() {
    let config = ServerConfig { analyzer: false, linter: false, ..ServerConfig::default() };
    let base = "<?php class Original {} function work(): int { return missing(); }";
    let mut h =
        Harness::start_with(&[("a.php", base), ("b.php", "<?php new Original(); work();")], config, json!({})).await;
    let definition = h.at("textDocument/definition", "b.php", 0, 12).await;
    assert_eq!(definition["uri"], h.url("a.php"));
    let symbols = h.request("workspace/symbol", json!({"query":"Original"})).await;
    assert_eq!(symbols.as_array().unwrap().len(), 1);
    let diagnostics = h.diagnostics().await;
    assert!(diagnostics.iter().all(|message| message["diagnostics"].as_array().unwrap().is_empty()), "{diagnostics:?}");

    h.open("a.php", base).await;
    h.change("a.php", "<?php class Replacement {} function work(): string { return missing(); }", 2).await;
    assert!(h.at("textDocument/definition", "b.php", 0, 12).await.is_null());
    let symbols = h.request("workspace/symbol", json!({"query":"Replacement"})).await;
    assert_eq!(symbols.as_array().unwrap().len(), 1);
    h.close("a.php").await;
    assert_eq!(h.at("textDocument/definition", "b.php", 0, 12).await["uri"], h.url("a.php"));
}

#[tokio::test]
async fn syntax_errors_remain_visible_with_analyzer_and_linter_disabled() {
    let config = ServerConfig { analyzer: false, linter: false, ..ServerConfig::default() };
    let mut h = Harness::start_with(&[("a.php", "<?php function broken( {")], config, json!({})).await;
    let initial = h.client.await_notification("textDocument/publishDiagnostics").await;
    assert!(!initial["params"]["diagnostics"].as_array().unwrap().is_empty());
    h.open("a.php", "<?php function fixed() {}").await;
    let fixed = h.client.await_notification("textDocument/publishDiagnostics").await;
    assert_eq!(fixed["params"]["version"], 1);
    assert!(fixed["params"]["diagnostics"].as_array().unwrap().is_empty());
}
