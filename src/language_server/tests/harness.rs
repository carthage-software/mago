//! Shared LSP test harness.
//!
//! Each test should fit in ~10 lines: spin up via [`Harness::start`], call
//! one of the dedicated capability helpers (e.g. [`Harness::folding`],
//! [`Harness::hover_at`]), and assert.

#![allow(dead_code)]

use std::path::Path;
use std::path::PathBuf;
use std::process;

use serde_json::Value;
use serde_json::json;
use tempfile::TempDir;

use super::client::LspClient;
use crate::language_server::ServerConfig;

const BOOTSTRAP_TIMEOUT_SECS: u64 = 60;
const REQUEST_TIMEOUT_SECS: u64 = 10;
const SETTLE_MS: u64 = 500;

pub struct Harness {
    pub client: LspClient,
    pub workspace: PathBuf,
    _dir: TempDir,
    _server: tokio::task::JoinHandle<()>,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl Harness {
    /// Spin up a workspace populated with `(name, contents)` pairs and
    /// wait for the bootstrap to finish.
    pub async fn start(files: &[(&str, &str)]) -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let workspace = dir.path().to_path_buf();
        for (name, contents) in files {
            std::fs::write(workspace.join(name), contents).unwrap();
        }

        let (client_stream, server_stream) = tokio::io::duplex(256 * 1024);
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        let server = tokio::spawn(async move {
            let (read_half, write_half) = tokio::io::split(server_stream);
            tokio::select! {
                _ = crate::language_server::serve(read_half, write_half, ServerConfig::default()) => {}
                _ = shutdown_rx => {}
            }
        });

        let mut client = LspClient::new(client_stream);
        let init_id = client
            .send_request(
                "initialize",
                json!({
                    "processId": process::id(),
                    "rootUri": url(&workspace),
                    "capabilities": {},
                }),
            )
            .await;
        let _ = client.await_response(init_id, BOOTSTRAP_TIMEOUT_SECS).await;
        client.send_notification("initialized", json!({})).await;

        let mut harness = Self { client, workspace, _dir: dir, _server: server, shutdown_tx: Some(shutdown_tx) };
        harness.wait_for_ready(files.first().map(|(n, _)| *n).unwrap_or("__bootstrap__.php")).await;
        harness
    }

    pub fn url(&self, name: &str) -> String {
        url(&self.workspace.canonicalize().unwrap().join(name))
    }

    /// Poll a cheap capability until the workspace transitions to `Ready`.
    pub async fn wait_for_ready(&mut self, name: &str) {
        let uri = self.url(name);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(BOOTSTRAP_TIMEOUT_SECS);
        loop {
            if std::time::Instant::now() > deadline {
                panic!("server never became ready");
            }
            let resp = self.request("textDocument/foldingRange", json!({ "textDocument": { "uri": &uri } })).await;
            if !resp.is_null() {
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }

    pub async fn open(&mut self, name: &str, contents: &str) {
        self.client
            .send_notification(
                "textDocument/didOpen",
                json!({
                    "textDocument": {
                        "uri": self.url(name),
                        "languageId": "php",
                        "version": 1,
                        "text": contents,
                    }
                }),
            )
            .await;
        self.settle().await;
    }

    pub async fn change(&mut self, name: &str, contents: &str, version: i32) {
        self.client
            .send_notification(
                "textDocument/didChange",
                json!({
                    "textDocument": { "uri": self.url(name), "version": version },
                    "contentChanges": [{ "text": contents }]
                }),
            )
            .await;
        self.settle().await;
    }

    pub async fn close(&mut self, name: &str) {
        self.client
            .send_notification("textDocument/didClose", json!({ "textDocument": { "uri": self.url(name) } }))
            .await;
        self.settle().await;
    }

    pub async fn watched(&mut self, name: &str, kind: i32) {
        self.client
            .send_notification(
                "workspace/didChangeWatchedFiles",
                json!({ "changes": [{ "uri": self.url(name), "type": kind }] }),
            )
            .await;
        self.settle().await;
    }

    /// Sleep just long enough for an async notification to be processed
    /// before a follow-up request races ahead.
    pub async fn settle(&mut self) {
        tokio::time::sleep(std::time::Duration::from_millis(SETTLE_MS)).await;
    }

    pub async fn request(&mut self, method: &str, params: Value) -> Value {
        let id = self.client.send_request(method, params).await;
        self.client.await_response(id, REQUEST_TIMEOUT_SECS).await["result"].clone()
    }

    pub async fn at(&mut self, method: &str, name: &str, line: u32, character: u32) -> Value {
        let uri = self.url(name);
        self.request(
            method,
            json!({ "textDocument": { "uri": uri }, "position": { "line": line, "character": character } }),
        )
        .await
    }

    pub async fn for_doc(&mut self, method: &str, name: &str) -> Value {
        let uri = self.url(name);
        self.request(method, json!({ "textDocument": { "uri": uri } })).await
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

pub fn url(path: &Path) -> String {
    let display = path.display().to_string();
    if display.starts_with('/') { format!("file://{display}") } else { format!("file:///{display}") }
}
