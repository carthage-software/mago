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
}

impl LspClient {
    pub fn new(stream: DuplexStream) -> Self {
        Self {
            stream,
            next_id: Arc::new(AtomicI64::new(1)),
            rx: Vec::with_capacity(8192),
            pending_notifications: Vec::new(),
        }
    }

    /// Drain notifications received since the last call.
    #[allow(dead_code)] // used by capabilities tests, not lifecycle tests
    pub fn take_pending_notifications(&mut self) -> Vec<Value> {
        std::mem::take(&mut self.pending_notifications)
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
            if msg.get("id").and_then(|v| v.as_i64()) == Some(expected_id) {
                return msg;
            }
            if msg.get("method").is_some() && msg.get("id").is_none() {
                // Notification; keep it for later inspection.
                self.pending_notifications.push(msg);
            }
            // Other server-to-client requests (e.g. workspace/applyEdit) are
            // ignored for now; tests don't simulate the editor's reply path.
        }
    }
}
