//! Capability smoke tests; one short test per LSP request, exercising
//! the happy path. Built on [`super::harness::Harness`] helpers to keep
//! each case ~10 lines.

use serde_json::Value;
use serde_json::json;

use super::harness::Harness;

/// Completion responses serialize either as a bare array or as a
/// `CompletionList` object; normalize both to the item array.
fn completion_array(result: &Value) -> Value {
    if result.is_array() {
        result.clone()
    } else {
        result.get("items").cloned().unwrap_or_else(|| Value::Array(Vec::new()))
    }
}

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
    let r = completion_array(&result);
    let labels: Vec<&str> = r.as_array().unwrap().iter().map(|i| i["label"].as_str().unwrap_or("")).collect();
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
    let r = completion_array(&result);
    let labels: Vec<&str> = r.as_array().unwrap().iter().map(|i| i["label"].as_str().unwrap_or("")).collect();
    assert!(labels.contains(&"Greeter") && labels.contains(&"Goodbye"), "got {labels:?}");
}

#[tokio::test]
async fn completion_methods_on_this() {
    let code = "<?php\nclass Greeter {\n    public function hello(string $n): string { return ''; }\n    public function howdy(string $n): string { return ''; }\n\n    public function dispatch(): void {\n        $this->h\n    }\n}\n";
    let mut h = Harness::start(&[("a.php", code)]).await;
    h.open("a.php", code).await;
    let result = h.at("textDocument/completion", "a.php", 6, 16).await;
    let r = completion_array(&result);
    let labels: Vec<&str> = r.as_array().unwrap().iter().map(|i| i["label"].as_str().unwrap_or("")).collect();
    assert!(labels.contains(&"hello") && labels.contains(&"howdy"), "got {labels:?}");
}

#[tokio::test]
async fn completion_methods_typed_param() {
    let code = "<?php\nclass Greeter {\n    public function hello(string $n): string { return ''; }\n    public function howdy(string $n): string { return ''; }\n}\n\nfunction dispatch(Greeter $g): void {\n    $g->h\n}\n";
    let mut h = Harness::start(&[("a.php", code)]).await;
    h.open("a.php", code).await;
    let result = h.at("textDocument/completion", "a.php", 7, 9).await;
    let r = completion_array(&result);
    let labels: Vec<&str> = r.as_array().unwrap().iter().map(|i| i["label"].as_str().unwrap_or("")).collect();
    assert!(labels.contains(&"hello") && labels.contains(&"howdy"), "got {labels:?}");
}

#[tokio::test]
async fn completion_properties_typed_param() {
    let code = "<?php\nclass Bag { public string $alpha = ''; public int $beta = 0; }\n\nfunction open(Bag $b): void {\n    $b->a\n}\n";
    let mut h = Harness::start(&[("a.php", code)]).await;
    h.open("a.php", code).await;
    let result = h.at("textDocument/completion", "a.php", 4, 9).await;
    let r = completion_array(&result);
    let labels: Vec<&str> = r.as_array().unwrap().iter().map(|i| i["label"].as_str().unwrap_or("")).collect();
    assert!(labels.contains(&"alpha"), "got {labels:?}");
}

#[tokio::test]
async fn completion_static_constants() {
    let code = "<?php\nfinal class Status {\n    public const string ACTIVE = 'active';\n    public const string ARCHIVED = 'archived';\n}\n\n$x = Status::A\n";
    let mut h = Harness::start(&[("a.php", code)]).await;
    h.open("a.php", code).await;
    let result = h.at("textDocument/completion", "a.php", 6, 14).await;
    let r = completion_array(&result);
    let labels: Vec<&str> = r.as_array().unwrap().iter().map(|i| i["label"].as_str().unwrap_or("")).collect();
    assert!(labels.contains(&"ACTIVE") && labels.contains(&"ARCHIVED"), "got {labels:?}");
}

#[tokio::test]
async fn completion_static_offers_static_members_only() {
    let code = "<?php\nclass Box {\n    public static function make(): void {}\n    public function open(): void {}\n    public const string TAG = 'x';\n}\n\nBox::\n";
    let mut h = Harness::start(&[("a.php", code)]).await;
    h.open("a.php", code).await;
    let result = h.at("textDocument/completion", "a.php", 7, 5).await;
    let r = completion_array(&result);
    let labels: Vec<&str> = r.as_array().unwrap().iter().map(|i| i["label"].as_str().unwrap_or("")).collect();
    assert!(labels.contains(&"make") && labels.contains(&"TAG"), "expected static members, got {labels:?}");
    assert!(!labels.contains(&"open"), "instance method must not appear after `::`, got {labels:?}");
}

#[tokio::test]
async fn completion_instance_offers_instance_members_only() {
    let code = "<?php\nclass Box {\n    public static function make(): void {}\n    public function open(): void {}\n}\n\nfunction run(Box $b): void {\n    $b->\n}\n";
    let mut h = Harness::start(&[("a.php", code)]).await;
    h.open("a.php", code).await;
    let result = h.at("textDocument/completion", "a.php", 7, 8).await;
    let r = completion_array(&result);
    let labels: Vec<&str> = r.as_array().unwrap().iter().map(|i| i["label"].as_str().unwrap_or("")).collect();
    assert!(labels.contains(&"open"), "expected instance method, got {labels:?}");
    assert!(!labels.contains(&"make"), "static method must not appear after `->`, got {labels:?}");
}

#[tokio::test]
async fn completion_does_not_offer_anonymous_classes() {
    let code = "<?php\nclass Real {}\n$x = new class {};\n$y = new R\n";
    let mut h = Harness::start(&[("a.php", code)]).await;
    h.open("a.php", code).await;
    let result = h.at("textDocument/completion", "a.php", 3, 10).await;
    let r = completion_array(&result);
    let labels: Vec<&str> = r.as_array().unwrap().iter().map(|i| i["label"].as_str().unwrap_or("")).collect();
    assert!(labels.contains(&"Real"), "expected the named class, got {labels:?}");
    assert!(!labels.iter().any(|l| l.starts_with('{')), "anonymous classes must not appear, got {labels:?}");
}

#[tokio::test]
async fn completion_variables_skip_the_partial_being_typed() {
    let code = "<?php\nfunction demo(): void {\n    $table = 1;\n    $t\n}\n";
    let mut h = Harness::start(&[("a.php", code)]).await;
    h.open("a.php", code).await;
    let result = h.at("textDocument/completion", "a.php", 3, 6).await;
    let r = completion_array(&result);
    let labels: Vec<&str> = r.as_array().unwrap().iter().map(|i| i["label"].as_str().unwrap_or("")).collect();
    assert!(labels.contains(&"$table"), "expected the in-scope variable, got {labels:?}");
    assert!(!labels.contains(&"$t"), "the partial being typed must not be offered, got {labels:?}");
}

#[tokio::test]
async fn completion_variable_edit_preserves_the_dollar_sign() {
    let code = "<?php\nfunction demo(): void {\n    $table = 1;\n    $t\n}\n";
    let mut h = Harness::start(&[("a.php", code)]).await;
    h.open("a.php", code).await;
    let result = h.at("textDocument/completion", "a.php", 3, 6).await;
    let r = completion_array(&result);
    let item = r
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["label"].as_str() == Some("$table"))
        .expect("expected $table completion");
    // The edit rewrites the whole `$table`, including the `$`, over the typed range.
    assert_eq!(item["textEdit"]["newText"].as_str(), Some("$table"), "got {item:?}");
    assert_eq!(item["textEdit"]["range"]["start"]["character"].as_u64(), Some(4), "should replace from the `$`");
    assert_eq!(item["textEdit"]["range"]["end"]["character"].as_u64(), Some(6));
}

#[tokio::test]
async fn completion_qualified_includes_sub_namespace_classes() {
    let lib = "<?php\nnamespace Foo\\Bar;\nclass Qux {}\n";
    let consumer = "<?php\n\\Foo\\\n";
    let mut h = Harness::start(&[("lib.php", lib), ("c.php", consumer)]).await;
    h.open("lib.php", lib).await;
    h.open("c.php", consumer).await;
    let result = h.at("textDocument/completion", "c.php", 1, 5).await;
    let r = completion_array(&result);
    let labels: Vec<&str> = r.as_array().unwrap().iter().map(|i| i["label"].as_str().unwrap_or("")).collect();
    assert!(labels.contains(&"Bar\\Qux"), "expected the sub-namespace class, got {labels:?}");
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
    let r = completion_array(&result);
    let labels: Vec<String> =
        r.as_array().unwrap().iter().map(|i| i["label"].as_str().unwrap_or("").to_string()).collect();
    assert!(labels.iter().any(|l| l == "$alpha"), "expected $alpha in {labels:?}");
    assert!(labels.iter().any(|l| l == "$beta"), "expected $beta in {labels:?}");
}

#[tokio::test]
async fn completion_after_arrow_offers_instance_members() {
    let code = "<?php\n\nclass Greeter {\n    public string $name = '';\n    public function hello(): string { return ''; }\n}\n\nfunction demo(Greeter $activity): void {\n    $activity->\n}\n";
    let mut h = Harness::start(&[("a.php", code)]).await;
    h.open("a.php", code).await;
    let result = h.at("textDocument/completion", "a.php", 8, 15).await;
    let r = completion_array(&result);
    let labels: Vec<String> =
        r.as_array().unwrap().iter().map(|i| i["label"].as_str().unwrap_or("").to_string()).collect();
    assert!(labels.iter().any(|l| l == "name"), "expected `name` property in {labels:?}");
    assert!(labels.iter().any(|l| l == "hello"), "expected `hello` method in {labels:?}");
    assert!(!labels.iter().any(|l| l.starts_with('$')), "did not expect variables in {labels:?}");
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

#[tokio::test]
async fn completion_ranks_acronym_above_substring() {
    let lib = "<?php\nnamespace App;\nclass GetAllTransactionsQueryHandler {}\nclass Gauge {}\n";
    let consumer = "<?php\nnamespace App;\n\n$x = new GATQH\n";
    let mut h = Harness::start(&[("lib.php", lib), ("c.php", consumer)]).await;
    h.open("c.php", consumer).await;
    let result = h.at("textDocument/completion", "c.php", 3, 14).await;
    let r = completion_array(&result);
    let labels: Vec<&str> = r.as_array().unwrap().iter().map(|i| i["label"].as_str().unwrap_or("")).collect();
    assert_eq!(labels.first().copied(), Some("GetAllTransactionsQueryHandler"), "got {labels:?}");
}

#[tokio::test]
async fn completion_after_new_offers_classes_not_functions() {
    let lib = "<?php\nnamespace App;\nclass Allocator {}\nfunction all_things(): void {}\n";
    let consumer = "<?php\nnamespace App;\n\n$x = new All\n";
    let mut h = Harness::start(&[("lib.php", lib), ("c.php", consumer)]).await;
    h.open("c.php", consumer).await;
    let result = h.at("textDocument/completion", "c.php", 3, 12).await;
    let r = completion_array(&result);
    let labels: Vec<&str> = r.as_array().unwrap().iter().map(|i| i["label"].as_str().unwrap_or("")).collect();
    assert!(labels.contains(&"Allocator"), "expected the class, got {labels:?}");
    assert!(!labels.contains(&"all_things"), "functions must not appear after `new`, got {labels:?}");
}

#[tokio::test]
async fn completion_inserts_fqcn_for_out_of_namespace_class() {
    let lib = "<?php\nnamespace App\\Models;\nclass User {}\n";
    let consumer = "<?php\nnamespace App;\n\n$x = new User\n";
    let mut h = Harness::start(&[("lib.php", lib), ("c.php", consumer)]).await;
    h.open("c.php", consumer).await;
    let result = h.at("textDocument/completion", "c.php", 3, 13).await;
    let r = completion_array(&result);
    let item =
        r.as_array().unwrap().iter().find(|i| i["label"].as_str() == Some("User")).expect("expected User completion");
    assert_eq!(item["insertText"].as_str(), Some("\\App\\Models\\User"), "got {item:?}");
}

#[tokio::test]
async fn completion_static_member_substring_matches() {
    let code = "<?php\nenum InvoiceStatus {\n    case Draft;\n    case Finalized;\n    case Uncollectible;\n}\n\n$x = InvoiceStatus::a\n";
    let mut h = Harness::start(&[("a.php", code)]).await;
    h.open("a.php", code).await;
    let result = h.at("textDocument/completion", "a.php", 7, 20).await;
    let r = completion_array(&result);
    let labels: Vec<&str> = r.as_array().unwrap().iter().map(|i| i["label"].as_str().unwrap_or("")).collect();
    assert!(labels.contains(&"Draft"), "expected Draft (contains a), got {labels:?}");
    assert!(labels.contains(&"Finalized"), "expected Finalized (contains a), got {labels:?}");
    assert!(labels.contains(&"cases"), "expected cases() method (contains a), got {labels:?}");
}

#[tokio::test]
async fn completion_imported_class_inserts_short_name() {
    let lib = "<?php\nnamespace App\\Enum;\nenum InvoiceStatus {\n    case Draft;\n}\n";
    let consumer = "<?php\nnamespace App\\Service;\nuse App\\Enum\\InvoiceStatus;\n\nfunction demo(): void {\n    $x = InvoiceS\n}\n";
    let mut h = Harness::start(&[("lib.php", lib), ("c.php", consumer)]).await;
    h.open("lib.php", lib).await;
    h.open("c.php", consumer).await;
    let result = h.at("textDocument/completion", "c.php", 5, 17).await;
    let r = completion_array(&result);
    let item = r
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["label"].as_str() == Some("InvoiceStatus"))
        .expect("expected InvoiceStatus completion");
    assert!(item["insertText"].is_null(), "imported class should insert its short name, got {item:?}");
}

#[tokio::test]
async fn completion_classifies_identifier_touching_closing_paren() {
    let lib = "<?php\nnamespace App;\nclass InvoiceStatus {}\n";
    let consumer = "<?php\nnamespace App;\n\nfunction demo(): void {\n    find(InvoiceS)\n}\n";
    let mut h = Harness::start(&[("lib.php", lib), ("c.php", consumer)]).await;
    h.open("lib.php", lib).await;
    h.open("c.php", consumer).await;
    let result = h.at("textDocument/completion", "c.php", 4, 17).await;
    let r = completion_array(&result);
    let labels: Vec<&str> = r.as_array().unwrap().iter().map(|i| i["label"].as_str().unwrap_or("")).collect();
    assert!(labels.contains(&"InvoiceStatus"), "identifier touching `)` should still complete, got {labels:?}");
}

#[tokio::test]
async fn completion_ranks_local_namespace_before_distant() {
    let near = "<?php\nnamespace App\\Service;\nclass Invoicer {}\n";
    let far = "<?php\nnamespace Other\\Deep\\Place;\nclass Invoicer {}\n";
    let consumer = "<?php\nnamespace App\\Service;\n\nfunction demo(): void {\n    $x = Invoice\n}\n";
    let mut h = Harness::start(&[("near.php", near), ("far.php", far), ("c.php", consumer)]).await;
    h.open("near.php", near).await;
    h.open("far.php", far).await;
    h.open("c.php", consumer).await;
    let result = h.at("textDocument/completion", "c.php", 4, 15).await;
    let r = completion_array(&result);
    let items = r.as_array().unwrap();
    let first = items
        .iter()
        .find(|i| i["label"].as_str() == Some("Invoicer"))
        .and_then(|i| i["documentation"]["value"].as_str())
        .expect("expected an Invoicer completion");
    assert_eq!(first, "App\\Service\\Invoicer", "same-namespace Invoicer should rank first, got {first:?}");
}

#[tokio::test]
async fn completion_static_member_on_imported_short_name() {
    let lib =
        "<?php\nnamespace App\\Enum;\nenum InvoiceStatus {\n    case Draft;\n    case Paid;\n    case Finalized;\n}\n";
    let consumer = "<?php\nnamespace App\\Service;\nuse App\\Enum\\InvoiceStatus;\n\nfunction demo(): void {\n    $x = InvoiceStatus::Pa\n}\n";
    let mut h = Harness::start(&[("lib.php", lib), ("c.php", consumer)]).await;
    h.open("lib.php", lib).await;
    h.open("c.php", consumer).await;
    let result = h.at("textDocument/completion", "c.php", 5, 26).await;
    let r = completion_array(&result);
    let labels: Vec<&str> = r.as_array().unwrap().iter().map(|i| i["label"].as_str().unwrap_or("")).collect();
    assert!(labels.contains(&"Paid"), "imported short name should resolve enum cases, got {labels:?}");
    assert!(!labels.is_empty(), "imported short name receiver must resolve, got {labels:?}");
}

#[tokio::test]
async fn multi_workspace_symbol_spans_every_folder() {
    let mut h = Harness::start_multi(&[
        ("alpha", &[("a.php", "<?php\nnamespace Alpha;\nclass AlphaService {}\n")]),
        ("beta", &[("b.php", "<?php\nnamespace Beta;\nclass BetaService {}\n")]),
    ])
    .await;
    let result = h.request("workspace/symbol", json!({ "query": "Service" })).await;
    let names: Vec<&str> = result.as_array().unwrap().iter().map(|s| s["name"].as_str().unwrap_or("")).collect();
    assert!(names.iter().any(|n| n.ends_with("AlphaService")), "missing symbol from first folder, got {names:?}");
    assert!(names.iter().any(|n| n.ends_with("BetaService")), "missing symbol from second folder, got {names:?}");
}

#[tokio::test]
async fn multi_workspace_hover_routes_to_owning_folder() {
    let mut h = Harness::start_multi(&[
        ("alpha", &[("a.php", "<?php\nnamespace Alpha;\nfinal class AlphaThing {}\n")]),
        ("beta", &[("b.php", "<?php\nnamespace Beta;\nfinal class BetaThing {}\n")]),
    ])
    .await;

    let alpha = h.at_uri("textDocument/hover", &h.url_in("alpha", "a.php"), 2, 13).await;
    let alpha_value = alpha["contents"]["value"].as_str().unwrap_or("");
    assert!(alpha_value.contains("AlphaThing"), "hover in first folder failed, got {alpha_value:?}");

    let beta = h.at_uri("textDocument/hover", &h.url_in("beta", "b.php"), 2, 13).await;
    let beta_value = beta["contents"]["value"].as_str().unwrap_or("");
    assert!(beta_value.contains("BetaThing"), "hover in second folder failed, got {beta_value:?}");
}

#[tokio::test]
async fn multi_workspace_applies_per_folder_config() {
    // The `tabs` folder ships its own mago.toml turning on tab indentation;
    // the `spaces` folder has none and inherits the (spaces) default. Each
    // workspace must format according to its own discovered config.
    let messy = "<?php\n\nfunction demo(): void {\necho 1;\n}\n";
    let mut h = Harness::start_multi(&[
        ("tabs", &[("a.php", messy), ("mago.toml", "[formatter]\nuse-tabs = true\n")]),
        ("spaces", &[("a.php", messy)]),
    ])
    .await;

    let tabs_uri = h.url_in("tabs", "a.php");
    let tabs = h
        .request(
            "textDocument/formatting",
            json!({ "textDocument": { "uri": tabs_uri }, "options": { "tabSize": 4, "insertSpaces": true } }),
        )
        .await;
    let tabs_text = tabs[0]["newText"].as_str().unwrap_or("");
    assert!(tabs_text.contains('\t'), "the `tabs` workspace config should format with tabs, got {tabs_text:?}");

    let spaces_uri = h.url_in("spaces", "a.php");
    let spaces = h
        .request(
            "textDocument/formatting",
            json!({ "textDocument": { "uri": spaces_uri }, "options": { "tabSize": 4, "insertSpaces": true } }),
        )
        .await;
    let spaces_text = spaces[0]["newText"].as_str().unwrap_or("");
    assert!(!spaces_text.contains('\t'), "the `spaces` workspace should not use tabs, got {spaces_text:?}");
}

#[tokio::test]
async fn multi_workspace_handles_dynamic_folder_add() {
    let mut h =
        Harness::start_multi(&[("alpha", &[("a.php", "<?php\nnamespace Alpha;\nclass AlphaThing {}\n")])]).await;

    let before = h.request("workspace/symbol", json!({ "query": "BetaThing" })).await;
    let before_names: Vec<&str> = before.as_array().unwrap().iter().map(|s| s["name"].as_str().unwrap_or("")).collect();
    assert!(!before_names.iter().any(|n| n.ends_with("BetaThing")), "beta should not exist yet, got {before_names:?}");

    h.add_folder("beta", &[("b.php", "<?php\nnamespace Beta;\nclass BetaThing {}\n")]).await;

    let after = h.request("workspace/symbol", json!({ "query": "BetaThing" })).await;
    let after_names: Vec<&str> = after.as_array().unwrap().iter().map(|s| s["name"].as_str().unwrap_or("")).collect();
    assert!(
        after_names.iter().any(|n| n.ends_with("BetaThing")),
        "beta should be added dynamically, got {after_names:?}"
    );
}
