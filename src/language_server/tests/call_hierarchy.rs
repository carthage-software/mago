use serde_json::Value;
use serde_json::json;
use tower_lsp_server::gen_lsp_types::CallHierarchyItem;
use tower_lsp_server::gen_lsp_types::Range;

use mago_database::file::File;

use super::harness::Harness;
use super::harness::url;
use crate::config::language_server::Provider;
use crate::language_server::ServerConfig;
use crate::language_server::capabilities::server_capabilities;
use crate::language_server::position::offset_at_position;
use crate::language_server::position::position_at_offset;

async fn prepare(h: &mut Harness, name: &str, source: &str, needle: &str) -> Value {
    let file = File::ephemeral(name.as_bytes().to_vec().into(), source.as_bytes().to_vec().into());
    let position = position_at_offset(&file, source.find(needle).unwrap() as u32);
    h.at("textDocument/prepareCallHierarchy", name, position.line, position.character).await
}

fn names(calls: &Value, direction: &str) -> Vec<String> {
    let mut names: Vec<_> =
        calls.as_array().unwrap().iter().map(|call| call[direction]["name"].as_str().unwrap().to_owned()).collect();
    names.sort();
    names
}

fn text_at<'a>(source: &'a str, range: &Value) -> &'a str {
    let range: Range = serde_json::from_value(range.clone()).unwrap();
    let file = File::ephemeral(b"test.php".as_slice().into(), source.as_bytes().to_vec().into());
    &source[offset_at_position(&file, range.start) as usize..offset_at_position(&file, range.end) as usize]
}

#[tokio::test]
async fn functions_methods_unions_and_constructors_keep_exact_calls() {
    let library = "<?php\nnamespace Lib; function work(): void {} class Base { function __construct() {} function run(): void { work(); } } class Child extends Base {} class Other { function run(): void {} }";
    let source = "<?php\nnamespace App; use function Lib\\work as go; use Lib\\{Child, Other}; /*😀*/ function start(Child|Other $worker): void { go(); go(); $worker->run(); $f = go(...); new Child(); }";
    let mut h = Harness::start(&[("a.php", source), ("b.php", library)]).await;
    let start = prepare(&mut h, "a.php", source, "start(").await[0].clone();
    let item: CallHierarchyItem = serde_json::from_value(start.clone()).unwrap();
    assert_eq!(text_at(source, &start["selectionRange"]), "start");
    assert!(item.range.start <= item.selection_range.start && item.selection_range.end <= item.range.end);
    let outgoing = h.request("callHierarchy/outgoingCalls", json!({"item": start})).await;
    assert_eq!(names(&outgoing, "to"), ["Lib\\Base::__construct", "Lib\\Base::run", "Lib\\Other::run", "Lib\\work"]);
    let work = outgoing.as_array().unwrap().iter().find(|call| call["to"]["name"] == "Lib\\work").unwrap();
    assert_eq!(work["fromRanges"].as_array().unwrap().len(), 2);
    assert!(work["fromRanges"].as_array().unwrap().iter().all(|range| text_at(source, range) == "go"));
    let targets = prepare(&mut h, "a.php", source, "run();").await;
    assert_eq!(targets.as_array().unwrap().len(), 2);
    let incoming = h.request("callHierarchy/incomingCalls", json!({"item": work["to"]})).await;
    assert_eq!(names(&incoming, "from"), ["App\\start", "Lib\\Base::run"]);
    assert_eq!(incoming, h.request("callHierarchy/incomingCalls", json!({"item": work["to"]})).await);
}

#[tokio::test]
async fn callable_invocations_keep_exact_calls_without_counting_creation() {
    let source = "<?php
function read(int $value, int $other = 0): int { return $value + $other; }
function caller(): void {
    $partial = read(value: ?, other: 2);
    $partial(value: 3);
    $callback = read(...);
    $callback(value: 1);
    (read(...))(value: 1);
    $callbacks = [read(...)];
    $callbacks[0](value: 1);
    (read(value: ?, other: 2))(value: 1);
    'read'(value: 1);
}";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let target = prepare(&mut h, "a.php", source, "read(").await[0].clone();
    let incoming = h.request("callHierarchy/incomingCalls", json!({"item": target})).await;
    assert_eq!(names(&incoming, "from"), ["caller"]);
    let expected = ["$partial", "$callback", "(read(...))", "$callbacks[0]", "(read(value: ?, other: 2))", "'read'"];
    let ranges = incoming[0]["fromRanges"].as_array().unwrap();
    assert_eq!(ranges.iter().map(|range| text_at(source, range)).collect::<Vec<_>>(), expected);
    let outgoing = h.request("callHierarchy/outgoingCalls", json!({"item": incoming[0]["from"]})).await;
    assert_eq!(names(&outgoing, "to"), ["read"]);
    assert_eq!(outgoing[0]["fromRanges"], incoming[0]["fromRanges"]);
    for call in expected {
        let needle = format!("{call}(value:");
        let items = prepare(&mut h, "a.php", source, &needle).await;
        assert_eq!(items.as_array().unwrap().len(), 1, "{call}: {items}");
        assert_eq!(items[0]["name"], "read", "{call}: {items}");
    }
}

#[tokio::test]
async fn namespaced_callable_hierarchy_survives_removing_the_direct_call() {
    let source = "<?php
namespace MagoEditorQa;
function read(int $value, int $other = 0): int { return $value + $other; }
function useRecord(): void {
    read(value: 1);
    $partial = read(value: ?, other: 2);
    $partial(value: 3);
    $callback = read(...);
    $callback(value: 1);
    (read(...))(value: 1);
    $callbacks = [read(...)];
    $callbacks[0](value: 1);
}";
    let without_direct = source.replace("    read(value: 1);\n", "");
    let mut h = Harness::start(&[("a.php", source)]).await;
    h.open("a.php", source).await;
    for (version, current) in [(2, source), (3, without_direct.as_str()), (4, source)] {
        h.change("a.php", current, version).await;
        let target = prepare(&mut h, "a.php", current, "read(").await[0].clone();
        let incoming = h.request("callHierarchy/incomingCalls", json!({"item": target})).await;
        assert_eq!(names(&incoming, "from"), ["MagoEditorQa\\useRecord"], "version {version}: {incoming}");
        let mut expected = vec!["$partial", "$callback", "(read(...))", "$callbacks[0]"];
        if current == source {
            expected.insert(0, "read");
        }
        assert_eq!(
            incoming[0]["fromRanges"]
                .as_array()
                .unwrap()
                .iter()
                .map(|range| text_at(current, range))
                .collect::<Vec<_>>(),
            expected,
            "version {version}"
        );
        let caller = prepare(&mut h, "a.php", current, "useRecord(").await[0].clone();
        let outgoing = h.request("callHierarchy/outgoingCalls", json!({"item": caller})).await;
        assert_eq!(names(&outgoing, "to"), ["MagoEditorQa\\read"], "version {version}: {outgoing}");
        assert_eq!(outgoing[0]["fromRanges"], incoming[0]["fromRanges"]);
    }
}

#[tokio::test]
async fn callable_returning_calls_keep_factory_and_returned_target_separate() {
    let source = "<?php
function read(int $value): int { return $value; }
function caller(): void { $factory = fn() => read(...); $factory()(value: 1); }";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let read = prepare(&mut h, "a.php", source, "read(").await[0].clone();
    let incoming = h.request("callHierarchy/incomingCalls", json!({"item": read})).await;
    assert_eq!(names(&incoming, "from"), ["caller"]);
    assert_eq!(text_at(source, &incoming[0]["fromRanges"][0]), "$factory()");
    let outgoing = h.request("callHierarchy/outgoingCalls", json!({"item": incoming[0]["from"]})).await;
    let calls = outgoing.as_array().unwrap();
    assert_eq!(calls.len(), 2);
    let factory = calls.iter().find(|call| call["to"]["name"] != "read").unwrap();
    assert_eq!(text_at(source, &factory["fromRanges"][0]), "$factory");
    assert_eq!(prepare(&mut h, "a.php", source, "$factory()(value:").await[0], factory["to"]);
    assert_eq!(h.request("callHierarchy/outgoingCalls", json!({"item": factory["to"]})).await, json!([]));
}

#[tokio::test]
async fn hierarchy_preparation_selects_nested_callable_names_without_counting_creation() {
    let source = "<?php
function innerFunction(): void {}
class Hooks { public static function innerMethod(): void {} }
function outer(Closure $callback, int $value): void {}
function caller(): void {
    (outer(callback: innerFunction(...), value: ?))(value: 1);
    (outer(callback: Hooks::innerMethod(...), value: ?))(value: 2);
}";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for (needle, name) in [("innerFunction(...)", "innerFunction"), ("innerMethod(...)", "Hooks::innerMethod")] {
        let items = prepare(&mut h, "a.php", source, needle).await;
        assert_eq!(items.as_array().unwrap().len(), 1, "{needle}: {items}");
        assert_eq!(items[0]["name"], name);
        assert_eq!(h.request("callHierarchy/incomingCalls", json!({"item": items[0]})).await, json!([]));
    }
    let items = prepare(&mut h, "a.php", source, "(outer(callback:").await;
    assert_eq!(items[0]["name"], "outer");
    let incoming = h.request("callHierarchy/incomingCalls", json!({"item": items[0]})).await;
    assert_eq!(names(&incoming, "from"), ["caller"]);
    assert_eq!(incoming[0]["fromRanges"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn partial_callable_origins_follow_values_and_keep_each_known_union_target() {
    let source = "<?php
function read(int $value, int $other = 0): int { return $value + $other; }
function other(int $value, int $other = 0): int { return $value - $other; }
function caller(bool $choose): void {
    $partial = read(value: ?, other: 2);
    $copy = $partial;
    $copy(value: 1);
    $callbacks = [$copy];
    $callbacks[0](value: 1);
    $same = $choose ? read(value: ?, other: 2) : read(value: ?, other: 3);
    $same(value: 1);
    $conflict = $choose ? read(value: ?, other: 2) : other(value: ?, other: 2);
    $conflict(value: 1);
    $firstClass = $choose ? read(...) : other(...);
    $firstClass(value: 1);
}";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let read = prepare(&mut h, "a.php", source, "read(").await[0].clone();
    let incoming = h.request("callHierarchy/incomingCalls", json!({"item": read})).await;
    assert_eq!(names(&incoming, "from"), ["caller"]);
    let ranges = incoming[0]["fromRanges"].as_array().unwrap();
    assert_eq!(
        ranges.iter().map(|range| text_at(source, range)).collect::<Vec<_>>(),
        ["$copy", "$callbacks[0]", "$same", "$conflict", "$firstClass"]
    );
    let other = prepare(&mut h, "a.php", source, "other(").await[0].clone();
    let incoming = h.request("callHierarchy/incomingCalls", json!({"item": other})).await;
    assert_eq!(names(&incoming, "from"), ["caller"]);
    assert_eq!(
        incoming[0]["fromRanges"].as_array().unwrap().iter().map(|range| text_at(source, range)).collect::<Vec<_>>(),
        ["$conflict", "$firstClass"]
    );
    for call in ["$conflict(value:", "$firstClass(value:"] {
        let targets = prepare(&mut h, "a.php", source, call).await;
        let mut names: Vec<_> =
            targets.as_array().unwrap().iter().map(|target| target["name"].as_str().unwrap()).collect();
        names.sort_unstable();
        assert_eq!(names, ["other", "read"], "{call}: {targets}");
    }
}

#[tokio::test]
async fn callable_hierarchy_rejects_uncertain_and_changed_targets() {
    let declarations = "<?php function read(int $value): int { return $value; } ";
    let source = format!("{declarations}function caller(): void {{ $callback = read(...); $callback(value: 1); }}");
    let mut h = Harness::start(&[("a.php", &source)]).await;
    let target = prepare(&mut h, "a.php", &source, "read(").await[0].clone();
    let request = json!({"item": target});
    assert_eq!(names(&h.request("callHierarchy/incomingCalls", request.clone()).await, "from"), ["caller"]);
    let changed = format!(
        "{declarations}
function overwritten(): void {{ $callback = read(...); $callback = 42; $callback(value: 1); }}
function uncertain(bool $choose): void {{ $callback = $choose ? read(...) : null; $callback(value: 1); }}
function missing(): void {{ $callbacks = [read(...)]; $callbacks[1](value: 1); }}
function creation(): void {{ $callback = read(...); $partial = read(value: ?); }}"
    );
    h.open("a.php", &changed).await;
    assert_eq!(h.request("callHierarchy/incomingCalls", request.clone()).await, json!([]));
    h.change("a.php", &source, 2).await;
    assert_eq!(names(&h.request("callHierarchy/incomingCalls", request).await, "from"), ["caller"]);
}

#[tokio::test]
async fn incoming_index_is_complete_without_lint_and_tracks_edits_and_deletions() {
    let target = "<?php function target(): void {}";
    let caller = "<?php function untouched(): void { target(); }";
    let config = ServerConfig { linter: false, ..ServerConfig::default() };
    let mut h = Harness::start_with(&[("a.php", target), ("b.php", caller)], config, json!({})).await;
    let item = prepare(&mut h, "a.php", target, "target").await[0].clone();
    let request = json!({"item": item});
    assert_eq!(names(&h.request("callHierarchy/incomingCalls", request.clone()).await, "from"), ["untouched"]);
    h.open("b.php", "<?php function untouched(): void {}").await;
    assert_eq!(h.request("callHierarchy/incomingCalls", request.clone()).await, json!([]));
    h.change("b.php", "<?php function edited(): void { target(); }", 2).await;
    assert_eq!(names(&h.request("callHierarchy/incomingCalls", request.clone()).await, "from"), ["edited"]);
    h.close("b.php").await;
    assert_eq!(names(&h.request("callHierarchy/incomingCalls", request.clone()).await, "from"), ["untouched"]);
    std::fs::remove_file(h.workspace.join("b.php")).unwrap();
    h.watched("b.php", 3).await;
    assert_eq!(h.request("callHierarchy/incomingCalls", request.clone()).await, json!([]));
    std::fs::write(h.workspace.join("c.php"), "<?php function added(): void { target(); }").unwrap();
    h.watched("c.php", 1).await;
    assert_eq!(names(&h.request("callHierarchy/incomingCalls", request).await, "from"), ["added"]);
}

#[tokio::test]
async fn closures_own_calls_without_adding_callable_references() {
    let source = "<?php function target(): void {} function outer(): void { $a = target(...); $b = function() { target(); }; $c = fn() => target(); function inner(): void { target(); } target(); }";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let target = prepare(&mut h, "a.php", source, "target()").await[0].clone();
    let incoming = h.request("callHierarchy/incomingCalls", json!({"item": target})).await;
    assert_eq!(incoming.as_array().unwrap().len(), 4);
    let outer = incoming.as_array().unwrap().iter().find(|call| call["from"]["name"] == "outer").unwrap();
    assert_eq!(outer["fromRanges"].as_array().unwrap().len(), 1);
    let outgoing = h.request("callHierarchy/outgoingCalls", json!({"item": outer["from"]})).await;
    assert_eq!(names(&outgoing, "to"), ["target"]);
    assert_eq!(outgoing[0]["fromRanges"].as_array().unwrap().len(), 1);
    for call in incoming.as_array().unwrap() {
        assert_eq!(text_at(source, &call["fromRanges"][0]), "target");
    }
}

#[tokio::test]
async fn vendor_bodies_keep_workspace_data_and_lexical_this_rules() {
    let vendor = tempfile::tempdir().unwrap();
    let path = vendor.path().join("vendor.php");
    let source = "<?php class Vendor { function target(): void {} function run(): void { $this->target(); $a = fn() => $this->target(); $b = static fn() => $this->target(); } static function invalid(): void { $this->target(); } }";
    std::fs::write(&path, source).unwrap();
    let mut config = ServerConfig::default();
    config.configuration.source.includes = vec![vendor.path().to_str().unwrap().to_owned()];
    let host = "<?php function host(Vendor $v): void { $v->run(); Vendor::invalid(); }";
    let mut h = Harness::start_with(&[("a.php", host)], config, json!({})).await;
    let run = prepare(&mut h, "a.php", host, "run();").await[0].clone();
    assert_eq!(run["uri"], url(&path.canonicalize().unwrap()));
    assert_eq!(run["data"]["workspace"], url(&h.workspace.canonicalize().unwrap()));
    let outgoing = h.request("callHierarchy/outgoingCalls", json!({"item": run})).await;
    assert_eq!(names(&outgoing, "to"), ["Vendor::target"]);
    assert_eq!(outgoing[0]["fromRanges"].as_array().unwrap().len(), 1);
    assert_eq!(text_at(source, &outgoing[0]["fromRanges"][0]), "target");
    let invalid = prepare(&mut h, "a.php", host, "invalid();").await[0].clone();
    assert_eq!(h.request("callHierarchy/outgoingCalls", json!({"item": invalid})).await, json!([]));
}

#[tokio::test]
async fn hierarchy_rejects_stale_data_and_obeys_provider_switches() {
    let source = "<?php function target(): void {} function caller(): void { target(); }";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let original = prepare(&mut h, "a.php", source, "target()").await[0].clone();
    for field in ["data", "uri", "name"] {
        let mut item = original.clone();
        item[field] = match field {
            "data" => Value::Null,
            "uri" => json!(h.url("missing.php")),
            _ => json!("wrong"),
        };
        for method in ["callHierarchy/incomingCalls", "callHierarchy/outgoingCalls"] {
            assert!(h.request(method, json!({"item":item})).await.is_null());
        }
    }
    h.open("a.php", &format!("\n{source}")).await;
    assert!(h.request("callHierarchy/incomingCalls", json!({"item":original})).await.is_null());
    for analyzer in [true, false] {
        let mut config = ServerConfig { analyzer, ..ServerConfig::default() };
        if analyzer {
            config.configuration.language_server.disable = vec![Provider::CallHierarchy];
        }
        assert!(serde_json::to_value(server_capabilities(&config)).unwrap()["callHierarchyProvider"].is_null());
        let mut disabled = Harness::start_with(&[("a.php", source)], config, json!({})).await;
        assert!(prepare(&mut disabled, "a.php", source, "target()").await.is_null());
        let mut item = original.clone();
        item["uri"] = json!(disabled.url("a.php"));
        item["data"]["workspace"] = json!(url(&disabled.workspace.canonicalize().unwrap()));
        for method in ["callHierarchy/incomingCalls", "callHierarchy/outgoingCalls"] {
            assert!(disabled.request(method, json!({"item":item})).await.is_null());
        }
    }
}

#[tokio::test]
async fn callable_union_hierarchy_keeps_known_origins_in_both_branch_orders() {
    let declarations =
        "<?php function read(int $value): int { return $value; } function other(int $value): int { return $value; }";
    let mut h = Harness::start(&[("lib.php", declarations), ("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    for (index, branches) in [
        "read(...) : other(...)",
        "other(...) : read(...)",
        "read(value: ?) : other(value: ?)",
        "other(value: ?) : read(value: ?)",
        "read(...) : other(value: ?)",
        "other(value: ?) : read(...)",
        "read(...) : $unknown",
        "$unknown : read(...)",
        "read(value: ?) : $unknown",
        "$unknown : read(value: ?)",
    ]
    .into_iter()
    .enumerate()
    {
        let source = format!(
            "<?php /** @param Closure(int): int $unknown */ function caller(bool $pick, Closure $unknown): int {{ $callback = $pick ? {branches}; return $callback(value: 2); }}"
        );
        h.change("a.php", &source, index as i32 + 2).await;
        let caller = prepare(&mut h, "a.php", &source, "caller(").await[0].clone();
        let outgoing = h.request("callHierarchy/outgoingCalls", json!({"item": caller})).await;
        let prepared = prepare(&mut h, "a.php", &source, "$callback(value:").await;
        if branches.contains("$unknown") {
            assert_eq!(outgoing, json!([]), "{branches}");
            assert!(prepared.is_null(), "{branches}: {prepared}");
        } else {
            assert_eq!(names(&outgoing, "to"), ["other", "read"], "{branches}");
            assert_eq!(prepared.as_array().unwrap().len(), 2, "{branches}: {prepared}");
            for call in outgoing.as_array().unwrap() {
                assert_eq!(call["fromRanges"].as_array().unwrap().len(), 1, "{branches}: {call}");
                assert_eq!(text_at(&source, &call["fromRanges"][0]), "$callback");
                let incoming = h.request("callHierarchy/incomingCalls", json!({"item": call["to"]})).await;
                assert_eq!(names(&incoming, "from"), ["caller"], "{branches}");
                assert_eq!(incoming[0]["fromRanges"], call["fromRanges"], "{branches}");
            }
        }
    }
}
