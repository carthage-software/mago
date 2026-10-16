use serde_json::{Value, json};

use super::harness::{Harness, client_capabilities};
use super::references::{apply, rename_at};
use crate::language_server::ServerConfig;

fn capabilities(document_changes: Option<bool>, literals: bool) -> Value {
    let mut capabilities = if literals { client_capabilities() } else { json!({}) };
    capabilities["workspace"] = json!({ "applyEdit": true });
    if let Some(enabled) = document_changes {
        capabilities["workspace"]["workspaceEdit"] = json!({ "documentChanges": enabled });
    }
    capabilities
}

fn check_edit(edit: &Value, uri: &str, version: Option<i32>, source: &str, expected: &str, versioned: bool) {
    let edits = if versioned {
        assert!(edit.get("changes").is_none(), "{edit}");
        let rows = edit["documentChanges"].as_array().unwrap();
        let row = rows.iter().find(|row| row["textDocument"]["uri"] == uri).unwrap_or_else(|| panic!("{edit}"));
        assert_eq!(rows.iter().filter(|row| row["textDocument"]["uri"] == uri).count(), 1, "{edit}");
        assert_eq!(row["textDocument"].get("version"), Some(&json!(version)), "{edit}");
        &row["edits"]
    } else {
        assert!(edit.get("documentChanges").is_none(), "{edit}");
        &edit["changes"][uri]
    };
    assert_eq!(apply(source, edits), expected);
}

async fn actions(h: &mut Harness, uri: &str, source: &str) -> Value {
    let before = &source[..source.find("\\DateTime").unwrap()];
    let position = json!({
        "line": before.bytes().filter(|byte| *byte == b'\n').count(),
        "character": before.rsplit('\n').next().unwrap().encode_utf16().count()
    });
    h.request(
        "textDocument/codeAction",
        json!({
            "textDocument": { "uri": uri }, "range": { "start": position, "end": position },
            "context": { "diagnostics": [], "only": ["refactor.rewrite"] }
        }),
    )
    .await
}

#[tokio::test]
async fn rename_versions_follow_accepted_buffers_and_closed_targets() {
    let disk = "<?php class Target {}";
    let source = "<?php\n/* 😀 */ class Target {}";
    let caller = "<?php new Target();";
    let mut h = Harness::start_with(
        &[("a.php", disk), ("b.php", caller)],
        ServerConfig::default(),
        capabilities(Some(true), true),
    )
    .await;
    h.open("a.php", disk).await;
    h.change("a.php", source, 8).await;
    h.change("a.php", "<?php class Stale {}", 7).await;
    let edit = rename_at(&mut h, "a.php", source, "Target", "Renamed").await.unwrap();
    assert_eq!(edit["documentChanges"].as_array().unwrap().len(), 2, "{edit}");
    check_edit(&edit, &h.url("a.php"), Some(8), source, &source.replace("Target", "Renamed"), true);
    check_edit(&edit, &h.url("b.php"), None, caller, &caller.replace("Target", "Renamed"), true);
    h.close("a.php").await;
    let edit = rename_at(&mut h, "a.php", disk, "Target", "Renamed").await.unwrap();
    check_edit(&edit, &h.url("a.php"), None, disk, &disk.replace("Target", "Renamed"), true);
}

#[tokio::test]
async fn literal_code_actions_use_versions_only_when_negotiated() {
    let source = "<?php class Target {} new \\DateTime();";
    for support in [None, Some(false), Some(true)] {
        let versioned = support == Some(true);
        let mut h =
            Harness::start_with(&[("a.php", source)], ServerConfig::default(), capabilities(support, true)).await;
        h.open("a.php", source).await;
        h.change("a.php", source, 13).await;
        let uri = h.url("a.php");
        let edit = rename_at(&mut h, "a.php", source, "Target", "Renamed").await.unwrap();
        check_edit(&edit, &uri, Some(13), source, &source.replace("Target", "Renamed"), versioned);
        for version in [Some(13), None] {
            if version.is_none() {
                h.close("a.php").await;
            }
            let result = actions(&mut h, &uri, source).await;
            assert_eq!(result.as_array().unwrap().len(), 1, "{result}");
            check_edit(&result[0]["edit"], &uri, version, source, &source.replace("\\DateTime", "DateTime"), versioned);
        }
    }
}

#[tokio::test]
async fn legacy_code_actions_convert_verified_edits_at_apply_time() {
    let source = "<?php new \\DateTime();";
    for support in [None, Some(false), Some(true)] {
        let mut h =
            Harness::start_with(&[("a.php", source)], ServerConfig::default(), capabilities(support, false)).await;
        let uri = h.url("a.php");
        h.open("a.php", source).await;
        for version in [Some(9), None] {
            if version.is_none() {
                h.close("a.php").await;
            }
            let result = actions(&mut h, &uri, source).await;
            let command = &result[0];
            assert_eq!(command["command"], "mago.applyCodeAction");
            if let Some(version) = version {
                // The contents still match the saved action; use the newer accepted version.
                h.change("a.php", source, version).await;
            }
            let id = h
                .client
                .send_request(
                    "workspace/executeCommand",
                    json!({
                        "command": command["command"], "arguments": command["arguments"]
                    }),
                )
                .await;
            let request = loop {
                let message = h.client.read_message(10).await;
                if message["method"] == "workspace/applyEdit" {
                    break message;
                }
                assert!(message.get("id").is_none(), "{message}");
            };
            check_edit(
                &request["params"]["edit"],
                &uri,
                version,
                source,
                "<?php new DateTime();",
                support == Some(true),
            );
            h.client.respond(request["id"].clone(), json!({ "applied": true })).await;
            let response = h.client.await_response(id, 10).await;
            assert!(response.get("error").is_none(), "{response}");
            assert!(response["result"].is_null(), "{response}");
        }
    }
}

#[cfg(unix)]
#[tokio::test]
async fn versioned_edits_keep_the_accepted_alias_uri() {
    let disk = "<?php class Disk {}";
    let source = "<?php\n/* 😀 */ class Target {} new \\DateTime();";
    let mut h = Harness::start_with(&[("a.php", disk)], ServerConfig::default(), capabilities(Some(true), true)).await;
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("source");
    std::os::unix::fs::symlink(h.workspace.canonicalize().unwrap(), &root).unwrap();
    let alias = super::harness::url(&root.join("a.php"));
    h.client
        .send_notification(
            "textDocument/didOpen",
            json!({
                "textDocument": { "uri": alias, "languageId": "php", "version": 17, "text": source }
            }),
        )
        .await;
    h.open("a.php", "<?php class Rejected {}").await;
    let edit = rename_at(&mut h, "a.php", source, "Target", "Renamed").await.unwrap();
    assert_eq!(edit["documentChanges"].as_array().unwrap().len(), 1, "{edit}");
    check_edit(&edit, &alias, Some(17), source, &source.replace("Target", "Renamed"), true);
    let result = actions(&mut h, &alias, source).await;
    assert_eq!(result.as_array().unwrap().len(), 1, "{result}");
    check_edit(&result[0]["edit"], &alias, Some(17), source, &source.replace("\\DateTime", "DateTime"), true);
}
