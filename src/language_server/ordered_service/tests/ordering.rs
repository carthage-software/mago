use super::*;

#[tokio::test(flavor = "current_thread")]
async fn handlers_run_in_arrival_order_off_the_runtime_thread() {
    let (mut service, ready, release) = blocked_service(8);
    let first = service.call(Request::build("block").id(1).finish());
    timeout(DEADLINE, ready).await.unwrap().unwrap();
    let second = service.call(Request::build("second").id(2).finish());
    let third = service.call(Request::build("third").id(3).finish());
    assert!(service.inner.calls.methods.lock().unwrap().is_empty());
    release.send(()).unwrap();
    // Poll replies in reverse order; their handlers must still run in wire order.
    assert!(timeout(DEADLINE, third).await.unwrap().unwrap().is_some());
    assert!(second.await.unwrap().is_some());
    assert!(first.await.unwrap().is_some());
    assert_eq!(*service.inner.calls.methods.lock().unwrap(), ["block", "second", "third"]);
}

#[tokio::test(flavor = "current_thread")]
async fn cancellation_runs_without_polling_ordinary_replies() {
    let (mut service, ready, release) = blocked_service(8);
    let mut waiting = vec![service.call(Request::build("block").id(1).finish())];
    timeout(DEADLINE, ready).await.unwrap().unwrap();
    for id in 2..=4 {
        waiting.push(service.call(Request::build("queued").id(id).finish()));
    }
    let cancel = service.call(Request::build("$/cancelRequest").params(json!({"id": 1})).finish());
    timeout(DEADLINE, service.inner.calls.control.notified()).await.unwrap();
    assert_eq!(*service.inner.calls.methods.lock().unwrap(), ["$/cancelRequest"]);
    release.send(()).unwrap();
    cancel.await.unwrap();
    for response in waiting {
        timeout(DEADLINE, response).await.unwrap().unwrap();
    }
}

#[tokio::test(flavor = "current_thread")]
async fn exit_releases_replies_while_a_handler_is_blocked() {
    let (mut service, ready, release) = blocked_service(8);
    let first = service.call(Request::build("block").id(1).finish());
    timeout(DEADLINE, ready).await.unwrap().unwrap();
    let queued = service.call(Request::build("queued").id(2).finish());
    service.call(Request::build("exit").finish()).await.unwrap();
    assert!(*service.stopped.borrow());
    assert!(timeout(DEADLINE, first).await.unwrap().unwrap().is_none());
    assert!(timeout(DEADLINE, queued).await.unwrap().unwrap().is_none());
    release.send(()).unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn a_full_queue_stops_the_session() {
    let (mut service, ready, release) = blocked_service(1);
    let first = service.call(Request::build("block").id(1).finish());
    timeout(DEADLINE, ready).await.unwrap().unwrap();
    let queued = service.call(Request::build("change").finish());
    let overflow = service.call(Request::build("another-change").finish());
    assert!(overflow.await.is_err());
    assert!(*service.stopped.borrow());
    assert!(service.session.lock().unwrap().result().unwrap_err().to_string().contains("queue is full"));
    assert!(first.await.unwrap().is_none());
    assert!(queued.await.unwrap().is_none());
    release.send(()).unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn dropping_the_service_releases_waiting_replies() {
    let (mut service, ready, release) = blocked_service(1);
    let response = service.call(Request::build("block").id(1).finish());
    timeout(DEADLINE, ready).await.unwrap().unwrap();
    drop(service);
    assert!(timeout(DEADLINE, response).await.unwrap().unwrap().is_none());
    release.send(()).unwrap();
}
