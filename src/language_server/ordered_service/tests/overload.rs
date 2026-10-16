use super::*;

#[tokio::test(flavor = "current_thread")]
async fn busy_requests_leave_handlers_ids_and_shutdown_unchanged() {
    let (mut service, ready, release) = blocked_service(1);
    service.call(Request::build("initialize").id(1).finish()).await.unwrap();
    let active = service.call(Request::build("block").id(2).finish());
    timeout(DEADLINE, ready).await.unwrap().unwrap();
    let queued = service.call(Request::build("queued").id(3).finish());
    let created = service.inner.calls.created.load(Ordering::Relaxed);
    for (request, code) in [
        (Request::build("query").id(4).finish(), ErrorCode::ServerError(-32000)),
        (Request::build("query").id(3).finish(), ErrorCode::InvalidRequest),
        (Request::build("shutdown").id(3).finish(), ErrorCode::InvalidRequest),
        (Request::build("shutdown").params(json!({})).id(4).finish(), ErrorCode::InvalidParams),
        (Request::build("shutdown").id(4).finish(), ErrorCode::ServerError(-32000)),
    ] {
        let response = timeout(DEADLINE, service.call(request)).await.unwrap().unwrap().unwrap();
        assert_eq!(response.error().unwrap().code, code);
        assert_eq!(service.inner.calls.created.load(Ordering::Relaxed), created);
        assert!(matches!(*service.session.lock().unwrap(), SessionState::Running));
        assert!(!*service.stopped.borrow());
    }
    assert!(service.queue.find(&4.into()).is_none());
    service.call(Request::build("$/cancelRequest").params(json!({"id": 3})).finish()).await.unwrap();
    assert_eq!(queued.await.unwrap().unwrap().error().unwrap().code, ErrorCode::RequestCancelled);
    let retry = service.call(Request::build("retry").id(4).finish());
    release.send(()).unwrap();
    assert!(timeout(DEADLINE, retry).await.unwrap().unwrap().unwrap().is_ok());
    assert!(active.await.unwrap().unwrap().is_ok());
    assert!(service.call(Request::build("shutdown").id(5).finish()).await.unwrap().unwrap().is_ok());
    service.call(Request::build("exit").finish()).await.unwrap();
    assert!(service.session.lock().unwrap().result().is_ok());
}

#[tokio::test(flavor = "current_thread")]
async fn request_byte_overflow_allows_retry_without_registering_the_id() {
    let (mut service, ready, release) = blocked_service(8);
    service.call(Request::build("initialize").id(1).finish()).await.unwrap();
    let block = Request::build("block").id(2).finish();
    service.input = InputBudget::new(encoded_size(&block, INPUT_BYTES_LIMIT).unwrap());
    let active = service.call(block);
    timeout(DEADLINE, ready).await.unwrap().unwrap();
    for method in ["query", "shutdown"] {
        let response = service.call(Request::build(method).id(3).finish()).await.unwrap().unwrap();
        assert_eq!(response.error().unwrap().code, ErrorCode::ServerError(-32000));
        assert_eq!(service.inner.calls.created.load(Ordering::Relaxed), 2);
        assert!(service.queue.find(&3.into()).is_none());
        assert!(matches!(*service.session.lock().unwrap(), SessionState::Running));
        assert!(!*service.stopped.borrow());
    }
    release.send(()).unwrap();
    assert!(timeout(DEADLINE, active).await.unwrap().unwrap().unwrap().is_ok());
    assert!(service.call(Request::build("retry").id(3).finish()).await.unwrap().unwrap().is_ok());
}

#[tokio::test(flavor = "current_thread")]
async fn busy_admission_keeps_lifecycle_errors_and_initialize_retry() {
    for (state, code) in [
        (SessionState::Starting, ErrorCode::ServerError(-32002)),
        (SessionState::Initializing, ErrorCode::ServerError(-32002)),
        (SessionState::Shutdown, ErrorCode::InvalidRequest),
    ] {
        let inner = MockService { calls: Arc::default(), gate: None };
        let mut service = OrderedService::new(inner, Worker::with_capacity(0).unwrap());
        *service.session.lock().unwrap() = state;
        let response = service.call(Request::build("query").id(1).finish()).await.unwrap().unwrap();
        assert_eq!(response.error().unwrap().code, code);
        assert_eq!(service.inner.calls.created.load(Ordering::Relaxed), 0);
        assert!(!*service.stopped.borrow());
    }
    let inner = MockService { calls: Arc::default(), gate: None };
    let mut service = OrderedService::new(inner, Worker::new().unwrap());
    service.input = InputBudget::new(0);
    let initialize = || Request::build("initialize").id(1).finish();
    let response = service.call(initialize()).await.unwrap().unwrap();
    assert_eq!(response.error().unwrap().code, ErrorCode::ServerError(-32000));
    assert!(matches!(*service.session.lock().unwrap(), SessionState::Starting));
    assert_eq!(service.inner.calls.created.load(Ordering::Relaxed), 0);
    service.input = InputBudget::default();
    assert!(service.call(initialize()).await.unwrap().unwrap().is_ok());
    assert!(matches!(*service.session.lock().unwrap(), SessionState::Running));
}
