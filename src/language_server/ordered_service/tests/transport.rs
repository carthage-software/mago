use super::*;

async fn write_request(stream: &mut DuplexStream, request: Request) {
    let body = serde_json::to_vec(&request).unwrap();
    let header = format!("Content-Length: {}\r\n\r\n", body.len());
    stream.write_all(header.as_bytes()).await.unwrap();
    stream.write_all(&body).await.unwrap();
}

async fn read_response(stream: &mut DuplexStream) -> Response {
    serde_json::from_value(read_message(stream).await).unwrap()
}

async fn read_message(stream: &mut DuplexStream) -> serde_json::Value {
    let mut header = Vec::new();
    while !header.ends_with(b"\r\n\r\n") {
        header.push(stream.read_u8().await.unwrap());
    }
    let length = std::str::from_utf8(&header)
        .unwrap()
        .lines()
        .find_map(|line| line.strip_prefix("Content-Length: "))
        .unwrap()
        .parse::<usize>()
        .unwrap();
    let mut body = vec![0; length];
    stream.read_exact(&mut body).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

#[tokio::test(flavor = "current_thread")]
async fn shutdown_response_flushes_before_exit() {
    let (mut client, server) = tokio::io::duplex(4096);
    let (inner, socket) = LspService::new(|_| TestServer(Arc::new(Notify::new()), Arc::default()));
    let service = OrderedService::new(inner, Worker::new().unwrap());
    let mut stopped = service.stopped();
    let server = tokio::spawn(async move {
        let (reader, writer) = tokio::io::split(server);
        tokio::select! {
            _ = tower_lsp_server::Server::new(reader, writer, socket)
                .concurrency_level(REPLY_CONCURRENCY).serve(service) => {}
            _ = stopped.changed() => {}
        }
    });
    let initialize = Request::build("initialize").params(json!({"capabilities": {}})).id(1).finish();
    write_request(&mut client, initialize).await;
    assert!(timeout(DEADLINE, read_response(&mut client)).await.unwrap().is_ok());
    write_request(&mut client, Request::build("shutdown").id(2).finish()).await;
    let response = timeout(DEADLINE, read_response(&mut client)).await.unwrap();
    assert_eq!(response.id(), &2.into());
    assert_eq!(response.result(), Some(&serde_json::Value::Null));
    write_request(&mut client, Request::build("exit").finish()).await;
    timeout(DEADLINE, server).await.unwrap().unwrap();
}

struct ShutdownPublisher {
    client: tower_lsp_server::Client,
    started: Arc<Notify>,
    release: Arc<Notify>,
    calls: Arc<AtomicUsize>,
}

impl LanguageServer for ShutdownPublisher {
    async fn initialize(&self, _: InitializeParams) -> JsonRpcResult<InitializeResult> {
        Ok(InitializeResult::default())
    }

    async fn shutdown(&self) -> JsonRpcResult<()> {
        Ok(())
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.calls.fetch_add(1, Ordering::Relaxed);
        self.started.notify_one();
        self.release.notified().await;
        for message in ["first", "second"] {
            let diagnostic =
                Diagnostic::new(Range::default(), None, None, None, None, message.into(), None, None, None);
            self.client
                .publish_diagnostics(
                    params.text_document.text_document_identifier.uri.clone(),
                    vec![diagnostic],
                    Some(params.text_document.version),
                )
                .await;
        }
    }
}

#[tokio::test(flavor = "current_thread")]
async fn shutdown_preserves_admitted_publications_and_rejects_later_changes() {
    let (mut client, server) = tokio::io::duplex(256);
    let started = Arc::new(Notify::new());
    let release = Arc::new(Notify::new());
    let calls = Arc::new(AtomicUsize::new(0));
    let (inner, socket) = LspService::new(|client| ShutdownPublisher {
        client,
        started: Arc::clone(&started),
        release: Arc::clone(&release),
        calls: Arc::clone(&calls),
    });
    let service = OrderedService::new(inner, Worker::new().unwrap());
    let session = service.session();
    let mut stopped = service.stopped();
    let server = tokio::spawn(async move {
        let (reader, writer) = tokio::io::split(server);
        tokio::select! {
            _ = tower_lsp_server::Server::new(reader, writer, socket)
                .concurrency_level(REPLY_CONCURRENCY).serve(service) => {}
            _ = stopped.changed() => {}
        }
    });
    write_request(&mut client, Request::build("initialize").params(json!({"capabilities": {}})).id(1).finish()).await;
    assert!(timeout(DEADLINE, read_response(&mut client)).await.unwrap().is_ok());
    write_request(&mut client, change("file:///test.php", 1, "before")).await;
    timeout(DEADLINE, started.notified()).await.unwrap();
    write_request(&mut client, Request::build("shutdown").id(2).finish()).await;
    timeout(DEADLINE, async {
        while !matches!(*session.lock().unwrap(), SessionState::Shutdown) {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    release.notify_one();

    let mut publications = Vec::new();
    let mut shutdown = false;
    // Notifications and responses use separate transport channels; accept either wire order.
    for _ in 0..3 {
        let message = timeout(DEADLINE, read_message(&mut client)).await.unwrap();
        if message.get("method").is_some() {
            assert_eq!(message["method"], "textDocument/publishDiagnostics");
            assert_eq!(message["params"]["version"], 1);
            publications.push(message["params"]["diagnostics"][0]["message"].clone());
        } else {
            assert_eq!(message, json!({"jsonrpc": "2.0", "id": 2, "result": null}));
            assert!(!shutdown);
            shutdown = true;
        }
    }
    assert!(shutdown);
    assert_eq!(publications, vec![json!("first"), json!("second")]);
    write_request(&mut client, change("file:///test.php", 2, "after")).await;
    write_request(
        &mut client,
        Request::build("textDocument/hover")
            .params(json!({
                "textDocument": {"uri": "file:///test.php"}, "position": {"line": 0, "character": 0}
            }))
            .id(3)
            .finish(),
    )
    .await;
    let response = timeout(DEADLINE, read_response(&mut client)).await.unwrap();
    assert_eq!(response.id(), &3.into());
    assert_eq!(response.error().unwrap().code, ErrorCode::InvalidRequest);
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    write_request(&mut client, Request::build("exit").finish()).await;
    timeout(DEADLINE, server).await.unwrap().unwrap();
    assert_eq!(timeout(DEADLINE, client.read_u8()).await.unwrap().unwrap_err().kind(), io::ErrorKind::UnexpectedEof);
}

async fn cancel_queued_through_transport(queued: usize) {
    let (mut client, server) = tokio::io::duplex(65536);
    let started = Arc::new(Notify::new());
    let calls = Arc::new(AtomicUsize::new(0));
    let backend = TestServer(Arc::clone(&started), Arc::clone(&calls));
    let (inner, socket) = LspService::new(move |_| backend);
    let service = OrderedService::new(inner, Worker::new().unwrap());
    let mut stopped = service.stopped();
    let server = tokio::spawn(async move {
        let (reader, writer) = tokio::io::split(server);
        tokio::select! {
            _ = tower_lsp_server::Server::new(reader, writer, socket)
                .concurrency_level(REPLY_CONCURRENCY).serve(service) => {}
            _ = stopped.changed() => {}
        }
    });
    write_request(&mut client, Request::build("initialize").params(json!({"capabilities": {}})).id(1).finish()).await;
    assert!(timeout(DEADLINE, read_response(&mut client)).await.unwrap().is_ok());
    let hover = |id| {
        Request::build("textDocument/hover")
            .params(json!({"textDocument": {"uri": "file:///test.php"}, "position": {"line": 0, "character": 0}}))
            .id(id)
            .finish()
    };
    let cancel = |id| Request::build("$/cancelRequest").params(json!({"id": id})).finish();
    write_request(&mut client, hover(2)).await;
    timeout(DEADLINE, started.notified()).await.unwrap();
    let last = queued as i64 + 2;
    for id in 3..=last {
        write_request(&mut client, hover(id)).await;
    }
    write_request(&mut client, cancel(last)).await;
    let response = timeout(DEADLINE, read_response(&mut client)).await.unwrap();
    assert_eq!(response.id(), &last.into());
    assert_eq!(response.error().unwrap().code, ErrorCode::RequestCancelled);
    assert_eq!(calls.load(Ordering::Relaxed), 1);

    for _ in 0..3 {
        // Cancellation must free a full queue slot before the client can reuse its ID.
        write_request(&mut client, hover(last)).await;
        write_request(&mut client, cancel(last)).await;
        let response = timeout(DEADLINE, read_response(&mut client)).await.unwrap();
        assert_eq!(response.id(), &last.into());
        assert_eq!(response.error().unwrap().code, ErrorCode::RequestCancelled);
    }
    // Repeated, unknown, and malformed cancellations must not send extra replies.
    write_request(&mut client, cancel(last)).await;
    write_request(&mut client, cancel(last + 100)).await;
    write_request(&mut client, Request::build("$/cancelRequest").params(json!({"id": false})).finish()).await;
    // Cancel queued work first, keeping the first handler blocked until every slot is cleared.
    for id in (2..last).rev() {
        write_request(&mut client, cancel(id)).await;
        let response = timeout(DEADLINE, read_response(&mut client)).await.unwrap();
        assert_eq!(response.id(), &id.into());
        assert_eq!(response.error().unwrap().code, ErrorCode::RequestCancelled);
    }
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    write_request(&mut client, Request::build("shutdown").id(last + 1).finish()).await;
    let response = timeout(DEADLINE, read_response(&mut client)).await.unwrap();
    assert_eq!(response.id(), &(last + 1).into());
    assert!(response.is_ok());
    write_request(&mut client, Request::build("exit").finish()).await;
    timeout(DEADLINE, server).await.unwrap().unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn queued_cancellation_passes_four_waiting_replies() {
    cancel_queued_through_transport(4).await;
}

#[tokio::test(flavor = "current_thread")]
async fn cancelled_requests_release_a_full_worker_queue_before_replying() {
    cancel_queued_through_transport(QUEUE_CAPACITY).await;
}

struct BurstServer {
    started: Arc<Notify>,
    release: Arc<Notify>,
    events: Arc<Mutex<Vec<String>>>,
    text: Mutex<String>,
}

impl LanguageServer for BurstServer {
    async fn initialize(&self, _: InitializeParams) -> JsonRpcResult<InitializeResult> {
        Ok(InitializeResult::default())
    }

    async fn shutdown(&self) -> JsonRpcResult<()> {
        Ok(())
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let document = params.text_document;
        let text = match params.content_changes.into_iter().last().unwrap() {
            TextDocumentContentChangeEvent::TextDocumentContentChangePartial(change) => change.text,
            TextDocumentContentChangeEvent::TextDocumentContentChangeWholeDocument(change) => change.text,
        };
        if document.text_document_identifier.uri.as_str() == "file:///test.php" {
            if document.version == 1 {
                self.started.notify_one();
                self.release.notified().await;
            }
            *self.text.lock().unwrap() = text.clone();
        }
        self.events.lock().unwrap().push(format!(
            "change:{}:{}:{text}",
            document.text_document_identifier.uri.as_str(),
            document.version
        ));
    }

    async fn did_save(&self, _: DidSaveTextDocumentParams) {
        self.events.lock().unwrap().push(format!("save:{}", self.text.lock().unwrap()));
    }

    async fn hover(&self, _: HoverParams) -> JsonRpcResult<Option<Hover>> {
        let text = self.text.lock().unwrap().clone();
        self.events.lock().unwrap().push(format!("hover:{text}"));
        Ok(Some(Hover { contents: Contents::MarkedString(text.into()), range: None }))
    }
}

#[tokio::test(flavor = "current_thread")]
async fn typing_bursts_keep_latest_text_and_do_not_cross_other_messages() {
    let (mut client, server) = tokio::io::duplex(65536);
    let started = Arc::new(Notify::new());
    let release = Arc::new(Notify::new());
    let events = Arc::new(Mutex::new(Vec::new()));
    let backend = BurstServer {
        started: Arc::clone(&started),
        release: Arc::clone(&release),
        events: Arc::clone(&events),
        text: Mutex::default(),
    };
    let (inner, socket) = LspService::new(move |_| backend);
    let service = OrderedService::new(inner, Worker::new().unwrap());
    let retained = service.input.counter();
    let mut stopped = service.stopped();
    let server = tokio::spawn(async move {
        let (reader, writer) = tokio::io::split(server);
        tokio::select! {
            _ = tower_lsp_server::Server::new(reader, writer, socket)
                .concurrency_level(REPLY_CONCURRENCY).serve(service) => {}
            _ = stopped.changed() => {}
        }
    });
    write_request(&mut client, Request::build("initialize").params(json!({"capabilities": {}})).id(1).finish()).await;
    assert!(timeout(DEADLINE, read_response(&mut client)).await.unwrap().is_ok());
    write_request(&mut client, change("file:///test.php", 1, "1")).await;
    timeout(DEADLINE, started.notified()).await.unwrap();
    for version in 2..=500 {
        write_request(&mut client, change("file:///test.php", version, &version.to_string())).await;
    }
    let hover = |id| {
        Request::build("textDocument/hover")
            .params(json!({
                "textDocument": {"uri": "file:///test.php"}, "position": {"line": 0, "character": 0}
            }))
            .id(id)
            .finish()
    };
    write_request(&mut client, hover(2)).await;
    for version in 501..=1000 {
        write_request(&mut client, change("file:///test.php", version, &version.to_string())).await;
    }
    write_request(
        &mut client,
        Request::build("textDocument/didSave").params(json!({"textDocument": {"uri": "file:///test.php"}})).finish(),
    )
    .await;
    for version in 1001..=1100 {
        write_request(&mut client, change("file:///test.php", version, &version.to_string())).await;
    }
    write_request(&mut client, change("file:///other.php", 1, "other")).await;
    write_request(&mut client, change("file:///test.php", 1101, "1101")).await;
    // A cancelled probe proves the input loop reached the end while the first change still waits.
    write_request(&mut client, hover(3)).await;
    write_request(&mut client, Request::build("$/cancelRequest").params(json!({"id": 3})).finish()).await;
    let response = timeout(DEADLINE, read_response(&mut client)).await.unwrap();
    assert_eq!(response.id(), &3.into());
    assert_eq!(response.error().unwrap().code, ErrorCode::RequestCancelled);
    assert!(retained.load(Ordering::Relaxed) < 8192);
    assert!(events.lock().unwrap().is_empty());
    release.notify_one();
    let response = timeout(DEADLINE, read_response(&mut client)).await.unwrap();
    assert_eq!(response.id(), &2.into());
    assert_eq!(response.result().unwrap()["contents"], "500");
    write_request(&mut client, Request::build("shutdown").id(4).finish()).await;
    assert!(timeout(DEADLINE, read_response(&mut client)).await.unwrap().is_ok());
    assert_eq!(
        *events.lock().unwrap(),
        [
            "change:file:///test.php:1:1",
            "change:file:///test.php:500:500",
            "hover:500",
            "change:file:///test.php:1000:1000",
            "save:1000",
            "change:file:///test.php:1101:1101",
            "change:file:///other.php:1:other",
        ]
    );
    assert_eq!(retained.load(Ordering::Relaxed), 0);
    write_request(&mut client, Request::build("exit").finish()).await;
    timeout(DEADLINE, server).await.unwrap().unwrap();
}

struct ClientReplyServer {
    client: tower_lsp_server::Client,
    calls: Arc<AtomicUsize>,
}

impl LanguageServer for ClientReplyServer {
    async fn initialize(&self, _: InitializeParams) -> JsonRpcResult<InitializeResult> {
        Ok(InitializeResult::default())
    }

    async fn shutdown(&self) -> JsonRpcResult<()> {
        Ok(())
    }

    async fn hover(&self, _: HoverParams) -> JsonRpcResult<Option<Hover>> {
        if self.calls.fetch_add(1, Ordering::Relaxed) == 0 {
            self.client.configuration(Vec::new()).await?;
            Ok(None)
        } else {
            std::future::pending().await
        }
    }
}

#[tokio::test(flavor = "current_thread")]
async fn request_overflow_keeps_cancellation_client_replies_and_reused_ids_alive() {
    let (mut client, server) = tokio::io::duplex(65536);
    let calls = Arc::new(AtomicUsize::new(0));
    let server_calls = Arc::clone(&calls);
    let (inner, socket) = LspService::new(move |client| ClientReplyServer { client, calls: server_calls });
    let service = OrderedService::new(inner, Worker::new().unwrap());
    let mut stopped = service.stopped();
    let server = tokio::spawn(async move {
        let (reader, writer) = tokio::io::split(server);
        tokio::select! {
            _ = tower_lsp_server::Server::new(reader, writer, socket)
                .concurrency_level(REPLY_CONCURRENCY).serve(service) => {}
            _ = stopped.changed() => {}
        }
    });
    write_request(&mut client, Request::build("initialize").params(json!({"capabilities": {}})).id(1).finish()).await;
    assert!(timeout(DEADLINE, read_response(&mut client)).await.unwrap().is_ok());
    let hover = |id| {
        Request::build("textDocument/hover")
            .params(json!({"textDocument": {"uri": "file:///test.php"}, "position": {"line": 0, "character": 0}}))
            .id(id)
            .finish()
    };
    let cancel = |id| Request::build("$/cancelRequest").params(json!({"id": id})).finish();
    write_request(&mut client, hover(2)).await;
    let configuration = timeout(DEADLINE, read_message(&mut client)).await.unwrap();
    assert_eq!(configuration["method"], "workspace/configuration");
    let last = QUEUE_CAPACITY as i64 + 2;
    for id in 3..=last {
        write_request(&mut client, hover(id)).await;
    }
    for id in 1000..1003 {
        write_request(&mut client, hover(id)).await;
        let response = timeout(DEADLINE, read_response(&mut client)).await.unwrap();
        assert_eq!(response.id(), &id.into());
        assert_eq!(response.error().unwrap().code, ErrorCode::ServerError(-32000));
    }
    write_request(&mut client, hover(3)).await;
    assert_eq!(
        timeout(DEADLINE, read_response(&mut client)).await.unwrap().error().unwrap().code,
        ErrorCode::InvalidRequest
    );
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    write_request(&mut client, cancel(3)).await;
    let response = timeout(DEADLINE, read_response(&mut client)).await.unwrap();
    assert_eq!(response.id(), &3.into());
    assert_eq!(response.error().unwrap().code, ErrorCode::RequestCancelled);
    write_request(&mut client, hover(1000)).await;
    write_request(&mut client, cancel(1000)).await;
    let response = timeout(DEADLINE, read_response(&mut client)).await.unwrap();
    assert_eq!(response.id(), &1000.into());
    assert_eq!(response.error().unwrap().code, ErrorCode::RequestCancelled);
    // Fill the freed slot before replying to the active handler's client request.
    write_request(&mut client, hover(1001)).await;
    let body = serde_json::to_vec(&json!({"jsonrpc": "2.0", "id": configuration["id"], "result": []})).unwrap();
    client.write_all(format!("Content-Length: {}\r\n\r\n", body.len()).as_bytes()).await.unwrap();
    client.write_all(&body).await.unwrap();
    let response = timeout(DEADLINE, read_response(&mut client)).await.unwrap();
    assert_eq!(response.id(), &2.into());
    assert!(response.is_ok());
    for id in std::iter::once(1001).chain((4..=last).rev()) {
        write_request(&mut client, cancel(id)).await;
        let response = timeout(DEADLINE, read_response(&mut client)).await.unwrap();
        assert_eq!(response.id(), &id.into());
        assert_eq!(response.error().unwrap().code, ErrorCode::RequestCancelled);
    }
    write_request(&mut client, Request::build("shutdown").id(1004).finish()).await;
    assert!(timeout(DEADLINE, read_response(&mut client)).await.unwrap().is_ok());
    write_request(&mut client, Request::build("exit").finish()).await;
    timeout(DEADLINE, server).await.unwrap().unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn client_replies_reach_a_waiting_handler_behind_notification_bursts() {
    let (mut client, server) = tokio::io::duplex(65536);
    let calls = Arc::new(AtomicUsize::new(0));
    let server_calls = Arc::clone(&calls);
    let (inner, socket) = LspService::new(move |client| ClientReplyServer { client, calls: server_calls });
    let service = OrderedService::new(inner, Worker::new().unwrap());
    let session = service.session();
    let mut stopped = service.stopped();
    let server = tokio::spawn(async move {
        let (reader, writer) = tokio::io::split(server);
        tokio::select! {
            _ = tower_lsp_server::Server::new(reader, writer, socket)
                .concurrency_level(REPLY_CONCURRENCY).serve(service) => {}
            _ = stopped.changed() => {}
        }
    });
    write_request(&mut client, Request::build("initialize").params(json!({"capabilities": {}})).id(1).finish()).await;
    assert!(timeout(DEADLINE, read_response(&mut client)).await.unwrap().is_ok());
    let hover = Request::build("textDocument/hover")
        .params(json!({"textDocument": {"uri": "file:///test.php"}, "position": {"line": 0, "character": 0}}))
        .id(2)
        .finish();
    write_request(&mut client, hover).await;
    let configuration = timeout(DEADLINE, read_message(&mut client)).await.unwrap();
    assert_eq!(configuration["method"], "workspace/configuration");
    timeout(DEADLINE, async {
        for index in 0..QUEUE_CAPACITY * 4 {
            write_request(&mut client, change(&format!("file:///{index}.php"), 1, "<?php")).await;
        }
    })
    .await
    .unwrap();
    let body = serde_json::to_vec(&json!({"jsonrpc": "2.0", "id": configuration["id"], "result": []})).unwrap();
    client.write_all(format!("Content-Length: {}\r\n\r\n", body.len()).as_bytes()).await.unwrap();
    client.write_all(&body).await.unwrap();
    let response = timeout(DEADLINE, read_response(&mut client)).await.unwrap();
    assert_eq!(response.id(), &2.into());
    assert!(response.is_ok());
    assert!(matches!(*session.lock().unwrap(), SessionState::Running));
    write_request(&mut client, Request::build("shutdown").id(3).finish()).await;
    assert!(timeout(DEADLINE, read_response(&mut client)).await.unwrap().is_ok());
    write_request(&mut client, Request::build("exit").finish()).await;
    timeout(DEADLINE, server).await.unwrap().unwrap();
}
