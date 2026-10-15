use mago_allocator::LocalArena;
use mago_database::file::File;
use mago_syntax::cst::{Modifier, Node};
use mago_syntax::parser::parse_file;
use serde_json::{Value, json};

use super::harness::{Harness, client_capabilities};
use super::references::apply;
use crate::config::language_server::Provider;
use crate::language_server::ServerConfig;

async fn actions(h: &mut Harness, source: &str, expression: &str, only: Value) -> Vec<Value> {
    let start = source.rfind(expression).unwrap();
    let position = |offset: usize| {
        let before = &source[..offset];
        json!({
            "line": before.bytes().filter(|byte| *byte == b'\n').count(),
            "character": before.rsplit('\n').next().unwrap().encode_utf16().count()
        })
    };
    let result = h
        .request(
            "textDocument/codeAction",
            json!({
                "textDocument": { "uri": h.url("a.php") },
                "range": { "start": position(start), "end": position(start + expression.len()) },
                "context": { "diagnostics": [], "only": only }
            }),
        )
        .await;
    result
        .as_array()
        .into_iter()
        .flatten()
        .filter(|row| row["title"].as_str().is_some_and(|title| title.starts_with("Extract method ")))
        .cloned()
        .collect()
}

fn applied(
    source: &str,
    expression: &str,
    action: &Value,
    uri: &str,
    method_name: &[u8],
    is_static: bool,
) -> (String, Vec<String>) {
    assert_eq!(action["kind"], "refactor.extract");
    let edits = &action["edit"]["changes"][uri];
    let rows = edits.as_array().unwrap();
    assert_eq!(rows.len(), 2);
    let insertion =
        rows.iter().find(|edit| edit["range"]["start"] == edit["range"]["end"]).unwrap()["newText"].as_str().unwrap();
    let replacement =
        rows.iter().find(|edit| edit["range"]["start"] != edit["range"]["end"]).unwrap()["newText"].as_str().unwrap();
    let changed = apply(source, edits);
    assert_eq!(changed.replace(insertion, "").replacen(replacement, expression, 1), source);
    let arena = LocalArena::new();
    let file = File::ephemeral(b"a.php".as_slice().into(), changed.as_bytes().to_vec().into());
    let program = parse_file(&arena, &file);
    assert!(!program.has_errors(), "{changed}");
    let mut nodes = vec![Node::Program(program)];
    while let Some(node) = nodes.pop() {
        if let Node::Method(method) = node
            && method.name.value == method_name
        {
            assert_eq!(method.is_static(), is_static);
            assert!(method.modifiers.iter().any(Modifier::is_private));
            let parameters = method
                .parameter_list
                .parameters
                .iter()
                .map(|parameter| {
                    assert!(
                        parameter.hint.is_some() && parameter.ampersand.is_none() && parameter.default_value.is_none()
                    );
                    String::from_utf8(parameter.variable.name.to_vec()).unwrap()
                })
                .collect();
            return (changed, parameters);
        }
        node.visit_children(|child| nodes.push(child));
    }
    panic!("missing extracted method: {changed}");
}

#[tokio::test]
async fn extract_method_uses_lexical_parameter_order_and_avoids_family_names() {
    let source = "<?php class Base { private function extracted() {} } class C extends Base { function calculate(int $a, int $b): int { return $b + $a * $b; } } class Child extends C { function extracted2() {} }";
    let expression = "$b + $a * $b";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let result = actions(&mut h, source, expression, json!(["refactor"])).await;
    assert_eq!(result.len(), 1);
    let (changed, parameters) = applied(source, expression, &result[0], &h.url("a.php"), b"extracted3", false);
    assert_eq!(parameters, ["$b", "$a"]);
    assert!(changed.contains("return $this->extracted3($b, $a);"));
    assert!(changed.contains("private function extracted3(int $b, int $a): int"));
    assert!(changed.contains("return $b + ($a * $b);"), "{changed}");
    assert!(actions(&mut h, source, "$a * $b", json!(["refactor.extract"])).await.is_empty());
    assert!(actions(&mut h, source, expression, json!(["quickfix"])).await.is_empty());
}

#[tokio::test]
async fn extract_method_preserves_staticness_short_circuit_reads_and_crlf() {
    let source = "<?php\r\nnamespace App {\r\n    class C {\r\n        // 😀 keep\r\n        public static function check(?string $a, string $b): bool { return $a !== null && $b === 'ok'; }\r\n    }\r\n}\r\n";
    let expression = "$a !== null && $b === 'ok'";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let result = actions(&mut h, source, expression, json!(["refactor.extract"])).await;
    assert_eq!(result.len(), 1);
    let (changed, parameters) = applied(source, expression, &result[0], &h.url("a.php"), b"extracted", true);
    assert_eq!(parameters, ["$a", "$b"]);
    assert!(changed.contains("return self::extracted($a, $b);"));
    assert!(changed.contains("        private static function extracted(?string $a, string $b): bool"));
    assert!(!changed.replace("\r\n", "").contains('\n'));
}

#[tokio::test]
async fn extract_method_declines_scope_and_context_hazards() {
    for (source, expression) in [
        ("<?php class C { function f(int &$a) { return $a + 1; } }", "$a + 1"),
        ("<?php class C { function f(int ...$a) { return $a; } }", "$a"),
        ("<?php class C { function &f(int $a) { return $a + 1; } }", "$a + 1"),
        ("<?php class C { function f($a) { return $a + 1; } }", "$a + 1"),
        ("<?php class C { function f(object $a) { return (string) $a; } }", "(string) $a"),
        ("<?php class C { function f(int $a) { $a = 2; return $a + 1; } }", "$a + 1"),
        ("<?php class C { function f(int $a) { mutate($a); return $a + 1; } }", "$a + 1"),
        ("<?php class C { function f(int $a) { unset($a); return $a + 1; } }", "$a + 1"),
        ("<?php class C { function f(int $a) { global $a; return $a + 1; } }", "$a + 1"),
        ("<?php class C { function f(int $a) { try { return $a + 1; } finally {} } }", "$a + 1"),
        ("<?php class C { function f(int $a) { extract([]); return $a + 1; } }", "$a + 1"),
        ("<?php class C { function f(int $a) { return $a + $undefined; } }", "$a + $undefined"),
        ("<?php class C { function f(int $a) { return self::value() + $a; } }", "self::value() + $a"),
        ("<?php class C { function f(int $a) { return $this->value + $a; } }", "$this->value + $a"),
        ("<?php class C { function f(int $a) { return $a + __LINE__; } }", "$a + __LINE__"),
        ("<?php declare(ticks=1); class C { function f(int $a) { return $a + 1; } }", "$a + 1"),
        ("<?php class C { function f(int $a) { /** @var int $a */ return $a + 1; } }", "$a + 1"),
        ("<?php class C { function f(?int $a): int { if ($a !== null) { return $a + 1; } return 0; } }", "$a + 1"),
        ("<?php class C { function f(?int $a): int { if ($a === null) { return 0; } return $a + 1; } }", "$a + 1"),
        ("<?php class C { function f(string $a = null): bool { return $a === null; } }", "$a === null"),
        ("<?php class C { /** @param numeric-string $a */ function f(string $a): int { return $a + 1; } }", "$a + 1"),
        ("<?php class C { /** @return positive-int */ function f(): int { return 42; } }", "42"),
        ("<?php class C { function f(int $a) { return $a /* keep */ + 1; } }", "$a /* keep */ + 1"),
        ("<?php class C { function f(int $a) { return $a + 1; } /** orphan */ }", "$a + 1"),
        ("<?php trait C { function f(int $a) { return $a + 1; } }", "$a + 1"),
        ("<?php class C { function f(int $a) { return function() use ($a) { return $a + 1; }; } }", "$a + 1"),
        ("<?php class C { function f(int $a) { return $a + 1; }", "$a + 1"),
    ] {
        let mut h = Harness::start(&[("a.php", source)]).await;
        assert!(actions(&mut h, source, expression, json!(["refactor.extract"])).await.is_empty(), "{source}");
    }
}

#[tokio::test]
async fn extract_method_preserves_native_types_and_return_coercion() {
    for (source, expression, signature) in [
        (
            "<?php class C { function add(int $left, int $right): int { return $left + $right; } }",
            "$left + $right",
            "private function extracted(int $left, int $right): int",
        ),
        (
            "<?php class C { function f(int $a = 1): string { return $a + 1; } }",
            "$a + 1",
            "private function extracted(int $a): string",
        ),
        (
            "<?php declare(strict_types=1); class C { function f(?string $a = null): ?string { return $a; } }",
            "$a",
            "private function extracted(?string $a): ?string",
        ),
        (
            "<?php class C { function f(string|null $a = null): bool { return $a === null; } }",
            "$a === null",
            "private function extracted(?string $a): bool",
        ),
    ] {
        let mut h = Harness::start(&[("a.php", source)]).await;
        h.open("a.php", source).await;
        let result = actions(&mut h, source, expression, json!(["refactor.extract"])).await;
        assert_eq!(result.len(), 1, "{source}");
        let (changed, _) = applied(source, expression, &result[0], &h.url("a.php"), b"extracted", false);
        assert!(changed.contains(signature), "{changed}");
        h.diagnostics().await;
        h.change("a.php", &changed, 2).await;
        let publications = h.diagnostics().await;
        let publication = publications.iter().rev().find(|item| item["uri"] == h.url("a.php")).unwrap();
        assert!(
            publication["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .all(|diagnostic| { !diagnostic["code"].as_str().is_some_and(|code| code.contains("mixed")) }),
            "{publication}"
        );
    }
}

#[tokio::test]
async fn extract_method_obeys_provider_settings() {
    let source = "<?php class C { function f(int $a) { return $a + 1; } }";
    let mut config = ServerConfig::default();
    config.configuration.language_server.disable.push(Provider::CodeAction);
    let mut h = Harness::start_with(&[("a.php", source)], config, client_capabilities()).await;
    assert!(actions(&mut h, source, "$a + 1", json!(["refactor.extract"])).await.is_empty());
}
