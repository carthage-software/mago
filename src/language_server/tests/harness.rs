//! Run protocol requests against a temporary workspace and an in-process server.

use std::path::Path;
use std::path::PathBuf;
use std::process;

use serde_json::Value;
use serde_json::json;
use tempfile::TempDir;
use tower_lsp_server::gen_lsp_types::Uri;

use super::client::LspClient;
use crate::language_server::ServerConfig;
use crate::language_server::uri::UriFilePath;

const BOOTSTRAP_TIMEOUT_SECS: u64 = 60;
const REQUEST_TIMEOUT_SECS: u64 = 10;

pub struct Harness {
    pub client: LspClient,
    pub workspace: PathBuf,
    pub server_capabilities: Value,
    _dir: TempDir,
    _server: tokio::task::JoinHandle<()>,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl Harness {
    /// Spin up a workspace populated with `(name, contents)` pairs and
    /// wait for the bootstrap to finish.
    pub async fn start(files: &[(&str, &str)]) -> Self {
        Self::start_with(files, ServerConfig::default(), client_capabilities()).await
    }

    /// Start with explicit server settings and client capabilities.
    pub async fn start_with(files: &[(&str, &str)], config: ServerConfig, capabilities: Value) -> Self {
        Self::start_with_root(files, config, capabilities, None).await
    }

    /// Override the root sent by the client without setting a server workspace override.
    pub async fn start_with_root(
        files: &[(&str, &str)],
        config: ServerConfig,
        capabilities: Value,
        root: Option<&Path>,
    ) -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let workspace = dir.path().to_path_buf();
        for (name, contents) in files {
            let path = workspace.join(name);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, contents).unwrap();
        }

        let (client_stream, server_stream) = tokio::io::duplex(256 * 1024);
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        let server = tokio::spawn(async move {
            let (read_half, write_half) = tokio::io::split(server_stream);
            tokio::select! {
                _ = crate::language_server::serve(read_half, write_half, config) => {}
                _ = shutdown_rx => {}
            }
        });

        let mut client = LspClient::new(client_stream);
        let init_id = client
            .send_request(
                "initialize",
                json!({
                    "processId": process::id(),
                    "rootUri": url(root.unwrap_or(&workspace)),
                    "capabilities": capabilities,
                }),
            )
            .await;
        let response = client.await_response(init_id, BOOTSTRAP_TIMEOUT_SECS).await;
        assert!(response.get("error").is_none(), "initialize failed: {response}");
        client.send_notification("initialized", json!({})).await;

        Self {
            client,
            workspace,
            server_capabilities: response["result"]["capabilities"].clone(),
            _dir: dir,
            _server: server,
            shutdown_tx: Some(shutdown_tx),
        }
    }

    pub fn url(&self, name: &str) -> String {
        url(&self.workspace.canonicalize().unwrap().join(name))
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
    }

    pub async fn close(&mut self, name: &str) {
        self.client
            .send_notification("textDocument/didClose", json!({ "textDocument": { "uri": self.url(name) } }))
            .await;
    }

    pub async fn watched(&mut self, name: &str, kind: i32) {
        self.client
            .send_notification(
                "workspace/didChangeWatchedFiles",
                json!({ "changes": [{ "uri": self.url(name), "type": kind }] }),
            )
            .await;
    }

    pub async fn request(&mut self, method: &str, params: Value) -> Value {
        let id = self.client.send_request(method, params).await;
        let response = self.client.await_response(id, REQUEST_TIMEOUT_SECS).await;
        assert!(response.get("error").is_none(), "{method} failed: {response}");
        response.get("result").unwrap_or_else(|| panic!("{method} returned no result: {response}")).clone()
    }

    /// Return a request's result, or its error object.
    pub async fn try_request(&mut self, method: &str, params: Value) -> Result<Value, Value> {
        let id = self.client.send_request(method, params).await;
        let mut response = self.client.await_response(id, REQUEST_TIMEOUT_SECS).await;
        if let Some(error) = response.get_mut("error") {
            return Err(error.take());
        }
        Ok(response.get_mut("result").unwrap_or_else(|| panic!("{method} returned no result")).take())
    }

    /// Wait until all earlier handlers' diagnostics reach the client.
    pub async fn diagnostics(&mut self) -> Vec<Value> {
        const MARKER: &str = "mago/testBarrier";
        self.client.send_notification(MARKER, json!({ "type": 4, "message": MARKER })).await;
        loop {
            let message = self.client.await_notification("window/logMessage").await;
            if message["params"]["message"] == MARKER {
                break;
            }
        }
        self.client
            .take_pending_notifications()
            .into_iter()
            .filter(|message| message["method"] == "textDocument/publishDiagnostics")
            .map(|message| message["params"].clone())
            .collect()
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

    /// Spin up a server with several workspace folders, each a subdirectory of
    /// a shared temp dir populated with `(name, contents)` files. Initializes
    /// via `workspaceFolders` so the registry holds one workspace per folder.
    pub async fn start_multi(folders: &[(&str, &[(&str, &str)])]) -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_path_buf();
        let mut folder_entries = Vec::new();
        for (folder, files) in folders {
            let folder_path = root.join(folder);
            std::fs::create_dir_all(&folder_path).unwrap();
            for (name, contents) in *files {
                std::fs::write(folder_path.join(name), contents).unwrap();
            }
            folder_entries.push(json!({ "uri": url(&folder_path), "name": folder }));
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
                    "workspaceFolders": folder_entries,
                    "capabilities": client_capabilities(),
                }),
            )
            .await;
        let response = client.await_response(init_id, BOOTSTRAP_TIMEOUT_SECS).await;
        assert!(response.get("error").is_none(), "initialize failed: {response}");
        client.send_notification("initialized", json!({})).await;

        Self {
            client,
            workspace: root,
            server_capabilities: response["result"]["capabilities"].clone(),
            _dir: dir,
            _server: server,
            shutdown_tx: Some(shutdown_tx),
        }
    }

    /// URL of `name` inside workspace subfolder `folder` (for [`start_multi`]).
    pub fn url_in(&self, folder: &str, name: &str) -> String {
        url(&self.workspace.canonicalize().unwrap().join(folder).join(name))
    }

    /// Create a new workspace subfolder, populate it, and notify the server via
    /// `workspace/didChangeWorkspaceFolders`.
    pub async fn add_folder(&mut self, folder: &str, files: &[(&str, &str)]) {
        let folder_path = self.workspace.join(folder);
        std::fs::create_dir_all(&folder_path).unwrap();
        for (name, contents) in files {
            std::fs::write(folder_path.join(name), contents).unwrap();
        }
        self.client
            .send_notification(
                "workspace/didChangeWorkspaceFolders",
                json!({
                    "event": {
                        "added": [{ "uri": url(&folder_path), "name": folder }],
                        "removed": []
                    }
                }),
            )
            .await;
    }

    /// Like [`at`](Self::at) but addressing a file by full URL.
    pub async fn at_uri(&mut self, method: &str, uri: &str, line: u32, character: u32) -> Value {
        self.request(
            method,
            json!({ "textDocument": { "uri": uri }, "position": { "line": line, "character": character } }),
        )
        .await
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

pub fn client_capabilities() -> Value {
    json!({ "textDocument": {
        "publishDiagnostics": { "relatedInformation": true, "codeDescriptionSupport": true, "tagSupport": { "valueSet": [1, 2] } },
        "codeAction": {
            "codeActionLiteralSupport": { "codeActionKind": { "valueSet": ["quickfix", "refactor"] } },
            "isPreferredSupport": true
        },
        "completion": { "completionItem": { "snippetSupport": true } },
        "rename": { "prepareSupport": true },
        "signatureHelp": { "signatureInformation": {
            "parameterInformation": { "labelOffsetSupport": true }, "activeParameterSupport": true
        } }
    } })
}

pub fn url(path: &Path) -> String {
    let Some(uri) = Uri::from_file_path(path) else {
        panic!("test workspace path is not absolute: {}", path.display());
    };

    uri.to_string()
}
