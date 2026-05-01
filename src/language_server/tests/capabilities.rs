//! Capability smoke tests; one short test per LSP request, exercising
//! the happy path. Built on [`super::harness::Harness`] helpers to keep
//! each case ~10 lines.

use serde_json::Value;
use serde_json::json;

use super::harness::Harness;

const SAMPLE: &str = "<?php
namespace App;

final class Greeter
{
    public function hello(string $name): string
    {
        return 'hi ' . $name;
    }
}

function farewell(string $name): string {
    return 'bye ' . $name;
}
";

#[tokio::test]
async fn folding_range() {
    let mut h = Harness::start(&[("a.php", SAMPLE)]).await;
    let result = h.for_doc("textDocument/foldingRange", "a.php").await;
    assert!(!result.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn hover_class() {
    let mut h = Harness::start(&[("a.php", SAMPLE)]).await;
    let result = h.at("textDocument/hover", "a.php", 3, 13).await;
    let value = result["contents"]["value"].as_str().unwrap_or("");
    assert!(value.contains("class") && value.contains("Greeter"), "got {value:?}");
}

#[tokio::test]
async fn goto_definition() {
    let code = "<?php\nclass Greeter {}\n\n$g = new Greeter();\n";
    let mut h = Harness::start(&[("a.php", code)]).await;
    let result = h.at("textDocument/definition", "a.php", 3, 9).await;
    assert_eq!(result["range"]["start"]["line"], 1);
}

#[tokio::test]
async fn goto_definition_on_use_statement() {
    let lib = "<?php\nnamespace Bar;\nclass G {}\n";
    let consumer = "<?php\nnamespace Foo;\nuse Bar\\G;\n";
    let mut h = Harness::start(&[("lib.php", lib), ("c.php", consumer)]).await;
    h.open("lib.php", lib).await;
    h.open("c.php", consumer).await;

    let result = h.at("textDocument/definition", "c.php", 2, 8).await;
    assert!(result["uri"].as_str().unwrap_or("").ends_with("lib.php"), "got {result:?}");
}

#[tokio::test]
async fn hover_on_use_statement() {
    let lib = "<?php\nnamespace Bar;\nclass G {}\n";
    let consumer = "<?php\nnamespace Foo;\nuse Bar\\G;\n";
    let mut h = Harness::start(&[("lib.php", lib), ("c.php", consumer)]).await;
    h.open("lib.php", lib).await;
    h.open("c.php", consumer).await;

    let result = h.at("textDocument/hover", "c.php", 2, 8).await;
    let value = result["contents"]["value"].as_str().unwrap_or("");
    assert!(value.contains("Bar") && value.contains('G'), "got {value:?}");
}

#[tokio::test]
async fn formatting() {
    let mut h = Harness::start(&[("a.php", "<?php\n$x   =   1   ;  \n")]).await;
    let result = h
        .request(
            "textDocument/formatting",
            json!({
                "textDocument": { "uri": h.url("a.php") },
                "options": { "tabSize": 4, "insertSpaces": true }
            }),
        )
        .await;
    let new_text = result[0]["newText"].as_str().unwrap_or("");
    assert!(new_text.contains("$x = 1;"), "got {new_text:?}");
}

#[tokio::test]
async fn semantic_tokens() {
    let mut h = Harness::start(&[("a.php", SAMPLE)]).await;
    let result = h.for_doc("textDocument/semanticTokens/full", "a.php").await;
    let data = result["data"].as_array().unwrap();
    assert!(!data.is_empty());
    assert_eq!(data.len() % 5, 0);
}

#[tokio::test]
async fn references_cross_file() {
    let mut h =
        Harness::start(&[("lib.php", "<?php\nclass Greeter {}\n"), ("c.php", "<?php\n$g = new Greeter();\n")]).await;
    h.open("lib.php", "<?php\nclass Greeter {}\n").await;
    h.open("c.php", "<?php\n$g = new Greeter();\n").await;
    let result = h
        .request(
            "textDocument/references",
            json!({
                "textDocument": { "uri": h.url("lib.php") },
                "position": { "line": 1, "character": 6 },
                "context": { "includeDeclaration": true },
            }),
        )
        .await;
    let uris: Vec<&str> = result.as_array().unwrap().iter().map(|l| l["uri"].as_str().unwrap_or("")).collect();
    assert!(uris.iter().any(|u| u.ends_with("lib.php")));
    assert!(uris.iter().any(|u| u.ends_with("c.php")));
}

#[tokio::test]
async fn references_follows_use_alias() {
    let lib = "<?php\nnamespace Bar;\nclass G {}\n";
    let consumer = "<?php\nnamespace Foo;\nuse Bar\\G as Qux;\n$x = new Qux();\n";
    let mut h = Harness::start(&[("lib.php", lib), ("c.php", consumer)]).await;
    h.open("lib.php", lib).await;
    h.open("c.php", consumer).await;

    let result = h
        .request(
            "textDocument/references",
            json!({
                "textDocument": { "uri": h.url("c.php") },
                "position": { "line": 3, "character": 10 },
                "context": { "includeDeclaration": true },
            }),
        )
        .await;
    let uris: Vec<&str> = result.as_array().unwrap().iter().map(|l| l["uri"].as_str().unwrap_or("")).collect();
    assert!(uris.iter().any(|u| u.ends_with("lib.php")), "missing declaration in lib.php, got {uris:?}");
    assert!(uris.iter().any(|u| u.ends_with("c.php")), "missing alias usage in c.php, got {uris:?}");
}

#[tokio::test]
async fn workspace_symbol() {
    let mut h = Harness::start(&[("a.php", SAMPLE)]).await;
    let result = h.request("workspace/symbol", json!({ "query": "Greet" })).await;
    let names: Vec<&str> = result.as_array().unwrap().iter().map(|s| s["name"].as_str().unwrap_or("")).collect();
    assert!(names.iter().any(|n| n.ends_with("Greeter")), "got {names:?}");
}

#[tokio::test]
async fn signature_help() {
    let code = "<?php\nfunction add(int $left, int $right): int { return $left + $right; }\n\nadd(1, ";
    let mut h = Harness::start(&[("a.php", code)]).await;
    h.open("a.php", code).await;
    let result = h.at("textDocument/signatureHelp", "a.php", 3, 7).await;
    assert!(result["signatures"][0]["label"].as_str().unwrap_or("").contains("add"));
    assert_eq!(result["activeParameter"], 1);
}

#[tokio::test]
async fn inlay_hints() {
    let code = "<?php\nfunction add(int $left, int $right): int { return $left + $right; }\n\nadd(1, 2);\n";
    let mut h = Harness::start(&[("a.php", code)]).await;
    h.open("a.php", code).await;
    let result = h
        .request(
            "textDocument/inlayHint",
            json!({
                "textDocument": { "uri": h.url("a.php") },
                "range": {
                    "start": { "line": 0, "character": 0 },
                    "end": { "line": 5, "character": 0 }
                }
            }),
        )
        .await;
    let labels: Vec<&str> = result.as_array().unwrap().iter().map(|h| h["label"].as_str().unwrap_or("")).collect();
    assert!(labels.contains(&"left:") && labels.contains(&"right:"), "got {labels:?}");
}

#[tokio::test]
async fn rename() {
    let code = "<?php\nclass Greeter {}\n$g = new Greeter();\n";
    let mut h = Harness::start(&[("a.php", code)]).await;
    h.open("a.php", code).await;
    let result = h
        .request(
            "textDocument/rename",
            json!({
                "textDocument": { "uri": h.url("a.php") },
                "position": { "line": 1, "character": 6 },
                "newName": "Speaker",
            }),
        )
        .await;
    let edits = result["changes"][&h.url("a.php")].as_array().unwrap();
    let texts: Vec<&str> = edits.iter().map(|e| e["newText"].as_str().unwrap_or("")).collect();
    assert!(texts.iter().all(|t| *t == "Speaker") && texts.len() >= 2);
}

#[tokio::test]
async fn document_link() {
    let lib = "<?php\nnamespace App;\nclass Greeter {}\n";
    let consumer = "<?php\nuse App\\Greeter;\n\n$g = new Greeter();\n";
    let mut h = Harness::start(&[("lib.php", lib), ("c.php", consumer)]).await;
    h.open("c.php", consumer).await;
    let result = h.for_doc("textDocument/documentLink", "c.php").await;
    let targets: Vec<&str> = result.as_array().unwrap().iter().filter_map(|l| l["target"].as_str()).collect();
    assert!(targets.iter().any(|t| t.ends_with("lib.php")), "got {targets:?}");
}

#[tokio::test]
async fn code_lens() {
    let mut h = Harness::start(&[("a.php", SAMPLE)]).await;
    h.open("a.php", SAMPLE).await;
    let result = h.for_doc("textDocument/codeLens", "a.php").await;
    let titles: Vec<&str> = result.as_array().unwrap().iter().filter_map(|l| l["command"]["title"].as_str()).collect();
    assert!(titles.iter().any(|t| t.contains("reference")), "got {titles:?}");
}

#[tokio::test]
async fn completion_variables() {
    let code = "<?php\nfunction demo(): void {\n    $alpha = 1;\n    $alphabet = 2;\n    $a\n}\n";
    let mut h = Harness::start(&[("a.php", code)]).await;
    h.open("a.php", code).await;
    let result = h.at("textDocument/completion", "a.php", 4, 6).await;
    let labels: Vec<&str> = result.as_array().unwrap().iter().map(|i| i["label"].as_str().unwrap_or("")).collect();
    assert!(labels.contains(&"$alpha") && labels.contains(&"$alphabet"));
}

#[tokio::test]
async fn completion_classes_bare_prefix() {
    let mut h = Harness::start(&[
        ("lib.php", "<?php\nnamespace App;\nclass Greeter {}\nclass Goodbye {}\n"),
        ("c.php", "<?php\nnamespace App;\n\n$g = new G\n"),
    ])
    .await;
    h.open("c.php", "<?php\nnamespace App;\n\n$g = new G\n").await;
    let result = h.at("textDocument/completion", "c.php", 3, 11).await;
    let labels: Vec<&str> = result.as_array().unwrap().iter().map(|i| i["label"].as_str().unwrap_or("")).collect();
    assert!(labels.contains(&"Greeter") && labels.contains(&"Goodbye"), "got {labels:?}");
}

#[tokio::test]
async fn completion_methods_on_this() {
    let code = "<?php\nclass Greeter {\n    public function hello(string $n): string { return ''; }\n    public function howdy(string $n): string { return ''; }\n\n    public function dispatch(): void {\n        $this->h\n    }\n}\n";
    let mut h = Harness::start(&[("a.php", code)]).await;
    h.open("a.php", code).await;
    let result = h.at("textDocument/completion", "a.php", 6, 16).await;
    let labels: Vec<&str> = result.as_array().unwrap().iter().map(|i| i["label"].as_str().unwrap_or("")).collect();
    assert!(labels.contains(&"hello") && labels.contains(&"howdy"), "got {labels:?}");
}

#[tokio::test]
async fn completion_methods_typed_param() {
    let code = "<?php\nclass Greeter {\n    public function hello(string $n): string { return ''; }\n    public function howdy(string $n): string { return ''; }\n}\n\nfunction dispatch(Greeter $g): void {\n    $g->h\n}\n";
    let mut h = Harness::start(&[("a.php", code)]).await;
    h.open("a.php", code).await;
    let result = h.at("textDocument/completion", "a.php", 7, 9).await;
    let labels: Vec<&str> = result.as_array().unwrap().iter().map(|i| i["label"].as_str().unwrap_or("")).collect();
    assert!(labels.contains(&"hello") && labels.contains(&"howdy"), "got {labels:?}");
}

#[tokio::test]
async fn completion_properties_typed_param() {
    let code = "<?php\nclass Bag { public string $alpha = ''; public int $beta = 0; }\n\nfunction open(Bag $b): void {\n    $b->a\n}\n";
    let mut h = Harness::start(&[("a.php", code)]).await;
    h.open("a.php", code).await;
    let result = h.at("textDocument/completion", "a.php", 4, 9).await;
    let labels: Vec<&str> = result.as_array().unwrap().iter().map(|i| i["label"].as_str().unwrap_or("")).collect();
    assert!(labels.contains(&"alpha"), "got {labels:?}");
}

#[tokio::test]
async fn completion_static_constants() {
    let code = "<?php\nfinal class Status {\n    public const string ACTIVE = 'active';\n    public const string ARCHIVED = 'archived';\n}\n\n$x = Status::A\n";
    let mut h = Harness::start(&[("a.php", code)]).await;
    h.open("a.php", code).await;
    let result = h.at("textDocument/completion", "a.php", 6, 14).await;
    let labels: Vec<&str> = result.as_array().unwrap().iter().map(|i| i["label"].as_str().unwrap_or("")).collect();
    assert!(labels.contains(&"ACTIVE") && labels.contains(&"ARCHIVED"), "got {labels:?}");
}

#[tokio::test]
async fn selection_range() {
    let code = "<?php\nclass A {\n    public function f(): void {\n        echo 1;\n    }\n}\n";
    let mut h = Harness::start(&[("a.php", code)]).await;
    h.open("a.php", code).await;
    let result = h
        .request(
            "textDocument/selectionRange",
            json!({
                "textDocument": { "uri": h.url("a.php") },
                "positions": [{ "line": 3, "character": 13 }]
            }),
        )
        .await;
    let mut depth = 0;
    let mut node: &Value = &result[0];
    while !node["parent"].is_null() && depth < 16 {
        depth += 1;
        node = &node["parent"];
    }
    assert!(depth >= 2);
}

#[tokio::test]
async fn completion_after_lone_dollar_offers_local_variables() {
    let code = "<?php\n\nfunction demo(): void {\n    $alpha = 1;\n    $beta = 2;\n    $\n}\n";
    let mut h = Harness::start(&[("a.php", code)]).await;
    h.open("a.php", code).await;
    let result = h.at("textDocument/completion", "a.php", 5, 5).await;
    let labels: Vec<String> =
        result.as_array().unwrap().iter().map(|i| i["label"].as_str().unwrap_or("").to_string()).collect();
    assert!(labels.iter().any(|l| l == "$alpha"), "expected $alpha in {labels:?}");
    assert!(labels.iter().any(|l| l == "$beta"), "expected $beta in {labels:?}");
}

#[tokio::test]
async fn completion_after_arrow_offers_instance_members() {
    let code = "<?php\n\nclass Greeter {\n    public string $name = '';\n    public function hello(): string { return ''; }\n}\n\nfunction demo(Greeter $activity): void {\n    $activity->\n}\n";
    let mut h = Harness::start(&[("a.php", code)]).await;
    h.open("a.php", code).await;
    let result = h.at("textDocument/completion", "a.php", 8, 15).await;
    let labels: Vec<String> =
        result.as_array().unwrap().iter().map(|i| i["label"].as_str().unwrap_or("").to_string()).collect();
    assert!(labels.iter().any(|l| l == "name"), "expected `name` property in {labels:?}");
    assert!(labels.iter().any(|l| l == "hello"), "expected `hello` method in {labels:?}");
    assert!(!labels.iter().any(|l| l.starts_with('$')), "did not expect variables in {labels:?}");
}

#[tokio::test]
async fn linter_diagnostics() {
    let mut h = Harness::start(&[("a.php", "<?php\nfunction check(mixed $x): bool { return $x === null; }\n")]).await;
    let mut diagnostics = h
        .client
        .take_pending_notifications()
        .into_iter()
        .filter(|m| {
            m.get("method").and_then(Value::as_str) == Some("textDocument/publishDiagnostics")
                && m["params"]["uri"].as_str() == Some(h.url("a.php").as_str())
        })
        .flat_map(|m| m["params"]["diagnostics"].as_array().cloned().unwrap_or_default())
        .collect::<Vec<_>>();
    if diagnostics.is_empty() {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while diagnostics.is_empty() && std::time::Instant::now() < deadline {
            let msg = h.client.read_message(10).await;
            if msg.get("method").and_then(Value::as_str) == Some("textDocument/publishDiagnostics")
                && msg["params"]["uri"].as_str() == Some(h.url("a.php").as_str())
                && let Some(arr) = msg["params"]["diagnostics"].as_array()
            {
                diagnostics = arr.clone();
            }
        }
    }
    assert!(!diagnostics.is_empty());
    assert!(diagnostics.iter().all(|d| d["source"] == "mago"));
}

#[tokio::test]
async fn linter_quickfixes() {
    let code = "<?php\nfunction check(mixed $x): bool { return $x === null; }\n";
    let mut h = Harness::start(&[("a.php", code)]).await;
    h.open("a.php", code).await;
    let result = h
        .request(
            "textDocument/codeAction",
            json!({
                "textDocument": { "uri": h.url("a.php") },
                "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 4, "character": 0 } },
                "context": { "diagnostics": [] }
            }),
        )
        .await;
    if let Some(arr) = result.as_array() {
        for action in arr {
            assert!(action["kind"].as_str().unwrap_or("").contains("quickfix"));
        }
    }
}
