//! Shared buffers follow workspace additions and removals.

use serde_json::{Value, json};

use super::harness::{Harness, client_capabilities};
use crate::language_server::ServerConfig;

const DISK: &str = "<?php function shared_value(int $disk): int { return $disk; }";
const STRING_BUFFER: &str = "<?php function shared_value(string $buffer): string { return $buffer; }";
const BOOL_BUFFER: &str = "<?php function shared_value(bool $latest): bool { return $latest; }";
const CALL: &str = "<?php shared_value(null);";
const DISK_SIGNATURE: &str = "function shared_value(int $disk): int";
const STRING_SIGNATURE: &str = "function shared_value(string $buffer): string";
const BOOL_SIGNATURE: &str = "function shared_value(bool $latest): bool";

async fn assert_signature(h: &mut Harness, folder: &str, expected: &str) {
    let uri = h.url_in(folder, "call.php");
    let result = h.at_uri("textDocument/signatureHelp", &uri, 0, CALL.find("null").unwrap() as u32).await;
    assert_eq!(result["signatures"][0]["label"], expected, "folder={folder}: {result}");
}

async fn remove_folder(h: &mut Harness, folder: &str) {
    let uri = h.url(folder);
    h.client
        .send_notification(
            "workspace/didChangeWorkspaceFolders",
            json!({ "event": { "added": [], "removed": [{ "uri": uri, "name": folder }] } }),
        )
        .await;
}

async fn take_diagnostics(h: &mut Harness, uri: &str) -> Vec<Value> {
    h.diagnostics().await.into_iter().filter(|message| message["uri"] == uri).collect()
}

#[tokio::test]
async fn shared_dependency_changes_refresh_signatures_in_both_roots() {
    let mut h = Harness::start_multi(&[
        ("owner", &[("lib.php", DISK), ("call.php", CALL)]),
        ("consumer", &[("call.php", CALL), ("mago.toml", "[source]\nincludes = ['../owner/lib.php']\n")]),
    ])
    .await;
    for folder in ["owner", "consumer"] {
        assert_signature(&mut h, folder, DISK_SIGNATURE).await;
    }

    h.open("owner/lib.php", DISK).await;
    h.change("owner/lib.php", STRING_BUFFER, 2).await;
    for folder in ["owner", "consumer"] {
        assert_signature(&mut h, folder, STRING_SIGNATURE).await;
    }

    h.change("owner/lib.php", BOOL_BUFFER, 1).await;
    for folder in ["owner", "consumer"] {
        assert_signature(&mut h, folder, STRING_SIGNATURE).await;
    }

    let uri = h.url("owner/lib.php");
    h.client.send_notification("textDocument/didSave", json!({ "textDocument": { "uri": uri } })).await;
    for folder in ["owner", "consumer"] {
        assert_signature(&mut h, folder, STRING_SIGNATURE).await;
    }
    h.watched("owner/lib.php", 2).await;
    for folder in ["owner", "consumer"] {
        assert_signature(&mut h, folder, STRING_SIGNATURE).await;
    }

    h.close("owner/lib.php").await;
    for folder in ["owner", "consumer"] {
        assert_signature(&mut h, folder, DISK_SIGNATURE).await;
    }
}

#[tokio::test]
async fn excluded_nested_root_keeps_the_parents_unsaved_buffer() {
    let mut h = Harness::start_multi(&[
        ("parent", &[("call.php", CALL)]),
        ("parent/nested", &[("lib.php", DISK), ("mago.toml", "[source]\nexcludes = ['lib.php']\n")]),
    ])
    .await;
    h.open("parent/nested/lib.php", STRING_BUFFER).await;
    assert_signature(&mut h, "parent", STRING_SIGNATURE).await;
    assert!(h.for_doc("textDocument/documentSymbol", "parent/nested/lib.php").await.is_null());

    h.change("parent/nested/lib.php", &format!("{BOOL_BUFFER}\nmissing_function();"), 2).await;
    assert_signature(&mut h, "parent", BOOL_SIGNATURE).await;
    let uri = h.url("parent/nested/lib.php");
    let _ = take_diagnostics(&mut h, &uri).await;

    remove_folder(&mut h, "parent/nested").await;
    assert_signature(&mut h, "parent", BOOL_SIGNATURE).await;
    let published = take_diagnostics(&mut h, &uri).await;
    assert_eq!(published.len(), 1, "{published:?}");
    assert_eq!(published[0]["version"], 2, "{published:?}");
    assert!(
        published[0]["diagnostics"].as_array().unwrap().iter().any(|issue| issue["code"] == "non-existent-function")
    );

    h.change("parent/nested/lib.php", STRING_BUFFER, 1).await;
    assert_signature(&mut h, "parent", BOOL_SIGNATURE).await;
    h.close("parent/nested/lib.php").await;
    assert_signature(&mut h, "parent", DISK_SIGNATURE).await;
}

#[tokio::test]
async fn closing_a_shared_virtual_dependency_removes_it_from_both_roots() {
    let mut h = Harness::start_multi(&[
        ("owner", &[("call.php", CALL)]),
        ("consumer", &[("call.php", CALL), ("mago.toml", "[source]\nincludes = ['../owner']\n")]),
    ])
    .await;
    assert!(!h.workspace.join("owner/new.php").exists());
    h.open("owner/new.php", STRING_BUFFER).await;
    for folder in ["owner", "consumer"] {
        assert_signature(&mut h, folder, STRING_SIGNATURE).await;
    }

    h.close("owner/new.php").await;
    for folder in ["owner", "consumer"] {
        let uri = h.url_in(folder, "call.php");
        let result = h.at_uri("textDocument/signatureHelp", &uri, 0, CALL.find("null").unwrap() as u32).await;
        assert!(result.is_null(), "folder={folder}: {result}");
    }
}

#[tokio::test]
async fn nested_workspace_changes_diagnostic_owner_without_losing_buffer_version() {
    let config = ServerConfig { linter: false, ..ServerConfig::default() };
    let mut h = Harness::start_with(
        &[
            ("anchor.php", "<?php"),
            ("mago.toml", "[analyzer]\nignore = [{ code = 'non-existent-function', in = ['nested/a.php'] }]\n"),
        ],
        config,
        client_capabilities(),
    )
    .await;
    let nested = h.workspace.join("nested");
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::write(nested.join("a.php"), "<?php\nmissing_function();\n").unwrap();
    let uri = h.url("nested/a.php");
    let _ = take_diagnostics(&mut h, &uri).await;

    h.open("nested/a.php", "<?php\nmissing_function();\n").await;
    h.change("nested/a.php", "<?php\n\nmissing_function();\n", 2).await;
    let opened = take_diagnostics(&mut h, &uri).await;
    let latest = opened.last().expect("outer workspace must publish the accepted buffer version");
    assert_eq!(latest["version"], 2, "{opened:?}");
    assert_eq!(latest["diagnostics"], json!([]), "{opened:?}");

    h.add_folder("nested", &[("mago.toml", "[analyzer]\nignore = []\n")]).await;
    let added = take_diagnostics(&mut h, &uri).await;
    assert_eq!(added.len(), 1, "{added:?}");
    assert_eq!(added[0]["version"], 2, "{added:?}");
    let issue = added[0]["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|issue| issue["code"] == "non-existent-function")
        .expect("nested workspace must apply its own analyzer ignores");
    assert_eq!(issue["range"]["start"]["line"], 2, "{added:?}");

    remove_folder(&mut h, "nested").await;
    let removed = take_diagnostics(&mut h, &uri).await;
    assert_eq!(removed.len(), 1, "{removed:?}");
    assert_eq!(removed[0]["version"], 2, "{removed:?}");
    assert_eq!(removed[0]["diagnostics"], json!([]), "{removed:?}");
}

#[tokio::test]
async fn open_buffer_survives_zero_roots_and_replays_its_latest_version() {
    let mut h = Harness::start_multi(&[("owner", &[("lib.php", DISK), ("call.php", CALL)])]).await;
    h.open("owner/lib.php", STRING_BUFFER).await;
    assert_signature(&mut h, "owner", STRING_SIGNATURE).await;

    remove_folder(&mut h, "owner").await;
    assert_eq!(h.request("workspace/symbol", json!({ "query": "shared_value" })).await, json!([]));
    h.change("owner/lib.php", BOOL_BUFFER, 2).await;
    h.add_folder("owner", &[]).await;
    assert_signature(&mut h, "owner", BOOL_SIGNATURE).await;

    h.change("owner/lib.php", STRING_BUFFER, 1).await;
    assert_signature(&mut h, "owner", BOOL_SIGNATURE).await;
    h.close("owner/lib.php").await;
    assert_signature(&mut h, "owner", DISK_SIGNATURE).await;
}

#[tokio::test]
async fn closing_a_buffer_with_zero_roots_discards_its_snapshot() {
    let mut h = Harness::start_multi(&[("owner", &[("lib.php", DISK), ("call.php", CALL)])]).await;
    h.open("owner/lib.php", STRING_BUFFER).await;
    assert_signature(&mut h, "owner", STRING_SIGNATURE).await;

    remove_folder(&mut h, "owner").await;
    assert_eq!(h.request("workspace/symbol", json!({ "query": "shared_value" })).await, json!([]));
    h.change("owner/lib.php", BOOL_BUFFER, 2).await;
    h.close("owner/lib.php").await;
    h.add_folder("owner", &[]).await;
    assert_signature(&mut h, "owner", DISK_SIGNATURE).await;

    h.change("owner/lib.php", BOOL_BUFFER, 3).await;
    assert_signature(&mut h, "owner", DISK_SIGNATURE).await;
}

#[cfg(unix)]
#[tokio::test]
async fn removing_a_workspace_uses_its_original_symlink_identity() {
    use std::os::unix::fs::symlink;

    use super::harness::url;

    for repoint in [false, true] {
        let roots = tempfile::tempdir().unwrap();
        let root = roots.path().canonicalize().unwrap();
        let removed = root.join("removed");
        let kept = root.join("kept");
        let alias = root.join("alias");
        std::fs::create_dir(&removed).unwrap();
        std::fs::create_dir(&kept).unwrap();
        std::fs::write(removed.join("a.php"), "<?php class Removed {} missing_function();").unwrap();
        std::fs::write(kept.join("b.php"), "<?php class Kept {}").unwrap();
        symlink(&removed, &alias).unwrap();
        let mut h = Harness::start(&[("anchor.php", "<?php")]).await;
        h.client
            .send_notification(
                "workspace/didChangeWorkspaceFolders",
                json!({ "event": {
            "added": [{ "uri": url(&alias), "name": "alias" }, { "uri": url(&kept), "name": "kept" }],
            "removed": []
        } }),
            )
            .await;
        let before = h.request("workspace/symbol", json!({ "query": "Removed" })).await;
        assert!(before.as_array().unwrap().iter().any(|symbol| symbol["name"] == "Removed"), "{before}");
        let removed_uri = url(&removed.join("a.php"));
        let before = take_diagnostics(&mut h, &removed_uri).await;
        assert!(
            before.iter().any(|item| item["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .any(|issue| { issue["code"] == "non-existent-function" })),
            "{before:?}"
        );

        std::fs::remove_file(&alias).unwrap();
        if repoint {
            symlink(&kept, &alias).unwrap();
        }
        h.client
            .send_notification(
                "workspace/didChangeWorkspaceFolders",
                json!({ "event": {
            "added": [], "removed": [{ "uri": url(&alias), "name": "alias" }]
        } }),
            )
            .await;
        assert_eq!(h.request("workspace/symbol", json!({ "query": "Removed" })).await, json!([]));
        let kept_symbols = h.request("workspace/symbol", json!({ "query": "Kept" })).await;
        assert!(kept_symbols.as_array().unwrap().iter().any(|symbol| symbol["name"] == "Kept"), "{kept_symbols}");
        let cleared = take_diagnostics(&mut h, &removed_uri).await;
        assert_eq!(cleared.len(), 1, "{cleared:?}");
        assert_eq!(cleared[0]["diagnostics"], json!([]));
        h.client
            .send_notification(
                "workspace/didChangeWorkspaceFolders",
                json!({ "event": {
            "added": [], "removed": [{ "uri": url(&alias), "name": "alias" }]
        } }),
            )
            .await;
        assert_eq!(h.request("workspace/symbol", json!({ "query": "Kept" })).await, kept_symbols);
    }
}
