//! Initialize handshake; the foundational test the entire harness rests on.

use std::process;

use serde_json::json;

use super::client::LspClient;
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
    assert_eq!(response["result"]["serverInfo"]["name"], "mago-server");

    client.send_notification("initialized", json!({})).await;
    let shutdown_id = client.send_request("shutdown", json!(null)).await;
    let _ = client.await_response(shutdown_id, 10).await;
    client.send_notification("exit", json!(null)).await;
    let _ = shutdown_tx.send(());
    let _ = server.await;
}
