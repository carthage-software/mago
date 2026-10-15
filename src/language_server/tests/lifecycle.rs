//! Workspace lifecycle: open / change / close + on-disk
//! create/change/delete via `workspace/didChangeWatchedFiles`.

use serde_json::json;

use super::harness::Harness;

const HOLDER: &str = "<?php\nclass Holder {\n    public int $value = 0;\n}\n";
const CONSUMER: &str = "<?php\nfunction pull(Holder $h): int {\n    return $h->value;\n}\n";

async fn workspace_symbol_names(h: &mut Harness, query: &str) -> Vec<String> {
    let result = h.request("workspace/symbol", json!({ "query": query })).await;
    result.as_array().cloned().unwrap_or_default().iter().filter_map(|s| s["name"].as_str().map(String::from)).collect()
}

#[tokio::test]
async fn buffer_changes_reflect_immediately() {
    let mut h = Harness::start(&[("h.php", HOLDER)]).await;
    h.open("h.php", HOLDER).await;
    h.change("h.php", "<?php\nclass Vault {}\n", 2).await;

    let names = workspace_symbol_names(&mut h, "Vault").await;
    assert!(names.iter().any(|n| n.ends_with("Vault")), "got {names:?}");
}

#[tokio::test]
async fn close_restores_disk_content() {
    let mut h = Harness::start(&[("h.php", HOLDER)]).await;
    h.open("h.php", HOLDER).await;
    h.change("h.php", "<?php\nclass Vault {}\n", 2).await;

    let dropped = workspace_symbol_names(&mut h, "Holder").await;
    assert!(!dropped.iter().any(|n| n.ends_with("Holder")), "expected Holder gone, got {dropped:?}");

    h.close("h.php").await;
    let restored = workspace_symbol_names(&mut h, "Holder").await;
    assert!(restored.iter().any(|n| n.ends_with("Holder")), "got {restored:?}");
}

#[tokio::test]
async fn external_create_picked_up() {
    let mut h = Harness::start(&[("seed.php", HOLDER)]).await;
    std::fs::write(h.workspace.join("late.php"), "<?php\nclass Late { public string $name = ''; }\n").unwrap();
    h.watched("late.php", 1).await;

    let names = workspace_symbol_names(&mut h, "Late").await;
    assert!(names.iter().any(|n| n.ends_with("Late")), "got {names:?}");
}

#[tokio::test]
async fn external_delete_invalidates() {
    let mut h = Harness::start(&[("h.php", HOLDER), ("c.php", CONSUMER)]).await;
    h.open("c.php", CONSUMER).await;
    let resolved = h.at("textDocument/definition", "c.php", 1, 16).await;
    assert!(!resolved.is_null());

    std::fs::remove_file(h.workspace.join("h.php")).unwrap();
    h.watched("h.php", 3).await;

    let result = h.at("textDocument/definition", "c.php", 1, 16).await;
    assert!(result.is_null(), "expected null, got {result}");
}

#[tokio::test]
async fn external_change_re_reads() {
    let mut h = Harness::start(&[("h.php", HOLDER)]).await;
    let updated = "<?php\nclass Alpha {}\nclass Beta {}\n";
    std::fs::write(h.workspace.join("h.php"), updated).unwrap();
    h.watched("h.php", 2).await;

    let names = workspace_symbol_names(&mut h, "").await;
    assert!(names.iter().any(|n| n.ends_with("Alpha")) && names.iter().any(|n| n.ends_with("Beta")), "got {names:?}");
}

#[tokio::test]
async fn change_without_open_is_noop() {
    let mut h = Harness::start(&[("a.php", HOLDER)]).await;
    h.change("a.php", "<?php\nclass Other {}\n", 1).await;

    let result = h.for_doc("textDocument/foldingRange", "a.php").await;
    assert!(!result.is_null());
}

#[tokio::test]
async fn old_versions_cannot_overwrite_the_open_buffer() {
    let mut h = Harness::start(&[("h.php", HOLDER)]).await;
    h.open("h.php", HOLDER).await;
    h.change("h.php", "<?php class Newest {}", 3).await;
    h.change("h.php", "<?php class Older {}", 2).await;
    h.change("h.php", "<?php class Duplicate {}", 3).await;
    h.open("h.php", HOLDER).await;

    let names = workspace_symbol_names(&mut h, "").await;
    assert!(names.iter().any(|n| n == "Newest"), "{names:?}");
    assert!(!names.iter().any(|n| matches!(n.as_str(), "Holder" | "Older" | "Duplicate")), "{names:?}");
}

#[tokio::test]
async fn saved_new_file_survives_close() {
    let mut h = Harness::start(&[("h.php", HOLDER)]).await;
    let text = "<?php class Saved {}";
    h.open("new.php", text).await;
    std::fs::write(h.workspace.join("new.php"), text).unwrap();
    h.watched("new.php", 1).await;
    h.close("new.php").await;

    assert!(workspace_symbol_names(&mut h, "Saved").await.iter().any(|n| n == "Saved"));
}

#[tokio::test]
async fn disk_changes_do_not_replace_open_buffers() {
    let mut h = Harness::start(&[("h.php", HOLDER)]).await;
    h.open("h.php", "<?php class Unsaved {}").await;
    std::fs::write(h.workspace.join("h.php"), "<?php class Disk {}").unwrap();
    h.watched("h.php", 2).await;
    assert!(workspace_symbol_names(&mut h, "Unsaved").await.iter().any(|n| n == "Unsaved"));

    std::fs::remove_file(h.workspace.join("h.php")).unwrap();
    h.watched("h.php", 3).await;
    assert!(workspace_symbol_names(&mut h, "Unsaved").await.iter().any(|n| n == "Unsaved"));

    h.close("h.php").await;
    assert!(workspace_symbol_names(&mut h, "Unsaved").await.is_empty());
}

#[tokio::test]
async fn unsaved_new_file_disappears_on_close() {
    let mut h = Harness::start(&[("h.php", HOLDER)]).await;
    h.open("new.php", "<?php class Unsaved {}").await;
    assert!(workspace_symbol_names(&mut h, "Unsaved").await.iter().any(|n| n == "Unsaved"));
    h.close("new.php").await;
    assert!(workspace_symbol_names(&mut h, "Unsaved").await.is_empty());
}

#[tokio::test]
async fn diagnostics_start_at_initialize_and_follow_buffer_versions() {
    let bad = "<?php\nmissing_function();\n";
    let config = crate::language_server::ServerConfig { linter: false, ..Default::default() };
    let mut h = Harness::start_with(&[("a.php", bad)], config, json!({})).await;
    h.for_doc("textDocument/foldingRange", "a.php").await;
    let initial = h.client.await_notification("textDocument/publishDiagnostics").await["params"].clone();
    assert!(!initial["diagnostics"].as_array().unwrap().is_empty());
    assert!(initial["version"].is_null());

    h.open("a.php", bad).await;
    let opened = h.client.await_notification("textDocument/publishDiagnostics").await["params"].clone();
    assert_eq!(opened["version"], 1);

    for version in [2, 3] {
        h.change("a.php", "<?php\n", version).await;
        let changed = h.client.await_notification("textDocument/publishDiagnostics").await["params"].clone();
        assert_eq!(changed["version"], version);
        assert!(changed["diagnostics"].as_array().unwrap().is_empty());
    }
    h.change("a.php", bad, 2).await;
    let stale = h.diagnostics().await;
    assert!(stale.is_empty(), "{stale:?}");

    h.close("a.php").await;
    h.for_doc("textDocument/foldingRange", "a.php").await;
    let closed = h.client.await_notification("textDocument/publishDiagnostics").await["params"].clone();
    assert!(closed["version"].is_null());
    assert!(!closed["diagnostics"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn closing_unchanged_or_deleted_buffers_releases_diagnostic_versions() {
    let clean = "<?php\n";
    let bad = "<?php\nmissing_function();\n";
    let config = crate::language_server::ServerConfig { linter: false, ..Default::default() };
    let mut h =
        Harness::start_with(&[("clean.php", clean), ("bad.php", bad), ("deleted.php", clean)], config, json!({})).await;
    h.diagnostics().await;

    for (name, contents, delete) in
        [("clean.php", clean, false), ("bad.php", bad, false), ("deleted.php", clean, true), ("new.php", clean, false)]
    {
        h.open(name, contents).await;
        let opened = h.diagnostics().await;
        assert_eq!(opened.len(), 1, "{name}: {opened:?}");
        assert_eq!(opened[0]["uri"], h.url(name));
        assert_eq!(opened[0]["version"], 1);

        if delete {
            std::fs::remove_file(h.workspace.join(name)).unwrap();
            h.watched(name, 3).await;
            assert!(h.diagnostics().await.is_empty());
        }
        h.close(name).await;
        let closed = h.diagnostics().await;
        assert_eq!(closed.len(), 1, "{name}: {closed:?}");
        assert_eq!(closed[0]["uri"], h.url(name));
        assert!(closed[0]["version"].is_null(), "{name}: {closed:?}");
        assert_eq!(closed[0]["diagnostics"], opened[0]["diagnostics"], "{name}: {closed:?}");

        h.close(name).await;
        assert!(h.diagnostics().await.is_empty());
    }
}

#[tokio::test]
async fn dependency_changes_refresh_member_completion() {
    let definitions = "<?php\nclass Before { public function before(): void {} }\nclass After { public function after(): void {} }\nfunction make(): Before { return new Before(); }\n";
    let caller = "<?php\n$value = make();\n$value->before();\n";
    let mut h = Harness::start(&[("definitions.php", definitions), ("caller.php", caller)]).await;
    h.open("definitions.php", definitions).await;

    let before = h.at("textDocument/completion", "caller.php", 2, 8).await;
    assert!(before["items"].as_array().unwrap().iter().any(|item| item["label"] == "before"), "{before}");

    let updated = definitions
        .replace("function make(): Before { return new Before(); }", "function make(): After { return new After(); }");
    h.change("definitions.php", &updated, 2).await;
    let after = h.at("textDocument/completion", "caller.php", 2, 8).await;
    let items = after["items"].as_array().unwrap();
    assert!(items.iter().any(|item| item["label"] == "after"), "{after}");
    assert!(!items.iter().any(|item| item["label"] == "before"), "{after}");
}

#[tokio::test]
async fn borrowed_diagnostics_keep_scoped_ignores_and_buffer_versions() {
    let bad = "<?php\nmissing_function();\n";
    let config = crate::language_server::ServerConfig { linter: false, ..Default::default() };
    let mut h = Harness::start_with(
        &[
            ("mago.toml", "[analyzer]\nignore = [{ code = \"non-existent-function\", in = [\"a.php\"] }]\n"),
            ("a.php", bad),
            ("b.php", bad),
        ],
        config,
        json!({}),
    )
    .await;
    let initial = h.client.await_notification("textDocument/publishDiagnostics").await;
    assert_eq!(initial["params"]["uri"], h.url("b.php"));
    assert!(
        initial["params"]["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue["code"] == "non-existent-function")
    );
    h.open("a.php", bad).await;
    let opened = h.client.await_notification("textDocument/publishDiagnostics").await;
    assert_eq!(opened["params"]["uri"], h.url("a.php"));
    assert_eq!(opened["params"]["version"], 1);
    assert!(opened["params"]["diagnostics"].as_array().unwrap().is_empty());
    h.change("a.php", "<?php\n\nmissing_function();\n", 2).await;
    let changed = h.client.await_notification("textDocument/publishDiagnostics").await;
    assert_eq!(changed["params"]["version"], 2);
    assert!(changed["params"]["diagnostics"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn watcher_batches_honor_exclusions_and_keep_the_latest_disk_state() {
    let config = "[source]\nexcludes = [\"ignored.php\"]\n";
    let mut h =
        Harness::start(&[("mago.toml", config), ("a.php", "<?php class Old {}"), ("b.php", "<?php class Removed {}")])
            .await;
    std::fs::write(h.workspace.join("a.php"), "<?php class NewName {}").unwrap();
    std::fs::remove_file(h.workspace.join("b.php")).unwrap();
    std::fs::write(h.workspace.join("ignored.php"), "<?php class Ignored {}").unwrap();
    std::fs::write(h.workspace.join("notes.txt"), "<?php class NotPHP {}").unwrap();
    h.client
        .send_notification(
            "workspace/didChangeWatchedFiles",
            json!({ "changes": [
        { "uri": h.url("a.php"), "type": 2 },
        { "uri": h.url("b.php"), "type": 3 },
        { "uri": h.url("ignored.php"), "type": 1 },
        { "uri": h.url("notes.txt"), "type": 1 },
        { "uri": h.url("a.php"), "type": 2 }
    ] }),
        )
        .await;
    assert!(workspace_symbol_names(&mut h, "NewName").await.iter().any(|name| name == "NewName"));
    for name in ["Old", "Removed", "Ignored", "NotPHP"] {
        assert!(workspace_symbol_names(&mut h, name).await.iter().all(|found| found != name), "{name}");
    }
}

#[tokio::test]
async fn shutdown_stops_watcher_registration_without_a_client_response() {
    use tokio::io::AsyncBufReadExt;
    use tokio::io::AsyncReadExt;
    use tokio::io::AsyncWriteExt;
    use tokio::io::BufReader;
    use tokio::io::DuplexStream;
    use tokio::time::Duration;
    use tokio::time::timeout;

    async fn send(client: &mut BufReader<DuplexStream>, message: serde_json::Value) {
        let body = message.to_string();
        let frame = format!("Content-Length: {}\r\n\r\n{body}", body.len());
        client.get_mut().write_all(frame.as_bytes()).await.unwrap();
    }

    async fn receive(client: &mut BufReader<DuplexStream>) -> serde_json::Value {
        let mut length = None;
        loop {
            let mut line = String::new();
            assert_ne!(client.read_line(&mut line).await.unwrap(), 0, "server closed the connection");
            if line == "\r\n" {
                break;
            }
            if let Some(value) = line.strip_prefix("Content-Length:") {
                length = Some(value.trim().parse::<usize>().unwrap());
            }
        }
        let mut body = vec![0; length.unwrap()];
        client.read_exact(&mut body).await.unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    let workspace = tempfile::tempdir().unwrap();
    let root_uri = super::harness::url(&workspace.path().canonicalize().unwrap());
    let (client, server) = tokio::io::duplex(8192);
    let server = tokio::spawn(async move {
        let (reader, writer) = tokio::io::split(server);
        crate::language_server::serve(reader, writer, crate::language_server::ServerConfig::default()).await
    });
    let mut client = BufReader::new(client);
    let deadline = Duration::from_secs(5);
    send(
        &mut client,
        json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
            "rootUri": root_uri,
            "capabilities": { "workspace": { "didChangeWatchedFiles": { "dynamicRegistration": true } } }
        } }),
    )
    .await;
    loop {
        let message = timeout(Duration::from_secs(60), receive(&mut client)).await.unwrap();
        if message["id"] == 1 && message.get("method").is_none() {
            assert!(message.get("error").is_none(), "{message}");
            break;
        }
    }
    send(&mut client, json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} })).await;
    let registration = loop {
        let message = timeout(deadline, receive(&mut client)).await.unwrap();
        if message["method"] == "client/registerCapability" {
            break message;
        }
        assert_eq!(message["method"], "window/logMessage", "{message}");
    };

    send(&mut client, json!({ "jsonrpc": "2.0", "id": 2, "method": "shutdown" })).await;
    let shutdown = timeout(deadline, receive(&mut client)).await.unwrap();
    assert_eq!(shutdown, json!({ "jsonrpc": "2.0", "id": 2, "result": null }));

    // A late failure must not wake a detached task and emit a log after shutdown.
    send(
        &mut client,
        json!({ "jsonrpc": "2.0", "id": registration["id"], "error": {
            "code": -32603, "message": "registration refused"
        } }),
    )
    .await;
    assert!(timeout(Duration::from_millis(100), receive(&mut client)).await.is_err());

    send(&mut client, json!({ "jsonrpc": "2.0", "method": "exit" })).await;
    assert!(timeout(deadline, server).await.unwrap().unwrap().is_ok());
}
