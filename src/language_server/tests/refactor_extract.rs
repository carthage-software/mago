use serde_json::{Value, json};

use mago_database::file::File;

use super::harness::Harness;
use super::references::apply;
use crate::language_server::position::position_at_offset;

async fn actions(h: &mut Harness, source: &str, needle: &str, only: Value) -> Value {
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let start = source.rfind(needle).unwrap() as u32;
    let mut result = h.request("textDocument/codeAction", json!({
        "textDocument":{"uri":h.url("a.php")},
        "range":{"start":position_at_offset(&file, start),"end":position_at_offset(&file, start + needle.len() as u32)},
        "context":{"diagnostics":[],"only":only}
    })).await;
    if let Some(actions) = result.as_array_mut() {
        actions.retain(|action| action["title"].as_str().is_some_and(|title| title.starts_with("Extract to $")));
    }
    result
}

#[tokio::test]
async fn extract_local_preserves_crlf_comments_indentation_and_utf16() {
    let source = "<?php\r\nfunction f(int $x): int {\r\n\t// 😀 keep\r\n\t$total = (int) $x + 2; // tail\r\n\treturn $total;\r\n}\r\n";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let result = actions(&mut h, source, "(int) $x + 2", json!(["refactor.extract"])).await;
    assert_eq!(result.as_array().unwrap().len(), 1, "{result}");
    assert_eq!(result[0]["kind"], "refactor.extract");
    assert_eq!(result[0]["title"], "Extract to $extracted");
    assert_eq!(
        apply(source, &result[0]["edit"]["changes"][h.url("a.php")]),
        "<?php\r\nfunction f(int $x): int {\r\n\t// 😀 keep\r\n\t$extracted = (int) $x + 2;\r\n\t$total = $extracted; // tail\r\n\treturn $total;\r\n}\r\n"
    );
    for only in [json!(["quickfix"]), json!(["refactor.rewrite"]), json!([]), json!(["refac"])] {
        assert!(actions(&mut h, source, "(int) $x + 2", only).await.as_array().unwrap().is_empty());
    }
}

#[tokio::test]
async fn extract_return_keeps_calls_in_order_and_avoids_nested_capture_names() {
    let source = "<?php function f($x) { $callback = function () use ($extracted) { return $extracted2; }; before(); return build(first($x), second()); }";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let result = actions(&mut h, source, "build(first($x), second())", json!(["refactor"])).await;
    assert_eq!(result.as_array().unwrap().len(), 1, "{result}");
    assert_eq!(
        apply(source, &result[0]["edit"]["changes"][h.url("a.php")]),
        "<?php function f($x) { $callback = function () use ($extracted) { return $extracted2; }; before(); $extracted3 = build(first($x), second()); return $extracted3; }"
    );
}

#[tokio::test]
async fn extract_local_supports_methods_and_braced_conditional_blocks() {
    for source in [
        "<?php class C { function f($x) { if ($x) { return strlen('é'); } } }",
        "<?php $f = function ($x) { if ($x) { return strlen('é'); } };",
    ] {
        let mut h = Harness::start(&[("a.php", source)]).await;
        let result = actions(&mut h, source, "strlen('é')", json!(["refactor.extract"])).await;
        assert_eq!(result.as_array().unwrap().len(), 1, "{source}: {result}");
        assert_eq!(
            apply(source, &result[0]["edit"]["changes"][h.url("a.php")]),
            source.replace("return strlen('é');", "$extracted = strlen('é'); return $extracted;")
        );
    }
}

#[tokio::test]
async fn extract_local_preserves_expression_grouping_without_adding_parentheses() {
    for expression in ["$value + $other", "($value + $other) * 2"] {
        let source = format!("<?php function f(int $value, int $other): int {{ return {expression}; }}");
        let mut h = Harness::start(&[("a.php", &source)]).await;
        let result = actions(&mut h, &source, expression, json!(["refactor.extract"])).await;
        assert_eq!(result.as_array().unwrap().len(), 1, "{result}");
        assert_eq!(
            apply(&source, &result[0]["edit"]["changes"][h.url("a.php")]),
            source.replace(&format!("return {expression};"), &format!("$extracted = {expression}; return $extracted;"))
        );
    }
}

#[tokio::test]
async fn extract_local_declines_nested_lazy_reference_and_lifetime_changes() {
    for (source, needle) in [
        ("<?php function f() { return 1 + 2; }", "1"),
        ("<?php declare(ticks=1); function f() { return 1 + 2; }", "1 + 2"),
        ("<?php function f() { if (true) return 1 + 2; }", "1 + 2"),
        ("<?php function f() { $x = make(); }", "make()"),
        ("<?php function f($o) { $x = $o; }", "$o"),
        ("<?php function f($o) { $o->value = 1 + 2; }", "1 + 2"),
        ("<?php function f(&$x) { $y =& $x; return 1 + 2; }", "1 + 2"),
        ("<?php function &f() { return 1 + 2; }", "1 + 2"),
        ("<?php function f($x) { return $x ? 1 : 2; }", "$x ? 1 : 2"),
        ("<?php function f($x) { return $x && work(); }", "$x && work()"),
        ("<?php function f($x) { return $x and work(); }", "$x and work()"),
        ("<?php function f($x) { return $x or work(); }", "$x or work()"),
        ("<?php function f($x) { return $x xor work(); }", "$x xor work()"),
        ("<?php function f($x) { return print $x; }", "print $x"),
        ("<?php function f($x) { return yield $x; }", "yield $x"),
        ("<?php function f($x) { return yield from $x; }", "yield from $x"),
        ("<?php function f($x) { return $x?->run(); }", "$x?->run()"),
        ("<?php function f($x) { return $x->await(); }", "$x->await()"),
        ("<?php function f() { return Fiber::suspend(); }", "Fiber::suspend()"),
        ("<?php function f() { yield 1; return 1 + 2; }", "1 + 2"),
        ("<?php function f() { try { return new C(); } finally { return null; } }", "new C()"),
        ("<?php function f() { /** @var int */ $x = 1 + 2; }", "1 + 2"),
        ("<?php function f() { /** @var int */ /* keep */ return 1 + 2; }", "1 + 2"),
        ("<?php return 1 + 2;", "1 + 2"),
        ("<?php function f() { return 1 + 2;", "1 + 2"),
    ] {
        let mut h = Harness::start(&[("a.php", source)]).await;
        assert!(
            actions(&mut h, source, needle, json!(["refactor.extract"])).await.as_array().unwrap().is_empty(),
            "{source}"
        );
    }
}

#[tokio::test]
async fn extract_local_declines_scope_reflection_including_imported_aliases() {
    for source in [
        "<?php function f() { get_defined_vars(); return 1 + 2; }",
        "<?php use function get_defined_vars as snapshot; function f() { snapshot(); return 1 + 2; }",
        "<?php use function Async\\await as wait; function f() { wait(); return 1 + 2; }",
        "<?php namespace App; function f() { get_defined_vars(); return 1 + 2; }",
        "<?php function f() { compact('extracted'); return 1 + 2; }",
        "<?php function f() { extract([]); return 1 + 2; }",
        "<?php function f($name) { $$name = 1; return 1 + 2; }",
        "<?php function f($call) { $call(); return 1 + 2; }",
        "<?php function f() { echo $GLOBALS['x']; return 1 + 2; }",
        "<?php function f() { include 'vars.php'; return 1 + 2; }",
        "<?php function f() { return 1 + 2; } echo __LINE__;",
    ] {
        let mut h = Harness::start(&[("a.php", source)]).await;
        assert!(
            actions(&mut h, source, "1 + 2", json!(["refactor.extract"])).await.as_array().unwrap().is_empty(),
            "{source}"
        );
    }
}
