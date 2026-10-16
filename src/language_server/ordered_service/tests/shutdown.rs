use super::*;

#[tokio::test(flavor = "current_thread")]
async fn tower_cancellation_and_shutdown_still_work() {
    let started = Arc::new(Notify::new());
    let signal = Arc::clone(&started);
    let (inner, _socket) = LspService::new(move |_| TestServer(signal, Arc::default()));
    let mut service = OrderedService::new(inner, Worker::new().unwrap());
    let initialize = Request::build("initialize").params(json!({"capabilities": {}})).id(1).finish();
    assert!(service.call(initialize).await.unwrap().unwrap().is_ok());
    let request = Request::build("textDocument/hover")
        .params(json!({"textDocument": {"uri": "file:///test.php"}, "position": {"line": 0, "character": 0}}))
        .id(2)
        .finish();
    let hover = service.call(request);
    timeout(DEADLINE, started.notified()).await.unwrap();
    service.call(Request::build("$/cancelRequest").params(json!({"id": 2})).finish()).await.unwrap();
    let response = timeout(DEADLINE, hover).await.unwrap().unwrap().unwrap();
    assert_eq!(response.error().unwrap().code, ErrorCode::RequestCancelled);
    let response = service.call(Request::build("shutdown").id(3).finish()).await.unwrap().unwrap();
    assert!(response.is_ok());
    assert!(!*service.stopped.borrow());
    service.call(Request::build("exit").finish()).await.unwrap();
    assert!(*service.stopped.borrow());
    assert!(service.session.lock().unwrap().result().is_ok());
}

#[tokio::test(flavor = "current_thread")]
async fn exit_without_an_accepted_shutdown_fails() {
    for initialize in [false, true] {
        let (inner, _socket) = LspService::new(|_| TestServer(Arc::default(), Arc::default()));
        let mut service = OrderedService::new(inner, Worker::new().unwrap());
        if initialize {
            let request = Request::build("initialize").params(json!({"capabilities": {}})).id(1).finish();
            assert!(service.call(request).await.unwrap().unwrap().is_ok());
        } else {
            assert!(service.call(Request::build("shutdown").id(2).finish()).await.unwrap().unwrap().is_error());
        }
        service.call(Request::build("exit").finish()).await.unwrap();
        assert!(service.session.lock().unwrap().result().unwrap_err().to_string().contains("before shutdown"));
    }
}

#[tokio::test(flavor = "current_thread")]
async fn bad_shutdown_requests_do_not_change_the_session() {
    let started = Arc::new(Notify::new());
    let signal = Arc::clone(&started);
    let (inner, _socket) = LspService::new(move |_| TestServer(signal, Arc::default()));
    let mut service = OrderedService::new(inner, Worker::new().unwrap());
    let initialize = Request::build("initialize").params(json!({"capabilities": {}})).id(1).finish();
    assert!(service.call(initialize).await.unwrap().unwrap().is_ok());
    assert!(service.call(Request::build("shutdown").finish()).await.unwrap().is_none());
    for params in [json!({}), json!([]), json!(false), json!(1), json!("invalid")] {
        let response = service.call(Request::build("shutdown").params(params).id(3).finish()).await.unwrap().unwrap();
        assert_eq!(response.error().unwrap().code, ErrorCode::InvalidParams);
    }
    let hover = service.call(
        Request::build("textDocument/hover")
            .params(json!({"textDocument": {"uri": "file:///test.php"}, "position": {"line": 0, "character": 0}}))
            .id(2)
            .finish(),
    );
    timeout(DEADLINE, started.notified()).await.unwrap();
    let response = service.call(Request::build("shutdown").id(2).finish()).await.unwrap().unwrap();
    assert_eq!(response.error().unwrap().code, ErrorCode::InvalidRequest);
    assert!(matches!(*service.session.lock().unwrap(), SessionState::Running));
    service.call(Request::build("$/cancelRequest").params(json!({"id": 2})).finish()).await.unwrap();
    assert_eq!(hover.await.unwrap().unwrap().error().unwrap().code, ErrorCode::RequestCancelled);
    assert!(
        service.call(Request::build("shutdown").params(json!(null)).id(4).finish()).await.unwrap().unwrap().is_ok()
    );
    service.call(Request::build("exit").finish()).await.unwrap();
    assert!(service.session.lock().unwrap().result().is_ok());
}

#[tokio::test(flavor = "current_thread")]
async fn exit_after_a_queued_shutdown_succeeds() {
    let (mut service, ready, release) = blocked_service(8);
    service.call(Request::build("initialize").id(1).finish()).await.unwrap();
    let blocked = service.call(Request::build("block").id(2).finish());
    timeout(DEADLINE, ready).await.unwrap().unwrap();
    let shutdown = service.call(Request::build("shutdown").id(3).finish());
    service.call(Request::build("exit").finish()).await.unwrap();
    assert!(service.session.lock().unwrap().result().is_ok());
    assert!(timeout(DEADLINE, shutdown).await.unwrap().unwrap().is_none());
    release.send(()).unwrap();
    drop(blocked);
}

#[tokio::test(flavor = "current_thread")]
async fn malformed_exit_after_shutdown_fails() {
    for exit in [
        Request::build("exit").id(3).finish(),
        Request::build("exit").params(json!({})).finish(),
        Request::build("exit").params(json!([])).finish(),
        Request::build("exit").params(json!(false)).finish(),
    ] {
        let (inner, _socket) = LspService::new(|_| TestServer(Arc::default(), Arc::default()));
        let mut service = OrderedService::new(inner, Worker::new().unwrap());
        let initialize = Request::build("initialize").params(json!({"capabilities": {}})).id(1).finish();
        service.call(initialize).await.unwrap();
        service.call(Request::build("shutdown").id(2).finish()).await.unwrap();
        service.call(exit).await.unwrap();
        assert!(service.session.lock().unwrap().result().unwrap_err().to_string().contains("invalid exit"));
    }
}

#[tokio::test(flavor = "current_thread")]
async fn worker_errors_and_panics_fail_the_session() {
    for method in ["error", "panic"] {
        let inner = MockService { calls: Arc::default(), gate: None };
        let mut service = OrderedService::new(inner, Worker::new().unwrap());
        let mut stopped = service.stopped();
        let reply = service.call(Request::build(method).id(1).finish());
        timeout(DEADLINE, stopped.changed()).await.unwrap().unwrap();
        assert!(service.session.lock().unwrap().result().is_err());
        drop(reply);
    }
}
