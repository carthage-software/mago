use super::*;

#[test]
fn ranged_changes_are_not_coalesced() {
    let request = change("file:///test.php", 1, "\"\n😀\\");
    let mut ranged = serde_json::to_value(&request).unwrap();
    ranged["params"]["contentChanges"][0]["range"] = json!({
        "start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 1}
    });

    assert!(full_change(&serde_json::from_value(ranged).unwrap()).is_none());
}

#[tokio::test(flavor = "current_thread")]
async fn replacing_text_reuses_its_byte_budget() {
    let (mut service, ready, release) = blocked_service(1);
    let block = Request::build("block").id(1).finish();
    let replacement = change("file:///test.php", 3, &"x".repeat(1024));
    let total =
        encoded_size(&block, INPUT_BYTES_LIMIT).unwrap() + encoded_size(&replacement, INPUT_BYTES_LIMIT).unwrap();
    service.input = InputBudget::new(total);
    let first = service.call(block);
    timeout(DEADLINE, ready).await.unwrap().unwrap();
    let queued = service.call(change("file:///test.php", 2, "old"));
    assert!(service.call(replacement).await.unwrap().is_none());
    assert_eq!(service.input.retained_bytes(), total);
    assert_eq!(service.inner.calls.created.load(Ordering::Relaxed), 3);
    release.send(()).unwrap();
    timeout(DEADLINE, queued).await.unwrap().unwrap();
    first.await.unwrap();
    assert_eq!(service.input.retained_bytes(), 0);
    assert_eq!(*service.inner.calls.methods.lock().unwrap(), ["block", "textDocument/didChange"]);
}

#[tokio::test(flavor = "current_thread")]
async fn coalescing_requires_rising_versions() {
    for version in [1, 2] {
        let (mut service, ready, release) = blocked_service(2);
        let first = service.call(Request::build("block").id(1).finish());
        timeout(DEADLINE, ready).await.unwrap().unwrap();
        let second = service.call(change("file:///test.php", 2, "new"));
        let third = service.call(change("file:///test.php", version, "old"));
        assert_eq!(service.inner.calls.created.load(Ordering::Relaxed), 3);
        release.send(()).unwrap();
        timeout(DEADLINE, third).await.unwrap().unwrap();
        second.await.unwrap();
        first.await.unwrap();
        assert_eq!(
            *service.inner.calls.methods.lock().unwrap(),
            ["block", "textDocument/didChange", "textDocument/didChange"]
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn notification_byte_overflow_stops_before_creating_a_handler() {
    let (mut service, ready, release) = blocked_service(1);
    let block = Request::build("block").id(1).finish();
    service.input = InputBudget::new(encoded_size(&block, INPUT_BYTES_LIMIT).unwrap());
    let first = service.call(block);
    timeout(DEADLINE, ready).await.unwrap().unwrap();
    assert!(service.call(Request::build("change").finish()).await.is_err());
    assert_eq!(service.inner.calls.created.load(Ordering::Relaxed), 1);
    assert!(*service.stopped.borrow());
    assert!(service.session.lock().unwrap().result().unwrap_err().to_string().contains("byte budget"));
    release.send(()).unwrap();
    assert!(first.await.unwrap().is_none());
}

#[tokio::test(flavor = "current_thread")]
async fn cancellation_frees_payload_bytes_when_the_budget_is_full() {
    let (mut service, ready, release) = blocked_service(1);
    let block = Request::build("block").id(1).finish();
    let request = Request::build("queued").id(2).finish();
    let block_size = encoded_size(&block, INPUT_BYTES_LIMIT).unwrap();
    let limit = block_size + encoded_size(&request, INPUT_BYTES_LIMIT).unwrap();
    service.input = InputBudget::new(limit);
    let first = service.call(block);
    timeout(DEADLINE, ready).await.unwrap().unwrap();
    let queued = service.call(request);
    assert_eq!(service.input.retained_bytes(), limit);
    service.call(Request::build("$/cancelRequest").params(json!({"id": 2})).finish()).await.unwrap();
    assert_eq!(queued.await.unwrap().unwrap().error().unwrap().code, ErrorCode::RequestCancelled);
    assert_eq!(service.input.retained_bytes(), block_size);
    release.send(()).unwrap();
    timeout(DEADLINE, first).await.unwrap().unwrap();
    assert_eq!(service.input.retained_bytes(), 0);
}

#[tokio::test(flavor = "current_thread")]
async fn dropping_the_service_releases_active_and_queued_payloads() {
    let started = Arc::new(Notify::new());
    let signal = Arc::clone(&started);
    let (inner, _socket) = LspService::new(move |_| TestServer(signal, Arc::default()));
    let mut service = OrderedService::new(inner).unwrap();
    let retained = service.input.counter();
    service.call(Request::build("initialize").params(json!({"capabilities": {}})).id(1).finish()).await.unwrap();
    let active = service.call(
        Request::build("textDocument/hover")
            .params(json!({"textDocument": {"uri": "file:///test.php"}, "position": {"line": 0, "character": 0}}))
            .id(2)
            .finish(),
    );
    timeout(DEADLINE, started.notified()).await.unwrap();
    let queued = service.call(change("file:///test.php", 1, &"x".repeat(1024)));
    assert!(retained.load(Ordering::Relaxed) > 1024);
    drop(service);
    assert!(timeout(DEADLINE, active).await.unwrap().unwrap().is_none());
    assert!(timeout(DEADLINE, queued).await.unwrap().unwrap().is_none());
    timeout(DEADLINE, async {
        while retained.load(Ordering::Relaxed) != 0 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
}
