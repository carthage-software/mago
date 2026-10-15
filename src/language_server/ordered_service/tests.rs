use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::time::Duration;

use serde_json::json;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::DuplexStream;
use tokio::sync::Notify;
use tokio::sync::oneshot;
use tokio::time::timeout;
use tower_lsp_server::LanguageServer;
use tower_lsp_server::LspService;
use tower_lsp_server::gen_lsp_types::Contents;
use tower_lsp_server::gen_lsp_types::Diagnostic;
use tower_lsp_server::gen_lsp_types::DidChangeTextDocumentParams;
use tower_lsp_server::gen_lsp_types::DidSaveTextDocumentParams;
use tower_lsp_server::gen_lsp_types::Hover;
use tower_lsp_server::gen_lsp_types::HoverParams;
use tower_lsp_server::gen_lsp_types::InitializeParams;
use tower_lsp_server::gen_lsp_types::InitializeResult;
use tower_lsp_server::gen_lsp_types::Range;
use tower_lsp_server::gen_lsp_types::TextDocumentContentChangeEvent;
use tower_lsp_server::jsonrpc::Error;
use tower_lsp_server::jsonrpc::ErrorCode;
use tower_lsp_server::jsonrpc::Result as JsonRpcResult;

use super::*;

const DEADLINE: Duration = Duration::from_secs(5);

#[derive(Default)]
struct Calls {
    created: AtomicUsize,
    methods: Mutex<Vec<String>>,
    control: Notify,
}

struct MockService {
    calls: Arc<Calls>,
    gate: Option<(oneshot::Sender<()>, mpsc::Receiver<()>)>,
}

impl Service<Request> for MockService {
    type Response = Option<Response>;
    type Error = io::Error;
    type Future = Handler;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: Request) -> Handler {
        self.calls.created.fetch_add(1, Ordering::Relaxed);
        let calls = Arc::clone(&self.calls);
        let gate = if request.method() == "block" { self.gate.take() } else { None };
        Box::pin(async move {
            assert_ne!(request.method(), "panic", "test worker panic");
            if request.method() == "error" {
                return Err(io::Error::other("test worker error"));
            }
            if let Some((started, release)) = gate {
                let _ = started.send(());
                release.recv_timeout(DEADLINE).unwrap();
            }
            calls.methods.lock().unwrap().push(request.method().into());
            if request.method() == "$/cancelRequest" {
                calls.control.notify_one();
            }
            Ok(request.id().cloned().map(|id| Response::from_ok(id, json!(request.method()))))
        })
    }
}

fn blocked_service(capacity: usize) -> (OrderedService<MockService>, oneshot::Receiver<()>, mpsc::Sender<()>) {
    let (started, ready) = oneshot::channel();
    let (release, wait) = mpsc::channel();
    let inner = MockService { calls: Arc::new(Calls::default()), gate: Some((started, wait)) };
    (OrderedService::with_capacity(inner, capacity).unwrap(), ready, release)
}

struct TestServer(Arc<Notify>, Arc<AtomicUsize>);

impl LanguageServer for TestServer {
    async fn initialize(&self, _: InitializeParams) -> JsonRpcResult<InitializeResult> {
        Ok(InitializeResult::default())
    }

    async fn shutdown(&self) -> JsonRpcResult<()> {
        Ok(())
    }

    async fn hover(&self, _: HoverParams) -> JsonRpcResult<Option<Hover>> {
        self.1.fetch_add(1, Ordering::Relaxed);
        self.0.notify_one();
        std::future::pending().await
    }
}

fn change(uri: &str, version: i32, text: &str) -> Request {
    Request::build("textDocument/didChange")
        .params(json!({"textDocument": {"uri": uri, "version": version}, "contentChanges": [{"text": text}]}))
        .finish()
}

mod initialization;
mod input;
mod ordering;
mod overload;
mod shutdown;
mod transport;
