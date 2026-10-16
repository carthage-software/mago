use mago_database::file::File;
use serde_json::Value;

use super::harness::Harness;
use crate::language_server::position::position_at_offset;

const SOURCE: &str = r"<?php
namespace CarthageSoftware\CQRS { interface CommandBusInterface {} }
namespace Symfony\Component\HttpFoundation { class Request {} class Response {} }
namespace Symfony\Component\Security\Csrf { interface CsrfTokenManagerInterface {} }
namespace Left { class Item {} }
namespace Right { class Item {} }
namespace App {
    use CarthageSoftware\CQRS\CommandBusInterface;
    use Symfony\Component\HttpFoundation\Request;
    use Symfony\Component\HttpFoundation\Response;
    use Symfony\Component\Security\Csrf\CsrfTokenManagerInterface;

    /**
     * @template T
     * @property-read Request $current
     */
    final class Controller {
        public \Left\Item|\Right\Item $item;
        /** @var T */
        public mixed $value;

        public function __construct(
            private CommandBusInterface $commandBus,
            private CsrfTokenManagerInterface $csrfTokens,
        ) {}

        /** @param non-empty-string $id */
        public function __invoke(Request $request, string $id): Response { return new Response(); }

        /** @return T */
        public function value(): mixed { return $this->value; }

        public function pick(\Left\Item $left, \Right\Item $right): \Left\Item { return $left; }

        public function run(): void {
            $this->commandBus;
            $this->current;
            $this->item;
            $this->__invoke(new Request(), 'id');
            $this->pick();
            $this->pick(left: new \Left\Item(), right: new \Right\Item());
        }
    }

    final class Other {
        public Response $current;
        public function pick(Response $left): Response { return $left; }
    }

    final class LeftHolder {
        public \Left\Item $held;
        public function take(\Left\Item $item): void {}
    }

    final class RightHolder {
        public \Right\Item $held;
        public function take(\Right\Item $item): void {}
    }

    function hold(LeftHolder|RightHolder $holder): void {
        $holder->held;
        $holder->take();
    }

    function either(Controller|Other $either): void {
        $either->current;
        $either->pick();
    }

    /**
     * @template K
     * @param list<K> $items
     * @return K
     */
    function first(array $items): mixed { return $items[0]; }

    function handle(Request $request): Response { return new Response(); }

    handle(new Request());
    first([1]);
    array_map(null, []);
}
";

async fn at(h: &mut Harness, offset: usize, method: &str) -> Value {
    let file = File::ephemeral(b"a.php".as_slice().into(), SOURCE.as_bytes().to_vec().into());
    let position = position_at_offset(&file, offset as u32);
    h.at(method, "a.php", position.line, position.character).await
}

async fn detail(h: &mut Harness, prefix: &str, label: &str) -> String {
    let offset = SOURCE.find(prefix).unwrap() + prefix.len();
    let completion = at(h, offset, "textDocument/completion").await;
    let item = completion["items"].as_array().unwrap().iter().find(|item| item["label"] == label);
    item.and_then(|item| item["detail"].as_str()).unwrap_or_else(|| panic!("{prefix} {label}: {completion}")).to_owned()
}

#[tokio::test]
async fn completion_details_spell_types_with_short_names() {
    let mut h = Harness::start(&[("a.php", SOURCE)]).await;
    for (prefix, label, expected) in [
        ("$this->comm", "commandBus", "CommandBusInterface"),
        ("$this->cur", "current", "Request"),
        (
            "$this->__",
            "__construct",
            "__construct(CommandBusInterface $commandBus, CsrfTokenManagerInterface $csrfTokens)",
        ),
        ("$this->__inv", "__invoke", "__invoke(Request $request, non-empty-string $id): Response"),
        ("$this->val", "value", "T"),
        ("$this->val", "value", "value(): T"),
        ("    handle", "handle", "handle(Request $request): Response"),
        ("    fir", "first", "first(list<K> $items): K"),
        ("$either->cur", "current", r"App\Controller: Request $current | App\Other: Response $current"),
        (
            "$either->pi",
            "pick",
            r"App\Controller: function pick(\Left\Item $left, \Right\Item $right): \Left\Item | App\Other: function pick(Response $left): Response",
        ),
    ] {
        let offset = SOURCE.find(prefix).unwrap() + prefix.len();
        let completion = at(&mut h, offset, "textDocument/completion").await;
        let found = completion["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["label"] == label && item["detail"] == expected);
        assert!(found, "{prefix} {label} {expected}: {completion}");
    }
}

#[tokio::test]
async fn completion_details_qualify_only_colliding_short_names() {
    let mut h = Harness::start(&[("a.php", SOURCE)]).await;
    assert_eq!(detail(&mut h, "$this->ite", "item").await, r"\Left\Item|\Right\Item");
    assert_eq!(detail(&mut h, "$this->pi", "pick").await, r"pick(\Left\Item $left, \Right\Item $right): \Left\Item");
    assert_eq!(detail(&mut h, "$this->pick(", "left:").await, r"\Left\Item $left");
}

#[tokio::test]
async fn union_member_details_qualify_short_names_shared_across_receivers() {
    let mut h = Harness::start(&[("a.php", SOURCE)]).await;
    assert_eq!(
        detail(&mut h, "$holder->hel", "held").await,
        r"App\LeftHolder: \Left\Item $held | App\RightHolder: \Right\Item $held"
    );
    assert_eq!(
        detail(&mut h, "$holder->ta", "take").await,
        r"App\LeftHolder: function take(\Left\Item $item): void | App\RightHolder: function take(\Right\Item $item): void"
    );
}

#[tokio::test]
async fn named_argument_details_share_the_call_clash_context() {
    let mut h = Harness::start(&[("a.php", SOURCE)]).await;
    let call = "$this->pick(";
    let help = at(&mut h, SOURCE.find(call).unwrap() + call.len(), "textDocument/signatureHelp").await;
    let signature = &help["signatures"][0];
    let label: Vec<_> = signature["label"].as_str().unwrap().encode_utf16().collect();
    for (index, name, expected) in [(0, "left", r"\Left\Item $left"), (1, "right", r"\Right\Item $right")] {
        let range = &signature["parameters"][index]["label"];
        let parameter = &label[range[0].as_u64().unwrap() as usize..range[1].as_u64().unwrap() as usize];
        assert_eq!(String::from_utf16(parameter).unwrap(), expected, "{help}");
        assert_eq!(detail(&mut h, call, &format!("{name}:")).await, expected);
        let hover = at(&mut h, SOURCE.find(&format!("{name}: new")).unwrap(), "textDocument/hover").await;
        assert!(hover["contents"]["value"].as_str().unwrap().lines().any(|line| line == expected), "{name}: {hover}");
    }
}

#[tokio::test]
async fn completion_details_match_hover_and_signature_help() {
    let mut h = Harness::start(&[("a.php", SOURCE)]).await;
    for (call, label, hover_declaration, signature_label) in [
        (
            "$this->__invoke(",
            "__invoke",
            "public function __invoke(Request $request, non-empty-string $id): Response",
            r"function App\Controller::__invoke(Request $request, non-empty-string $id): Response",
        ),
        (
            "$this->pick(",
            "pick",
            r"public function pick(\Left\Item $left, \Right\Item $right): \Left\Item",
            r"function App\Controller::pick(\Left\Item $left, \Right\Item $right): \Left\Item",
        ),
        (
            "    handle(",
            "handle",
            "function handle(Request $request): Response",
            r"function App\handle(Request $request): Response",
        ),
    ] {
        let start = SOURCE.find(call).unwrap();
        let hover = at(&mut h, start + call.len() - 1 - label.len(), "textDocument/hover").await;
        assert!(
            hover["contents"]["value"].as_str().unwrap().lines().any(|line| line == hover_declaration),
            "{call}: {hover}"
        );
        let help = at(&mut h, start + call.len(), "textDocument/signatureHelp").await;
        assert_eq!(help["signatures"][0]["label"], signature_label, "{call}: {help}");
        let completion = detail(&mut h, &call[..call.len() - 1], label).await;
        assert_eq!(completion, hover_declaration.split_once("function ").unwrap().1, "{call}");
    }

    let call = "array_map(";
    let start = SOURCE.find(call).unwrap();
    let help = at(&mut h, start + call.len(), "textDocument/signatureHelp").await;
    let signature = help["signatures"][0]["label"].as_str().unwrap();
    let completion = detail(&mut h, "array_ma", "array_map").await;
    assert_eq!(Some(completion.as_str()), signature.strip_prefix("function "), "{help}");
    assert!(!completion.contains("array_map()::"), "{completion}");
}
