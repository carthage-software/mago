//! Tiny LSP client used by integration tests.
//!
//! Speaks JSON-RPC over a `tokio::io::DuplexStream` half; pair the other half
//! with [`mago_server::serve`] to drive the server in-process.

use std::sync::Arc;
use std::sync::atomic::AtomicI64;
use std::sync::atomic::Ordering;

use serde_json::Value;
use serde_json::json;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::DuplexStream;
use tokio::time::Duration;
use tokio::time::timeout;

pub struct LspClient {
    stream: DuplexStream,
    next_id: Arc<AtomicI64>,
    /// Persistent receive buffer; multiple LSP messages may arrive in a
    /// single read, so we accumulate and parse out one at a time.
    rx: Vec<u8>,
    /// Notifications received while waiting for a request response. Tests
    /// can drain this with [`Self::take_pending_notifications`] to inspect
    /// publishDiagnostics, log messages, etc., without racing the request
    /// loop.
    pending_notifications: Vec<Value>,
    server_requests: Vec<Value>,
}

impl LspClient {
    pub fn take_pending_notifications(&mut self) -> Vec<Value> {
        std::mem::take(&mut self.pending_notifications)
    }

    pub fn server_requests(&self) -> &[Value] {
        &self.server_requests
    }

    pub async fn await_notification(&mut self, method: &str) -> Value {
        if let Some(index) = self.pending_notifications.iter().position(|message| message["method"] == method) {
            return self.pending_notifications.remove(index);
        }
        loop {
            let message = self.read_message(10).await;
            if message["method"] == method && message.get("id").is_none() {
                return message;
            }
            if message.get("method").is_some() && message.get("id").is_some() {
                self.write(json!({ "jsonrpc": "2.0", "id": message["id"], "result": null })).await;
                self.server_requests.push(message);
            } else {
                self.pending_notifications.push(message);
            }
        }
    }

    pub fn new(stream: DuplexStream) -> Self {
        Self {
            stream,
            next_id: Arc::new(AtomicI64::new(1)),
            rx: Vec::with_capacity(8192),
            pending_notifications: Vec::new(),
            server_requests: Vec::new(),
        }
    }

    pub async fn send_request(&mut self, method: &str, params: Value) -> i64 {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);

        let request = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });

        self.write(request).await;

        id
    }

    pub async fn send_notification(&mut self, method: &str, params: Value) {
        let notification = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        });

        self.write(notification).await;
    }

    pub async fn respond(&mut self, id: Value, result: Value) {
        self.write(json!({ "jsonrpc": "2.0", "id": id, "result": result })).await;
    }

    pub async fn respond_error(&mut self, id: Value, error: Value) {
        self.write(json!({ "jsonrpc": "2.0", "id": id, "error": error })).await;
    }

    async fn write(&mut self, msg: Value) {
        let body = serde_json::to_string(&msg).expect("json encode");
        let framed = format!("Content-Length: {}\r\n\r\n{}", body.len(), body);
        self.stream.write_all(framed.as_bytes()).await.expect("lsp write");
    }

    pub async fn read_message(&mut self, timeout_secs: u64) -> Value {
        timeout(Duration::from_secs(timeout_secs), async {
            loop {
                if let Some(msg) = self.try_pop_message() {
                    return msg;
                }

                let mut chunk = [0u8; 4096];
                let n = self.stream.read(&mut chunk).await.expect("lsp read");
                if n == 0 {
                    panic!("lsp stream closed before a complete message arrived");
                }
                self.rx.extend_from_slice(&chunk[..n]);
            }
        })
        .await
        .expect("lsp read timed out")
    }

    /// Try to extract a single complete LSP message from `self.rx`.
    /// Returns `None` if more bytes are needed.
    fn try_pop_message(&mut self) -> Option<Value> {
        const SEPARATOR: &[u8] = b"\r\n\r\n";
        let separator_pos = self.rx.windows(SEPARATOR.len()).position(|w| w == SEPARATOR)?;
        let header = std::str::from_utf8(&self.rx[..separator_pos]).expect("lsp header utf-8");

        let content_length: usize = header
            .lines()
            .find_map(|line| line.strip_prefix("Content-Length:"))
            .map(|v| v.trim())
            .and_then(|v| v.parse().ok())
            .expect("Content-Length header");

        let body_start = separator_pos + SEPARATOR.len();
        if self.rx.len() < body_start + content_length {
            return None;
        }

        let body_end = body_start + content_length;
        let value: Value = serde_json::from_slice(&self.rx[body_start..body_end]).expect("lsp body json");

        self.rx.drain(..body_end);

        Some(value)
    }

    /// Read messages until one matches the given response id. Any
    /// notifications observed while waiting are stashed in
    /// `pending_notifications` so tests don't accidentally drop them.
    pub async fn await_response(&mut self, expected_id: i64, timeout_secs: u64) -> Value {
        loop {
            let msg = self.read_message(timeout_secs).await;
            if msg.get("method").is_none() && msg.get("id").and_then(|v| v.as_i64()) == Some(expected_id) {
                return msg;
            }
            if msg.get("method").is_some() && msg.get("id").is_none() {
                // Notification; keep it for later inspection.
                self.pending_notifications.push(msg);
            } else if msg.get("method").is_some() {
                self.write(json!({ "jsonrpc": "2.0", "id": msg["id"], "result": null })).await;
                self.server_requests.push(msg);
            }
        }
    }
}

#[tokio::test]
async fn server_request_with_same_id_does_not_complete_client_request() {
    let (client_stream, server_stream) = tokio::io::duplex(4096);
    let mut client = LspClient::new(client_stream);
    let mut server = LspClient::new(server_stream);
    let id = client.send_request("workspace/executeCommand", json!({ "command": "test" })).await;
    assert_eq!(server.read_message(1).await["id"], id);

    let request = json!({ "jsonrpc": "2.0", "id": id, "method": "client/registerCapability", "params": {} });
    let response = json!({ "jsonrpc": "2.0", "id": id, "result": "complete" });
    server.write(request.clone()).await;
    server.write(response.clone()).await;

    assert_eq!(client.await_response(id, 1).await, response);
    assert_eq!(client.server_requests(), &[request]);
    assert_eq!(server.read_message(1).await, json!({ "jsonrpc": "2.0", "id": id, "result": null }));
}

#[tokio::test]
async fn notification_marker_collects_diagnostics_after_a_response() {
    let (client_stream, server_stream) = tokio::io::duplex(4096);
    let mut client = LspClient::new(client_stream);
    let mut server = LspClient::new(server_stream);
    let id = client.send_request("workspace/symbol", json!({ "query": "__barrier__" })).await;
    assert_eq!(server.read_message(1).await["id"], id);

    let response = json!({ "jsonrpc": "2.0", "id": id, "result": [] });
    let diagnostic = json!({ "jsonrpc": "2.0", "method": "textDocument/publishDiagnostics",
        "params": { "uri": "file:///workspace/a.php", "version": 2, "diagnostics": [] } });
    let marker = json!({ "jsonrpc": "2.0", "method": "window/logMessage",
        "params": { "type": 4, "message": "mago/testBarrier" } });
    server.write(response.clone()).await;
    server.write(diagnostic.clone()).await;
    server.write(marker.clone()).await;

    assert_eq!(client.await_response(id, 1).await, response);
    assert!(client.take_pending_notifications().is_empty());
    assert_eq!(client.await_notification("window/logMessage").await, marker);
    assert_eq!(client.take_pending_notifications(), vec![diagnostic]);
}
