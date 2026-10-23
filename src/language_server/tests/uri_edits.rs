//! Outgoing edits and locations keep the URI used to open an unsaved buffer.

#![cfg(unix)]

use std::path::PathBuf;

use mago_database::file::File;
use serde_json::{Value, json};
use tempfile::TempDir;
use tower_lsp_server::gen_lsp_types::Position;

use super::harness::{Harness, url};
use super::references::apply;
use crate::language_server::position::position_at_offset;

fn alias_root(h: &Harness) -> (TempDir, PathBuf) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("source");
    std::os::unix::fs::symlink(h.workspace.canonicalize().unwrap(), &path).unwrap();
    (directory, path)
}

async fn open(h: &mut Harness, uri: &str, source: &str) {
    h.client
        .send_notification(
            "textDocument/didOpen",
            json!({ "textDocument": { "uri": uri, "languageId": "php", "version": 7, "text": source } }),
        )
        .await;
}

fn position(source: &str, needle: &str) -> Position {
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    position_at_offset(&file, source.find(needle).unwrap() as u32)
}

async fn rename(h: &mut Harness, uri: &str, source: &str, needle: &str, name: &str) -> Value {
    h.request(
        "textDocument/rename",
        json!({ "textDocument": { "uri": uri }, "position": position(source, needle), "newName": name }),
    )
    .await
}

fn check_edits(edit: &Value, files: &[(&str, &str, String)]) {
    let changes = edit["changes"].as_object().unwrap_or_else(|| panic!("missing changes: {edit}"));
    assert_eq!(changes.len(), files.len(), "{edit}");
    for (uri, source, expected) in files {
        let edits = changes.get(*uri).unwrap_or_else(|| panic!("missing active URI {uri}: {edit}"));
        assert_eq!(apply(source, edits), *expected);
    }
}

#[tokio::test]
async fn local_rename_edits_use_the_active_alias_and_buffer_offsets() {
    let disk = "<?php function disk() { $item = 0; return $item; }";
    let source = "<?php\nfunction buffer() { /* 😀 */ $item = 1; return $item; }\n";
    let mut h = Harness::start(&[("a.php", disk)]).await;
    let (_directory, root) = alias_root(&h);
    let alias = url(&root.join("a.php"));
    let canonical = h.url("a.php");
    open(&mut h, &alias, source).await;
    for query_uri in [&alias, &canonical] {
        let edit = rename(&mut h, query_uri, source, "$item", "value").await;
        check_edits(&edit, &[(&alias, source, source.replace("$item", "$value"))]);
    }
    assert_eq!(std::fs::read_to_string(h.workspace.join("a.php")).unwrap(), disk);
}

#[tokio::test]
async fn global_rename_edits_use_each_open_files_alias() {
    let disk = "<?php class Disk {}";
    let caller_disk = "<?php new Disk();";
    let source = "<?php\n/* unsaved 😀 */ class Target {}\n";
    let caller = "<?php\n/* 😀 */ new Target();\n";
    let mut h = Harness::start(&[("a.php", disk), ("b.php", caller_disk)]).await;
    let (_directory, root) = alias_root(&h);
    let alias = url(&root.join("a.php"));
    let caller_alias = url(&root.join("b.php"));
    open(&mut h, &alias, source).await;
    open(&mut h, &caller_alias, caller).await;
    let edit = rename(&mut h, &alias, source, "Target", "RenamedTarget").await;
    check_edits(
        &edit,
        &[
            (&alias, source, source.replace("Target", "RenamedTarget")),
            (&caller_alias, caller, caller.replace("Target", "RenamedTarget")),
        ],
    );
    assert_eq!(std::fs::read_to_string(h.workspace.join("a.php")).unwrap(), disk);
    assert_eq!(std::fs::read_to_string(h.workspace.join("b.php")).unwrap(), caller_disk);
}

#[tokio::test]
async fn import_alias_rename_edits_use_the_active_document_uri() {
    let disk = "<?php namespace App; use Lib\\Thing as Disk; new Disk();";
    let source = "<?php\nnamespace App;\nuse Lib\\Thing as Alias;\n/* 😀 */ $thing = new Alias();\n";
    let mut h = Harness::start(&[("a.php", disk), ("lib.php", "<?php namespace Lib; class Thing {}")]).await;
    let (_directory, root) = alias_root(&h);
    let alias = url(&root.join("a.php"));
    open(&mut h, &alias, source).await;
    let edit = rename(&mut h, &alias, source, "Alias;", "Renamed").await;
    check_edits(&edit, &[(&alias, source, source.replace("Alias", "Renamed"))]);
    assert_eq!(std::fs::read_to_string(h.workspace.join("a.php")).unwrap(), disk);
}

#[tokio::test]
async fn code_action_edits_use_the_active_alias_and_buffer_offsets() {
    let disk = "<?php new \\DateTime();";
    let source = "<?php\r\n/* 😀 */ $time = new \\DateTime();\r\n";
    let mut h = Harness::start(&[("a.php", disk)]).await;
    let (_directory, root) = alias_root(&h);
    let alias = url(&root.join("a.php"));
    open(&mut h, &alias, source).await;
    let position = position(source, "\\DateTime");
    let actions = h
        .request(
            "textDocument/codeAction",
            json!({
                "textDocument": { "uri": alias },
                "range": { "start": position, "end": position },
                "context": { "diagnostics": [], "only": ["refactor.rewrite"] }
            }),
        )
        .await;
    let actions = actions.as_array().unwrap();
    assert_eq!(actions.len(), 1, "{actions:?}");
    check_edits(&actions[0]["edit"], &[(&alias, source, source.replace("\\DateTime", "DateTime"))]);
    assert_eq!(std::fs::read_to_string(h.workspace.join("a.php")).unwrap(), disk);
}

#[tokio::test]
async fn definition_and_references_point_to_open_aliases_and_buffer_ranges() {
    let source = "<?php\n/* unsaved 😀 */ class Target {}\n";
    let caller = "<?php\n/* 😀 */ new Target();\n";
    let mut h = Harness::start(&[("a.php", "<?php class Disk {}"), ("b.php", "<?php new Disk();")]).await;
    let (_directory, root) = alias_root(&h);
    let alias = url(&root.join("a.php"));
    let caller_alias = url(&root.join("b.php"));
    open(&mut h, &alias, source).await;
    open(&mut h, &caller_alias, caller).await;
    let params = json!({ "textDocument": { "uri": h.url("b.php") }, "position": position(caller, "Target") });
    let definition = h.request("textDocument/definition", params.clone()).await;
    assert_eq!(definition["uri"], alias);
    assert_eq!(definition["range"]["start"], json!(position(source, "Target")));
    let mut params = params;
    params["context"] = json!({ "includeDeclaration": true });
    let references = h.request("textDocument/references", params).await;
    let references = references.as_array().unwrap();
    assert_eq!(references.len(), 2, "{references:?}");
    for (uri, contents) in [(&alias, source), (&caller_alias, caller)] {
        let location =
            references.iter().find(|location| location["uri"] == *uri).unwrap_or_else(|| panic!("{references:?}"));
        let start = position(contents, "Target");
        let end = Position { line: start.line, character: start.character + "Target".len() as u32 };
        assert_eq!(location["range"], json!({ "start": start, "end": end }));
    }
}
