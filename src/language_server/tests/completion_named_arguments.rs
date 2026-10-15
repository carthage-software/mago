use serde_json::{Value, json};
use tower_lsp_server::gen_lsp_types::TextEdit;

use mago_allocator::LocalArena;
use mago_database::file::File;
use mago_php_version::PHPVersion;
use mago_syntax::parser::parse_file_borrowed_with_settings;

use crate::language_server::ServerConfig;
use crate::language_server::position::{offset_at_position, position_at_offset};

use super::harness::Harness;

const LIBRARY: &str = r"<?php namespace Demo;
function deliver(string $message = '', int $priority = 0, bool $urgent = false): void {}
class Client {
    public function __construct(string $name = '', int $priority = 0) {}
    public function send(string $message = '', int $priority = 0): void {}
    public static function build(int $priority = 0): self { return new self(); }
}
class Child extends Client {}
class Alternate { public function send(string $body = '', int $priority = 0): void {} }
class Swapped { public function send(int $priority = 0, string $message = ''): void {} }
class Magic { public function __call(string $name, array $arguments): mixed { return null; } }
function client(): Client { return new Client(); }
/** @no-named-arguments */ function positional(int $priority = 0): void {}
function café(string $名 = ''): void {}
function cases(int $value = 0, int $Value = 0): void {}
";

async fn complete(h: &mut Harness, marked: &str, version: i32) -> (String, Vec<Value>) {
    let offset = marked.find("<cursor>").unwrap();
    let source = marked.replacen("<cursor>", "", 1);
    h.change("a.php", &source, version).await;
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let position = position_at_offset(&file, offset as u32);
    let result = h.at("textDocument/completion", "a.php", position.line, position.character).await;
    (source, result["items"].as_array().unwrap().clone())
}

fn named<'a>(items: &'a [Value], name: &str) -> &'a Value {
    items
        .iter()
        .find(|item| item["label"] == format!("{name}:"))
        .unwrap_or_else(|| panic!("missing {name}: completion: {items:?}"))
}

fn named_labels(items: &[Value]) -> Vec<&str> {
    items.iter().filter_map(|item| item["label"].as_str()).filter(|label| label.ends_with(':')).collect()
}

fn apply(source: &str, item: &Value, value: &str) -> (String, u32) {
    assert_ne!(item["insertTextFormat"], 2, "named argument edits must be plain text: {item}");
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let edit: TextEdit = serde_json::from_value(item["textEdit"].clone()).unwrap();
    let start = offset_at_position(&file, edit.range.start) as usize;
    let end = offset_at_position(&file, edit.range.end) as usize;
    let insertion = edit.new_text + value;
    let mut result = source.to_owned();
    result.replace_range(start..end, &insertion);
    (result, (start + insertion.len()) as u32)
}

async fn check_binding(h: &mut Harness, source: &str, offset: u32, parameter: u32, version: i32) {
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    {
        let arena = LocalArena::new();
        assert!(!parse_file_borrowed_with_settings(&arena, &file, Default::default()).has_errors(), "{source}");
    }
    let position = position_at_offset(&file, offset);
    h.change("a.php", source, version).await;
    let signature = h.at("textDocument/signatureHelp", "a.php", position.line, position.character).await;
    assert_eq!(signature["activeParameter"], parameter, "{source}: {signature}");
}

#[tokio::test]
async fn named_argument_completion_resolves_functions_aliases_and_methods() {
    let mut h = Harness::start(&[("lib.php", LIBRARY), ("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    for (index, (marked, parameter)) in [
        ("<?php \\Demo\\deliver(pr<cursor>);", 1),
        ("<?php namespace Demo; deliver(pr<cursor>);", 1),
        ("<?php namespace App; use function Demo\\deliver as send; send(pr<cursor>);", 1),
        ("<?php new \\Demo\\Child(pr<cursor>);", 1),
        ("<?php \\Demo\\Client::build(pr<cursor>);", 0),
        ("<?php \\Demo\\client()->send(pr<cursor>);", 1),
        ("<?php function f(\\Demo\\Client $c): void { $c->send(pr<cursor>); }", 1),
        ("<?php function f(?\\Demo\\Client $c): void { $c?->send(pr<cursor>); }", 1),
    ]
    .iter()
    .enumerate()
    {
        let version = index as i32 * 2 + 2;
        let (source, items) = complete(&mut h, marked, version).await;
        let item = named(&items, "priority");
        assert_eq!(item["textEdit"]["newText"], "priority: ");
        let (applied, offset) = apply(&source, item, "3");
        assert_eq!(applied, marked.replace("pr<cursor>", "priority: 3"));
        check_binding(&mut h, &applied, offset, *parameter, version + 1).await;
    }
}

#[tokio::test]
async fn named_argument_completion_excludes_bound_parameters_and_preserves_colons() {
    let mut h = Harness::start(&[("lib.php", LIBRARY), ("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    for (version, marked, name, excluded) in [
        (2, "<?php \\Demo\\deliver('x', <cursor>urgent: true, priority: 3);", "urgent", &["message:", "priority:"][..]),
        (
            3,
            "<?php \\Demo\\deliver(message: implode(',', ['a', 'b']), /* comma, */ pr<cursor>iority: 3);",
            "priority",
            &["message:"][..],
        ),
        (4, "<?php \\Demo\\cases(value: 1, <cursor>Value: 2);", "Value", &["value:"][..]),
    ] {
        let (source, items) = complete(&mut h, marked, version).await;
        let item = named(&items, name);
        assert_eq!(item["textEdit"]["newText"], name);
        assert_eq!(apply(&source, item, "").0, source);
        for label in excluded {
            assert!(!named_labels(&items).contains(label), "{marked}: {items:?}");
        }
    }
}

#[tokio::test]
async fn named_argument_completion_agrees_across_union_receivers() {
    let mut h = Harness::start(&[("lib.php", LIBRARY), ("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    let marked = "<?php function f(\\Demo\\Client|\\Demo\\Alternate $c): void { $c->send(<cursor>); }";
    let (source, items) = complete(&mut h, marked, 2).await;
    assert_eq!(named_labels(&items), ["priority:"], "{items:?}");
    let (applied, offset) = apply(&source, named(&items, "priority"), "3");
    assert_eq!(applied, marked.replace("<cursor>", "priority: 3"));
    check_binding(&mut h, &applied, offset, 1, 3).await;
    for (version, marked) in [
        (4, "<?php function f(\\Demo\\Client|\\Demo\\Swapped $c): void { $c->send('x', <cursor>); }"),
        (5, "<?php function f(\\Demo\\Client|string $c): void { $c->send(pr<cursor>); }"),
        (6, "<?php function f(\\Demo\\Client|Missing $c): void { $c->send(pr<cursor>); }"),
    ] {
        let (_, items) = complete(&mut h, marked, version).await;
        assert!(named_labels(&items).is_empty(), "{marked}: {items:?}");
    }
}

#[tokio::test]
async fn named_argument_completion_keeps_each_target_detail_and_description() {
    let library = r"<?php namespace Demo;
class A {
    /** @param int $value Numeric **input**. */
    public function send(int $value = 1): void {}
}
class B {
    /** @param string $value Text **input**. */
    public function send(string $value = 'x'): void {}
}
class InheritedA extends A {}
class InheritedB extends A {}
/** @param int $value Numeric **input**. */
function left(int $value = 1): void {}
/** @param string $value Text **input**. */
function right(string $value = 'x'): void {}
";
    for format in ["markdown", "plaintext"] {
        let mut h = Harness::start_with(
            &[("lib.php", library), ("a.php", "<?php")],
            ServerConfig::default(),
            json!({"textDocument":{"completion":{"completionItem":{"documentationFormat":[format]}}}}),
        )
        .await;
        h.open("a.php", "<?php").await;
        for (index, (marked, owners, declarations)) in [
            (
                r"<?php function inspect(\Demo\A|\Demo\B $receiver): void { $receiver->send(va<cursor>lue: 1); }",
                [r"Demo\A::send", r"Demo\B::send"],
                ["int $value = 1", "string $value = 'x'"],
            ),
            (
                r"<?php function inspect(bool $pick): void { $call = $pick ? \Demo\left(...) : \Demo\right(value: ?); $call(va<cursor>lue: 1); }",
                [r"Demo\left", r"Demo\right"],
                ["int $value = 1", "string $value"],
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let (source, items) = complete(&mut h, marked, index as i32 + 2).await;
            assert_eq!(named_labels(&items), ["value:"], "{items:?}");
            let item = named(&items, "value");
            assert_eq!(item["textEdit"]["newText"], "value");
            assert_eq!(apply(&source, item, "").0, source);
            assert_eq!(item["documentation"]["kind"], format, "{item}");
            let detail = item["detail"].as_str().unwrap();
            let document = item["documentation"]["value"].as_str().unwrap();
            for (owner, declaration) in owners.into_iter().zip(declarations) {
                assert!(detail.contains(owner) && detail.contains(declaration), "{item}");
                assert!(document.contains(owner) && document.contains(declaration), "{item}");
            }
            for description in ["Numeric **input**.", "Text **input**."] {
                assert!(document.contains(description), "{item}");
            }
            assert_eq!(document.contains("```php\n"), format == "markdown", "{item}");
        }
        let (_, items) = complete(
            &mut h,
            r"<?php function inspect(\Demo\InheritedA|\Demo\InheritedB $receiver): void { $receiver->send(va<cursor>lue: 1); }",
            4,
        )
        .await;
        let item = named(&items, "value");
        assert_eq!(item["detail"], "int $value = 1", "{item}");
        assert_eq!(item["documentation"], json!({"kind":format,"value":"Numeric **input**."}), "{item}");
    }
}

#[tokio::test]
async fn named_argument_completion_hides_anonymous_callable_ids() {
    let marked = r"<?php
function inspect(bool $pick): void {
    $call = $pick
        ? function(int $value = 1): int { return $value; }
        : fn(string $value = 'x'): string => $value;
    $call(va<cursor>lue: 1);
}";
    let mut h = Harness::start(&[("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    let (source, items) = complete(&mut h, marked, 2).await;
    assert_eq!(named_labels(&items), ["value:"], "{items:?}");
    let item = named(&items, "value");
    assert_eq!(apply(&source, item, "").0, source);
    let detail = item["detail"].as_str().unwrap();
    assert!(detail.contains("closure: int $value = 1"), "{item}");
    assert!(detail.contains("arrow function: string $value = 'x'"), "{item}");
    let document = item["documentation"]["value"].as_str().unwrap();
    for text in [detail, document] {
        assert!(text.contains("closure") && text.contains("arrow function"), "{item}");
        assert!(!text.contains("{closure:") && !text.contains("a.php"), "{item}");
    }
}

#[tokio::test]
async fn named_argument_completion_handles_empty_and_unfinished_calls() {
    let mut h = Harness::start(&[("lib.php", LIBRARY), ("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    for (version, marked) in [
        (2, "<?php \\Demo\\deliver(<cursor>);"),
        (3, "<?php \\Demo\\deliver(message: 'x', pr<cursor>"),
        (4, "<?php $c = new \\Demo\\Client(); $c->send(pr<cursor>"),
    ] {
        let (source, items) = complete(&mut h, marked, version).await;
        let item = named(&items, "priority");
        let expected = if marked.contains("pr<cursor>") {
            marked.replace("pr<cursor>", "priority: 3")
        } else {
            marked.replace("<cursor>", "priority: 3")
        };
        assert_eq!(apply(&source, item, "3").0, expected);
    }
}

#[tokio::test]
async fn named_argument_completion_handles_unclosed_calls_before_following_code() {
    let mut h = Harness::start(&[("lib.php", LIBRARY), ("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    for (version, marked) in [
        (2, "<?php \\Demo\\deliver(<cursor>\n$x = 1;"),
        (3, "<?php function f() { \\Demo\\deliver(message: 'x', pr<cursor>\n}"),
        (4, "<?php function f($a) { new \\Demo\\Client(<cursor>\n    if ($a) { b(); }\n}"),
        (5, "<?php function f() { \\Demo\\deliver(fn() => 1, pr<cursor>\n    $x = [1, 2];\n}"),
        (6, "<?php $c = new \\Demo\\Client(); $c->send(<cursor>\nreturn $c;"),
    ] {
        let (source, items) = complete(&mut h, marked, version).await;
        let item = named(&items, "priority");
        assert_eq!(item["textEdit"]["newText"], "priority: ", "{marked}");
        let expected = if marked.contains("pr<cursor>") {
            marked.replace("pr<cursor>", "priority: 3")
        } else {
            marked.replace("<cursor>", "priority: 3")
        };
        assert_eq!(apply(&source, item, "3").0, expected);
    }
    let marked = "<?php function f() { \\Demo\\deliver(pr<cursor>iority: 3\n}";
    let (source, items) = complete(&mut h, marked, 7).await;
    let item = named(&items, "priority");
    assert_eq!(item["textEdit"]["newText"], "priority");
    assert_eq!(apply(&source, item, "").0, source);
    for (index, marked) in [
        "<?php \\Demo\\deliver(['v' => pr<cursor>\n$x = 1;",
        "<?php function f() { \\Demo\\deliver(message: pr<cursor>\n}",
        "<?php \\Demo\\deliver('x' . pr<cursor>\n$x = 1;",
    ]
    .iter()
    .enumerate()
    {
        let (_, items) = complete(&mut h, marked, index as i32 + 8).await;
        assert!(named_labels(&items).is_empty(), "{marked}: {items:?}");
    }
}

#[tokio::test]
async fn named_argument_completion_keeps_unclosed_call_arguments_on_the_cursor_line() {
    let mut h = Harness::start(&[("lib.php", LIBRARY), ("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    let mut version = 1;
    for (arguments, expected) in [
        ("<cursor>, 2", &[][..]),
        ("pr<cursor>, 2", &[][..]),
        ("<cursor>, 2, // note\n    3", &[][..]),
        ("<cursor>, message: 'x'", &["priority:", "urgent:"][..]),
        ("'x', <cursor>, urgent: true", &["priority:"][..]),
        ("<cursor> /* note */, message: 'x'", &["priority:", "urgent:"][..]),
    ] {
        for marked in [
            format!("<?php \\Demo\\deliver({arguments}"),
            format!("<?php function f() {{\n    \\Demo\\deliver({arguments}\n    return 1;\n}}"),
        ] {
            version += 1;
            let (_, items) = complete(&mut h, &marked, version).await;
            let mut labels = named_labels(&items);
            labels.sort_unstable();
            assert_eq!(labels, expected, "{marked}: {items:?}");
        }
    }
}

#[tokio::test]
async fn named_argument_completion_keeps_later_lines_of_calls_unclosed_at_end_of_file() {
    let mut h = Harness::start(&[("lib.php", LIBRARY), ("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    for (version, (marked, expected)) in [
        ("<?php \\Demo\\deliver('x',\n    <cursor>\n    , urgent: true", &["priority:"][..]),
        ("<?php \\Demo\\deliver(<cursor> // c\n    , message: 'x'", &["priority:", "urgent:"][..]),
        ("<?php \\Demo\\deliver(<cursor>\r\n    , message: 'x'", &["priority:", "urgent:"][..]),
        ("<?php \\Demo\\deliver(<cursor> /* multi\n line */, message: 'x'", &["priority:", "urgent:"][..]),
        (
            "<?php function f() {\n    \\Demo\\deliver(<cursor> /* multi\n line */, message: 'x'\n    return 1;\n}",
            &["priority:", "urgent:"][..],
        ),
        (
            "<?php function f() {\n    \\Demo\\deliver(<cursor> /** multi\n line */, message: 'x'\n    return 1;\n}",
            &["priority:", "urgent:"][..],
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let (_, items) = complete(&mut h, marked, version as i32 + 2).await;
        let mut labels = named_labels(&items);
        labels.sort_unstable();
        assert_eq!(labels, expected, "{marked}: {items:?}");
    }
}

#[tokio::test]
async fn named_argument_completion_uses_property_types_for_static_calls() {
    let mut h = Harness::start(&[("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    for (version, (head, expected)) in [
        ("foreach ($this->kind::create(<cursor>", &["name:", "size:"][..]),
        ("foreach ($this->kind::create('a', <cursor>", &["size:"][..]),
        ("$x = match ($this->kind::create(<cursor>", &["name:", "size:"][..]),
        ("if ($this->kind::create(<cursor>", &["name:", "size:"][..]),
    ]
    .into_iter()
    .enumerate()
    {
        let marked = format!(
            "<?php
class Kind {{ public static function create(bool $flag, string $mode = 'x'): Kind {{ return new Kind(); }} }}
class Bus {{ public static function create(string $name, int $size = 0): Bus {{ return new Bus(); }} }}
final class Controller {{
    private Bus $kind;
    public function run(): int {{
        {head}
        return 1;
    }}
}}"
        );
        let (_, items) = complete(&mut h, &marked, version as i32 + 2).await;
        let mut labels = named_labels(&items);
        labels.sort_unstable();
        assert_eq!(labels, expected, "{marked}: {items:?}");
    }
}

#[tokio::test]
async fn named_argument_completion_declines_values_unpacking_magic_and_declarations() {
    let mut h = Harness::start(&[("lib.php", LIBRARY), ("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    for (index, marked) in [
        "<?php \\Demo\\deliver(message: pr<cursor>);",
        "<?php \\Demo\\deliver(['value' => pr<cursor>]);",
        "<?php \\Demo\\deliver('x' . pr<cursor>);",
        "<?php \\Demo\\deliver(...$values, pr<cursor>);",
        "<?php \\Demo\\deliver(...<cursor>);",
        "<?php \\Demo\\positional(pr<cursor>);",
        "<?php (new \\Demo\\Magic())->send(pr<cursor>);",
        "<?php function deliver(pr<cursor>) {}",
        "<?php $fn = function ($priority) use (pr<cursor>) {};",
        "<?php $fn = fn(pr<cursor>) => 1;",
        "<?php \\Demo\\deliver(function () { pr<cursor>; });",
    ]
    .iter()
    .enumerate()
    {
        let (_, items) = complete(&mut h, marked, index as i32 + 2).await;
        assert!(named_labels(&items).is_empty(), "{marked}: {items:?}");
    }
}

#[tokio::test]
async fn named_argument_completion_obeys_php_version_and_plain_unicode_edits() {
    let mut config = ServerConfig::default();
    config.configuration.php_version = PHPVersion::PHP74;
    let mut h = Harness::start_with(&[("a.php", "<?php function send($priority = 0) {}")], config, json!({})).await;
    h.open("a.php", "<?php function send($priority = 0) {}").await;
    let (_, items) = complete(&mut h, "<?php function send($priority = 0) {} send(pr<cursor>);", 2).await;
    assert!(named_labels(&items).is_empty(), "{items:?}");
    for snippets in [false, true] {
        let capabilities = json!({"textDocument": {"completion": {"completionItem": {"snippetSupport": snippets}}}});
        let mut h =
            Harness::start_with(&[("lib.php", LIBRARY), ("a.php", "<?php")], ServerConfig::default(), capabilities)
                .await;
        h.open("a.php", "<?php").await;
        let marked = "<?php /* 😀 */ \\Demo\\café(名<cursor>);";
        let (source, items) = complete(&mut h, marked, 2).await;
        let item = named(&items, "名");
        assert_eq!(item["textEdit"]["newText"], "名: ");
        let (applied, offset) = apply(&source, item, "'x'");
        assert_eq!(applied, marked.replace("名<cursor>", "名: 'x'"));
        check_binding(&mut h, &applied, offset, 0, 3).await;
    }
}

#[tokio::test]
async fn named_completion_uses_remaining_partial_parameters() {
    let mut h = Harness::start(&[("lib.php", LIBRARY), ("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    for (index, (source, expected)) in [
        (r"$partial = \Demo\deliver(message: 'x', priority: ?, urgent: ?); $partial(<cursor>);", vec!["priority:", "urgent:"]),
        (r"$partial = \Demo\deliver(urgent: ?, priority: ?, message: 'x'); $partial(true, <cursor>);", vec!["priority:"]),
        (r"$partial = \Demo\deliver(message: 'x', priority: ?, urgent: ?); $again = $partial(priority: ?, urgent: true); $again(<cursor>);", vec!["priority:"]),
        (r"function run(bool $pick) { $partial = $pick ? \Demo\deliver(message: '', priority: ?, urgent: false) : \Demo\deliver(message: '', priority: 1, urgent: ?); $partial(<cursor>); }", vec![]),
    ].into_iter().enumerate() {
        let (_, items) = complete(&mut h, &format!("<?php {source}"), index as i32 + 2).await;
        let mut labels = named_labels(&items);
        labels.sort_unstable();
        assert_eq!(labels, expected, "{source}: {items:?}");
    }
}
