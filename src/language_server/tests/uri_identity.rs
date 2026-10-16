//! File URI aliases must resolve to the loaded source and its open buffer.

#![cfg(unix)]

use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use tempfile::TempDir;
use tower_lsp_server::gen_lsp_types::Location;

use super::harness::{Harness, client_capabilities, url};
use super::references::{READ_ONLY_STUBS, refusable};
use crate::config::LoadOptions;
use crate::language_server::ServerConfig;
use crate::language_server::uri::UriFilePath;

fn alias_to(path: &Path) -> (TempDir, PathBuf) {
    let alias = tempfile::tempdir().unwrap();
    let link = alias.path().join("source");
    std::os::unix::fs::symlink(path, &link).unwrap();
    (alias, link)
}

async fn open(h: &mut Harness, uri: &str, text: &str, version: i32) {
    h.client
        .send_notification(
            "textDocument/didOpen",
            json!({ "textDocument": { "uri": uri, "languageId": "php", "version": version, "text": text } }),
        )
        .await;
}

async fn outline(h: &mut Harness, uri: &str) -> Value {
    h.request("textDocument/documentSymbol", json!({ "textDocument": { "uri": uri } })).await
}

async fn diagnostics(h: &mut Harness) -> Vec<Value> {
    h.diagnostics().await
}

async fn open_physical_alias(disk: &str, buffer: &str) -> (Harness, TempDir, String) {
    let config = ServerConfig { linter: false, ..ServerConfig::default() };
    let mut h = Harness::start_with(&[("a.php", "<?php")], config, client_capabilities()).await;
    let external = tempfile::tempdir().unwrap();
    let physical = external.path().join("Original.php");
    std::fs::write(&physical, disk).unwrap();
    std::os::unix::fs::symlink(&physical, h.workspace.join("Alias.php")).unwrap();
    h.watched("Alias.php", 1).await;
    diagnostics(&mut h).await;
    let uri = url(&physical.canonicalize().unwrap());
    open(&mut h, &uri, buffer, 7).await;
    // Accept the buffer before the caller changes the link on disk.
    outline(&mut h, &uri).await;
    (h, external, uri)
}

#[tokio::test]
async fn symlink_and_parent_segment_uris_find_the_loaded_source() {
    let mut h = Harness::start(&[("a.php", "<?php class Loaded {}")]).await;
    let root = h.workspace.canonicalize().unwrap();
    let (_alias, link) = alias_to(&root);
    std::fs::create_dir(root.join("child")).unwrap();
    for path in [link.join("a.php"), root.join("child/../a.php")] {
        let uri = url(&path);
        let result = outline(&mut h, &uri).await;
        assert_eq!(result[0]["name"], "Loaded", "uri={uri}: {result}");
    }
}

#[tokio::test]
async fn open_alias_updates_the_loaded_file_and_blocks_canonical_disk_events() {
    let mut h = Harness::start(&[("a.php", "<?php class Disk {}")]).await;
    let (_alias, link) = alias_to(&h.workspace.canonicalize().unwrap());
    let alias_uri = url(&link.join("a.php"));
    let canonical_uri = h.url("a.php");
    open(&mut h, &alias_uri, "<?php class Buffer {}", 4).await;
    assert_eq!(outline(&mut h, &canonical_uri).await[0]["name"], "Buffer");
    assert_eq!(h.request("workspace/symbol", json!({ "query": "Disk" })).await, json!([]));

    h.watched("a.php", 2).await;
    assert_eq!(outline(&mut h, &canonical_uri).await[0]["name"], "Buffer");
    h.client.send_notification("textDocument/didSave", json!({ "textDocument": { "uri": alias_uri } })).await;
    assert_eq!(outline(&mut h, &canonical_uri).await[0]["name"], "Buffer");

    h.client.send_notification("textDocument/didClose", json!({ "textDocument": { "uri": alias_uri } })).await;
    assert_eq!(outline(&mut h, &canonical_uri).await[0]["name"], "Disk");
    assert_eq!(h.request("workspace/symbol", json!({ "query": "Buffer" })).await, json!([]));
}

#[tokio::test]
async fn missing_alias_buffer_uses_its_existing_parents_identity() {
    let mut h = Harness::start(&[("anchor.php", "<?php")]).await;
    let (_alias, link) = alias_to(&h.workspace.canonicalize().unwrap());
    let alias_uri = url(&link.join("new/deep/a.php"));
    let canonical_uri = h.url("new/deep/a.php");
    open(&mut h, &alias_uri, "<?php class Unsaved {}", 1).await;
    assert_eq!(outline(&mut h, &canonical_uri).await[0]["name"], "Unsaved");
    assert!(!h.workspace.join("new").exists());

    h.client.send_notification("textDocument/didClose", json!({ "textDocument": { "uri": alias_uri } })).await;
    assert!(outline(&mut h, &canonical_uri).await.is_null());
}

#[tokio::test]
async fn duplicate_uri_for_an_open_file_cannot_change_or_close_its_buffer() {
    let mut h = Harness::start(&[("a.php", "<?php class Disk {}")]).await;
    let canonical_uri = h.url("a.php");
    let alias_uri = format!("{}%61.php", canonical_uri.strip_suffix("a.php").unwrap());
    h.open("a.php", "<?php class First {}").await;
    open(&mut h, &alias_uri, "<?php class Rejected {}", 99).await;
    assert_eq!(outline(&mut h, &canonical_uri).await[0]["name"], "First");

    h.client
        .send_notification(
            "textDocument/didChange",
            json!({ "textDocument": { "uri": alias_uri, "version": 100 }, "contentChanges": [{ "text": "<?php class Wrong {}" }] }),
        )
        .await;
    h.client.send_notification("textDocument/didClose", json!({ "textDocument": { "uri": alias_uri } })).await;
    assert_eq!(outline(&mut h, &canonical_uri).await[0]["name"], "First");
    h.change("a.php", "<?php class Latest {}", 2).await;
    assert_eq!(outline(&mut h, &canonical_uri).await[0]["name"], "Latest");
    h.close("a.php").await;
    assert_eq!(outline(&mut h, &canonical_uri).await[0]["name"], "Disk");
}

#[tokio::test]
async fn opening_an_alias_moves_diagnostics_to_the_open_uri_and_version() {
    let config = ServerConfig { linter: false, ..ServerConfig::default() };
    let mut h = Harness::start_with(&[("a.php", "<?php missing_function();")], config, client_capabilities()).await;
    let canonical_uri = h.url("a.php");
    let initial = h.client.await_notification("textDocument/publishDiagnostics").await;
    assert_eq!(initial["params"]["uri"], canonical_uri, "{initial}");
    assert!(!initial["params"]["diagnostics"].as_array().unwrap().is_empty(), "{initial}");

    let (_alias, link) = alias_to(&h.workspace.canonicalize().unwrap());
    let alias_uri = url(&link.join("a.php"));
    open(&mut h, &alias_uri, "<?php", 3).await;
    let opened = diagnostics(&mut h).await;
    assert!(
        opened.iter().any(|message| message["uri"] == canonical_uri && message["diagnostics"] == json!([])),
        "{opened:?}"
    );
    assert!(
        opened.iter().any(|message| message["uri"] == alias_uri
            && message["version"] == 3
            && message["diagnostics"] == json!([])),
        "{opened:?}"
    );

    h.client.send_notification("textDocument/didClose", json!({ "textDocument": { "uri": alias_uri } })).await;
    let closed = diagnostics(&mut h).await;
    assert!(
        closed.iter().any(|message| message["uri"] == alias_uri && message["diagnostics"] == json!([])),
        "{closed:?}"
    );
    assert!(
        closed.iter().any(|message| message["uri"] == canonical_uri
            && message["version"].is_null()
            && !message["diagnostics"].as_array().unwrap().is_empty()),
        "{closed:?}"
    );
}

#[tokio::test]
async fn builtin_cache_alias_keeps_navigation_and_edit_guards() {
    let mut h = Harness::start(&[("a.php", "<?php strlen('x');")]).await;
    let location: Location = serde_json::from_value(h.at("textDocument/definition", "a.php", 0, 8).await).unwrap();
    let path = location.uri.to_file_path().unwrap().into_owned();
    let (_alias, link) = alias_to(path.parent().unwrap());
    let alias_uri = url(&link.join(path.file_name().unwrap()));
    let params = json!({ "textDocument": { "uri": alias_uri }, "position": location.range.start });
    let hover = h.request("textDocument/hover", params.clone()).await;
    assert!(hover["contents"]["value"].as_str().is_some_and(|value| value.contains("function strlen(")), "{hover}");
    open(&mut h, &alias_uri, "<?php", 1).await;
    assert_eq!(h.request("textDocument/hover", params.clone()).await, hover);
    assert_eq!(refusable(&mut h, "textDocument/prepareRename", params).await, Err(READ_ONLY_STUBS.into()));
}

#[tokio::test]
async fn loaded_symlink_descendants_accept_buffer_changes() {
    let workspace = tempfile::tempdir().unwrap();
    let dependency = tempfile::tempdir().unwrap();
    let disk = "<?php function external_value(int $disk): int { return $disk; }";
    let edited = "<?php function external_value(string $buffer): string { return $buffer; }";
    let caller = "<?php external_value(null);";
    std::fs::write(dependency.path().join("lib.php"), disk).unwrap();
    std::fs::create_dir(workspace.path().join("vendor")).unwrap();
    std::os::unix::fs::symlink(dependency.path(), workspace.path().join("vendor/package")).unwrap();
    std::fs::write(workspace.path().join("caller.php"), caller).unwrap();
    std::fs::write(workspace.path().join("mago.toml"), "[source]\npaths = ['caller.php']\nincludes = ['vendor']\n")
        .unwrap();
    let config = ServerConfig {
        launch: LoadOptions { workspace: Some(workspace.path().to_path_buf()), ..LoadOptions::default() },
        ..ServerConfig::default()
    };
    let mut h = Harness::start_with(&[], config, client_capabilities()).await;
    let caller_uri = url(&workspace.path().canonicalize().unwrap().join("caller.php"));
    let definition = h.at_uri("textDocument/definition", &caller_uri, 0, 10).await;
    let uri = definition["uri"].as_str().expect("loader must include the symlinked dependency").to_owned();
    let position = caller.find("null").unwrap() as u32;
    let before = h.at_uri("textDocument/signatureHelp", &caller_uri, 0, position).await;
    assert_eq!(before["signatures"][0]["label"], "function external_value(int $disk): int");

    open(&mut h, &uri, edited, 1).await;
    let after = h.at_uri("textDocument/signatureHelp", &caller_uri, 0, position).await;
    assert_eq!(after["signatures"][0]["label"], "function external_value(string $buffer): string");
}

#[tokio::test]
async fn renamed_file_symlinks_resolve_from_the_physical_source_uri() {
    let workspace = tempfile::tempdir().unwrap();
    let dependency = tempfile::tempdir().unwrap();
    let disk = "<?php function external_value(int $disk): int { return $disk; }";
    let edited = "<?php function external_value(string $buffer): string { return $buffer; }";
    let caller = "<?php external_value(null);";
    let physical_path = dependency.path().join("Actual.php");
    std::fs::write(&physical_path, disk).unwrap();
    std::fs::create_dir(workspace.path().join("vendor")).unwrap();
    std::os::unix::fs::symlink(&physical_path, workspace.path().join("vendor/Alias.php")).unwrap();
    std::fs::write(workspace.path().join("caller.php"), caller).unwrap();
    std::fs::write(workspace.path().join("mago.toml"), "[source]\npaths = ['caller.php']\nincludes = ['vendor']\n")
        .unwrap();
    let config = ServerConfig {
        launch: LoadOptions { workspace: Some(workspace.path().to_path_buf()), ..LoadOptions::default() },
        ..ServerConfig::default()
    };
    let mut h = Harness::start_with(&[], config, client_capabilities()).await;
    let caller_uri = url(&workspace.path().canonicalize().unwrap().join("caller.php"));
    let before = h.at_uri("textDocument/signatureHelp", &caller_uri, 0, caller.find("null").unwrap() as u32).await;
    assert_eq!(before["signatures"][0]["label"], "function external_value(int $disk): int");

    let physical_uri = url(&physical_path.canonicalize().unwrap());
    assert_eq!(outline(&mut h, &physical_uri).await[0]["name"], "external_value");
    open(&mut h, &physical_uri, edited, 1).await;
    let after = h.at_uri("textDocument/signatureHelp", &caller_uri, 0, caller.find("null").unwrap() as u32).await;
    assert_eq!(after["signatures"][0]["label"], "function external_value(string $buffer): string");
}

#[tokio::test]
async fn open_renamed_link_keeps_its_buffer_after_the_target_is_deleted() {
    let workspace = tempfile::tempdir().unwrap();
    let dependency = tempfile::tempdir().unwrap();
    let physical_path = dependency.path().join("Actual.php");
    let caller = "<?php external_value(null);";
    std::fs::write(&physical_path, "<?php function external_value(int $disk): int { return $disk; }").unwrap();
    std::fs::create_dir(workspace.path().join("vendor")).unwrap();
    std::os::unix::fs::symlink(&physical_path, workspace.path().join("vendor/Alias.php")).unwrap();
    std::fs::write(workspace.path().join("caller.php"), caller).unwrap();
    std::fs::write(workspace.path().join("mago.toml"), "[source]\npaths = ['caller.php']\nincludes = ['vendor']\n")
        .unwrap();
    let config = ServerConfig {
        launch: LoadOptions { workspace: Some(workspace.path().to_path_buf()), ..LoadOptions::default() },
        ..ServerConfig::default()
    };
    let mut h = Harness::start_with(&[], config, client_capabilities()).await;
    let caller_uri = url(&workspace.path().canonicalize().unwrap().join("caller.php"));
    let physical_uri = url(&physical_path.canonicalize().unwrap());
    let alias_uri = url(&workspace.path().canonicalize().unwrap().join("vendor/Alias.php"));
    let position = caller.find("null").unwrap() as u32;
    open(&mut h, &alias_uri, "<?php function external_value(string $buffer): string { return $buffer; }", 1).await;
    let opened = h.at_uri("textDocument/signatureHelp", &caller_uri, 0, position).await;
    assert_eq!(opened["signatures"][0]["label"], "function external_value(string $buffer): string");

    std::fs::remove_file(&physical_path).unwrap();
    h.client
        .send_notification(
            "textDocument/didChange",
            json!({ "textDocument": { "uri": alias_uri, "version": 2 },
                "contentChanges": [{ "text": "<?php function external_value(bool $latest): bool { return $latest; }" }] }),
        )
        .await;
    let changed = h.at_uri("textDocument/signatureHelp", &caller_uri, 0, position).await;
    assert_eq!(changed["signatures"][0]["label"], "function external_value(bool $latest): bool");

    for uri in [&alias_uri, &physical_uri] {
        h.client
            .send_notification("workspace/didChangeWatchedFiles", json!({ "changes": [{ "uri": uri, "type": 3 }] }))
            .await;
        let watched = h.at_uri("textDocument/signatureHelp", &caller_uri, 0, position).await;
        assert_eq!(watched["signatures"][0]["label"], "function external_value(bool $latest): bool", "uri={uri}");
    }

    h.client.send_notification("textDocument/didClose", json!({ "textDocument": { "uri": alias_uri } })).await;
    assert_eq!(h.request("workspace/symbol", json!({ "query": "external_value" })).await, json!([]));
    assert!(h.at_uri("textDocument/signatureHelp", &caller_uri, 0, position).await.is_null());
}

#[tokio::test]
async fn physical_buffer_close_restores_a_repointed_loaded_alias() {
    let (mut h, external, physical_uri) =
        open_physical_alias("<?php class Original {}", "<?php class Unsaved {}").await;
    let alias_uri = h.url("Alias.php");
    let replacement = external.path().join("Replacement.php");
    std::fs::write(&replacement, "<?php class Replacement {}").unwrap();
    std::fs::remove_file(h.workspace.join("Alias.php")).unwrap();
    std::os::unix::fs::symlink(replacement, h.workspace.join("Alias.php")).unwrap();
    h.watched("Alias.php", 2).await;
    assert_eq!(outline(&mut h, &alias_uri).await[0]["name"], "Unsaved");

    h.client.send_notification("textDocument/didClose", json!({ "textDocument": { "uri": physical_uri } })).await;
    assert_eq!(outline(&mut h, &alias_uri).await[0]["name"], "Replacement");
    for name in ["Original", "Unsaved"] {
        assert_eq!(h.request("workspace/symbol", json!({ "query": name })).await, json!([]));
    }
}

#[tokio::test]
async fn physical_buffer_close_removes_a_deleted_loaded_alias() {
    let (mut h, external, physical_uri) =
        open_physical_alias("<?php class Original {}", "<?php class Unsaved {}").await;
    let alias_uri = h.url("Alias.php");
    std::fs::remove_file(h.workspace.join("Alias.php")).unwrap();
    h.watched("Alias.php", 3).await;
    assert_eq!(outline(&mut h, &alias_uri).await[0]["name"], "Unsaved");

    h.client.send_notification("textDocument/didClose", json!({ "textDocument": { "uri": physical_uri } })).await;
    assert!(outline(&mut h, &alias_uri).await.is_null());
    assert!(external.path().join("Original.php").is_file());
    for name in ["Original", "Unsaved"] {
        assert_eq!(h.request("workspace/symbol", json!({ "query": name })).await, json!([]));
    }
}

#[tokio::test]
async fn closing_a_buffer_under_a_replaced_directory_removes_its_symbols() {
    let config = ServerConfig { linter: false, ..ServerConfig::default() };
    let mut h = Harness::start_with(&[("a.php", "<?php")], config, client_capabilities()).await;
    let directory = h.workspace.join("src");
    std::fs::create_dir(&directory).unwrap();
    std::fs::write(directory.join("Model.php"), "<?php class Model {}").unwrap();
    h.watched("src", 1).await;
    h.open("src/Model.php", "<?php class UnsavedModel {}").await;
    let uri = h.url("src/Model.php");
    assert_eq!(outline(&mut h, &uri).await[0]["name"], "UnsavedModel");

    std::fs::remove_dir_all(&directory).unwrap();
    std::fs::write(&directory, "replacement file").unwrap();
    h.watched("src", 3).await;
    assert_eq!(outline(&mut h, &uri).await[0]["name"], "UnsavedModel");
    h.close("src/Model.php").await;
    assert!(outline(&mut h, &uri).await.is_null());
    assert_eq!(h.request("workspace/symbol", json!({"query": "UnsavedModel"})).await, json!([]));
}

#[tokio::test]
async fn physical_buffer_close_restores_each_workspaces_loaded_path() {
    let (mut h, external, physical_uri) =
        open_physical_alias("<?php class Original {}", "<?php class Unsaved {}").await;
    let alias_uri = h.url("Alias.php");
    h.client
        .send_notification(
            "workspace/didChangeWorkspaceFolders",
            json!({ "event": { "added": [{ "uri": url(&external.path().canonicalize().unwrap()), "name": "external" }], "removed": [] } }),
        )
        .await;
    assert_eq!(outline(&mut h, &physical_uri).await[0]["name"], "Unsaved");

    let replacement = external.path().join("Replacement.php");
    std::fs::write(&replacement, "<?php class Replacement {}").unwrap();
    std::fs::remove_file(h.workspace.join("Alias.php")).unwrap();
    std::os::unix::fs::symlink(replacement, h.workspace.join("Alias.php")).unwrap();
    h.watched("Alias.php", 2).await;
    for uri in [&physical_uri, &alias_uri] {
        assert_eq!(outline(&mut h, uri).await[0]["name"], "Unsaved");
    }

    h.client.send_notification("textDocument/didClose", json!({ "textDocument": { "uri": physical_uri } })).await;
    assert_eq!(outline(&mut h, &alias_uri).await[0]["name"], "Replacement");
    assert_eq!(outline(&mut h, &physical_uri).await[0]["name"], "Original");
}

#[tokio::test]
async fn physical_buffer_close_moves_unchanged_diagnostics_to_a_repointed_alias() {
    let source = "<?php missing_function();";
    let (mut h, external, physical_uri) = open_physical_alias(source, source).await;
    let alias_uri = h.url("Alias.php");
    let opened = diagnostics(&mut h).await;
    let buffer = opened.iter().find(|message| message["uri"] == physical_uri).unwrap();
    assert_eq!(buffer["version"], 7);
    assert!(buffer["diagnostics"].as_array().unwrap().iter().any(|issue| issue["code"] == "non-existent-function"));

    let replacement = external.path().join("Replacement.php");
    std::fs::write(&replacement, source).unwrap();
    std::fs::remove_file(h.workspace.join("Alias.php")).unwrap();
    std::os::unix::fs::symlink(replacement, h.workspace.join("Alias.php")).unwrap();
    h.watched("Alias.php", 2).await;
    assert!(diagnostics(&mut h).await.is_empty());

    h.client.send_notification("textDocument/didClose", json!({ "textDocument": { "uri": physical_uri } })).await;
    let closed = diagnostics(&mut h).await;
    assert_eq!(closed.len(), 2, "{closed:?}");
    let cleared = closed.iter().find(|message| message["uri"] == physical_uri).unwrap();
    assert!(cleared["version"].is_null());
    assert_eq!(cleared["diagnostics"], json!([]));
    let restored = closed.iter().find(|message| message["uri"] == alias_uri).unwrap();
    assert!(restored["version"].is_null());
    assert_eq!(restored["diagnostics"], buffer["diagnostics"]);
}

#[tokio::test]
async fn closing_an_unreadable_alias_ends_its_diagnostic_uri_and_version() {
    let config = ServerConfig { linter: false, ..ServerConfig::default() };
    let mut h = Harness::start_with(&[("a.php", "<?php")], config, client_capabilities()).await;
    let canonical_uri = h.url("a.php");
    let (_alias, link) = alias_to(&h.workspace.canonicalize().unwrap());
    let alias_uri = url(&link.join("a.php"));
    diagnostics(&mut h).await;
    open(&mut h, &alias_uri, "<?php class Retained {} missing_function();", 7).await;
    let opened = diagnostics(&mut h).await;
    assert!(
        opened.iter().any(|message| message["uri"] == alias_uri
            && message["version"] == 7
            && message["diagnostics"].as_array().unwrap().iter().any(|issue| issue["code"] == "non-existent-function")),
        "{opened:?}"
    );

    let path = h.workspace.join("a.php");
    std::fs::remove_file(&path).unwrap();
    std::fs::create_dir(&path).unwrap();
    h.client.send_notification("textDocument/didClose", json!({ "textDocument": { "uri": alias_uri } })).await;
    let closed = diagnostics(&mut h).await;
    assert!(
        closed.iter().any(|message| message["uri"] == alias_uri
            && message["version"].is_null()
            && message["diagnostics"] == json!([])),
        "{closed:?}"
    );
    assert!(
        closed.iter().any(|message| message["uri"] == canonical_uri
            && message["version"].is_null()
            && message["diagnostics"].as_array().unwrap().iter().any(|issue| issue["code"] == "non-existent-function")),
        "{closed:?}"
    );
    assert_eq!(outline(&mut h, &canonical_uri).await[0]["name"], "Retained");
}

#[tokio::test]
async fn open_uri_chooses_one_diagnostic_owner_across_loaded_aliases() {
    let source = "<?php missing_function();";
    let mut h = Harness::start_multi(&[(
        "long-root",
        &[("Actual.php", source), ("mago.toml", "[analyzer]\nignore = ['non-existent-function']\n")],
    )])
    .await;
    let alias_root = h.workspace.join("a");
    std::fs::create_dir(&alias_root).unwrap();
    std::os::unix::fs::symlink(h.workspace.join("long-root/Actual.php"), alias_root.join("Alias.php")).unwrap();
    h.add_folder("a", &[]).await;
    let alias_uri = h.url_in("a", "Alias.php");
    let physical_uri = h.url_in("long-root", "Actual.php");
    let initial = diagnostics(&mut h).await;
    assert!(
        initial.iter().any(|message| message["uri"] == alias_uri
            && message["version"].is_null()
            && message["diagnostics"].as_array().unwrap().iter().any(|issue| issue["code"] == "non-existent-function")),
        "{initial:?}"
    );

    open(&mut h, &physical_uri, source, 3).await;
    let opened = diagnostics(&mut h).await;
    let alias: Vec<_> = opened.iter().filter(|message| message["uri"] == alias_uri).collect();
    assert_eq!(alias.len(), 1, "{opened:?}");
    assert_eq!(alias[0]["diagnostics"], json!([]), "{opened:?}");
    assert!(alias[0]["version"].is_null(), "{opened:?}");
    let physical: Vec<_> = opened.iter().filter(|message| message["uri"] == physical_uri).collect();
    assert_eq!(physical.len(), 1, "{opened:?}");
    assert_eq!(physical[0]["version"], 3, "{opened:?}");
    assert!(
        physical[0]["diagnostics"].as_array().unwrap().iter().all(|issue| issue["code"] != "non-existent-function"),
        "{opened:?}"
    );
    assert!(diagnostics(&mut h).await.is_empty());
}
