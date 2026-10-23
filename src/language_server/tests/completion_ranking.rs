//! Completion order: match quality, then group, test demotion, origin, and proximity.

use serde_json::Value;
use serde_json::json;

use mago_database::file::File;

use crate::language_server::ServerConfig;
use crate::language_server::position::position_at_offset;

use super::harness::Harness;
use super::harness::client_capabilities;

const LIBRARY: &str = "[source]\npaths = ['a.php']\nincludes = ['lib.php']\n";

async fn complete_in(h: &mut Harness, source: &str, marked: &str) -> Vec<Value> {
    let cursor = marked.find('|').unwrap();
    let needle = marked.replace('|', "");
    let offset = source.find(&needle).unwrap_or_else(|| panic!("missing {needle}")) + cursor;
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let position = position_at_offset(&file, offset as u32);
    let result = h.at("textDocument/completion", "a.php", position.line, position.character).await;
    let items = result["items"].as_array().unwrap().clone();
    let mut sorted = items.clone();
    sorted.sort_by(|left, right| client_key(left).cmp(client_key(right)));
    assert_eq!(sorted, items, "sorting by sortText must keep the server order: {result}");
    items
}

async fn complete(files: &[(&str, &str)], marked: &str) -> Vec<Value> {
    let source = marked.replace('|', "");
    let mut all = vec![("a.php", source.as_str())];
    all.extend_from_slice(files);
    let mut h = Harness::start(&all).await;
    complete_in(&mut h, &source, marked).await
}

fn client_key(item: &Value) -> &str {
    item["sortText"].as_str().or_else(|| item["label"].as_str()).unwrap()
}

fn labels(items: &[Value]) -> Vec<&str> {
    items.iter().map(|item| item["label"].as_str().unwrap()).collect()
}

fn position_of(items: &[Value], key: &str, value: &str) -> usize {
    let found = |item: &Value| match key {
        "newText" => item["textEdit"]["newText"] == value,
        _ => item[key] == value,
    };
    items.iter().position(found).unwrap_or_else(|| panic!("missing {value}: {items:#?}"))
}

#[tokio::test]
async fn variables_rank_locals_then_this_then_superglobals() {
    let items = complete(&[], "<?php\nclass C { function run(string $id) { $request = 1; $|; } }\n").await;
    assert_eq!(
        labels(&items),
        [
            "$id",
            "$request",
            "$this",
            "$GLOBALS",
            "$_SERVER",
            "$_GET",
            "$_POST",
            "$_FILES",
            "$_COOKIE",
            "$_SESSION",
            "$_REQUEST",
            "$_ENV"
        ]
    );
    let items = complete(&[], "<?php\nfunction run(int $id, int $idx) { $id|; }\n").await;
    assert_eq!(labels(&items), ["$id", "$idx"]);
    assert_eq!(items[0]["preselect"], true, "{items:#?}");
    assert!(items[1]["preselect"].is_null(), "{items:#?}");
}

#[tokio::test]
async fn class_completion_prefers_import_affinity() {
    let library = r"<?php
namespace Faker\Provider { class Uuid {} }
namespace Acme\Component\Uid { class Uuid {} }
namespace Acme\Component\Http { class Request {} }
";
    let items = complete(
        &[("lib.php", library), ("mago.toml", LIBRARY)],
        "<?php\nnamespace App;\nuse Acme\\Component\\Http\\Request;\nnew Uuid|;\n",
    )
    .await;
    let edits: Vec<_> = items.iter().take(2).map(|item| item["textEdit"]["newText"].as_str().unwrap()).collect();
    assert_eq!(edits, [r"\Acme\Component\Uid\Uuid", r"\Faker\Provider\Uuid"], "{items:#?}");
}

#[tokio::test]
async fn class_completion_demotes_tests_outside_tests() {
    let library = r"<?php
namespace Acme\V\Tests { class CurrencyTest {} }
namespace Acme\V\Constraints\Deep { class CurrencyValidator {} }
";
    let files = [("lib.php", library), ("mago.toml", LIBRARY)];
    let items = complete(&files, "<?php\nnamespace App;\nCurren|;\n").await;
    assert!(position_of(&items, "label", "CurrencyValidator") < position_of(&items, "label", "CurrencyTest"));
    let items = complete(&files, "<?php\nnamespace App\\Tests;\nCurren|;\n").await;
    assert!(position_of(&items, "label", "CurrencyTest") < position_of(&items, "label", "CurrencyValidator"));
}

#[tokio::test]
async fn first_letter_case_outranks_exactness() {
    let source = "<?php\nnamespace App;\nCurren;\nLedg;\nstrl;\nzorb;\n";
    let host = "<?php\nnamespace App\\Enum { enum Currency {} enum Ledger {} }\nnamespace App { const LEDGER_LIMIT = 1; function ledger_total(): int { return 0; } function zorbify(): void {} }\n";
    let mut h = Harness::start(&[
        ("a.php", source),
        ("host.php", host),
        ("lib.php", "<?php\nclass Strl {}\nclass Zorb {}\n"),
        ("mago.toml", "[source]\npaths = ['a.php', 'host.php']\nincludes = ['lib.php']\n"),
    ])
    .await;
    let items = complete_in(&mut h, source, "\nCurren|;").await;
    assert_eq!(labels(&items)[..2], ["Currency", "CURRENCY_SYMBOL"], "{items:#?}");
    let items = complete_in(&mut h, source, "\nLedg|;").await;
    assert_eq!(items[0]["label"], "Ledger", "{items:#?}");
    assert!(position_of(&items, "label", "LEDGER_LIMIT") < position_of(&items, "label", "ledger_total"));
    let items = complete_in(&mut h, source, "\nstrl|;").await;
    assert_eq!(items[0]["label"], "strlen", "{items:#?}");
    let items = complete_in(&mut h, source, "\nzorb|;").await;
    assert_eq!(items[0]["label"], "zorbify", "{items:#?}");
    assert_eq!(items[1]["label"], "Zorb", "{items:#?}");
}

#[tokio::test]
async fn core_then_affinity_then_unrelated() {
    let library = r"<?php
namespace Acme\Http\Exception { interface ExceptionInterface {} }
namespace Acme\Http { class Client {} }
namespace Other { class Exception extends \Exception {} }
";
    let items = complete(
        &[("lib.php", library), ("mago.toml", LIBRARY)],
        "<?php\nnamespace App;\nuse Acme\\Http\\Client;\nExce|;\n",
    )
    .await;
    let core = position_of(&items, "newText", r"\Exception");
    let related = position_of(&items, "newText", r"\Acme\Http\Exception\ExceptionInterface");
    let unrelated = position_of(&items, "newText", r"\Other\Exception");
    assert!(core < related && related < unrelated, "{items:#?}");
}

#[tokio::test]
async fn member_tiers_groups_and_magic() {
    let source = r"<?php
class B { function redirect() {} function render() {} }
final class R extends B { function redirectToRoute() {} function __toString(): string { return ''; } }
function f(R $r) { $r->red }
function g(R $r) { $r-> }
function h(R $r) { $r->__ }
";
    let mut h = Harness::start(&[("a.php", source)]).await;
    assert_eq!(labels(&complete_in(&mut h, source, "$r->red| }").await), ["redirectToRoute", "redirect", "render"]);
    assert_eq!(
        labels(&complete_in(&mut h, source, "function g(R $r) { $r->| }").await),
        ["redirectToRoute", "redirect", "render", "__toString"]
    );
    assert_eq!(labels(&complete_in(&mut h, source, "$r->__| }").await)[0], "__toString");
}

#[tokio::test]
async fn static_enum_cases_first() {
    let items = complete(
        &[],
        "<?php\nenum S: string { case Open = 'o'; const D = self::Open; static function all(): array { return []; } }\nfunction f() { S::| }\n",
    )
    .await;
    assert_eq!(labels(&items), ["Open", "D", "all", "cases", "from", "tryFrom"]);
}

#[tokio::test]
async fn parent_static_access_keeps_magic_methods_in_place() {
    let items = complete(
        &[],
        "<?php\nclass B { public function __construct() {} public function zeta(): void {} }\nclass C extends B { public function __construct() { parent::| } }\n",
    )
    .await;
    assert_eq!(labels(&items), ["__construct", "zeta"]);
}

#[tokio::test]
async fn qualified_functions_rank_host_before_vendor() {
    let items = complete(
        &[("lib.php", "<?php\nnamespace App;\nfunction fmt_a(): void {}\n"), ("mago.toml", LIBRARY)],
        "<?php\nnamespace App;\nfunction fmt_b(): void {}\n\\App\\fmt|;\n",
    )
    .await;
    assert_eq!(labels(&items), ["fmt_b", "fmt_a"]);
}

#[tokio::test]
async fn exact_unique_match_is_preselected() {
    let source = r"<?php
namespace App;
class Foo {}
class FooBar {}
new Foo;
new Fo;
new foo;
new Clock;
";
    let clocks = "<?php\nnamespace First { class Clock {} }\nnamespace Second { class Clock {} }\n";
    let mut h = Harness::start(&[("a.php", source), ("clocks.php", clocks)]).await;
    let items = complete_in(&mut h, source, "new Foo|;").await;
    assert_eq!(labels(&items)[..2], ["Foo", "FooBar"]);
    assert_eq!(items[0]["preselect"], true);
    assert!(items[1..].iter().all(|item| item["preselect"].is_null()), "{items:#?}");
    for needle in ["new Fo|;", "new foo|;", "new Clock|;"] {
        let items = complete_in(&mut h, source, needle).await;
        assert!(items.iter().all(|item| item["preselect"].is_null()), "{needle}: {items:#?}");
    }
}

#[tokio::test]
async fn deprecated_symbols_are_tagged() {
    let source = r"<?php
/** @deprecated */
function old_thing(): void {}
function old_other(): void {}
class Box { /** @deprecated */ public function oldPack(): void {} public function oldWrap(): void {} }
old_;
function f(Box $box) { $box->old }
";
    for supported in [false, true] {
        let mut capabilities = client_capabilities();
        if supported {
            capabilities["textDocument"]["completion"]["completionItem"]["tagSupport"] = json!({ "valueSet": [1] });
        }
        let mut h = Harness::start_with(&[("a.php", source)], ServerConfig::default(), capabilities).await;
        let (tags, flag) = if supported { (json!([1]), Value::Null) } else { (Value::Null, json!(true)) };
        for (needle, deprecated, current) in
            [("\nold_|;", "old_thing", "old_other"), ("$box->old| }", "oldPack", "oldWrap")]
        {
            let items = complete_in(&mut h, source, needle).await;
            let item = |label: &str| &items[position_of(&items, "label", label)];
            assert_eq!(item(deprecated)["tags"], tags, "{items:#?}");
            assert_eq!(item(deprecated)["deprecated"], flag, "{items:#?}");
            assert!(item(current)["tags"].is_null() && item(current)["deprecated"].is_null(), "{items:#?}");
        }
    }
}

#[tokio::test]
async fn methods_match_their_declared_spelling() {
    let source = r"<?php
class Responder {
    public function __construct() {}
    public function getCurrentRequest(): void {}
    public function redirectToRoute(): void {}
    public function getFlashBag(): void {}
    public function getSession(): void {}
}
function f(Responder $r) { $r->tr; $r->fb; $r->cr; $r->getSession; }
";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let items = complete_in(&mut h, source, "$r->tr|;").await;
    let mut loose = labels(&items);
    assert_eq!(loose.remove(0), "redirectToRoute", "{items:#?}");
    loose.sort_unstable();
    assert_eq!(loose, ["__construct", "getCurrentRequest"], "{items:#?}");
    assert_eq!(labels(&complete_in(&mut h, source, "$r->fb|;").await), ["getFlashBag"]);
    assert_eq!(labels(&complete_in(&mut h, source, "$r->cr|;").await), ["getCurrentRequest", "__construct"]);
    let items = complete_in(&mut h, source, "$r->getSession|;").await;
    assert_eq!(labels(&items), ["getSession"]);
    assert_eq!(items[0]["preselect"], true, "{items:#?}");
}

#[tokio::test]
async fn anchored_and_imported_names_lead_loose_matches() {
    let library = r"<?php
namespace Lib { class Identity {} class Request {} class Response {} }
namespace Doctrine\ORM\Exception { class InvalidEntityRepository {} }
";
    let files = [("lib.php", library), ("mago.toml", LIBRARY)];
    let source = "<?php\nnamespace App;\nuse Lib\\Identity;\nuse Lib\\Request;\nuse Lib\\Response;\nidentity;\nreq;\nresp;\nrequest;\nIdentity;\n";
    let mut all = vec![("a.php", source)];
    all.extend_from_slice(&files);
    let mut h = Harness::start(&all).await;
    for (needle, expected) in
        [("\nidentity|;", "Identity"), ("\nreq|;", "Request"), ("\nresp|;", "Response"), ("\nrequest|;", "Request")]
    {
        let items = complete_in(&mut h, source, needle).await;
        assert_eq!(items[0]["label"], expected, "{needle}: {items:#?}");
        assert!(items[0]["preselect"].is_null(), "{needle}: {items:#?}");
    }
    let items = complete_in(&mut h, source, "\nIdentity|;").await;
    assert_eq!(items[0]["label"], "Identity", "{items:#?}");
    assert_eq!(items[0]["preselect"], true, "{items:#?}");
    let items = complete(&files, "<?php\nnamespace App;\nidentity|;\n").await;
    assert!(
        position_of(&items, "label", "Identity") < position_of(&items, "label", "InvalidEntityRepository"),
        "{items:#?}"
    );
}

#[tokio::test]
async fn exact_spelling_beats_imported_case_variants() {
    let library =
        "<?php\nnamespace Acme\\Facades { class Route {} }\nnamespace { function route(): void {} const ROUTE = 1; }\n";
    let source = "<?php\nnamespace App;\nuse Acme\\Facades\\Route;\nroute;\nRoute;\n";
    let mut h = Harness::start(&[("a.php", source), ("lib.php", library), ("mago.toml", LIBRARY)]).await;
    let items = complete_in(&mut h, source, "\nroute|;").await;
    assert_eq!(labels(&items)[..2], ["route", "Route"], "{items:#?}");
    assert_eq!(items[0]["preselect"], true, "{items:#?}");
    let items = complete_in(&mut h, source, "\nRoute|;").await;
    assert_eq!(labels(&items)[..2], ["Route", "ROUTE"], "{items:#?}");
    assert_eq!(items[0]["preselect"], true, "{items:#?}");
    assert!(items[1..].iter().all(|item| item["preselect"].is_null()), "{items:#?}");
}

#[tokio::test]
async fn capitalized_queries_prefer_word_starts_of_that_case() {
    let source = "<?php\nnamespace App;\nclass ApiTest {}\nfunction test_helper(): void {}\nTest;\ntest;\n";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let items = complete_in(&mut h, source, "\nTest|;").await;
    assert!(position_of(&items, "label", "ApiTest") < position_of(&items, "label", "test_helper"), "{items:#?}");
    let items = complete_in(&mut h, source, "\ntest|;").await;
    assert!(position_of(&items, "label", "test_helper") < position_of(&items, "label", "ApiTest"), "{items:#?}");
}

#[tokio::test]
async fn merged_lists_preselect_at_most_the_first_entry() {
    let source = "<?php\nfunction target(int $zeta): void {}\narray_key_exists(key);\ntarget(zeta);\n";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let items = complete_in(&mut h, source, "(key|)").await;
    assert_eq!(labels(&items)[..2], ["key:", "key"], "{items:#?}");
    assert!(items.iter().all(|item| item["preselect"].is_null()), "{items:#?}");
    let items = complete_in(&mut h, source, "(zeta|)").await;
    assert_eq!(items[0]["label"], "zeta:", "{items:#?}");
    assert_eq!(items[0]["preselect"], true, "{items:#?}");
    assert!(items[1..].iter().all(|item| item["preselect"].is_null()), "{items:#?}");
}

#[tokio::test]
async fn named_arguments_keep_declaration_order() {
    let declaration = "<?php\nfunction target(int $zeta, int $alpha = 1, int $mid = 2): void {}\n";
    let items = complete(&[], &format!("{declaration}target(|);\n")).await;
    assert_eq!(labels(&items)[..3], ["zeta:", "alpha:", "mid:"], "{items:#?}");
    let items = complete(&[], &format!("{declaration}target(1, |);\n")).await;
    assert_eq!(labels(&items)[..2], ["alpha:", "mid:"], "{items:#?}");
}
