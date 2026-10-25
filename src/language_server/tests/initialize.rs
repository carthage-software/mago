//! Initialize handshake; the foundational test the entire harness rests on.

use std::process;

use serde_json::json;

use super::client::LspClient;
use super::harness::Harness;
use super::harness::url;
use crate::language_server::ServerConfig;

#[tokio::test]
async fn initialize_handshake() {
    let dir = tempfile::tempdir().expect("tempdir");
    let workspace = dir.path().to_path_buf();
    std::fs::write(workspace.join("hello.php"), "<?php\necho 'hi';\n").unwrap();

    let (client_stream, server_stream) = tokio::io::duplex(64 * 1024);
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let server = tokio::spawn(async move {
        let (rh, wh) = tokio::io::split(server_stream);
        tokio::select! {
            _ = crate::language_server::serve(rh, wh, ServerConfig::default()) => {}
            _ = shutdown_rx => {}
        }
    });

    let mut client = LspClient::new(client_stream);
    let init_id = client
        .send_request(
            "initialize",
            json!({
                "processId": process::id(),
                "rootUri": url(&workspace.canonicalize().unwrap()),
                "capabilities": {},
            }),
        )
        .await;
    let response = client.await_response(init_id, 30).await;
    assert!(response["result"]["capabilities"].is_object());
    assert!(response["result"]["capabilities"].get("semanticTokensProvider").is_none());
    assert_eq!(response["result"]["serverInfo"]["name"], "mago-server");

    client.send_notification("initialized", json!({})).await;
    let shutdown_id = client.send_request("shutdown", json!(null)).await;
    let _ = client.await_response(shutdown_id, 10).await;
    client.send_notification("exit", json!(null)).await;
    let _ = shutdown_tx.send(());
    let _ = server.await;
}

#[tokio::test]
async fn initial_diagnostics_reach_the_client_once() {
    let mut h = Harness::start(&[("a.php", "<?php\nmissing_function();\n")]).await;
    let initial = h.client.await_notification("textDocument/publishDiagnostics").await;
    assert_eq!(initial["params"]["uri"], h.url("a.php"));
    assert!(initial["params"]["diagnostics"].as_array().is_some_and(|issues| !issues.is_empty()));

    h.client.send_notification("initialized", json!({})).await;
    let repeated = h.diagnostics().await;
    assert!(repeated.is_empty(), "{repeated:?}");
}

#[tokio::test]
async fn editor_settings_do_not_override_workspace_configuration() {
    for disabled in [false, true] {
        let config = format!("[language-server]\ndisable = {}\n", if disabled { "['hover']" } else { "[]" });
        let mut h = Harness::start(&[("a.php", "<?php class Example {}"), ("mago.toml", &config)]).await;
        h.client
            .send_notification(
                "workspace/didChangeConfiguration",
                json!({
                    "settings": { "language-server": { "disable": if disabled { vec![] } else { vec!["hover"] } } }
                }),
            )
            .await;
        assert_eq!(h.at("textDocument/hover", "a.php", 0, 14).await.is_null(), disabled);
        assert!(h.client.server_requests().is_empty());
    }
}
