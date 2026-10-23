use serde_json::Value;

use super::harness::Harness;

const DECLARATIONS: &str = r"<?php
class Worker {
    public function __construct(string $name, int $count = 1) {}
    public function run(string $name, int $count = 1): bool { return true; }
    public static function build(string $name, int $count = 1): self { return new self($name, $count); }
}
class Child extends Worker {}
class Other {
    public function run(int $count, string $name): bool { return true; }
}
function work(mixed $first, mixed $second, mixed $third = null): void {}
function inner(int $left, int $right): int { return $left + $right; }
function café(string &$名, string $suffix = '😀', int ...$rest): string { return $名; }
function collect(int $first = 1, int $second = 2, int ...$values): void {}
";

async fn signature(h: &mut Harness, source: &str, version: i32) -> Value {
    let offset = source.find("<cursor>").unwrap();
    let before = &source[..offset];
    let line = before.bytes().filter(|byte| *byte == b'\n').count() as u32;
    let character = before.rsplit('\n').next().unwrap().encode_utf16().count() as u32;
    let source = source.replacen("<cursor>", "", 1);
    h.change("a.php", &source, version).await;
    h.at("textDocument/signatureHelp", "a.php", line, character).await
}

fn check(result: &Value, label: &str, active: u32) {
    assert_eq!(result["signatures"][0]["label"], label, "{result}");
    assert_eq!(result["activeParameter"], active, "{result}");
    assert_eq!(result["signatures"][0]["activeParameter"], active, "{result}");
}

#[tokio::test]
async fn signature_help_resolves_methods_and_constructors() {
    let mut h = Harness::start(&[("lib.php", DECLARATIONS), ("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    for (index, (source, label)) in [
        ("$w = new Worker('x'); $w->run('x', <cursor>2);", "function Worker::run(string $name, int $count = 1): bool"),
        ("$w = new Worker('x'); $w?->run('x', <cursor>2);", "function Worker::run(string $name, int $count = 1): bool"),
        ("Child::build('x', <cursor>2);", "function Worker::build(string $name, int $count = 1): Worker"),
        ("new Child('x', <cursor>2);", "function Child::__construct(string $name, int $count = 1)"),
        ("Worker::build('x')->run('x', <cursor>2);", "function Worker::run(string $name, int $count = 1): bool"),
        ("new Worker('x', <cursor>", "function Worker::__construct(string $name, int $count = 1)"),
        ("Worker::build('x', <cursor>", "function Worker::build(string $name, int $count = 1): Worker"),
        ("$w = new Worker('x'); $w->run('x', <cursor>", "function Worker::run(string $name, int $count = 1): bool"),
    ]
    .iter()
    .enumerate()
    {
        let result = signature(&mut h, &format!("<?php\n{source}"), index as i32 + 2).await;
        assert!(!result.is_null(), "no signature for {source}");
        check(&result, label, 1);
    }
}

#[tokio::test]
async fn incomplete_signature_help_uses_the_anonymous_class_scope() {
    let mut h = Harness::start(&[("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    for (index, (receiver, parameter)) in
        [("self", "$inner"), ("static", "$inner"), ("parent", "$base")].iter().enumerate()
    {
        let source = format!(
            "<?php
            class Base {{ public static function send(string $base): void {{}} }}
            class Outer {{
                public static function send(string $outer): void {{}}
                public function make(): object {{
                    return new class extends Base {{
                        public static function send(string $inner): void {{}}
                        public function run(): void {{ {receiver}::send(<cursor>
                        }}
                    }};
                }}
            }}"
        );
        let result = signature(&mut h, &source, index as i32 + 2).await;
        let label = result["signatures"][0]["label"].as_str().expect("anonymous class signature");
        assert!(label.ends_with(&format!("::send(string {parameter}): void")), "{result}");
        assert!(!label.contains("Outer::"), "{result}");
        assert_eq!(result["activeParameter"], 0, "{result}");
    }
}

#[tokio::test]
async fn signature_help_resolves_callable_expressions() {
    let mut h = Harness::start(&[("lib.php", DECLARATIONS), ("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    for (index, source) in [
        "$callback = inner(...); $callback(right: <cursor>2, left: 1);",
        "$callback = inner(...); ($callback)(right: <cursor>2, left: 1);",
        "$callback = inner(...); (($callback))(right: <cursor>2, left: 1);",
        "(inner(...))(right: <cursor>2, left: 1);",
        "$callbacks = [inner(...)]; $callbacks[0](right: <cursor>2, left: 1);",
        "$callbacks = [inner(...)]; ($callbacks[0])(right: <cursor>2, left: 1);",
        "'inner'(right: <cursor>2, left: 1);",
    ]
    .iter()
    .enumerate()
    {
        let result = signature(&mut h, &format!("<?php\n{source}"), index as i32 + 2).await;
        assert!(!result.is_null(), "no signature for {source}");
        check(&result, "function inner(int $left, int $right): int", 1);
    }
}

#[tokio::test]
async fn signature_help_does_not_reuse_unresolved_callable_expressions() {
    let mut h = Harness::start(&[("lib.php", DECLARATIONS), ("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    for (index, source) in [
        "$callback = inner(...); $callback = 42; ($callback)(right: <cursor>2, left: 1);",
        "$callbacks = [inner(...)]; $callbacks[0] = 42; $callbacks[0](right: <cursor>2, left: 1);",
        "$callbacks = [inner(...)]; $callbacks[1](right: <cursor>2, left: 1);",
        "(42)(right: <cursor>2, left: 1);",
        "'missing'(right: <cursor>2, left: 1);",
    ]
    .iter()
    .enumerate()
    {
        let result = signature(&mut h, &format!("<?php\n{source}"), index as i32 + 2).await;
        assert!(result.is_null(), "unexpected signature for {source}: {result}");
    }
}

#[tokio::test]
async fn incomplete_signature_help_does_not_reuse_old_or_outer_variable_types() {
    let mut h = Harness::start(&[("lib.php", DECLARATIONS), ("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    for (index, source) in [
        "$w = new Worker('x'); $w = 42; $w->run('x', <cursor>",
        "$w = new Worker('x'); function f(): void { $w->run('x', <cursor>",
        "$w = new Worker('x'); function f(int $w): void { $w->run('x', <cursor>",
        "$w = new Worker('x'); $closure = function (int $w): void { $w->run('x', <cursor>",
    ]
    .iter()
    .enumerate()
    {
        let result = signature(&mut h, &format!("<?php\n{source}"), index as i32 + 2).await;
        assert!(result.is_null(), "unexpected signature for {source}: {result}");
    }
}

#[tokio::test]
async fn incomplete_signature_help_uses_parameters_and_skips_complete_closures() {
    let mut h = Harness::start(&[("lib.php", DECLARATIONS), ("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    for (index, source) in [
        "function f(Worker $w): void {\n    foreach ($w->run('x', <cursor>\n    return;\n}",
        "$w = new Worker('x');\n$double = fn (int $v): int => $v * 2;\nforeach ($w->run('x', <cursor>\n$x = 1;",
    ]
    .iter()
    .enumerate()
    {
        let result = signature(&mut h, &format!("<?php\n{source}"), index as i32 + 2).await;
        assert!(!result.is_null(), "no signature for {source}");
        check(&result, "function Worker::run(string $name, int $count = 1): bool", 1);
    }
}

#[tokio::test]
async fn incomplete_signature_help_skips_static_and_dynamic_property_names() {
    let mut h = Harness::start(&[("lib.php", DECLARATIONS), ("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    for (index, receiver) in
        ["$w", "Holder::$w", "static::$w", "Holder::$w?", "$this->$w", "$$w", "$this->holder::$w"].iter().enumerate()
    {
        let source = format!(
            "<?php
final class Holder {{
    public static int $w = 0;
    private Holder $holder;
    public function f(Worker $w): void {{
        foreach ({receiver}->run('x', <cursor>
        return;
    }}
}}"
        );
        let result = signature(&mut h, &source, index as i32 + 2).await;
        if index == 0 {
            check(&result, "function Worker::run(string $name, int $count = 1): bool", 1);
        } else {
            assert!(result.is_null(), "unexpected signature for {receiver}: {result}");
        }
    }
}

#[tokio::test]
async fn incomplete_signature_help_resolves_property_fetch_receivers() {
    let mut h = Harness::start(&[("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    for (index, (head, label)) in [
        ("foreach ($this->kind::create('a', <cursor>", "function Bus::create(string $name, int $size = 0): Bus"),
        ("$x = match ($this->kind::create('a', <cursor>", "function Bus::create(string $name, int $size = 0): Bus"),
        ("if ($this->kind::create('a', <cursor>", "function Bus::create(string $name, int $size = 0): Bus"),
        ("foreach ($this->kind->send('a', <cursor>", "function Bus::send(string $body, int $size = 0): void"),
        ("foreach ($o->send('a', <cursor>", "function Node::send(string $body, int $level = 0): void"),
    ]
    .iter()
    .enumerate()
    {
        let source = format!(
            "<?php
class Kind {{ public static function create(bool $flag, string $mode = 'x'): Kind {{ return new Kind(); }} }}
class Bus {{
    public static function create(string $name, int $size = 0): Bus {{ return new Bus(); }}
    public function send(string $body, int $size = 0): void {{}}
}}
class Node {{ public function send(string $body, int $level = 0): void {{}} }}
final class Controller extends Node {{
    private Bus $kind;
    public function run(parent $o): int {{
        {head}
        return 1;
    }}
}}"
        );
        let result = signature(&mut h, &source, index as i32 + 2).await;
        assert!(!result.is_null(), "no signature for {head}");
        check(&result, label, 1);
    }
}

#[tokio::test]
async fn signature_help_tracks_nested_and_named_arguments() {
    let mut h = Harness::start(&[("lib.php", DECLARATIONS), ("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    for (index, (source, active)) in [
        ("work([1, 2, 3], <cursor>null);", 1),
        ("work(array(1, 2), <cursor>null);", 1),
        ("work(function ($a, $b) { return [$a, $b]; }, <cursor>null);", 1),
        ("work(fn($a, $b) => [$a, $b], <cursor>null);", 1),
        ("work(inner(1, 2), <cursor>null);", 1),
        ("work(third: <cursor>null, first: 1, second: 2);", 2),
        ("work(third: null, first: <cursor>1, second: 2);", 0),
        ("work([1, 2], /* comma, */ <cursor>", 1),
    ]
    .iter()
    .enumerate()
    {
        let result = signature(&mut h, &format!("<?php\n{source}"), index as i32 + 2).await;
        check(&result, "function work(mixed $first, mixed $second, mixed $third = null): void", *active);
    }
    let result = signature(&mut h, "<?php work(null, inner(1, <cursor>2));", 20).await;
    check(&result, "function inner(int $left, int $right): int", 1);
    let result = signature(&mut h, "<?php function work(mixed $first, <cursor>mixed $second) {}", 21).await;
    assert!(result.is_null(), "declarations are not calls: {result}");
}

#[tokio::test]
async fn signature_help_keeps_all_union_targets_and_named_positions() {
    let code = "<?php function use_worker(Worker|Other $w): void { $w->run(name: <cursor>'x', count: 1); }";
    let mut h = Harness::start(&[("lib.php", DECLARATIONS), ("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    let result = signature(&mut h, code, 2).await;
    let signatures = result["signatures"].as_array().expect("signatures");
    assert_eq!(signatures.len(), 2, "{result}");
    assert_eq!(signatures[0]["label"], "function Other::run(int $count, string $name): bool");
    assert_eq!(signatures[0]["activeParameter"], 1);
    assert_eq!(signatures[1]["label"], "function Worker::run(string $name, int $count = 1): bool");
    assert_eq!(signatures[1]["activeParameter"], 0);
}

#[tokio::test]
async fn signature_help_resolves_namespaces_and_imports() {
    let library = "<?php namespace Lib; function work(int $local): void {}";
    let mut h = Harness::start(&[("global.php", DECLARATIONS), ("lib.php", library), ("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    for (index, (source, label, active)) in [
        (
            "namespace App; work(1, <cursor>2);",
            "function work(mixed $first, mixed $second, mixed $third = null): void",
            1,
        ),
        ("namespace Lib; work(<cursor>1);", "function Lib\\work(int $local): void", 0),
        (
            "namespace App; use function Lib\\work as local; local(<cursor>1);",
            "function Lib\\work(int $local): void",
            0,
        ),
        (
            "namespace App; \\work(1, <cursor>2);",
            "function work(mixed $first, mixed $second, mixed $third = null): void",
            1,
        ),
        (
            "namespace App; use Worker as Alias; new Alias('x', <cursor>2);",
            "function Worker::__construct(string $name, int $count = 1)",
            1,
        ),
    ]
    .iter()
    .enumerate()
    {
        let result = signature(&mut h, &format!("<?php\n{source}"), index as i32 + 2).await;
        check(&result, label, *active);
    }
    let result = signature(&mut h, "<?php namespace App; use function Missing\\work; work(<cursor>1);", 20).await;
    assert!(result.is_null(), "an imported function must not fall back to a global function: {result}");
}

#[tokio::test]
async fn signature_help_renders_defaults_references_variadics_and_utf16_labels() {
    let mut h = Harness::start(&[("lib.php", DECLARATIONS), ("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    let result = signature(&mut h, "<?php $value = 'x'; café($value, '😀', 1, <cursor>2);", 2).await;
    check(&result, "function café(string &$名, string $suffix = '😀', int ...$rest): string", 2);
    let signature = &result["signatures"][0];
    let label: Vec<_> = signature["label"].as_str().unwrap().encode_utf16().collect();
    let expected = ["string &$名", "string $suffix = '😀'", "int ...$rest"];
    for (parameter, expected) in signature["parameters"].as_array().unwrap().iter().zip(expected) {
        let start = parameter["label"][0].as_u64().unwrap() as usize;
        let end = parameter["label"][1].as_u64().unwrap() as usize;
        assert_eq!(String::from_utf16(&label[start..end]).unwrap(), expected);
    }
}

#[tokio::test]
async fn signature_help_selects_variadics_for_unknown_named_arguments() {
    let mut h = Harness::start(&[("lib.php", DECLARATIONS), ("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    for (index, (argument, active)) in [("extra", 2), ("second", 1), ("values", 2)].iter().enumerate() {
        let result = signature(&mut h, &format!("<?php collect({argument}: <cursor>1);"), index as i32 + 2).await;
        check(&result, "function collect(int $first = 1, int $second = 2, int ...$values): void", *active);
    }
}

#[tokio::test]
async fn signature_help_skips_anonymous_function_parameters_and_captures() {
    let mut h = Harness::start(&[("lib.php", DECLARATIONS), ("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    for (index, source) in [
        "work(function ($a, <cursor>$b) {}, null);",
        "work(function &($a, <cursor>$b) { return $a; }, null);",
        "work(fn($a, <cursor>$b) => $a, null);",
        "work(fn&($a, <cursor>$b) => $a, null);",
        "$a = $b = 1; work(function () use ($a, <cursor>$b) {}, null);",
    ]
    .iter()
    .enumerate()
    {
        let result = signature(&mut h, &format!("<?php {source}"), index as i32 + 2).await;
        assert!(result.is_null(), "unexpected signature for {source}: {result}");
    }
}

#[tokio::test]
async fn signature_help_uses_remaining_partial_parameters_in_placeholder_order() {
    let mut h = Harness::start(&[("lib.php", DECLARATIONS), ("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    for (index, (source, label, active)) in [
        ("$partial = inner(left: 1, right: ?); $partial(right: <cursor>2);", "function inner(int $right): int", 0),
        (
            "$partial = inner(right: ?, left: ?); $partial(2, <cursor>1);",
            "function inner(int $right, int $left): int",
            1,
        ),
        (
            "$partial = inner(right: ?, left: ?); $partial(left: <cursor>1, right: 2);",
            "function inner(int $right, int $left): int",
            1,
        ),
        (
            "$partial = inner(right: ?, left: ?); $again = $partial(right: 2, left: ?); ($again)(left: <cursor>1);",
            "function inner(int $left): int",
            0,
        ),
        (
            "$partials = [inner(left: 1, right: ?)]; $partials[0](right: <cursor>2);",
            "function inner(int $right): int",
            0,
        ),
        (
            "$worker = new Worker('x'); $partial = $worker->run(name: 'x', count: ?); $partial(count: <cursor>2);",
            "function run(int $count = 1): bool",
            0,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let result = signature(&mut h, &format!("<?php {source}"), index as i32 + 2).await;
        check(&result, label, active);
    }
}

#[tokio::test]
async fn signature_help_keeps_closure_class_context_and_defaults() {
    let mut h = Harness::start(&[("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    for (index, closure) in [
        "function (parent $value, int $count = 2): parent { return $value; }",
        "fn(parent $value, int $count = 2): parent => $value",
    ]
    .iter()
    .enumerate()
    {
        let source = format!(
            "<?php class Base {{}} class Child extends Base {{ function run(): void {{ $cb = {closure}; $cb(value: <cursor>new Base()); }} }}"
        );
        let result = signature(&mut h, &source, index as i32 + 2).await;
        let label = result["signatures"][0]["label"].as_str().unwrap();
        assert!(label.ends_with("(Base $value, int $count = 2): Base"), "{result}");
        assert_eq!(result["activeParameter"], 0);
    }
}
