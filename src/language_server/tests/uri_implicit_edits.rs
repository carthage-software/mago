//! Implicit document edits must use the URI of the accepted open buffer.

#![cfg(unix)]

use serde_json::{Value, json};

use super::harness::{Harness, url};
use super::references::apply;

async fn format(h: &mut Harness, method: &str, uri: &str) -> Value {
    let mut params = json!({
        "textDocument": { "uri": uri },
        "options": { "tabSize": 4, "insertSpaces": true }
    });
    if method == "textDocument/rangeFormatting" {
        params["range"] = json!({
            "start": { "line": 1, "character": 0 },
            "end": { "line": 1, "character": 8 }
        });
    }
    h.request(method, params).await
}

async fn open(h: &mut Harness, uri: &str, source: &str, version: i32) {
    h.client
        .send_notification(
            "textDocument/didOpen",
            json!({ "textDocument": { "uri": uri, "languageId": "php", "version": version, "text": source } }),
        )
        .await;
}

async fn check_implicit_edits(method: &str) {
    let disk = "<?php\n$disk=0;\n";
    let accepted = "<?php\n$kept=1;\n";
    let rejected = "<?php\n$lost=2;\n";
    let mut h = Harness::start(&[("a.php", disk)]).await;
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("source");
    std::os::unix::fs::symlink(h.workspace.canonicalize().unwrap(), &root).unwrap();
    let alias = url(&root.join("a.php"));
    let canonical = h.url("a.php");

    // A closed file has no accepted URI, so either path may format its disk text.
    let edits = format(&mut h, method, &alias).await;
    assert!(apply(disk, &edits).contains("$disk = 0;"), "{edits}");

    open(&mut h, &alias, accepted, 1).await;
    open(&mut h, &canonical, rejected, 2).await;
    let edits = format(&mut h, method, &alias).await;
    let formatted = apply(accepted, &edits);
    assert!(formatted.contains("$kept = 1;"), "{edits}");
    assert!(!formatted.contains("$lost") && !formatted.contains("$disk"), "{formatted}");

    // These responses carry no URI. The client applies them to the requested document.
    let edits = format(&mut h, method, &canonical).await;
    assert!(edits.is_null(), "{method} returned another buffer's edits: {edits}");

    h.client.send_notification("textDocument/didClose", json!({ "textDocument": { "uri": alias } })).await;
    for uri in [&alias, &canonical] {
        let edits = format(&mut h, method, uri).await;
        assert!(apply(disk, &edits).contains("$disk = 0;"), "{edits}");
    }
    assert_eq!(std::fs::read_to_string(h.workspace.join("a.php")).unwrap(), disk);
}

#[tokio::test]
async fn formatting_rejects_a_uri_that_did_not_open_the_buffer() {
    check_implicit_edits("textDocument/formatting").await;
}

#[tokio::test]
async fn range_formatting_rejects_a_uri_that_did_not_open_the_buffer() {
    check_implicit_edits("textDocument/rangeFormatting").await;
}

async fn completion_at_name(h: &mut Harness, uri: &str) -> Value {
    h.request(
        "textDocument/completion",
        json!({ "textDocument": { "uri": uri }, "position": { "line": 2, "character": 7 } }),
    )
    .await
}

fn check_class_completion(result: &Value, source: &str, prefix: &str, name: &str) {
    let items = result["items"].as_array().unwrap_or_else(|| panic!("{result}"));
    let item = items.iter().find(|item| item["label"] == name).unwrap_or_else(|| panic!("missing {name}: {result}"));
    let edits = json!([item["textEdit"]]);
    assert_eq!(apply(source, &edits), source.replace(&format!("{prefix}()"), &format!("{name}()")));
}

#[tokio::test]
async fn completion_edits_require_the_accepted_buffer_uri() {
    let disk = "<?php\nclass DiskClass {}\nnew Dis();\n";
    let accepted = "<?php\nclass KeptClass {}\nnew Kep();\n";
    let rejected = "<?php\nclass LostClass {}\nnew Los();\n";
    let mut h = Harness::start(&[("a.php", disk)]).await;
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("source");
    std::os::unix::fs::symlink(h.workspace.canonicalize().unwrap(), &root).unwrap();
    let alias = url(&root.join("a.php"));
    let canonical = h.url("a.php");

    for uri in [&alias, &canonical] {
        let result = completion_at_name(&mut h, uri).await;
        check_class_completion(&result, disk, "Dis", "DiskClass");
    }

    open(&mut h, &alias, accepted, 1).await;
    open(&mut h, &canonical, rejected, 2).await;
    let result = completion_at_name(&mut h, &alias).await;
    check_class_completion(&result, accepted, "Kep", "KeptClass");

    let result = completion_at_name(&mut h, &canonical).await;
    assert!(
        result.is_null() || result["items"].as_array().is_some_and(Vec::is_empty),
        "completion returned another buffer's edits: {result}"
    );

    h.client.send_notification("textDocument/didClose", json!({ "textDocument": { "uri": alias } })).await;
    for uri in [&alias, &canonical] {
        let result = completion_at_name(&mut h, uri).await;
        check_class_completion(&result, disk, "Dis", "DiskClass");
    }
    assert_eq!(std::fs::read_to_string(h.workspace.join("a.php")).unwrap(), disk);
}
