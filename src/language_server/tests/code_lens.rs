use serde_json::Value;
use serde_json::json;
use std::time::Duration;
use tower_lsp_server::gen_lsp_types::Position;

use mago_database::file::File;

use super::harness::Harness;
use crate::config::language_server::Provider;
use crate::language_server::ServerConfig;
use crate::language_server::position::position_at_offset;

async fn check(h: &mut Harness, path: &str, source: &str, expected: &[(&str, usize)]) {
    let lenses = h.for_doc("textDocument/codeLens", path).await;
    let lenses = lenses.as_array().unwrap();
    assert_eq!(lenses.len(), expected.len(), "{lenses:?}");
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    for (lens, (needle, count)) in lenses.iter().zip(expected) {
        let position = position_at_offset(&file, source.find(needle).unwrap() as u32);
        assert_eq!(lens["range"], json!({"start":position,"end":position}));
        assert!(lens.get("command").is_none(), "references must stay lazy: {lens}");
        assert_eq!(lens["data"]["uri"], h.url(path));
        let resolved = h.request("codeLens/resolve", lens.clone()).await;
        check_command(h, path, position, &resolved["command"], *count).await;
    }
}

async fn check_command(h: &mut Harness, path: &str, position: Position, command: &Value, count: usize) {
    assert_eq!(command["title"], format!("{count} reference{}", if count == 1 { "" } else { "s" }));
    assert_eq!(command["command"], "editor.action.showReferences");
    let arguments = &command["arguments"];
    assert_eq!(arguments[0], h.url(path));
    assert_eq!(arguments[1], json!(position));
    assert_eq!(arguments[2].as_array().unwrap().len(), count);
    let references = h
        .request(
            "textDocument/references",
            json!({
                "textDocument":{"uri":h.url(path)}, "position":position,
                "context":{"includeDeclaration":false},
            }),
        )
        .await;
    assert_eq!(arguments[2], references);
}

#[tokio::test]
async fn code_lenses_keep_class_function_and_constant_names_separate() {
    let source = "<?php /*😀*/ class Shared {} function Shared(): void {} const Shared = 1; new Shared(); Shared(); Shared(); echo Shared, Shared, Shared;";
    let mut h = Harness::start(&[("a.php", source)]).await;
    assert_eq!(h.server_capabilities["codeLensProvider"]["resolveProvider"], true);
    check(&mut h, "a.php", source, &[("Shared {}", 1), ("Shared():", 2), ("Shared =", 3)]).await;
}

#[tokio::test]
async fn code_lenses_keep_same_names_in_other_files_and_read_untouched_files() {
    let class = "<?php class Shared {}";
    let function = "<?php function Shared(): void {}";
    let constant = "<?php const Shared = 1;";
    let use_site = "<?php new Shared(); Shared(); Shared(); echo Shared, Shared, Shared;";
    let mut h = Harness::start_with(
        &[("a.php", class), ("b.php", function), ("c.php", constant), ("d.php", use_site)],
        ServerConfig { linter: false, ..ServerConfig::default() },
        json!({}),
    )
    .await;
    for (path, source, needle, count) in
        [("a.php", class, "Shared", 1), ("b.php", function, "Shared", 2), ("c.php", constant, "Shared", 3)]
    {
        check(&mut h, path, source, &[(needle, count)]).await;
    }
}

#[tokio::test]
async fn code_lenses_count_fallback_imports_and_shadowing_after_edits() {
    let global = "<?php function work(): void {} const FLAG = 1; const flag = 2; class work {}";
    let caller = "<?php namespace App; WORK(); echo FLAG, flag; new work();";
    let local =
        "<?php namespace Shadow; function work(): void {} const FLAG = 3; work(); echo FLAG; \\work(); echo \\FLAG;";
    let imports = "<?php namespace Imports; use function work as go; use const FLAG as VALUE; go(); echo VALUE;";
    let edited = "<?php namespace App; function work(): void {} const FLAG = 5; WORK(); echo FLAG, flag;";
    for hide in [false, true] {
        let mut config = ServerConfig::default();
        config.configuration.language_server.hide_zero_reference_code_lenses = hide;
        let files = [("a.php", global), ("b.php", caller), ("c.php", local), ("d.php", imports)];
        let mut h = Harness::start_with(&files, config, super::harness::client_capabilities()).await;
        let shown = |expected: &[(&'static str, usize)]| {
            expected.iter().copied().filter(|(_, count)| !hide || *count > 0).collect::<Vec<_>>()
        };
        let before = shown(&[("work():", 4), ("FLAG =", 4), ("flag =", 1), ("work {}", 0)]);
        check(&mut h, "a.php", global, &before).await;
        check(&mut h, "c.php", local, &[("work():", 1), ("FLAG =", 1)]).await;
        h.open("b.php", edited).await;
        check(&mut h, "a.php", global, &shown(&[("work():", 3), ("FLAG =", 3), ("flag =", 1), ("work {}", 0)])).await;
        check(&mut h, "b.php", edited, &shown(&[("work():", 1), ("FLAG =", 1)])).await;
        h.close("b.php").await;
        check(&mut h, "a.php", global, &before).await;
    }
}

#[tokio::test]
async fn code_lenses_hide_zero_references_when_configured() {
    let declarations = "<?php class Used {} class Idle {} function call(): void {} function rest(): void {} const KEPT = 1; const SPARE = 2;";
    let mut config = ServerConfig { linter: false, ..ServerConfig::default() };
    config.configuration.language_server.hide_zero_reference_code_lenses = true;
    let mut h = Harness::start_with(
        &[
            ("a.php", declarations),
            ("b.php", "<?php new Used(); call(); call(); echo KEPT;"),
            ("c.php", "<?php class Lonely {}"),
        ],
        config,
        json!({}),
    )
    .await;
    check(&mut h, "a.php", declarations, &[("Used {}", 1), ("call():", 2), ("KEPT =", 1)]).await;
    h.open("b.php", "<?php new Idle();").await;
    check(&mut h, "a.php", declarations, &[("Idle {}", 1)]).await;
    h.close("b.php").await;
    check(&mut h, "a.php", declarations, &[("Used {}", 1), ("call():", 2), ("KEPT =", 1)]).await;
    assert_eq!(h.for_doc("textDocument/codeLens", "c.php").await, json!([]));
}

#[tokio::test]
async fn code_lenses_hide_zero_references_per_workspace() {
    let source = "<?php class Lonely {}";
    let mut h = Harness::start_multi(&[
        ("shown", &[("a.php", source)]),
        ("hidden", &[("a.php", source), ("mago.toml", "[language-server]\nhide-zero-reference-code-lenses = true\n")]),
    ])
    .await;
    let shown = h.request("textDocument/codeLens", json!({"textDocument":{"uri":h.url_in("shown", "a.php")}})).await;
    let shown = shown.as_array().unwrap();
    assert_eq!(shown.len(), 1);
    assert!(shown[0].get("command").is_none(), "references must stay lazy: {}", shown[0]);
    let resolved = h.request("codeLens/resolve", shown[0].clone()).await;
    assert_eq!(resolved["command"]["title"], "0 references");
    let hidden = h.request("textDocument/codeLens", json!({"textDocument":{"uri":h.url_in("hidden", "a.php")}})).await;
    assert_eq!(hidden, json!([]));
}

#[tokio::test]
async fn code_lenses_resolve_current_references_and_reject_stale_or_invalid_data() {
    let source = "<?php class Target {}";
    let mut h = Harness::start(&[("a.php", source), ("b.php", "<?php new Target();")]).await;
    let lens = h.for_doc("textDocument/codeLens", "a.php").await[0].clone();
    h.open("b.php", "<?php new Target(); new Target();").await;
    let resolved = h.request("codeLens/resolve", lens.clone()).await;
    assert_eq!(resolved["command"]["title"], "2 references");
    assert_eq!(resolved["command"]["arguments"][2].as_array().unwrap().len(), 2);
    let repeated = h.request("codeLens/resolve", resolved).await;
    assert_eq!(repeated["command"]["title"], "2 references");

    for data in [json!({"uri":42}), json!({"uri":h.url("a.php"),"content_hash":"0"})] {
        let mut invalid = lens.clone();
        invalid["data"] = data;
        expect_resolve_error(&mut h, invalid, -32602).await;
    }
    for start in [json!({"line":u32::MAX,"character":0}), json!({"line":0,"character":0})] {
        let mut moved = lens.clone();
        moved["range"] = json!({"start":start,"end":start});
        assert_eq!(h.request("codeLens/resolve", moved).await, repeated);
    }

    let outside = tempfile::NamedTempFile::new().unwrap();
    for uri in ["https://example.com/a.php".to_owned(), super::harness::url(outside.path())] {
        let mut invalid = lens.clone();
        invalid["data"]["uri"] = json!(uri);
        expect_resolve_error(&mut h, invalid, -32803).await;
    }
    h.open("a.php", "<?php class Renamed {}").await;
    expect_resolve_error(&mut h, lens, -32801).await;
}

#[tokio::test]
async fn code_lenses_resolve_against_the_current_declaration_after_edits() {
    let source =
        "<?php\nclass Target {}\nfunction work(): void {}\nconst LIMIT = 1;\nnew Target(); work(); echo LIMIT;";
    let edited = "<?php\n// 😀 moved\n\ninterface Other {}\nclass Target {}\nfunction work(): void { echo 1; }\nconst LIMIT = 2;\nnew Target(); new Target(); work(); echo LIMIT, LIMIT;";
    for hide in [false, true] {
        let mut config = ServerConfig { linter: false, ..ServerConfig::default() };
        config.configuration.language_server.hide_zero_reference_code_lenses = hide;
        let files = [("a.php", source), ("b.php", "<?php new Target(); work();")];
        let mut h = Harness::start_with(&files, config, json!({})).await;
        h.open("a.php", source).await;
        let lenses = h.for_doc("textDocument/codeLens", "a.php").await;
        let lenses = lenses.as_array().unwrap();
        assert_eq!(lenses.len(), 3, "{lenses:?}");
        h.change("a.php", edited, 2).await;
        for (lens, (needle, count)) in lenses.iter().zip([("Target {}", 3), ("work()", 2), ("LIMIT =", 2)]) {
            resolve_at(&mut h, "a.php", lens, position_of(edited, needle, 0), count).await;
        }
    }
}

#[tokio::test]
async fn code_lenses_reject_resolving_renamed_deleted_or_retyped_declarations() {
    let source = "<?php class Target {} function work(): void {} const LIMIT = 1;";
    let mut h = Harness::start(&[("a.php", source)]).await;
    h.open("a.php", source).await;
    let lenses = h.for_doc("textDocument/codeLens", "a.php").await;
    let lenses = lenses.as_array().unwrap();
    assert_eq!(lenses.len(), 3, "{lenses:?}");
    h.change("a.php", "<?php class Renamed {} const work = 1;", 2).await;
    for lens in lenses {
        expect_resolve_error(&mut h, lens.clone(), -32801).await;
    }
    h.change("a.php", source, 3).await;
    for (lens, needle) in lenses.iter().zip(["Target", "work", "LIMIT"]) {
        resolve_at(&mut h, "a.php", lens, position_of(source, needle, 0), 0).await;
    }
}

#[tokio::test]
async fn code_lenses_resolve_duplicate_declarations_to_their_own_position() {
    let source = "<?php\nif (A) { function twice(): void {} }\nif (B) { function twice(): void {} }\nif (C) { function twice(): void {} }\ntwice();\n";
    let gap = source.match_indices("twice()").nth(1).unwrap().0 - source.find("twice()").unwrap();
    let above = |line: String| source.replacen("<?php\n", &format!("<?php\n{line}\n"), 1);
    let shifted = above(format!("// {}", "x".repeat(200)));
    let aligned = above(format!("//{}", "x".repeat(gap - 3)));
    let surrounded = above(format!("//{}", "x".repeat(97))) + &format!("//{}\n", "x".repeat(57));
    let pruned = source.replacen("function twice(): void {} ", "", 1);
    let mut h = Harness::start(&[("a.php", source)]).await;
    h.open("a.php", source).await;
    let lenses = h.for_doc("textDocument/codeLens", "a.php").await;
    let lenses = lenses.as_array().unwrap();
    assert_eq!(lenses.len(), 3, "{lenses:?}");
    for (version, edited) in [(2, &shifted), (3, &aligned), (4, &surrounded)] {
        h.change("a.php", edited, version).await;
        for (nth, lens) in lenses.iter().enumerate() {
            resolve_at(&mut h, "a.php", lens, position_of(edited, "twice()", nth), 1).await;
        }
    }
    h.change("a.php", &pruned, 5).await;
    for (lens, nth) in lenses.iter().zip([0, 0, 1]) {
        resolve_at(&mut h, "a.php", lens, position_of(&pruned, "twice()", nth), 1).await;
    }
}

fn position_of(source: &str, needle: &str, nth: usize) -> Position {
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    position_at_offset(&file, source.match_indices(needle).nth(nth).unwrap().0 as u32)
}

async fn resolve_at(h: &mut Harness, path: &str, lens: &Value, position: Position, count: usize) {
    let resolved = h.request("codeLens/resolve", lens.clone()).await;
    assert_eq!(resolved["range"], json!({"start":position,"end":position}));
    check_command(h, path, position, &resolved["command"], count).await;
}

async fn expect_resolve_error(h: &mut Harness, lens: Value, code: i32) {
    let id = h.client.send_request("codeLens/resolve", lens).await;
    let response = h.client.await_response(id, 10).await;
    assert_eq!(response["error"]["code"], code, "{response}");
}

#[tokio::test]
async fn code_lens_refresh_requires_client_support_and_an_enabled_provider() {
    for (supported, enabled) in [(false, true), (true, false)] {
        let mut config = ServerConfig { linter: false, ..ServerConfig::default() };
        if !enabled {
            config.configuration.language_server.disable.push(Provider::CodeLens);
        }
        let mut h = Harness::start_with(
            &[("a.php", "<?php class Target {}")],
            config,
            json!({"workspace":{"codeLens":{"refreshSupport":supported}}}),
        )
        .await;
        h.open("a.php", "<?php class Changed {}").await;
        h.for_doc("textDocument/documentSymbol", "a.php").await;
        tokio::time::sleep(Duration::from_millis(350)).await;
        h.for_doc("textDocument/documentSymbol", "a.php").await;
        assert!(h.client.server_requests().iter().all(|message| message["method"] != "workspace/codeLens/refresh"));
    }
}

#[tokio::test]
async fn code_lens_refresh_coalesces_changes_and_does_not_block_queries_or_shutdown() {
    let mut h = Harness::start_with(
        &[("a.php", "<?php class Target {}"), ("b.php", "<?php new Target();")],
        ServerConfig { linter: false, ..ServerConfig::default() },
        json!({"workspace":{"codeLens":{"refreshSupport":true}}}),
    )
    .await;
    let lens = h.for_doc("textDocument/codeLens", "a.php").await[0].clone();
    h.open("b.php", "<?php new Target(); new Target();").await;
    let first = next_refresh(&mut h).await;
    // Hold the response while edits, disk events and a close settle.
    for version in 2..=10 {
        h.change("b.php", "<?php new Target(); new Target(); new Target();", version).await;
    }
    h.close("b.php").await;
    h.for_doc("textDocument/documentSymbol", "b.php").await;
    std::fs::write(h.workspace.join("b.php"), "<?php new Target(); new Target(); new Target(); new Target();").unwrap();
    h.watched("b.php", 2).await;
    let resolved = h.request("codeLens/resolve", lens.clone()).await;
    assert_eq!(resolved["command"]["title"], "4 references");
    tokio::time::sleep(Duration::from_millis(350)).await;
    h.for_doc("textDocument/documentSymbol", "a.php").await;
    assert!(h.client.server_requests().iter().all(|message| message["method"] != "workspace/codeLens/refresh"));
    h.client.respond(first, Value::Null).await;
    let second = next_refresh(&mut h).await;
    h.client.respond(second, Value::Null).await;

    h.add_folder("extra", &[("extra.php", "<?php class Extra {}")]).await;
    let added = next_refresh(&mut h).await;
    h.client.respond(added, Value::Null).await;
    h.client
        .send_notification(
            "workspace/didChangeWorkspaceFolders",
            json!({
                "event":{"added":[],"removed":[{"uri":h.url("extra"),"name":"extra"}]},
            }),
        )
        .await;
    let _pending = next_refresh(&mut h).await;
    // Shutdown must cancel an outstanding client request without waiting for its response.
    assert_eq!(h.request("shutdown", Value::Null).await, Value::Null);
}

async fn next_refresh(h: &mut Harness) -> Value {
    loop {
        let message = h.client.read_message(10).await;
        if message["method"] == "workspace/codeLens/refresh" {
            return message["id"].clone();
        }
        if message.get("method").is_some() && message.get("id").is_some() {
            h.client.respond(message["id"].clone(), Value::Null).await;
        }
    }
}
