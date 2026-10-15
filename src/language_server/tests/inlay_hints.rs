use serde_json::Value;
use serde_json::json;

use mago_database::file::File;

use super::harness::Harness;
use crate::language_server::position::position_at_offset;

const DECLARATIONS: &str = r"<?php
function work(mixed $first, mixed $second, mixed $third = null): void {}
function inner(int $left, int $right): int { return $left + $right; }
function collect(int $head, int ...$values): void {}
class Worker {
    public function __construct(string $name) {}
    public function run(string $name, int $count): void {}
    public static function build(string $name): self { return new self($name); }
}
class Child extends Worker {}
class Other { public function run(string $name, int $amount): void {} }
class Missing {}
";

async fn hints(h: &mut Harness, source: &str, start: usize, end: usize) -> Value {
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    h.request(
        "textDocument/inlayHint",
        json!({"textDocument":{"uri":h.url("a.php")},"range":{
            "start":position_at_offset(&file, start as u32),"end":position_at_offset(&file, end as u32)
        }}),
    )
    .await
}

fn check(source: &str, result: &Value, expected: &[(&str, &str)]) {
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let mut expected: Vec<_> = expected
        .iter()
        .map(|(needle, label)| {
            let offset = source.find(needle).unwrap() as u32;
            (offset, json!({"position":position_at_offset(&file, offset),"label":label,"kind":2,"paddingRight":true}))
        })
        .collect();
    expected.sort_unstable_by_key(|(offset, _)| *offset);
    assert_eq!(result, &json!(expected.into_iter().map(|(_, hint)| hint).collect::<Vec<_>>()), "{source}");
}

#[tokio::test]
async fn inlay_hints_resolve_function_fallback_imports_and_shadowing() {
    let library = "<?php namespace Lib; function work(int $local): void {}";
    let source = r"<?php namespace App; use function Lib\work as local;
work(10, 20); local(30); \work(40, 50); Missing\work(60); namespace\work(70); work(...);
use function Missing\work as absent; absent(80);
";
    let mut h = Harness::start(&[("lib.php", DECLARATIONS), ("namespaced.php", library), ("a.php", source)]).await;
    check(
        source,
        &hints(&mut h, source, 0, source.len()).await,
        &[("10", "first:"), ("20", "second:"), ("30", "local:"), ("40", "first:"), ("50", "second:")],
    );
    let shadow = "<?php namespace App; function work(int $local): void {} work(90);";
    h.open("a.php", shadow).await;
    check(shadow, &hints(&mut h, shadow, 0, shadow.len()).await, &[("90", "local:")]);
}

#[tokio::test]
async fn inlay_hints_cover_inherited_methods_static_calls_and_constructors() {
    let source = "<?php $w = new Child('ctor'); $w->run('method', 11); $w?->run('safe', 12); Child::build('static')->run('chain', 13);";
    let mut h = Harness::start(&[("lib.php", DECLARATIONS), ("a.php", source)]).await;
    check(
        source,
        &hints(&mut h, source, 0, source.len()).await,
        &[
            ("'ctor'", "name:"),
            ("'method'", "name:"),
            ("11", "count:"),
            ("'safe'", "name:"),
            ("12", "count:"),
            ("'static'", "name:"),
            ("'chain'", "name:"),
            ("13", "count:"),
        ],
    );
}

#[tokio::test]
async fn inlay_hints_require_union_agreement_and_known_targets() {
    let source = "<?php function known(Worker|Other $w): void { $w->run('same', 10); } function unknown(Worker|Missing $w): void { $w->run('unknown', 20); }";
    let mut h = Harness::start(&[("lib.php", DECLARATIONS), ("a.php", source)]).await;
    check(source, &hints(&mut h, source, 0, source.len()).await, &[("'same'", "name:")]);
}

#[tokio::test]
async fn inlay_hints_skip_nested_commas_named_arguments_and_unpacking() {
    let source = "<?php work([1, 2], function ($a, $b) { return [$a, $b]; }, (3)); work(inner(4, 5), match (6) { 6, 7 => 8 }, third /* gap */ : 9); collect(10, 11, 12); collect(head: 13, extra: 14); collect(15, ...[16, 17]); work(fn($a, $b) => [$a, $b], null);";
    let mut h = Harness::start(&[("lib.php", DECLARATIONS), ("a.php", source)]).await;
    check(
        source,
        &hints(&mut h, source, 0, source.len()).await,
        &[
            ("[1, 2]", "first:"),
            ("function ($a", "second:"),
            ("(3)", "third:"),
            ("inner(4", "first:"),
            ("4, 5", "left:"),
            ("5)", "right:"),
            ("match (6)", "second:"),
            ("10", "head:"),
            ("11", "values:"),
            ("12", "values:"),
            ("15", "head:"),
            ("fn($a", "first:"),
            ("null", "second:"),
        ],
    );
}

#[tokio::test]
async fn inlay_hints_use_utf16_positions_and_half_open_ranges() {
    let source = "<?php /*😀*/ work('é', '名');";
    let mut h = Harness::start(&[("lib.php", DECLARATIONS), ("a.php", source)]).await;
    let first = source.find("'é'").unwrap();
    let second = source.find("'名'").unwrap();
    check(source, &hints(&mut h, source, first, second).await, &[("'é'", "first:")]);
    check(source, &hints(&mut h, source, second, source.len()).await, &[("'名'", "second:")]);
    check(source, &hints(&mut h, source, first, first).await, &[]);
    h.open("a.php", source).await;
    let incomplete = "<?php /*😀*/ work('é', ";
    h.change("a.php", incomplete, 2).await;
    check(incomplete, &hints(&mut h, incomplete, 0, incomplete.len()).await, &[]);
}

#[tokio::test]
async fn inlay_hints_resolve_callable_values() {
    let source = r"<?php
inner(101, 102);
$full = inner(...); $full(103, 104);
$closure = static fn(int $left, int $right): int => $left + $right; $closure(105, 106);
($full)(107, 108);
$callbacks = [$full]; $callbacks[0](109, 110);
$string = 'inner'; $string(111, 112);
";
    let mut h = Harness::start(&[("lib.php", DECLARATIONS), ("a.php", source)]).await;
    check(
        source,
        &hints(&mut h, source, 0, source.len()).await,
        &[
            ("101", "left:"),
            ("102", "right:"),
            ("103", "left:"),
            ("104", "right:"),
            ("105", "left:"),
            ("106", "right:"),
            ("107", "left:"),
            ("108", "right:"),
            ("109", "left:"),
            ("110", "right:"),
            ("111", "left:"),
            ("112", "right:"),
        ],
    );
}

#[tokio::test]
async fn inlay_hints_use_remaining_partial_parameters_and_variadic_flags() {
    let source = r"<?php
$partial = inner(left: 1, right: ?); $partial(201); $partial(right: 202);
$reordered = inner(right: ?, left: ?); $reordered(203, 204);
$again = $reordered(right: 2, left: ?); ($again)(205);
$partials = [$partial]; $partials[0](206);
$tail = collect(1, ...); $tail(207, 208);
$full = collect(...); $full(209, 210, 211);
$partial(...[212]);
";
    let mut h = Harness::start(&[("lib.php", DECLARATIONS), ("a.php", source)]).await;
    check(
        source,
        &hints(&mut h, source, 0, source.len()).await,
        &[
            ("201", "right:"),
            ("203", "right:"),
            ("204", "left:"),
            ("205", "left:"),
            ("206", "right:"),
            ("207", "values:"),
            ("208", "values:"),
            ("209", "head:"),
            ("210", "values:"),
            ("211", "values:"),
        ],
    );
}

#[tokio::test]
async fn inlay_hints_require_agreeing_callable_parameter_names() {
    let source = r"<?php
function same(int $left, int $right): int { return $left + $right; }
function different(int $left, int $amount): int { return $left + $amount; }
/** @param Closure(int, int): int $unknown */
function calls(bool $pick, Closure $unknown): void {
    $same = $pick ? inner(...) : same(...); $same(301, 302);
    $different = $pick ? inner(...) : different(...); $different(303, 304);
    $partial = $pick ? inner(left: ?, right: 1) : inner(left: 1, right: ?); $partial(305);
    $unknownTarget = $pick ? inner(...) : $unknown; $unknownTarget(306, 307);
    $reordered = $pick ? inner(...) : inner(right: ?, left: ?); $reordered(308, 309);
}
";
    let mut h = Harness::start(&[("lib.php", DECLARATIONS), ("a.php", source)]).await;
    check(
        source,
        &hints(&mut h, source, 0, source.len()).await,
        &[("301", "left:"), ("302", "right:"), ("303", "left:")],
    );
}
