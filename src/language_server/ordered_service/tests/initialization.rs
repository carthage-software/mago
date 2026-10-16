use super::*;

struct InitializingServer {
    calls: Arc<AtomicUsize>,
    started: Arc<Notify>,
    release: Arc<Notify>,
    fail_first: bool,
}

impl LanguageServer for InitializingServer {
    async fn initialize(&self, _: InitializeParams) -> JsonRpcResult<InitializeResult> {
        if self.calls.fetch_add(1, Ordering::Relaxed) == 0 {
            self.started.notify_one();
            self.release.notified().await;
            if self.fail_first {
                return Err(Error::internal_error());
            }
        }
        Ok(InitializeResult::default())
    }

    async fn shutdown(&self) -> JsonRpcResult<()> {
        Ok(())
    }
}

#[tokio::test(flavor = "current_thread")]
async fn a_pending_initialize_rejects_another_initialize() {
    let calls = Arc::new(AtomicUsize::new(0));
    let started = Arc::new(Notify::new());
    let release = Arc::new(Notify::new());
    let backend = InitializingServer {
        calls: Arc::clone(&calls),
        started: Arc::clone(&started),
        release: Arc::clone(&release),
        fail_first: false,
    };
    let (inner, _socket) = LspService::new(move |_| backend);
    let mut service = OrderedService::new(inner, Worker::new().unwrap());
    let initialize = |id| Request::build("initialize").params(json!({"capabilities": {}})).id(id).finish();
    assert!(
        service
            .call(Request::build("initialize").params(json!({"capabilities": {}})).finish())
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(calls.load(Ordering::Relaxed), 0);
    let first = service.call(initialize(1));
    timeout(DEADLINE, started.notified()).await.unwrap();
    let second = service.call(initialize(2));
    release.notify_one();
    assert!(timeout(DEADLINE, first).await.unwrap().unwrap().unwrap().is_ok());
    let response = timeout(DEADLINE, second).await.unwrap().unwrap().unwrap();
    assert_eq!(response.error().map(|error| error.code), Some(ErrorCode::InvalidRequest));
    assert_eq!(calls.load(Ordering::Relaxed), 1);
}

#[tokio::test(flavor = "current_thread")]
async fn initialize_can_retry_after_an_error() {
    let calls = Arc::new(AtomicUsize::new(0));
    let release = Arc::new(Notify::new());
    release.notify_one();
    let backend = InitializingServer { calls: Arc::clone(&calls), started: Arc::default(), release, fail_first: true };
    let (inner, _socket) = LspService::new(move |_| backend);
    let mut service = OrderedService::new(inner, Worker::new().unwrap());
    let initialize = |id| Request::build("initialize").params(json!({"capabilities": {}})).id(id).finish();
    let response = timeout(DEADLINE, service.call(initialize(1))).await.unwrap().unwrap().unwrap();
    assert_eq!(response.error().unwrap().code, ErrorCode::InternalError);
    assert!(matches!(*service.session.lock().unwrap(), SessionState::Starting));
    assert!(timeout(DEADLINE, service.call(initialize(2))).await.unwrap().unwrap().unwrap().is_ok());
    assert_eq!(calls.load(Ordering::Relaxed), 2);
    assert!(matches!(*service.session.lock().unwrap(), SessionState::Running));
}

#[tokio::test(flavor = "current_thread")]
async fn dropping_a_queued_initialize_reply_still_initializes_and_frees_its_id() {
    let calls = Arc::new(AtomicUsize::new(0));
    let release = Arc::new(Notify::new());
    release.notify_one();
    let backend = InitializingServer { calls: Arc::clone(&calls), started: Arc::default(), release, fail_first: false };
    let (inner, _socket) = LspService::new(move |_| backend);
    let mut service = OrderedService::new(inner, Worker::new().unwrap());
    let (release, wait) = oneshot::channel();
    let (_, blocker, _) = service
        .queue
        .enqueue(
            None,
            Box::pin(async move {
                wait.await.unwrap();
                Ok(None)
            }),
            service.stopped(),
        )
        .unwrap();
    let initialize = |id| Request::build("initialize").params(json!({"capabilities": {}})).id(id).finish();
    drop(service.call(initialize(1)));
    release.send(()).unwrap();
    timeout(DEADLINE, blocker).await.unwrap().unwrap();
    // This reply proves the worker has passed the dropped initialize reply.
    timeout(DEADLINE, service.call(Request::build("before-retry").id(2).finish())).await.unwrap().unwrap();
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    assert!(matches!(*service.session.lock().unwrap(), SessionState::Running));
    let reuse = Request::build("textDocument/hover")
        .params(json!({"textDocument": {"uri": "file:///test.php"}, "position": {"line": 0, "character": 0}}))
        .id(1)
        .finish();
    let response = timeout(DEADLINE, service.call(reuse)).await.unwrap().unwrap().unwrap();
    assert_eq!(response.error().unwrap().code, ErrorCode::MethodNotFound);
}

#[tokio::test(flavor = "current_thread")]
async fn dropping_a_queued_request_reply_frees_its_id() {
    let release = Arc::new(Notify::new());
    release.notify_one();
    let backend = InitializingServer { calls: Arc::default(), started: Arc::default(), release, fail_first: false };
    let (inner, _socket) = LspService::new(move |_| backend);
    let mut service = OrderedService::new(inner, Worker::new().unwrap());
    service.call(Request::build("initialize").params(json!({"capabilities": {}})).id(1).finish()).await.unwrap();
    let (release, wait) = oneshot::channel();
    let (_, blocker, _) = service
        .queue
        .enqueue(
            None,
            Box::pin(async move {
                wait.await.unwrap();
                Ok(None)
            }),
            service.stopped(),
        )
        .unwrap();
    let hover = || {
        Request::build("textDocument/hover")
            .params(json!({"textDocument": {"uri": "file:///test.php"}, "position": {"line": 0, "character": 0}}))
            .id(2)
            .finish()
    };
    drop(service.call(hover()));
    release.send(()).unwrap();
    timeout(DEADLINE, blocker).await.unwrap().unwrap();
    timeout(DEADLINE, service.call(Request::build("before-retry").id(3).finish())).await.unwrap().unwrap();
    let response = timeout(DEADLINE, service.call(hover())).await.unwrap().unwrap().unwrap();
    assert_eq!(response.error().unwrap().code, ErrorCode::MethodNotFound);
}

#[tokio::test(flavor = "current_thread")]
async fn cancelled_initialization_allows_retry() {
    for queued in [false, true] {
        let calls = Arc::new(AtomicUsize::new(0));
        let started = Arc::new(Notify::new());
        let release = Arc::new(Notify::new());
        let backend = InitializingServer {
            calls: Arc::clone(&calls),
            started: Arc::clone(&started),
            release: Arc::clone(&release),
            fail_first: false,
        };
        let (inner, _socket) = LspService::new(move |_| backend);
        let mut service = OrderedService::new(inner, Worker::new().unwrap());
        let (unblock, wait) = oneshot::channel();
        let blocker = queued.then(|| {
            service
                .queue
                .enqueue(
                    None,
                    Box::pin(async move {
                        wait.await.unwrap();
                        Ok(None)
                    }),
                    service.stopped(),
                )
                .unwrap()
                .1
        });
        let initialize = |id| Request::build("initialize").params(json!({"capabilities": {}})).id(id).finish();
        let reply = service.call(initialize(1));
        if !queued {
            timeout(DEADLINE, started.notified()).await.unwrap();
        }
        service.call(Request::build("$/cancelRequest").params(json!({"id": 1})).finish()).await.unwrap();
        let response = timeout(DEADLINE, reply).await.unwrap().unwrap().unwrap();
        assert_eq!(response.error().unwrap().code, ErrorCode::RequestCancelled);
        assert!(matches!(*service.session.lock().unwrap(), SessionState::Starting));
        if let Some(blocker) = blocker {
            unblock.send(()).unwrap();
            timeout(DEADLINE, blocker).await.unwrap().unwrap();
            assert_eq!(calls.load(Ordering::Relaxed), 0);
        }
        release.notify_one();
        assert!(timeout(DEADLINE, service.call(initialize(1))).await.unwrap().unwrap().unwrap().is_ok());
        assert_eq!(calls.load(Ordering::Relaxed), if queued { 1 } else { 2 });
        assert!(matches!(*service.session.lock().unwrap(), SessionState::Running));
    }
}
