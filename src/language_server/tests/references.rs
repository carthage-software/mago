use serde_json::Value;
use serde_json::json;

use super::harness::Harness;

async fn references(h: &mut Harness, line: u32, character: u32, declaration: bool) -> Value {
    h.request(
        "textDocument/references",
        json!({
            "textDocument": { "uri": h.url("a.php") },
            "position": { "line": line, "character": character },
            "context": { "includeDeclaration": declaration }
        }),
    )
    .await
}

pub(super) fn apply(source: &str, edits: &Value) -> String {
    let file = mago_database::file::File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let mut edits: Vec<tower_lsp_server::gen_lsp_types::TextEdit> = serde_json::from_value(edits.clone()).unwrap();
    edits.sort_by_key(|edit| std::cmp::Reverse(edit.range.start));
    let mut result = source.to_owned();
    for edit in edits {
        let start = crate::language_server::position::offset_at_position(&file, edit.range.start) as usize;
        let end = crate::language_server::position::offset_at_position(&file, edit.range.end) as usize;
        result.replace_range(start..end, &edit.new_text);
    }
    let arena = mago_allocator::LocalArena::new();
    let edited =
        mago_database::file::File::ephemeral(b"edited.php".as_slice().into(), result.as_bytes().to_vec().into());
    let parsed = mago_syntax::parser::parse_file_borrowed_with_settings(&arena, &edited, Default::default());
    assert!(!parsed.has_errors(), "rename produced invalid PHP: {result}");
    result
}

pub(super) async fn rename_at(
    h: &mut Harness,
    path: &str,
    source: &str,
    needle: &str,
    name: &str,
) -> Result<Value, String> {
    let offset = source.find(needle).unwrap();
    let before = &source[..offset];
    let line = before.bytes().filter(|byte| *byte == b'\n').count();
    let character = before.rsplit('\n').next().unwrap().encode_utf16().count();
    let params = json!({
        "textDocument": { "uri": h.url(path) }, "position": { "line": line, "character": character }, "newName": name
    });
    rename_or_refusal(h, params).await
}

/// Return a rename result, or the message of its refusal.
pub(super) async fn rename_or_refusal(h: &mut Harness, params: Value) -> Result<Value, String> {
    refusable(h, "textDocument/rename", params).await
}

/// Return a prepared rename, or the message of its refusal.
pub(super) async fn prepare_at(h: &mut Harness, path: &str, line: u32, character: u32) -> Result<Value, String> {
    let params =
        json!({ "textDocument": { "uri": h.url(path) }, "position": { "line": line, "character": character } });
    refusable(h, "textDocument/prepareRename", params).await
}

/// Accept rename refusals without hiding other request errors.
pub(super) async fn refusable(h: &mut Harness, method: &str, params: Value) -> Result<Value, String> {
    h.try_request(method, params).await.map_err(|error| {
        assert_eq!(error["code"], -32803, "{method} failed: {error}");
        error["message"].as_str().unwrap().to_owned()
    })
}

pub(super) const READ_ONLY_STUBS: &str = "Cannot rename: PHP's builtin stubs are read-only.";

pub(super) async fn rename_messages(h: &mut Harness) -> Vec<Value> {
    const MARKER: &str = "rename-feedback";
    h.client.send_notification("mago/testBarrier", json!({ "type": 4, "message": MARKER })).await;
    loop {
        if h.client.await_notification("window/logMessage").await["params"]["message"] == MARKER {
            break;
        }
    }
    h.client
        .take_pending_notifications()
        .into_iter()
        .filter(|message| message["method"] == "window/showMessage")
        .map(|message| message["params"].clone())
        .collect()
}

#[tokio::test]
async fn rename_refusal_fails_once_without_invalidating_the_prepared_symbol() {
    let source = "<?php function read(int $value, int $other = 0): int { return $value + $other; } read(value: 1);";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let position = source.find("$value").unwrap() as u32 + 1;
    assert_eq!(prepare_at(&mut h, "a.php", 0, position).await.unwrap()["placeholder"], "value");
    assert!(prepare_at(&mut h, "a.php", 0, 0).await.unwrap().is_null());
    for (name, reason) in [
        ("other", "$other is already defined in the same scope at a.php:1."),
        ("bad-name", "it is not a valid PHP identifier."),
    ] {
        let params = json!({
            "textDocument": { "uri": h.url("a.php") }, "position": { "line": 0, "character": position }, "newName": name
        });
        assert_eq!(rename_or_refusal(&mut h, params).await, Err(format!("Cannot rename to \"{name}\": {reason}")));
        assert!(rename_messages(&mut h).await.is_empty());
    }
    assert_eq!(std::fs::read_to_string(h.workspace.join("a.php")).unwrap(), source);
    assert_eq!(prepare_at(&mut h, "a.php", 0, position).await.unwrap()["placeholder"], "value");
    let edit = rename_at(&mut h, "a.php", source, "$value", "item").await.unwrap();
    assert_eq!(apply(source, &edit["changes"][h.url("a.php")]), source.replace("value", "item"));
    assert!(rename_messages(&mut h).await.is_empty());
}

#[tokio::test]
async fn unavailable_rename_routes_keep_null_results() {
    let source = "<?php $value = 1;";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for uri in [
        h.url("missing.php"),
        super::harness::url(&h.workspace.parent().unwrap().join("outside.php")),
        "untitled:outside.php".into(),
    ] {
        assert!(
            h.request(
                "textDocument/rename",
                json!({
                    "textDocument": { "uri": uri }, "position": { "line": 0, "character": 7 }, "newName": "item"
                })
            )
            .await
            .is_null(),
        );
    }
    assert!(rename_messages(&mut h).await.is_empty());
    let mut config = crate::language_server::ServerConfig::default();
    config.configuration.language_server.disable.push(crate::config::language_server::Provider::Rename);
    let mut h = Harness::start_with(&[("a.php", source)], config, json!({})).await;
    assert!(
        h.request(
            "textDocument/rename",
            json!({
                "textDocument": { "uri": h.url("a.php") }, "position": { "line": 0, "character": 7 }, "newName": "item"
            })
        )
        .await
        .is_null(),
    );
    assert!(rename_messages(&mut h).await.is_empty());
}

#[tokio::test]
async fn local_references_and_rename_follow_captures_without_crossing_scopes() {
    let source = "<?php\nfunction a() { $item = 1; $f = function() use ($item) { return $item; }; return $item; }\nfunction b($item) { return $item; }\n// $item\n";
    let mut h = Harness::start(&[("a.php", source)]).await;
    assert_eq!(references(&mut h, 1, 16, true).await.as_array().unwrap().len(), 4);
    assert_eq!(references(&mut h, 1, 16, false).await.as_array().unwrap().len(), 3);
    assert!(references(&mut h, 3, 5, true).await.as_array().unwrap().is_empty());
    let renamed = h.request("textDocument/rename", json!({
        "textDocument": { "uri": h.url("a.php") }, "position": { "line": 1, "character": 16 }, "newName": "value"
    })).await;
    let edited = apply(source, &renamed["changes"][h.url("a.php")]);
    let expected =
        source.replacen(source.lines().nth(1).unwrap(), &source.lines().nth(1).unwrap().replace("$item", "$value"), 1);
    assert_eq!(edited, expected);
}

#[tokio::test]
async fn references_keep_symbol_kinds_and_constant_case_separate() {
    let source = "<?php\nclass Same {}\nfunction Same(): int { return 1; }\nconst Same = 1;\nconst SAME = 2;\nnew Same();\nSame();\necho Same;\necho SAME;\n";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for (line, character, declaration_line) in [(5, 5, 1), (6, 1, 2), (7, 6, 3), (8, 6, 4)] {
        let refs = references(&mut h, line, character, true).await;
        let refs = refs.as_array().unwrap();
        assert_eq!(refs.len(), 2, "{refs:?}");
        assert_eq!(refs[0]["range"]["start"]["line"], declaration_line);
        assert_eq!(refs[1]["range"]["start"]["line"], line);
    }
}

#[tokio::test]
async fn rename_preserves_qualified_name_prefixes() {
    let declaration = "<?php namespace App; class OldName {}";
    let caller = "<?php new \\App\\OldName();";
    let mut h = Harness::start(&[("a.php", declaration), ("b.php", caller)]).await;
    let renamed = h.request("textDocument/rename", json!({
        "textDocument": { "uri": h.url("a.php") }, "position": { "line": 0, "character": 29 }, "newName": "NewName"
    })).await;
    assert_eq!(apply(declaration, &renamed["changes"][h.url("a.php")]), declaration.replace("OldName", "NewName"));
    assert_eq!(apply(caller, &renamed["changes"][h.url("b.php")]), caller.replace("OldName", "NewName"));
}

#[tokio::test]
async fn global_rename_preserves_explicit_aliases_including_matching_names() {
    let declaration = "<?php namespace Lib; class Thing {}";
    let source = "<?php namespace App; use Lib\\{Thing as Alias}; new Alias(); namespace Other; use Lib\\Thing as Thing; new Thing(); namespace Third; use Lib\\Thing; new Thing(); new \\Lib\\Thing();";
    let mut h = Harness::start(&[("a.php", declaration), ("b.php", source)]).await;
    let edit = rename_at(&mut h, "a.php", declaration, "Thing", "Renamed").await.unwrap();
    assert_eq!(apply(declaration, &edit["changes"][h.url("a.php")]), declaration.replace("Thing", "Renamed"));
    assert_eq!(
        apply(source, &edit["changes"][h.url("b.php")]),
        "<?php namespace App; use Lib\\{Renamed as Alias}; new Alias(); namespace Other; use Lib\\Renamed as Thing; new Thing(); namespace Third; use Lib\\Renamed; new Renamed(); new \\Lib\\Renamed();"
    );
}

#[tokio::test]
async fn alias_rename_stays_in_one_namespace_block_and_preserves_suffixes() {
    let source = "<?php namespace App { use Lib\\Bar as B; new B\\Thing(); B\\run(); echo B\\FLAG; } namespace App { use Lib\\Bar as B; new B\\Thing(); }";
    let other = "<?php use Lib\\Bar as B; new B\\Thing();";
    let mut h = Harness::start(&[("a.php", source), ("b.php", other)]).await;
    for needle in ["B;", "B\\Thing"] {
        let edit = rename_at(&mut h, "a.php", source, needle, "Longer").await.unwrap();
        assert_eq!(edit["changes"].as_object().unwrap().len(), 1);
        assert_eq!(
            apply(source, &edit["changes"][h.url("a.php")]),
            "<?php namespace App { use Lib\\Bar as Longer; new Longer\\Thing(); Longer\\run(); echo Longer\\FLAG; } namespace App { use Lib\\Bar as B; new B\\Thing(); }"
        );
    }
}

#[tokio::test]
async fn typed_group_aliases_and_implicit_aliases_keep_their_targets() {
    let source = "<?php use Lib\\{Thing, function work as run, const FLAG as VALUE}; new Thing(); run(); echo VALUE;";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for (needle, name, expected) in [
        (
            "Thing();",
            "Renamed",
            "<?php use Lib\\{Thing as Renamed, function work as run, const FLAG as VALUE}; new Renamed(); run(); echo VALUE;",
        ),
        (
            "run,",
            "execute",
            "<?php use Lib\\{Thing, function work as execute, const FLAG as VALUE}; new Thing(); execute(); echo VALUE;",
        ),
        (
            "VALUE}",
            "OTHER",
            "<?php use Lib\\{Thing, function work as run, const FLAG as OTHER}; new Thing(); run(); echo OTHER;",
        ),
    ] {
        let edit = rename_at(&mut h, "a.php", source, needle, name).await.unwrap();
        assert_eq!(apply(source, &edit["changes"][h.url("a.php")]), expected);
    }
}

#[tokio::test]
async fn symbol_rename_after_namespace_alias_preserves_alias_prefix() {
    let declaration = "<?php namespace Lib\\Bar; class Thing {}";
    let source = "<?php use Lib\\Bar as B; new B\\Thing();";
    let mut h = Harness::start(&[("a.php", declaration), ("b.php", source)]).await;
    let edit = rename_at(&mut h, "b.php", source, "Thing", "Renamed").await.unwrap();
    assert_eq!(apply(source, &edit["changes"][h.url("b.php")]), source.replace("Thing", "Renamed"));
    assert_eq!(apply(declaration, &edit["changes"][h.url("a.php")]), declaration.replace("Thing", "Renamed"));
}

#[tokio::test]
async fn rename_rejects_collisions_and_reserved_names_but_accepts_non_ascii() {
    let source = "<?php class First {} class Second {} function f() { $old = 1; $taken = 2; return $old; } use Lib\\Thing as Alias; use Lib\\Other as Taken; new Alias(); // $comment\n";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for (needle, name) in [
        ("First", "Second"),
        ("First", "class"),
        ("First", "int"),
        ("$old", "taken"),
        ("$old", "this"),
        ("Alias;", "Taken"),
        ("Alias;", "bad-name"),
    ] {
        assert!(rename_at(&mut h, "a.php", source, needle, name).await.is_err(), "{needle} -> {name}");
    }
    let edit = rename_at(&mut h, "a.php", source, "$old", "donnée").await.unwrap();
    assert_eq!(apply(source, &edit["changes"][h.url("a.php")]), source.replace("$old", "$donnée"));
    let prepare = h
        .request(
            "textDocument/prepareRename",
            json!({
                "textDocument": { "uri": h.url("a.php") },
                "position": { "line": 0, "character": source.find("$comment").unwrap() + 2 }
            }),
        )
        .await;
    assert!(prepare.is_null());
}

#[tokio::test]
async fn global_rename_rejects_collisions_in_other_files_and_import_scopes() {
    let source = "<?php namespace App; use Other\\Thing as Alias; class First {}";
    let other = "<?php namespace App; class Second {}";
    let mut h = Harness::start(&[("a.php", source), ("b.php", other)]).await;
    for name in ["Second", "Alias"] {
        assert!(rename_at(&mut h, "a.php", source, "First", name).await.is_err());
    }
}

#[tokio::test]
async fn global_rename_follows_function_and_constant_namespace_fallback() {
    let declaration = "<?php function work(): void {} const TOKEN = 1;";
    let caller = "<?php namespace App; WoRk(); echo TOKEN;";
    let other = "<?php namespace Other; function work(): void {} const TOKEN = 2; work(); echo TOKEN;";
    let mut h = Harness::start(&[("a.php", declaration), ("b.php", caller), ("c.php", other)]).await;
    for (old, needle, name) in [("work", "WoRk", "execute"), ("TOKEN", "TOKEN", "VALUE")] {
        for (path, source, selection) in [("a.php", declaration, old), ("b.php", caller, needle)] {
            let edit = rename_at(&mut h, path, source, selection, name).await.unwrap();
            assert_eq!(edit["changes"].as_object().unwrap().len(), 2);
            assert_eq!(apply(declaration, &edit["changes"][h.url("a.php")]), declaration.replace(old, name));
            assert_eq!(apply(caller, &edit["changes"][h.url("b.php")]), caller.replace(needle, name));
        }
    }
}

#[tokio::test]
async fn global_rename_rejects_namespace_symbols_that_would_shadow_fallback_calls() {
    for (declaration, caller, shadow, old, name) in [
        (
            "<?php function work(): void {}",
            "<?php namespace App; work();",
            "<?php namespace App; function execute(): void {}",
            "work",
            "execute",
        ),
        (
            "<?php const TOKEN = 1;",
            "<?php namespace App; echo TOKEN;",
            "<?php namespace App; const VALUE = 2;",
            "TOKEN",
            "VALUE",
        ),
    ] {
        let mut h = Harness::start(&[("a.php", declaration), ("b.php", caller), ("c.php", shadow)]).await;
        for (path, source) in [("a.php", declaration), ("b.php", caller)] {
            assert!(rename_at(&mut h, path, source, old, name).await.is_err(), "{source}");
        }
    }
}

#[tokio::test]
async fn namespace_rename_rejects_capture_of_other_global_fallback_uses() {
    for (declaration, global, caller, old, name) in [
        (
            "<?php namespace App; function work(): void {}",
            "<?php function execute(): void {}",
            "<?php namespace App; execute();",
            "work",
            "execute",
        ),
        (
            "<?php namespace App; const TOKEN = 1;",
            "<?php const VALUE = 2;",
            "<?php namespace App; echo VALUE;",
            "TOKEN",
            "VALUE",
        ),
    ] {
        let mut h = Harness::start(&[("a.php", declaration), ("b.php", global), ("c.php", caller)]).await;
        assert!(rename_at(&mut h, "a.php", declaration, old, name).await.is_err(), "{declaration}");
    }
    let declaration = "<?php namespace App; function work(): void {}";
    let global = "<?php function execute(): void {}";
    let caller = "<?php namespace Other; execute(); \\App\\work();";
    let mut h = Harness::start(&[("a.php", declaration), ("b.php", global), ("c.php", caller)]).await;
    let edit = rename_at(&mut h, "a.php", declaration, "work", "execute").await.unwrap();
    assert_eq!(edit["changes"].as_object().unwrap().len(), 2);
    assert_eq!(apply(declaration, &edit["changes"][h.url("a.php")]), declaration.replace("work", "execute"));
    assert_eq!(apply(caller, &edit["changes"][h.url("c.php")]), caller.replace("work", "execute"));
}

#[tokio::test]
async fn rename_rejects_shared_namespace_prefix_imports_and_prefix_cursors() {
    let declaration = "<?php namespace Lib; class Bar {}";
    let source = "<?php use Lib\\Bar; new Bar(); new Bar\\Thing(); new \\Lib\\Bar();";
    let mut h = Harness::start(&[("a.php", declaration), ("b.php", source)]).await;
    assert!(rename_at(&mut h, "a.php", declaration, "Bar", "Renamed").await.is_err());
    assert!(rename_at(&mut h, "b.php", source, "\\Lib\\Bar", "Renamed").await.is_err());
    let position = source.find("\\Lib\\Bar").unwrap() as u32 + 1;
    let refusal = prepare_at(&mut h, "b.php", 0, position).await.unwrap_err();
    assert!(refusal.contains("namespace part of \\Lib\\Bar"), "{refusal}");
}

#[tokio::test]
async fn alias_references_stay_in_their_namespace_and_select_only_the_prefix() {
    let source = "<?php namespace App { use Lib\\Bar as B; new B\\Thing(); B\\run(); } namespace App { use Lib\\Bar as B; new B\\Thing(); }";
    let other = "<?php namespace Lib\\Bar; class Thing {}";
    let mut h = Harness::start(&[("a.php", source), ("b.php", other)]).await;
    for needle in ["B;", "B\\Thing"] {
        for declaration in [false, true] {
            let refs = references(&mut h, 0, source.find(needle).unwrap() as u32, declaration).await;
            let refs = refs.as_array().unwrap();
            assert_eq!(refs.len(), if declaration { 3 } else { 2 });
            for reference in refs {
                assert_eq!(reference["uri"], h.url("a.php"));
                let start = reference["range"]["start"]["character"].as_u64().unwrap() as usize;
                let end = reference["range"]["end"]["character"].as_u64().unwrap() as usize;
                assert_eq!(&source[start..end], "B");
                assert!(start < source.find("} namespace").unwrap());
            }
        }
    }
    let refs = references(&mut h, 0, source.find("Thing").unwrap() as u32, true).await;
    assert_eq!(refs.as_array().unwrap().len(), 3);
    assert!(refs.as_array().unwrap().iter().any(|reference| reference["uri"] == h.url("b.php")));
}
