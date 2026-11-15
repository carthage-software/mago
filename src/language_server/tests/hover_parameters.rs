use super::harness::Harness;

async fn hover(h: &mut Harness, source: &str, marker: &str) -> serde_json::Value {
    let offset = source.find(marker).unwrap();
    let prefix = &source[..offset];
    h.at(
        "textDocument/hover",
        "a.php",
        prefix.bytes().filter(|byte| *byte == b'\n').count() as u32,
        prefix.rsplit('\n').next().unwrap().encode_utf16().count() as u32 + 1,
    )
    .await
}

#[tokio::test]
async fn hover_parameter_declarations_keep_type_default_reference_and_variadic() {
    let source = r"<?php
function change(string &$text = 'draft', int ...$items): void {}
class Service { public function run(?int $count = null): void {} }
$closure = function (float $rate = 1.5): void {};
$arrow = fn (bool $active = true): bool => $active;
$object = new class { public function send(string $payload = 'body'): void {} };
";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for (marker, declaration) in [
        ("$text =", "string &$text = 'draft'"),
        ("$items)", "int ...$items"),
        ("$rate =", "float $rate = 1.5"),
        ("$active =", "bool $active = true"),
        ("$payload =", "string $payload = 'body'"),
    ] {
        let value = hover(&mut h, source, marker).await;
        assert_eq!(value["contents"]["value"], declaration, "{marker}: {value}");
    }
    let value = hover(&mut h, source, "$count =").await;
    let text = value["contents"]["value"].as_str().unwrap();
    assert!(text.contains("int") && text.contains("null") && text.ends_with("$count = null"), "{value}");
}

#[tokio::test]
async fn hover_parameter_declarations_use_phpdoc_type_and_exact_scope() {
    let source = r"<?php
/** @param non-empty-string $value */
function outer(string $value): void {
    $inner = function (int $value): int { return $value; };
    $arrow = fn (bool $value): bool => $value;
}
";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for (marker, declaration) in
        [("$value): void", "non-empty-string $value"), ("$value): int", "int $value"), ("$value): bool", "bool $value")]
    {
        let value = hover(&mut h, source, marker).await;
        assert_eq!(value["contents"]["value"], declaration, "{marker}: {value}");
    }
}

#[tokio::test]
async fn hover_parameter_declarations_preserve_promoted_property_precedence() {
    let source = "<?php class Record { public function __construct(public string $title) {} }";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let value = hover(&mut h, source, "$title").await;
    assert_eq!(value["contents"]["value"], "Record\n\npublic string $title");
}

#[tokio::test]
async fn hover_parameter_descriptions_belong_to_the_exact_binding() {
    let source = r"<?php
/** @param string $value Outer value. */
function outer(string $value): void {
    $inner = /** @param int $value Inner value. */ function (int $value): int { return $value; };
    $arrow = fn (bool $value): bool => $value;
}
class Service {
    /**
     * Sends an item.
     * @param string $titles Other values.
     * @param string $title The title to send.
     * @return string The receipt.
     */
    public function send(string $title): string { return $title; }
}
";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for (marker, expected) in [
        ("$value): void", "string $value\n\nOuter value."),
        ("$value): int", "int $value\n\nInner value."),
        ("$value): bool", "bool $value"),
        ("$title): string", "string $title\n\nThe title to send."),
    ] {
        assert_eq!(hover(&mut h, source, marker).await["contents"]["value"], expected, "{marker}");
    }
}

#[tokio::test]
async fn hover_parameter_reads_keep_docs_flow_types_and_capture_bindings() {
    let source = r"<?php
/** @param ?string $value Outer value. */
function outer(?string $value): void {
    if ($value === null) { return; }
    echo $value; // narrowed
    $closure = function () use ($value): string { return $value; /* captured */ };
    $arrow = fn (): string => $value; // arrow
    $shadow = /** @param int $value Inner value. */ function (int $value): int { return $value; /* shadowed */ };
    $local = function (): int { $value = 1; return $value; /* local */ };
    $value = 'changed';
    echo $value; // assigned
}
";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for (marker, expected) in [
        ("$value; // narrowed", "string $value\n\nOuter value."),
        ("$value; /* captured */", "string $value\n\nOuter value."),
        ("$value; // arrow", "string $value\n\nOuter value."),
        ("$value; /* shadowed */", "int $value\n\nInner value."),
        ("$value; /* local */", "int $value = 1"),
        ("$value; // assigned", "string $value = 'changed'\n\nOuter value."),
    ] {
        let value = hover(&mut h, source, marker).await;
        assert_eq!(value["contents"]["value"], expected, "{marker}: {value}");
        let prefix = &source[..source.find(marker).unwrap()];
        let column = prefix.rsplit('\n').next().unwrap().encode_utf16().count();
        assert_eq!(value["range"]["start"]["character"], column);
        assert_eq!(value["range"]["end"]["character"], column + "$value".len());
    }
    h.open("a.php", source).await;
    let changed = source.replace("Outer value.", "Fresh value.");
    h.change("a.php", &changed, 2).await;
    assert_eq!(
        hover(&mut h, &changed, "$value; // narrowed").await["contents"]["value"],
        "string $value\n\nFresh value.",
    );
}

#[tokio::test]
async fn hover_promoted_parameter_reads_keep_constructor_docs() {
    let source = r"<?php
class Record {
    /** @param string $title The record title. */
    public function __construct(public string $title) { echo $title; }
}
";
    let mut h = Harness::start(&[("a.php", source)]).await;
    assert_eq!(hover(&mut h, source, "$title;").await["contents"]["value"], "string $title\n\nThe record title.",);
}

#[tokio::test]
async fn hover_parameter_descriptions_keep_markdown_without_other_tags() {
    let source = r"<?php
/**
 * Does work.
 * @param $name The **name**.
 *
 * ```php
 * @param int $fake
 * ```
 * @custom Other details.
 */
function run(string $name): void {}
";
    let mut h = Harness::start_with(
        &[("a.php", source)],
        crate::language_server::ServerConfig::default(),
        serde_json::json!({"textDocument":{"hover":{"contentFormat":["markdown"]}}}),
    )
    .await;
    assert_eq!(
        hover(&mut h, source, "$name):").await["contents"]["value"],
        "```php\nstring $name\n```\n\n---\n\nThe **name**.\n\n```php\n@param int $fake\n```",
    );
}

async fn assert_argument_hover(h: &mut Harness, source: &str, marker: &str, expected: &str) {
    let value = hover(h, source, marker).await;
    assert_eq!(value["contents"]["value"], expected, "{marker}: {value}");
    let prefix = &source[..source.find(marker).unwrap()];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() as u32;
    let column = prefix.rsplit('\n').next().unwrap().encode_utf16().count() as u32;
    let length = marker.split(':').next().unwrap().encode_utf16().count() as u32;
    assert_eq!(
        value["range"],
        serde_json::json!({
            "start": {"line": line, "character": column},
            "end": {"line": line, "character": column + length},
        }),
        "{marker}",
    );
}

#[tokio::test]
async fn hover_named_arguments_use_inherited_constructor_docs_and_label_ranges() {
    let parent = r"<?php namespace Lib;
#[\Attribute] class Base {
    /** @param positive-int $bar Number of items. */
    public function __construct(public int $bar = 1) {}
}
#[\Attribute] class Child extends Base {}
";
    let source = r"<?php namespace App;
use Lib\Child as Alias;
#[Alias(bar: 1)] class First {}
#[Alias(bar: 2), \Lib\Child(bar: 3)] class Second {}
/* 😀 */ $child = new Alias(bar: 4);
$partial = $child->__construct(bar: ?);
";
    let mut h = Harness::start_with(
        &[("a.php", source), ("lib.php", parent)],
        crate::language_server::ServerConfig::default(),
        serde_json::json!({"textDocument":{"hover":{"contentFormat":["markdown"]}}}),
    )
    .await;
    let expected = "```php\npositive-int $bar = 1\n```\n\n---\n\nNumber of items.";
    for marker in ["bar: 1", "bar: 2", "bar: 3", "bar: 4", "bar: ?"] {
        assert_argument_hover(&mut h, source, marker, expected).await;
    }
    h.open("lib.php", parent).await;
    h.change("lib.php", &parent.replace("Number of items.", "Updated count."), 2).await;
    assert_argument_hover(&mut h, source, "bar: 4", &expected.replace("Number of items.", "Updated count.")).await;
}

#[tokio::test]
async fn hover_named_arguments_cover_functions_methods_partial_calls_and_callables() {
    let source = r"<?php namespace App;
/** @param non-empty-string $text Text to send. */
function send(string $text): void {}
class Service {
    /** @param int $limit Result limit. */
    public function read(int $limit = 2): void {}
    /** @param int $limit Result limit. */
    public static function fetch(int $limit = 2): void {}
}
send(text: 'direct');
$partial = send(text: ?);
$partial(text: 'partial');
$again = $partial(text: ?);
$again(text: 'nested');
$callable = send(...);
$callable(text: 'callable');
($callable)(text: 'parenthesized');
$callbacks = [$callable];
$callbacks[0](text: 'indexed');
$service = new Service();
$service->read(limit: 3);
$service?->read(limit: 4);
Service::fetch(limit: 5);
$partialMethod = $service->read(limit: ?);
$partialMethod(limit: 9);
$method = Service::fetch(...);
$method(limit: 6);
$closure = /** @param bool $enabled Enable work. */ function (bool $enabled): void {};
$closure(enabled: true);
$arrow = fn(int $value): int => $value;
$arrow(value: 7);
/** @param string $café The label. */
function label(string $café): void {}
/* 😀 */ label(café: 'hello');
function keyword(int $class): void {}
keyword(class: 8);
";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for marker in [
        "text: 'direct'",
        "text: ?",
        "text: 'callable'",
        "text: 'parenthesized'",
        "text: 'indexed'",
        "text: 'partial'",
        "text: 'nested'",
    ] {
        assert_argument_hover(&mut h, source, marker, "non-empty-string $text\n\nText to send.").await;
    }
    for marker in ["limit: 3", "limit: 4", "limit: 5", "limit: ?", "limit: 6"] {
        assert_argument_hover(&mut h, source, marker, "int $limit = 2\n\nResult limit.").await;
    }
    assert_argument_hover(&mut h, source, "limit: 9", "int $limit = 2\n\nResult limit.").await;
    assert_argument_hover(&mut h, source, "enabled: true", "bool $enabled\n\nEnable work.").await;
    assert_argument_hover(&mut h, source, "value: 7", "int $value").await;
    assert_argument_hover(&mut h, source, "café: 'hello'", "string $café\n\nThe label.").await;
    assert_argument_hover(&mut h, source, "class: 8", "int $class").await;
}

#[tokio::test]
async fn hover_named_arguments_decline_unknown_or_uncertain_bindings() {
    let source = r"<?php
class Known { public function send(int $value): void {} }
class Unknown {}
function mixed_call(Known|Unknown $object): void { $object->send(value: 1); }
function unknown(callable $callback): void { $callback(value: 2); }
missing(value: 3);
new Missing(value: 4);
#[Missing(value: 5)] class Tagged {}
function known(int $value): void {}
known(other: 6);
/** @method void send(int $value) */
class Magic { public function __call(string $method, array $arguments): mixed { return null; } }
(new Magic())->send(value: 7);
known(true ? value: 8);
value: goto value;
$text = 'known(value: 9)';
";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for marker in
        ["value: 1", "value: 2", "value: 3", "value: 4", "value: 5", "other: 6", "value: 7", "value: 8", "value: goto"]
    {
        assert!(hover(&mut h, source, marker).await.is_null(), "{marker}");
    }
    assert_eq!(hover(&mut h, source, "value: 9").await["contents"]["value"], "string\n'known(value: 9)'");
}

#[tokio::test]
async fn named_argument_hover_qualifies_colliding_types_across_callable_targets() {
    let source = r"<?php
namespace Left {
    interface Item {}
    enum State { case Open; }
    function take(Item $value, int $other = 0): Item { return $value; }
    /** @param array{state: State, label: 'Right\State'} $value */
    function shape(array $value, int $other = 0): void {}
}
namespace Right {
    interface Item {}
    enum State { case Open; }
    function take(Item $value, int $other = 0): Item { return $value; }
    /** @param array{state: State, label: 'Right\State'} $value */
    function shape(array $value, int $other = 0): void {}
    /**
     * @template T
     * @param T $seed
     * @param T $value
     * @return T
     */
    function bind(mixed $seed, mixed $value, int $other = 0): mixed { return $value; }
}
namespace App {
    function inspect(bool $pick, \Left\Item&\Right\Item $value, \Right\Item $right): void {
        $full = $pick ? \Left\take(...) : \Right\take(...);
        $partial = $pick ? \Left\take(value: ?, other: 1) : \Right\take(value: ?, other: 1);
        $mixed = $pick ? \Left\take(...) : \Right\take(value: ?, other: 1);
        $generic = $pick ? \Left\take(...) : \Right\bind(seed: $right, value: ?, other: 1);
        $shape = $pick ? \Left\shape(...) : \Right\shape(value: ?, other: 1);
        $full(value: $value);
        $partial(value: ($value));
        $mixed(value: $value /* mixed */);
        $generic(value: $value /* generic */);
        $shape(value: ['state' => \Left\State::Open, 'label' => 'Right\State']);
    }
}
";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for (marker, left) in [
        ("value: $value);", r"\Left\Item"),
        ("value: ($value)", r"\Left\Item"),
        ("value: $value /* mixed */", r"\Left\Item"),
        ("value: $value /* generic */", r"\Left\Item"),
    ] {
        let result = hover(&mut h, source, marker).await;
        let text = result["contents"]["value"].as_str().unwrap();
        assert!(text.contains(&format!("{left} $value")), "{marker}: {result}");
        assert!(text.contains(r"\Right\Item $value"), "{marker}: {result}");
        assert_eq!(text.matches("$value").count(), 2, "{marker}: {result}");

        let before = &source[..source.find(marker).unwrap() + "value: ".len()];
        let result = h
            .at(
                "textDocument/signatureHelp",
                "a.php",
                before.bytes().filter(|byte| *byte == b'\n').count() as u32,
                before.rsplit('\n').next().unwrap().encode_utf16().count() as u32,
            )
            .await;
        let signatures = result["signatures"].as_array().unwrap();
        assert_eq!(signatures.len(), 2, "{marker}: {result}");
        for expected in [left, r"\Right\Item"] {
            assert!(
                signatures.iter().any(|signature| {
                    let label = signature["label"].as_str().unwrap();
                    label.contains(&format!("({expected} $value")) && label.ends_with(&format!("): {expected}"))
                }),
                "{marker}: {result}"
            );
        }
    }
    let result = hover(&mut h, source, "value: ['state'").await;
    let text = result["contents"]["value"].as_str().unwrap();
    for state in [r"\Left\State", r"\Right\State"] {
        assert!(
            text.contains(&format!(r"array{{'label': string('Right\State'), 'state': {state}}} $value")),
            "{result}"
        );
    }
    assert!(!text.contains("enum("), "{result}");
}

#[tokio::test]
async fn partial_argument_hover_uses_exact_type_display_for_changed_shapes_and_collisions() {
    let source = r"<?php
namespace Left; class User {}
namespace Right; class User {}
namespace App;
class Payload {}
/**
 * @template T
 * @param T $seed
 * @param array{label: 'App\Payload', value: T} $value Changed payload.
 */
function shape(mixed $seed, array $value): void {}
function link(\Left\User $left, \Right\User $right): \Left\User { return $left; }
$shape = shape(seed: new Payload(), value: ?);
$shape(value: ['label' => 'App\\Payload', 'value' => new Payload()]);
$link = link(left: ?, right: ?);
$link(left: new \Left\User(), right: new \Right\User());
";
    let mut h = Harness::start(&[("a.php", source)]).await;
    assert_argument_hover(
        &mut h,
        source,
        "value: ['label'",
        "array{'label': string('App\\Payload'), 'value': Payload} $value\n\nChanged payload.",
    )
    .await;
    assert_argument_hover(&mut h, source, "left: new", r"\Left\User $left").await;
    assert_argument_hover(&mut h, source, "right: new", r"\Right\User $right").await;
}

#[tokio::test]
async fn closure_argument_hover_keeps_bound_class_types_docs_and_defaults() {
    let source = r"<?php
class Base {}
class Child extends Base {
    function run(): void {
        $closure = /** @param parent $value The parent value. */
            function (parent $value, int $count = 2): parent { return $value; };
        $arrow = /** @param self $value The child value. */ fn(self $value): self => $value;
        $closure(value: new Base(), count: 3);
        $arrow(value: new Child());
    }
}
";
    let mut h = Harness::start(&[("a.php", source)]).await;
    assert_argument_hover(&mut h, source, "value: new Base", "Base $value\n\nThe parent value.").await;
    assert_argument_hover(&mut h, source, "count: 3", "int $count = 2").await;
    assert_argument_hover(&mut h, source, "value: new Child", "Child $value\n\nThe child value.").await;
}

#[tokio::test]
async fn closure_conditional_types_keep_class_context_in_hover_and_signature() {
    let source = r"<?php
class Base {}
class Child extends Base {
    function run(bool $flag, Base $value): Base {
        $copy = /**
         * @param ($flag is true ? parent : self) $item Selected value.
         * @return ($flag is true ? parent : self)
         */ static function (bool $flag, Base $item): Base { throw new \RuntimeException(); };
        return $copy(flag: $flag, item: $value);
    }
}
";
    let mut h = Harness::start(&[("a.php", source)]).await;
    assert_argument_hover(&mut h, source, "item: $value", "($flag is true ? Base : Child) $item\n\nSelected value.")
        .await;
    let before = &source[..source.find("item: $value").unwrap() + "item: ".len()];
    let signature = h
        .at(
            "textDocument/signatureHelp",
            "a.php",
            before.bytes().filter(|byte| *byte == b'\n').count() as u32,
            before.rsplit('\n').next().unwrap().encode_utf16().count() as u32,
        )
        .await;
    let label = signature["signatures"][0]["label"].as_str().unwrap();
    assert!(
        label.ends_with("(bool $flag, ($flag is true ? Base : Child) $item): ($flag is true ? Base : Child)"),
        "{signature}"
    );
    assert_eq!(signature["activeParameter"], 1);
}
