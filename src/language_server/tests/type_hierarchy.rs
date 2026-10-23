use serde_json::Value;
use serde_json::json;
use tower_lsp_server::gen_lsp_types::Location;
use tower_lsp_server::gen_lsp_types::Range;
use tower_lsp_server::gen_lsp_types::TypeHierarchyItem;

use mago_database::file::File;

use super::harness::Harness;
use super::harness::url;
use crate::config::language_server::Provider;
use crate::language_server::ServerConfig;
use crate::language_server::capabilities::server_capabilities;
use crate::language_server::position::offset_at_position;
use crate::language_server::position::position_at_offset;

async fn at(h: &mut Harness, method: &str, source: &str, needle: &str) -> Value {
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let position = position_at_offset(&file, source.find(needle).unwrap() as u32);
    h.at(method, "a.php", position.line, position.character).await
}

async fn prepare(h: &mut Harness, source: &str, needle: &str) -> Value {
    let result = at(h, "textDocument/prepareTypeHierarchy", source, needle).await;
    assert_eq!(result.as_array().unwrap().len(), 1, "prepare must return one type");
    result[0].clone()
}

fn names(items: &Value) -> Vec<&str> {
    items.as_array().unwrap().iter().map(|item| item["name"].as_str().unwrap()).collect()
}

#[tokio::test]
async fn type_hierarchy_returns_direct_edges_and_exact_selection() {
    let source = "<?php\ninterface Root {}\ninterface Mid extends Root {}\nclass Base implements Mid {}\nclass Child extends Base {}\nenum Flag implements Root { case On; }\ntrait Helper {}\nclass User { use Helper; }\n";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let root = prepare(&mut h, source, "Root").await;
    let item: TypeHierarchyItem = serde_json::from_value(root.clone()).unwrap();
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    assert_eq!(offset_at_position(&file, item.selection_range.start), source.find("Root").unwrap() as u32);
    assert_eq!(offset_at_position(&file, item.selection_range.end), source.find("Root").unwrap() as u32 + 4);
    assert!(item.range.start <= item.selection_range.start && item.selection_range.end <= item.range.end);
    assert_eq!(root["data"]["workspace"], url(&h.workspace.canonicalize().unwrap()));
    let children = h.request("typeHierarchy/subtypes", json!({ "item": root })).await;
    assert_eq!(names(&children), ["Flag", "Mid"]);
    assert_eq!(children[0]["kind"], 10);
    let child = prepare(&mut h, source, "Child").await;
    assert_eq!(names(&h.request("typeHierarchy/supertypes", json!({ "item": child })).await), ["Base"]);
    let helper = prepare(&mut h, source, "Helper").await;
    assert_eq!(h.request("typeHierarchy/subtypes", json!({ "item": helper })).await, json!([]));
    let consumers = at(&mut h, "textDocument/implementation", source, "Helper").await;
    assert_eq!(consumers.as_array().unwrap().len(), 1);
    assert!(at(&mut h, "textDocument/prepareTypeHierarchy", source, "<?php").await.is_null());
}

#[tokio::test]
async fn type_hierarchy_items_start_at_declared_names() {
    let source = "<?php\n/** Doc. */\n#[Mark]\ninterface Root {}\n/**\n * Doc.\n */\n#[Mark]\nabstract class Base implements Root {}\n#[Mark]\nfinal readonly class Leaf extends Base {}\n/** Doc. */\n#[Mark]\nenum Flag: string implements Root { case On = 'on'; }\n";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let span = |item: &Value, field: &str| {
        let range: Range = serde_json::from_value(item[field].clone()).unwrap();
        &source[offset_at_position(&file, range.start) as usize..offset_at_position(&file, range.end) as usize]
    };
    let leaf = prepare(&mut h, source, "Leaf").await;
    assert_eq!(span(&leaf, "selectionRange"), "Leaf");
    assert_eq!(span(&leaf, "range"), "Leaf extends Base {}");
    let parents = h.request("typeHierarchy/supertypes", json!({ "item": leaf })).await;
    assert_eq!(span(&parents[0], "range"), "Base implements Root {}");
    let roots = h.request("typeHierarchy/supertypes", json!({ "item": parents[0] })).await;
    assert_eq!(span(&roots[0], "selectionRange"), "Root");
    assert_eq!(span(&roots[0], "range"), "Root {}");
    let children = h.request("typeHierarchy/subtypes", json!({ "item": roots[0] })).await;
    assert_eq!(names(&children), ["Base", "Flag"]);
    assert_eq!(span(&children[1], "range"), "Flag: string implements Root { case On = 'on'; }");
    let flag = prepare(&mut h, source, "Flag").await;
    assert_eq!(flag, children[1]);
    let builtins = h.request("typeHierarchy/supertypes", json!({ "item": flag })).await;
    let backed = builtins.as_array().unwrap().iter().find(|item| item["name"] == "BackedEnum").unwrap();
    let item: TypeHierarchyItem = serde_json::from_value(backed.clone()).unwrap();
    assert_eq!(item.range.start, item.selection_range.start);
    assert_eq!(item.selection_range.end.character - item.selection_range.start.character, "BackedEnum".len() as u32);
    assert!(item.selection_range.end < item.range.end);
}

#[tokio::test]
async fn implementation_returns_declared_bodies_once() {
    let source = "<?php\ninterface Task { public function run(): void; }\nabstract class AbstractTask implements Task {}\nclass Basic implements Task { public function run(): void {} }\nclass Inherited extends Basic {}\nclass Special extends Basic { public function run(): void {} }\nclass Leaf extends Special {}\nfunction invoke(Task $task): void { $task->run(); }\n";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for needle in ["run(): void;", "run();"] {
        let result = at(&mut h, "textDocument/implementation", source, needle).await;
        let locations: Vec<Location> = serde_json::from_value(result).unwrap();
        assert_eq!(locations.iter().map(|l| l.range.start.line).collect::<Vec<_>>(), [3, 5]);
        assert!(locations.iter().all(|l| l.range.end.character - l.range.start.character == 3));
    }
    let overrides = at(&mut h, "textDocument/implementation", source, "run(): void {}").await;
    assert_eq!(overrides.as_array().unwrap().len(), 1);
    assert_eq!(overrides[0]["range"]["start"]["line"], 5);
    assert!(at(&mut h, "textDocument/implementation", source, "<?php").await.is_null());
}

#[tokio::test]
async fn hierarchy_rejects_missing_mismatched_and_stale_data() {
    let source = "<?php class Base {} class Child extends Base {}";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let original = prepare(&mut h, source, "Base").await;
    let mut missing = original.clone();
    missing["data"] = Value::Null;
    let mut wrong_key = original.clone();
    wrong_key["data"]["key"] = json!(b"child");
    let mut wrong_workspace = original.clone();
    wrong_workspace["data"]["workspace"] = json!(h.url("a.php"));
    let mut wrong_uri = original.clone();
    wrong_uri["uri"] = json!(h.url("missing.php"));
    for item in [missing, wrong_key, wrong_workspace, wrong_uri] {
        assert!(h.request("typeHierarchy/subtypes", json!({ "item": item })).await.is_null());
    }
    h.open("a.php", "<?php\nclass Base {} class Child extends Base {}").await;
    assert!(h.request("typeHierarchy/supertypes", json!({ "item": original })).await.is_null());
}

#[tokio::test]
async fn vendor_hierarchy_keeps_the_request_workspace() {
    let vendor = tempfile::tempdir().unwrap();
    let vendor_path = vendor.path().join("base.php");
    std::fs::write(&vendor_path, "<?php class VendorBase {}").unwrap();
    let mut config = ServerConfig::default();
    config.configuration.source.includes = vec![vendor.path().to_str().unwrap().to_owned()];
    let source = "<?php /*😀*/ class Café extends VendorBase {}";
    let mut h = Harness::start_with(&[("a.php", source)], config, json!({})).await;
    let parent = prepare(&mut h, source, "VendorBase").await;
    assert_eq!(parent["uri"], url(&vendor_path.canonicalize().unwrap()));
    let children = h.request("typeHierarchy/subtypes", json!({ "item": parent })).await;
    assert_eq!(names(&children), ["Café"]);
    assert_eq!(children[0]["uri"], h.url("a.php"));
    assert_eq!(
        children[0]["selectionRange"]["start"]["character"],
        source[..source.find("Café").unwrap()].encode_utf16().count()
    );
    assert_eq!(names(&h.request("typeHierarchy/supertypes", json!({ "item": children[0] })).await), ["VendorBase"]);
}

#[tokio::test]
async fn hierarchy_followups_obey_provider_and_analyzer_switches() {
    let source = "<?php class Base {} class Child extends Base {}";
    let mut enabled = Harness::start(&[("a.php", source)]).await;
    let original = prepare(&mut enabled, source, "Base").await;
    for analyzer in [true, false] {
        let mut config = ServerConfig { analyzer, ..ServerConfig::default() };
        if analyzer {
            config.configuration.language_server.disable = vec![Provider::TypeHierarchy, Provider::Implementation];
        }
        let caps = serde_json::to_value(server_capabilities(&config)).unwrap();
        assert!(caps["typeHierarchyProvider"].is_null());
        assert!(caps["implementationProvider"].is_null());
        assert!(caps["experimental"].is_null());
        let mut h = Harness::start_with(&[("a.php", source)], config, json!({})).await;
        let mut item = original.clone();
        item["uri"] = json!(h.url("a.php"));
        item["data"]["workspace"] = json!(url(&h.workspace.canonicalize().unwrap()));
        for method in ["typeHierarchy/supertypes", "typeHierarchy/subtypes"] {
            assert!(h.request(method, json!({ "item": item })).await.is_null());
        }
        assert!(at(&mut h, "textDocument/prepareTypeHierarchy", source, "Base").await.is_null());
        assert!(at(&mut h, "textDocument/implementation", source, "Base").await.is_null());
    }
}

#[test]
fn hierarchy_advertisement_uses_the_standard_capability() {
    let caps = serde_json::to_value(server_capabilities(&ServerConfig::default())).unwrap();
    assert_eq!(caps["typeHierarchyProvider"], true);
    assert!(caps["experimental"].is_null());
}

async fn check_trait_implementations(source: &str, call: &str, declarations: &[&str]) {
    let mut h = Harness::start(&[("a.php", source)]).await;
    let result = at(&mut h, "textDocument/implementation", source, call).await;
    if declarations.is_empty() {
        assert!(result.is_null(), "{call}: {result}");
        return;
    }
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let locations: Vec<Location> = serde_json::from_value(result).unwrap();
    let expected: Vec<_> = declarations.iter().map(|declaration| source.find(declaration).unwrap() as u32).collect();
    assert_eq!(
        locations.iter().map(|location| offset_at_position(&file, location.range.start)).collect::<Vec<_>>(),
        expected,
        "{call}: {locations:?}"
    );
    for (location, declaration) in locations.iter().zip(declarations) {
        assert_eq!(location.uri.as_str(), h.url("a.php"));
        assert_eq!(
            offset_at_position(&file, location.range.end) - offset_at_position(&file, location.range.start),
            declaration.find('(').unwrap() as u32
        );
    }
}

#[tokio::test]
async fn implementation_trait_alias_uses_the_exposed_name() {
    let source = r"<?php
trait Methods { public function run(): void {} }
class Base { use Methods { run as ExEcUtE; } }
class Child extends Base {
    public function execute(): void {}
    public function run(): void { /* child */ }
}
function invoke(Base $base): void { $base->execute(); $base->run(); }
";
    check_trait_implementations(source, "execute();", &["execute(): void {}"]).await;
    check_trait_implementations(source, "run();", &["run(): void { /* child */ }"]).await;
}

#[tokio::test]
async fn implementation_trait_alias_uses_its_public_visibility() {
    let source = r"<?php
trait Methods { private function run(): void {} }
class Base { use Methods { run as public execute; } }
class Child extends Base { public function execute(): void {} }
function invoke(Base $base): void { $base->execute(); }
";
    check_trait_implementations(source, "execute();", &["execute(): void {}"]).await;
}

#[tokio::test]
async fn implementation_trait_body_uses_its_public_visibility() {
    let source = r"<?php
interface Task { public function run(): void; }
trait Methods { private function run(): void {} }
class Worker implements Task { use Methods { run as public; } }
class ChildWorker extends Worker {}
function invoke(Task $task): void { $task->run(); }
";
    check_trait_implementations(source, "run();", &["run(): void {}"]).await;
}

#[tokio::test]
async fn implementation_trait_private_adaptation_has_no_overrides() {
    let source = r"<?php
trait Methods { public function run(): void {} }
class Base {
    use Methods { run as private; }
    public function invoke(): void { $this->run(); }
}
class Child extends Base { public function run(): void { /* separate method */ } }
";
    check_trait_implementations(source, "run();", &[]).await;
}

#[tokio::test]
async fn implementation_trait_final_alias_does_not_follow_original_overrides() {
    let source = r"<?php
trait Methods { public function run(): void {} }
class Base { use Methods { run as final locked; } }
class Child extends Base { public function run(): void { /* original override */ } }
function invoke(Base $base): void { $base->locked(); $base->run(); }
";
    check_trait_implementations(source, "locked();", &[]).await;
    check_trait_implementations(source, "run();", &["run(): void { /* original override */ }"]).await;
}

#[tokio::test]
async fn enum_hierarchy_hides_internal_backed_contracts() {
    let source = "<?php\ninterface Contract {}\nenum Suit: string implements Contract { case H = 'h'; }\nenum Flag { case On; }\nfunction f(BackedEnum $b): void {}\n";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for (needle, expected) in [("Suit", &["BackedEnum", "Contract"][..]), ("Flag", &["UnitEnum"][..])] {
        let item = prepare(&mut h, source, needle).await;
        assert_eq!(names(&h.request("typeHierarchy/supertypes", json!({ "item": item })).await), expected, "{needle}");
    }
    let backed = prepare(&mut h, source, "BackedEnum").await;
    let subtypes = h.request("typeHierarchy/subtypes", json!({ "item": backed })).await;
    assert!(names(&subtypes).iter().all(|name| !name.contains("INTERNAL_DO_NOT_USE")), "{subtypes}");
    let suit = subtypes.as_array().unwrap().iter().find(|item| item["name"] == "Suit").unwrap();
    assert_eq!(suit["uri"], h.url("a.php"));
}
