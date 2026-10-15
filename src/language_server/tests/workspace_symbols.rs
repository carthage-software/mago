//! Workspace search keeps declared names, symbol kinds, and source destinations.

use mago_database::file::File;
use serde_json::{Value, json};

use super::harness::Harness;
use crate::language_server::position::position_at_offset;

const SOURCE: &str = r"<?php
#[Mark]
/*😀*/ final class ReviewClass {}
#[Mark]
interface ReviewInterface {}
#[Mark]
trait ReviewTrait {}
#[Mark]
enum ReviewEnum {}
#[Mark]
/*😀*/ function ReviewFunction(): void {}
/*😀*/ const ReviewFlag = 1;
";

fn check_symbol(symbol: &Value, uri: &str, name: &str, kind: u32, declaration: &str) {
    let file = File::ephemeral(b"a.php".as_slice().into(), SOURCE.as_bytes().to_vec().into());
    let start = SOURCE.find(declaration).unwrap();
    assert_eq!(symbol["name"], name);
    assert_eq!(symbol["kind"], kind);
    assert_eq!(symbol["location"]["uri"], uri);
    assert_eq!(
        symbol["location"]["range"],
        json!({
            "start": position_at_offset(&file, start as u32),
            "end": position_at_offset(&file, (start + declaration.len()) as u32)
        })
    );
}

#[tokio::test]
async fn workspace_symbol_finds_constants_with_case_insensitive_queries() {
    let mut h = Harness::start(&[("a.php", SOURCE)]).await;
    for query in ["ReviewFlag", "reviewflag", "REVIEWFLAG", "vIeWfLa"] {
        let result = h.request("workspace/symbol", json!({ "query": query })).await;
        let symbols = result.as_array().unwrap();
        assert_eq!(symbols.len(), 1, "query {query}: {result}");
        check_symbol(&symbols[0], &h.url("a.php"), "ReviewFlag", 14, "ReviewFlag = 1");
    }
    assert_eq!(h.request("workspace/symbol", json!({ "query": "ReviewMissing" })).await, json!([]));
}

#[tokio::test]
async fn workspace_symbol_targets_names_after_attributes_and_modifiers() {
    let mut h = Harness::start(&[("a.php", SOURCE)]).await;
    for query in ["review", "REVIEW", ""] {
        let result = h.request("workspace/symbol", json!({ "query": query })).await;
        let symbols = result.as_array().unwrap();
        assert_eq!(symbols.len(), 6, "query {query}: {result}");
        for (name, kind, declaration) in [
            ("ReviewClass", 5, "ReviewClass"),
            ("ReviewInterface", 11, "ReviewInterface"),
            ("ReviewTrait", 5, "ReviewTrait"),
            ("ReviewEnum", 10, "ReviewEnum"),
            ("ReviewFunction", 12, "ReviewFunction"),
            ("ReviewFlag", 14, "ReviewFlag = 1"),
        ] {
            let symbol = symbols.iter().find(|symbol| symbol["name"] == name).unwrap_or_else(|| panic!("{result}"));
            check_symbol(symbol, &h.url("a.php"), name, kind, declaration);
        }
    }
}

const SHARED_SYMBOL_SOURCE: &str = "<?php\nclass SharedRow {}\nfunction SharedRow(): void {}\nconst SharedRow = 1;\n";

async fn check_shared_symbol_rows(h: &mut Harness, uris: &[String]) {
    let result = h.request("workspace/symbol", json!({ "query": "SharedRow" })).await;
    let symbols = result.as_array().unwrap();
    assert_eq!(symbols.len(), uris.len() * 3, "each declaration needs one row: {result}");
    let file = File::ephemeral(b"shared.php".as_slice().into(), SHARED_SYMBOL_SOURCE.as_bytes().to_vec().into());
    for uri in uris {
        for (kind, declaration, selection) in [
            (5, "class SharedRow {}", "SharedRow"),
            (12, "function SharedRow(): void {}", "SharedRow"),
            (14, "SharedRow = 1", "SharedRow = 1"),
        ] {
            let start = SHARED_SYMBOL_SOURCE.find(declaration).unwrap() + declaration.find(selection).unwrap();
            let expected = json!({
                "uri": uri,
                "range": {
                    "start": position_at_offset(&file, start as u32),
                    "end": position_at_offset(&file, (start + selection.len()) as u32)
                }
            });
            assert_eq!(
                symbols
                    .iter()
                    .filter(|symbol| {
                        symbol["name"] == "SharedRow" && symbol["kind"] == kind && symbol["location"] == expected
                    })
                    .count(),
                1,
                "missing or repeated declaration: {expected}; {result}"
            );
        }
    }
}

#[tokio::test]
async fn workspace_symbols_do_not_repeat_files_in_nested_roots() {
    let mut h = Harness::start_multi(&[
        ("parent", &[("anchor.php", "<?php")]),
        ("parent/nested", &[("shared.php", SHARED_SYMBOL_SOURCE)]),
        ("separate", &[("same_name.php", SHARED_SYMBOL_SOURCE)]),
    ])
    .await;
    let uris = [h.url_in("parent/nested", "shared.php"), h.url_in("separate", "same_name.php")];
    check_shared_symbol_rows(&mut h, &uris).await;
}

#[tokio::test]
async fn workspace_symbols_do_not_repeat_shared_dependency_files() {
    let mut h = Harness::start_multi(&[
        ("owner", &[("shared.php", SHARED_SYMBOL_SOURCE)]),
        ("consumer", &[("anchor.php", "<?php"), ("mago.toml", "[source]\nincludes = ['../owner/shared.php']\n")]),
        ("separate", &[("same_name.php", SHARED_SYMBOL_SOURCE)]),
    ])
    .await;
    let uris = [h.url_in("owner", "shared.php"), h.url_in("separate", "same_name.php")];
    check_shared_symbol_rows(&mut h, &uris).await;
}

const LIBRARY: &str = "[source]\npaths = ['a.php']\nincludes = ['lib.php']\n";

async fn symbol_names(h: &mut Harness, query: &str) -> Vec<String> {
    let result = h.request("workspace/symbol", json!({ "query": query })).await;
    result.as_array().unwrap().iter().map(|symbol| symbol["name"].as_str().unwrap().to_owned()).collect()
}

#[tokio::test]
async fn workspace_symbols_rank_host_then_library_then_tests() {
    let host = "<?php\nnamespace App\\Enum { enum Currency {} }\nnamespace App\\Twig { class CurrencyExtension {} }\n";
    let library = r"<?php
namespace Acme\Validator\Tests { class CurrencyTest {} }
namespace Acme\Validator\Constraints { class CurrencyValidator {} class Currency {} }
";
    let mut h = Harness::start(&[("a.php", host), ("lib.php", library), ("mago.toml", LIBRARY)]).await;
    assert_eq!(
        symbol_names(&mut h, "Currency").await,
        [
            r"App\Enum\Currency",
            r"App\Twig\CurrencyExtension",
            r"Acme\Validator\Constraints\Currency",
            r"Acme\Validator\Constraints\CurrencyValidator",
            r"Acme\Validator\Tests\CurrencyTest",
        ]
    );
    assert_eq!(symbol_names(&mut h, r"Constraints\Curr").await[0], r"Acme\Validator\Constraints\Currency");
    assert_eq!(symbol_names(&mut h, r"\App\Enum\Currency").await, [r"App\Enum\Currency"]);
    assert!(symbol_names(&mut h, r"\Enum\Currency").await.is_empty());
}

#[tokio::test]
async fn workspace_symbols_find_members_with_containers() {
    let source = r"<?php
namespace App;
final class Responder {
    public const ROUTE = 'r';
    public string $route = '';
    public function redirectToRoute(): void {}
}
enum Status { case RouteOpen; }
";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let result = h.request("workspace/symbol", json!({ "query": "ROUTE" })).await;
    let rows: Vec<_> = result
        .as_array()
        .unwrap()
        .iter()
        .map(|symbol| {
            (
                symbol["name"].as_str().unwrap(),
                symbol["kind"].as_u64().unwrap(),
                symbol["containerName"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        rows,
        [
            (r"App\Responder::ROUTE", 14, r"App\Responder"),
            (r"App\Status::RouteOpen", 22, r"App\Status"),
            (r"App\Responder::redirectToRoute", 6, r"App\Responder"),
            (r"App\Responder::$route", 7, r"App\Responder"),
        ]
    );
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let start = source.find("$route").unwrap() as u32;
    let property = result.as_array().unwrap().iter().find(|symbol| symbol["kind"] == 7).unwrap();
    assert_eq!(property["location"]["range"]["start"], json!(position_at_offset(&file, start)));
    assert_eq!(symbol_names(&mut h, "Responder::red").await, [r"App\Responder::redirectToRoute"]);
    assert_eq!(symbol_names(&mut h, r"\App\Status::").await, [r"App\Status::RouteOpen"]);
    assert_eq!(symbol_names(&mut h, r"App\red").await, [r"App\Responder"]);
}

#[tokio::test]
async fn workspace_symbols_rank_word_starts_by_query_case() {
    let source = r"<?php
namespace App\Tests;
abstract class AbstractTestCase { public function testFoo(): void {} }
final class UserTest { public function testBar(): void {} }
";
    let mut h = Harness::start(&[("a.php", source)]).await;
    assert_eq!(
        symbol_names(&mut h, "Test").await,
        [
            r"App\Tests\AbstractTestCase",
            r"App\Tests\UserTest",
            r"App\Tests\UserTest::testBar",
            r"App\Tests\AbstractTestCase::testFoo",
        ]
    );
    assert_eq!(
        symbol_names(&mut h, "test").await,
        [
            r"App\Tests\UserTest::testBar",
            r"App\Tests\AbstractTestCase::testFoo",
            r"App\Tests\AbstractTestCase",
            r"App\Tests\UserTest",
        ]
    );
}

#[tokio::test]
async fn workspace_symbols_match_humps_then_fuzzy() {
    let source = "<?php\nnamespace App;\nclass DeleteController {}\nclass DashboardLinkCollector {}\n";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for query in ["dc", "dlc", "DLC"] {
        assert_eq!(
            symbol_names(&mut h, query).await,
            [r"App\DashboardLinkCollector", r"App\DeleteController"],
            "{query}"
        );
    }
}

#[tokio::test]
async fn workspace_symbols_skip_closures_and_anonymous_classes() {
    let source = "<?php\nclass Named {}\n$x = new class {};\n$f = function () {};\n$g = fn () => 1;\n";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for query in ["closure", "anonymous", "a.php", "{"] {
        assert!(symbol_names(&mut h, query).await.is_empty(), "{query}");
    }
    assert_eq!(symbol_names(&mut h, "").await, ["Named"]);
}

#[tokio::test]
async fn workspace_symbol_empty_query_is_host_only() {
    let host = "<?php\nclass HostThing { public function run(): void {} }\nfunction host_fn(): void {}\n";
    let library = "<?php\nclass VendorThing {}\n";
    let mut h = Harness::start(&[("a.php", host), ("lib.php", library), ("mago.toml", LIBRARY)]).await;
    assert_eq!(symbol_names(&mut h, "").await, ["host_fn", "HostThing"]);
    assert_eq!(symbol_names(&mut h, "Thing").await, ["HostThing", "VendorThing"]);
}

#[tokio::test]
async fn workspace_symbols_merge_ranked_across_workspaces() {
    let mut h = Harness::start_multi(&[
        ("one", &[("a.php", "<?php\nnamespace One;\nclass CurrencyExtension {}\n")]),
        ("two", &[("b.php", "<?php\nnamespace Two;\nclass Currency {}\n")]),
    ])
    .await;
    assert_eq!(symbol_names(&mut h, "Currency").await, [r"Two\Currency", r"One\CurrencyExtension"]);
}

const QUALIFIED_SOURCE: &str = r"<?php
namespace App\Enum { enum Currency: string { case EUR = 'eur'; public function getLabel(): string { return ''; } } }
namespace App\Twig { class CurrencyExtension { public function getFunctions(): array { return []; } } }
namespace Psr\Http\Client { interface ClientInterface {} }
namespace Symfony\Component\HttpClient\Exception { class ClientException {} }
namespace App\Event\Client {
    class ClientUpdatedEvent { public string $previousAddress = ''; }
    class Client { public string $address = ''; public function address(): string { return ''; } }
}
";

#[tokio::test]
async fn workspace_symbol_qualifiers_match_whole_segments() {
    let mut h = Harness::start(&[("a.php", QUALIFIED_SOURCE)]).await;
    assert_eq!(symbol_names(&mut h, "Currency::").await, [r"App\Enum\Currency::EUR", r"App\Enum\Currency::getLabel"]);
    assert_eq!(symbol_names(&mut h, r"Enum\Currency::get").await, [r"App\Enum\Currency::getLabel"]);
    assert_eq!(symbol_names(&mut h, r"Http\Client").await, [r"Psr\Http\Client\ClientInterface"]);
    assert_eq!(symbol_names(&mut h, "Client::$address").await, [r"App\Event\Client\Client::$address"]);
    assert!(symbol_names(&mut h, "Curr::").await.is_empty());
    assert!(symbol_names(&mut h, r"Ap\Currency").await.is_empty());
}

#[tokio::test]
async fn workspace_symbol_property_queries_search_properties() {
    let mut h = Harness::start(&[("a.php", QUALIFIED_SOURCE)]).await;
    assert_eq!(
        symbol_names(&mut h, "$address").await,
        [r"App\Event\Client\Client::$address", r"App\Event\Client\ClientUpdatedEvent::$previousAddress"]
    );
    assert_eq!(
        symbol_names(&mut h, "address").await,
        [
            r"App\Event\Client\Client::$address",
            r"App\Event\Client\Client::address",
            r"App\Event\Client\ClientUpdatedEvent::$previousAddress"
        ]
    );
}
