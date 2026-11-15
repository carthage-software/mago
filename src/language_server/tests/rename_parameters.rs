use serde_json::json;

use super::harness::Harness;
use super::references::{apply, rename_at};

#[tokio::test]
async fn parameter_rename_updates_captures_docs_named_calls_and_partial_calls() {
    let declaration = "<?php namespace Lib; /** @param int $value A $value in prose. */ function read(int $value, int $other = 0): int { $fn = function () use ($value) { return $value; }; return $fn() + $other; }";
    let caller = "<?php use function Lib\\read as fetch; fetch(value: 1); $partial = fetch(value: ?, other: 2); $captured = fetch(value: 1, other: ?); $partial(value: 4); ($partial)(value: 5); $again = $partial(value: ?); $again(value: 6); $captured(other: 7); function unrelated($value) { return $value; } unrelated(value: 3);";
    let mut h = Harness::start(&[("lib.php", declaration), ("call.php", caller)]).await;
    let expected_declaration = declaration.replace("$value", "$item").replace("A $item in prose", "A $value in prose");
    let expected_caller = caller.replacen("value:", "item:", 7);
    for (path, source, needle) in [
        ("lib.php", declaration, "$value,"),
        ("lib.php", declaration, "$value;"),
        ("call.php", caller, "value: 1"),
        ("call.php", caller, "value: ?"),
        ("call.php", caller, "value: 4"),
        ("call.php", caller, "value: 6"),
    ] {
        let edit = rename_at(&mut h, path, source, needle, "item").await.unwrap();
        assert_eq!(apply(declaration, &edit["changes"][h.url("lib.php")]), expected_declaration, "{needle}: {edit}");
        assert_eq!(apply(caller, &edit["changes"][h.url("call.php")]), expected_caller, "{needle}: {edit}");
    }
}

#[tokio::test]
async fn promoted_property_rename_updates_all_bindings_calls_and_attributes() {
    let declaration = "<?php namespace Lib; #[\\Attribute] class Foo { function __construct(public int $bar) { echo $bar; } function read(): int { return $this->bar; } }";
    let caller = "<?php use Lib\\Foo as Alias; #[Alias(bar: 1)] class Tagged {} #[\\Lib\\Foo(bar: 2), Alias(bar: 3)] function tagged() {} $foo = new Alias(bar: 4); echo $foo->bar; $partial = $foo->__construct(bar: ?); $partial(bar: 6); class Child extends Alias {} new Child(bar: 5);";
    let mut h = Harness::start(&[("lib.php", declaration), ("call.php", caller)]).await;
    for (path, source, needle) in [
        ("lib.php", declaration, "$bar)"),
        ("lib.php", declaration, "$bar;"),
        ("lib.php", declaration, "bar; }"),
        ("call.php", caller, "bar: 1"),
        ("call.php", caller, "bar: 4"),
        ("call.php", caller, "bar: 6"),
        ("call.php", caller, "bar;"),
    ] {
        let edit = rename_at(&mut h, path, source, needle, "amount").await.unwrap();
        assert_eq!(
            apply(declaration, &edit["changes"][h.url("lib.php")]),
            declaration.replace("bar", "amount"),
            "{needle}: {edit}"
        );
        assert_eq!(
            apply(caller, &edit["changes"][h.url("call.php")]),
            caller.replace("bar", "amount"),
            "{needle}: {edit}"
        );
        let prepare = h.request("textDocument/prepareRename", json!({
            "textDocument": {"uri": h.url(path)},
            "position": {"line": 0, "character": source.find(needle).unwrap() + usize::from(needle.starts_with('$'))}
        })).await;
        assert_eq!(prepare["placeholder"], "bar", "{needle}: {prepare}");
    }
}

#[tokio::test]
async fn method_parameter_rename_follows_overrides_and_resolved_receivers() {
    let source = "<?php interface Contract { function run(int $value): int; } class C implements Contract { function run(int $value): int { return $value; } } class D extends C { function run(int $different): int { return $different; } } function call(Contract $object): int { return $object->run(value: 1); } (new C())->run(value: 2); (new D())->run(different: 3); class Other { function run(int $value): int { return $value; } } (new Other())->run(value: 4);";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let edit = rename_at(&mut h, "a.php", source, "$value", "item").await.unwrap();
    let expected = source.replacen("$value", "$item", 3).replacen("value:", "item:", 2);
    assert_eq!(apply(source, &edit["changes"][h.url("a.php")]), expected, "{edit}");
}

#[tokio::test]
async fn parameter_rename_rejects_binding_and_property_collisions() {
    for (source, needle) in [
        ("<?php function f($value, $taken) { return $value; } f(value: 1, taken: 2);", "$value"),
        ("<?php function f($value) { $taken = 2; return $value; }", "$value"),
        ("<?php function f($value) { return fn($taken) => $value + $taken; }", "$value"),
        ("<?php class C { public int $taken; function __construct(public int $value) {} }", "$value"),
        (
            "<?php class C { function __construct(public int $value, int $taken) {} } new C(value: 1, taken: 2);",
            "value: 1",
        ),
    ] {
        let mut h = Harness::start(&[("a.php", source)]).await;
        assert!(rename_at(&mut h, "a.php", source, needle, "taken").await.is_err(), "{source}");
    }
}

#[tokio::test]
async fn closure_parameter_rename_follows_known_callable_labels() {
    let source = "<?php $cb = function (int $value): int { return $value; }; $cb(value: 1); $arrow = fn(int $value): int => $value; $arrow(value: 2);";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let edit = rename_at(&mut h, "a.php", source, "$value", "item").await.unwrap();
    assert_eq!(
        apply(source, &edit["changes"][h.url("a.php")]),
        source.replacen("$value", "$item", 2).replacen("value:", "item:", 1),
        "{edit}"
    );
}

#[tokio::test]
async fn closure_parameter_rename_keeps_bound_class_context_and_declines_mixed_origins() {
    let source = "<?php class Base {} class Child extends Base { function run(): void { $cb = /** @param parent $value Parent value. */ function (parent $value): parent { return $value; }; $cb(value: new Base()); } }";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for needle in ["$value):", "value: new"] {
        let edit = rename_at(&mut h, "a.php", source, needle, "item").await.unwrap();
        assert_eq!(
            apply(source, &edit["changes"][h.url("a.php")]),
            source.replace("$value", "$item").replace("value:", "item:"),
            "{edit}"
        );
    }
    h.open("a.php", source).await;
    for (index, alternatives) in ["$first : $second", "$second : $first"].iter().enumerate() {
        let source = format!(
            "<?php class Base {{}} class Child extends Base {{ function run(bool $pick): void {{ $first = function (parent $value): parent {{ return $value; }}; $second = fn(parent $value): parent => $value; $cb = $pick ? {alternatives}; $cb(value: new Base()); }} }}"
        );
        h.change("a.php", &source, index as i32 + 2).await;
        assert!(rename_at(&mut h, "a.php", &source, "$value):", "item").await.is_err(), "{source}");
    }
}

#[tokio::test]
async fn method_parameter_rename_covers_partial_static_and_nullsafe_calls() {
    let source = "<?php class C { static function read(int $value, int $other = 0): int { return $value + $other; } } C::read(value: 1); $first = C::read(value: ?, other: 2); $second = C::read(value: 1, other: ?); $object = new C(); $object->read(value: 3); $third = $object->read(value: ?, other: 4); $object?->read(value: 5); $cb = C::read(...); $cb(value: 6);";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for needle in ["$value", "value: ?", "value: 6"] {
        let edit = rename_at(&mut h, "a.php", source, needle, "item").await.unwrap();
        assert_eq!(
            apply(source, &edit["changes"][h.url("a.php")]),
            source.replace("value", "item"),
            "{needle}: {edit}"
        );
    }
}

#[tokio::test]
async fn promoted_property_family_rename_updates_each_constructor_binding() {
    let source = "<?php class Base { function __construct(public int $value) { echo $value; } } class Child extends Base { function __construct(public int $value) { parent::__construct(value: $value); } } new Base(value: 1); new Child(value: 2); echo (new Child(3))->value;";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let edit = rename_at(&mut h, "a.php", source, "$value", "item").await.unwrap();
    assert_eq!(apply(source, &edit["changes"][h.url("a.php")]), source.replace("value", "item"), "{edit}");
}

#[tokio::test]
async fn parameter_rename_keeps_unrelated_labels_prose_and_literal_keys() {
    let source = "<?php /* 😀 */ function f($café) { return ['café' => $café]; } function unrelated($café) { return $café; } f(café: unrelated(café: 1)); $text = 'café:';";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let edit = rename_at(&mut h, "a.php", source, "$café", "match").await.unwrap();
    let expected = source.replacen("$café", "$match", 2).replacen("f(café:", "f(match:", 1);
    assert_eq!(apply(source, &edit["changes"][h.url("a.php")]), expected, "{edit}");
}

#[tokio::test]
async fn parameter_rename_declines_labels_with_unrelated_union_targets_and_builtins() {
    let source = "<?php class A { function f($value) { return $value; } } class B { function f($value) { return $value; } } function call(A|B $object) { return $object->f(value: 1); } strlen(string: 'x');";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for needle in ["$value", "value: 1", "string:"] {
        assert!(rename_at(&mut h, "a.php", source, needle, "item").await.is_err(), "{needle}");
    }
}

#[tokio::test]
async fn variadic_parameter_rename_preserves_named_argument_array_keys() {
    let source = "<?php function collect(...$values): array { return $values; } collect(values: 1, items: 2);";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let edit = rename_at(&mut h, "a.php", source, "$values", "items").await.unwrap();
    assert_eq!(apply(source, &edit["changes"][h.url("a.php")]), source.replace("$values", "$items"), "{edit}");
    assert!(rename_at(&mut h, "a.php", source, "values: 1", "items").await.is_err());
}

#[tokio::test]
async fn parameter_rename_updates_conditional_phpdoc_types_without_capturing_callable_parameters() {
    let source = "<?php\n/**\n * @param int|string $value\n * @param callable(int $value): ($value is int ? int : string) $callback\n * @param callable(): ($value is int ? int : string) $factory\n * @return ($value is int ? int : string)\n * @custom Prose mentioning $value.\n */\nfunction f(int|string $value, callable $callback, callable $factory): int|string { return $value; }\nf(value: 1, callback: fn($value) => $value, factory: fn() => 1);";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let edit = rename_at(&mut h, "a.php", source, "$value, callable", "item").await.unwrap();
    let expected = source
        .replace("int|string $value", "int|string $item")
        .replace("callable(): ($value", "callable(): ($item")
        .replace("@return ($value", "@return ($item")
        .replace("return $value", "return $item")
        .replace("f(value:", "f(item:");
    assert_eq!(apply(source, &edit["changes"][h.url("a.php")]), expected, "{edit}");
}

#[tokio::test]
async fn parameter_rename_updates_inline_phpdoc_bindings_without_crossing_scopes() {
    for (source, expected) in [
        (
            "<?php function f(mixed $value): int { /** @var int $value A $value in prose. */ return $value; }",
            "<?php function f(mixed $item): int { /** @var int $item A $value in prose. */ return $item; }",
        ),
        (
            "<?php function f(mixed $value): int { $read = function () use (&$value): int { /** @phpstan-var int $value */ return $value; }; return $read(); }",
            "<?php function f(mixed $item): int { $read = function () use (&$item): int { /** @phpstan-var int $item */ return $item; }; return $read(); }",
        ),
        (
            "<?php function f(mixed $value): int { $read = function (mixed $value): int { /** @var int $value */ return $value; }; /** @psalm-var int $value */ return $value; }",
            "<?php function f(mixed $item): int { $read = function (mixed $value): int { /** @var int $value */ return $value; }; /** @psalm-var int $item */ return $item; }",
        ),
        (
            "<?php function f(mixed $value): int { $object = new class { /** @var int $value */ public mixed $value; }; /** @var int $value */ return $value; }",
            "<?php function f(mixed $item): int { $object = new class { /** @var int $value */ public mixed $value; }; /** @var int $item */ return $item; }",
        ),
        (
            "<?php function f(mixed $value): mixed { global $value; /** @var int $value */ return $value; }",
            "<?php function f(mixed $item): mixed { global $value; /** @var int $value */ return $value; }",
        ),
        (
            "<?php function f(mixed $value): mixed { /** @var int $value */ $copy = $value; global $value; /** @var string $value */ return $value; }",
            "<?php function f(mixed $item): mixed { /** @var int $item */ $copy = $item; global $value; /** @var string $value */ return $value; }",
        ),
        (
            "<?php function f(mixed $value): mixed { /** @psalm-trace $value A $value in prose. */ $copy = $value; global $value; /** @psalm-trace $value */ return $value; }",
            "<?php function f(mixed $item): mixed { /** @psalm-trace $item A $value in prose. */ $copy = $item; global $value; /** @psalm-trace $value */ return $value; }",
        ),
        (
            "<?php function f(mixed $value): mixed { return function () use ($value): int { /** @var int $value */ return 1; }; }",
            "<?php function f(mixed $item): mixed { return function () use ($item): int { /** @var int $item */ return 1; }; }",
        ),
        (
            "<?php function f(int|string $value): mixed { /** @var ($value is int ? int : string) $result */ $result = $value; return $result; }",
            "<?php function f(int|string $item): mixed { /** @var ($item is int ? int : string) $result */ $result = $item; return $result; }",
        ),
        (
            "<?php function f(mixed $value): mixed { /** @return ($value is int ? int : string) */ function nested(int|string $value) { return $value; } return $value; }",
            "<?php function f(mixed $item): mixed { /** @return ($value is int ? int : string) */ function nested(int|string $value) { return $value; } return $item; }",
        ),
        (
            "<?php function f(mixed $value): mixed { $read = /** @return ($value is int ? int : string) */ function (int|string $value) { return $value; }; return $value; }",
            "<?php function f(mixed $item): mixed { $read = /** @return ($value is int ? int : string) */ function (int|string $value) { return $value; }; return $item; }",
        ),
        (
            concat!(
                "<?php function f(int|string $value): mixed { ",
                "/** @var callable(int $value): ($value is int ? int : string) $read */ $read = fn($x) => $x; ",
                "/** @var callable(): ($value is int ? int : string) $factory */ $factory = fn() => $value; ",
                "return [$read, $factory]; }",
            ),
            concat!(
                "<?php function f(int|string $item): mixed { ",
                "/** @var callable(int $value): ($value is int ? int : string) $read */ $read = fn($x) => $x; ",
                "/** @var callable(): ($item is int ? int : string) $factory */ $factory = fn() => $item; ",
                "return [$read, $factory]; }",
            ),
        ),
    ] {
        let mut h = Harness::start(&[("a.php", source)]).await;
        let edit = rename_at(&mut h, "a.php", source, "$value", "item").await.unwrap();
        assert_eq!(apply(source, &edit["changes"][h.url("a.php")]), expected, "{edit}");
    }
}

#[tokio::test]
async fn parameter_rename_follows_parenthesized_and_indexed_callable_expressions() {
    let source = "<?php function read(int $value): int { return $value; } $cb = read(...); ($cb)(value: 1); (($cb))(value: 2); (read(...))(value: 3); $callbacks = [read(...)]; $callbacks[0](value: 4);";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for needle in ["$value", "value: 1", "value: 4"] {
        let edit = rename_at(&mut h, "a.php", source, needle, "item").await.unwrap();
        assert_eq!(
            apply(source, &edit["changes"][h.url("a.php")]),
            source.replace("value", "item"),
            "{needle}: {edit}"
        );
    }
}

#[tokio::test]
async fn parameter_rename_follows_immediate_closures_and_declines_mixed_callable_unions() {
    let source =
        "<?php (function (int $value): int { return $value; })(value: 1); (fn(int $other): int => $other)(other: 2);";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for (needle, original) in [("$value", "value"), ("other: 2", "other")] {
        let edit = rename_at(&mut h, "a.php", source, needle, "item").await.unwrap();
        assert_eq!(apply(source, &edit["changes"][h.url("a.php")]), source.replace(original, "item"), "{edit}");
    }
    let source = "<?php function read(int $value): int { return $value; } function call(bool $condition): void { $cb = $condition ? read(...) : 1; ($cb)(value: 1); }";
    let mut h = Harness::start(&[("a.php", source)]).await;
    assert!(rename_at(&mut h, "a.php", source, "$value", "item").await.is_err());
}

#[tokio::test]
async fn parameter_rename_follows_direct_and_inherited_invokable_objects() {
    let source = "<?php class Handler { function __invoke(int $value): int { return $value; } } class Child extends Handler {} $handler = new Handler(); $handler(value: 1); $child = new Child(); ($child)(value: 2);";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for needle in ["$value", "value: 1", "value: 2"] {
        let edit = rename_at(&mut h, "a.php", source, needle, "item").await.unwrap();
        assert_eq!(
            apply(source, &edit["changes"][h.url("a.php")]),
            source.replace("value", "item"),
            "{needle}: {edit}"
        );
    }
    let source = "<?php class Handler { function __invoke(int $value): int { return $value; } } class Other {} function call(bool $condition): void { $handler = $condition ? new Handler() : new Other(); ($handler)(value: 1); }";
    let mut h = Harness::start(&[("a.php", source)]).await;
    assert!(rename_at(&mut h, "a.php", source, "$value", "item").await.is_err());
}

#[tokio::test]
async fn parameter_rename_follows_known_function_strings_and_rejects_unknown_union_branches() {
    let source = "<?php namespace Lib; function read(int $value): int { return $value; } $cb = 'Lib\\read'; $cb(value: 1); $rooted = '\\Lib\\READ'; ($rooted)(value: 2);";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for needle in ["$value", "value: 1", "value: 2"] {
        let edit = rename_at(&mut h, "a.php", source, needle, "item").await.unwrap();
        assert_eq!(
            apply(source, &edit["changes"][h.url("a.php")]),
            source.replace("value", "item"),
            "{needle}: {edit}"
        );
    }
    let source = "<?php function read(int $value): int { return $value; } function call(bool $condition): void { $cb = $condition ? 'read' : 'missing'; $cb(value: 1); }";
    let mut h = Harness::start(&[("a.php", source)]).await;
    assert!(rename_at(&mut h, "a.php", source, "$value", "item").await.is_err());
}

#[tokio::test]
async fn parameter_rename_follows_object_arrays_and_inherited_static_callables() {
    let source = "<?php class Base { function run(int $value): int { return $value; } static function read(int $other): int { return $other; } } class Child extends Base {} $object = new Child(); $cb = [$object, 'run']; $cb(value: 1); $keyed = [1 => 'run', 0 => $object]; ($keyed)(value: 2); $static = 'Child::READ'; $static(other: 3); $array = [Child::class, 'read']; $array(other: 4);";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for (needle, original) in [("$value", "value"), ("value: 1", "value"), ("$other", "other"), ("other: 4", "other")] {
        let edit = rename_at(&mut h, "a.php", source, needle, "item").await.unwrap();
        assert_eq!(
            apply(source, &edit["changes"][h.url("a.php")]),
            source.replace(original, "item"),
            "{needle}: {edit}"
        );
    }
}

#[tokio::test]
async fn parameter_rename_declines_partial_calls_with_ambiguous_remaining_names() {
    for branches in [
        "read(value: ?, other: 2) : read(value: 1, other: ?)",
        "read(value: 1, other: ?) : read(value: ?, other: 2)",
        "read(value: ?, other: 2) : 42",
        "read(...) : read(other: ?, value: ?)",
        "read(other: ?, value: ?) : read(...)",
    ] {
        let source = format!(
            "<?php function read(int $value, int $other): int {{ return $value + $other; }} function call(bool $pick): int {{ $partial = $pick ? {branches}; return $partial(value: 3); }}"
        );
        let mut h = Harness::start(&[("a.php", &source)]).await;
        assert!(rename_at(&mut h, "a.php", &source, "$value", "item").await.is_err(), "{source}");
        assert!(rename_at(&mut h, "a.php", &source, "value: 3", "item").await.is_err(), "{source}");
    }
}

#[tokio::test]
async fn parameter_rename_declines_callable_unions_with_different_or_unknown_origins() {
    let declaration = "<?php function read(int $value): int { return $value; }";
    let other = "<?php function other(int $value): int { return $value; }";
    let mut h = Harness::start(&[("read.php", declaration), ("other.php", other), ("call.php", "<?php")]).await;
    h.open("call.php", "<?php").await;
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
            "<?php /** @param Closure(int): int $unknown */ function call(bool $pick, Closure $unknown): int {{ $callback = $pick ? {branches}; return $callback(value: 2); }}"
        );
        h.change("call.php", &source, index as i32 + 2).await;
        assert!(rename_at(&mut h, "read.php", declaration, "$value", "item").await.is_err(), "{branches}");
        assert!(rename_at(&mut h, "call.php", &source, "value: 2", "item").await.is_err(), "{branches}");
        if branches.contains("other(") {
            assert!(rename_at(&mut h, "other.php", other, "$value", "item").await.is_err(), "{branches}");
        }
    }
}
