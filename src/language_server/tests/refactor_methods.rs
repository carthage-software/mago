use mago_allocator::LocalArena;
use mago_database::file::File;
use mago_php_version::PHPVersion;
use mago_syntax::parser::parse_file;
use serde_json::{Value, json};

use super::harness::{Harness, client_capabilities};
use super::references::apply;
use crate::config::language_server::Provider;
use crate::language_server::ServerConfig;

const CONTRACT: &str = "<?php namespace Lib; class Value {} interface Contract { function &build(Value &$value, array $options = [Value::class]): ?Value; static function make(): static; }";

async fn actions(h: &mut Harness, source: &str, needle: &str, only: Value) -> Vec<Value> {
    let offset = source.find(needle).unwrap();
    let before = &source[..offset];
    let position = json!({ "line": before.bytes().filter(|byte| *byte == b'\n').count(),
        "character": before.rsplit('\n').next().unwrap().encode_utf16().count() });
    let result = h
        .request(
            "textDocument/codeAction",
            json!({
                "textDocument": { "uri": h.url("a.php") },
                "range": { "start": position, "end": position }, "context": { "diagnostics": [], "only": only }
            }),
        )
        .await;
    result
        .as_array()
        .into_iter()
        .flatten()
        .filter(|action| action["title"] == "Implement inherited methods")
        .cloned()
        .collect()
}

fn checked_edit(source: &str, action: &Value, uri: &str) -> String {
    assert_eq!(action["kind"], "quickfix");
    let edits = &action["edit"]["changes"][uri];
    assert_eq!(edits.as_array().unwrap().len(), 1);
    assert_eq!(edits[0]["range"]["start"], edits[0]["range"]["end"]);
    let changed = apply(source, edits);
    let arena = LocalArena::new();
    let file = File::ephemeral(b"a.php".as_slice().into(), changed.as_bytes().to_vec().into());
    assert!(!parse_file(&arena, &file).has_errors(), "{changed}");
    changed
}

#[tokio::test]
async fn implement_methods_preserves_existing_bytes_indentation_and_crlf() {
    let source = "<?php\r\nnamespace App {\r\n    /** 😀 Keep class docs. */\r\n    final class C implements \\Lib\\Contract\r\n    {\r\n        // Keep this method.\r\n        public function untouched( ) { return 1; }\r\n    }\r\n}\r\n";
    let mut h = Harness::start(&[("a.php", "<?php"), ("lib.php", CONTRACT)]).await;
    h.open("a.php", source).await;
    let result = actions(&mut h, source, "C implements", json!(["quickfix"])).await;
    assert_eq!(result.len(), 1);
    let changed = checked_edit(source, &result[0], &h.url("a.php"));
    assert!(changed.contains("        public function untouched( ) { return 1; }\r\n\r\n"));
    for expected in [
        "        public function &build(\\Lib\\Value &$value",
        "\\Lib\\Value::class",
        "        public static function make(): static",
        "            throw new \\LogicException('Not implemented.');",
    ] {
        assert!(changed.contains(expected), "missing {expected}: {changed}");
    }
    let insertion = result[0]["edit"]["changes"][h.url("a.php")][0]["newText"].as_str().unwrap();
    assert!(!insertion.replace("\r\n", "").contains('\n'));
    assert_eq!(changed.replace(insertion, ""), source);
    assert!(actions(&mut h, source, "untouched", json!(["quickfix"])).await.is_empty());
    assert!(actions(&mut h, source, "C implements", json!(["refactor"])).await.is_empty());
}

#[tokio::test]
async fn implement_methods_supports_abstract_parents_and_enums() {
    for (source, needle, expected) in [
        (
            "<?php abstract class Base { abstract protected function run(): self; } class C extends Base {}",
            "C extends",
            "public function run(): \\Base",
        ),
        (
            "<?php interface I { function text(): string; } enum E: int implements I { case A = 1; }",
            "E: int",
            "public function text(): string",
        ),
        ("<?php class C implements \\Countable {}", "C implements", "public function count(): int"),
    ] {
        let mut h = Harness::start(&[("a.php", source)]).await;
        let result = actions(&mut h, source, needle, json!(["quickfix"])).await;
        assert_eq!(result.len(), 1, "{source}");
        assert!(checked_edit(source, &result[0], &h.url("a.php")).contains(expected));
    }
}

#[tokio::test]
async fn implement_methods_marks_overrides_but_not_constructors_from_php83() {
    let source = "<?php namespace App; class Override {} interface Contract { function __construct(); function send(string $message, int $count = 0): bool; } class C implements Contract {}";
    for version in [PHPVersion::PHP82, PHPVersion::PHP83] {
        let mut config = ServerConfig::default();
        config.configuration.php_version = version;
        let mut h = Harness::start_with(&[("a.php", source)], config, client_capabilities()).await;
        h.open("a.php", source).await;
        let result = actions(&mut h, source, "C implements", json!(["quickfix"])).await;
        assert_eq!(result.len(), 1);
        let changed = checked_edit(source, &result[0], &h.url("a.php"));
        assert_eq!(changed.matches("#[\\Override]").count(), usize::from(version >= PHPVersion::PHP83));
        if version >= PHPVersion::PHP83 {
            assert!(changed.contains("#[\\Override]\n    public function send("), "{changed}");
        }
        h.diagnostics().await;
        h.change("a.php", &changed, 2).await;
        let publications = h.diagnostics().await;
        let publication = publications.iter().rev().find(|item| item["uri"] == h.url("a.php")).unwrap();
        assert!(
            publication["diagnostics"].as_array().unwrap().iter().all(|diagnostic| {
                !matches!(
                    diagnostic["code"].as_str(),
                    Some("missing-override-attribute" | "invalid-override-attribute")
                )
            }),
            "{publication}"
        );
    }
}

#[tokio::test]
async fn implement_methods_declines_unsafe_or_unneeded_edits() {
    for source in [
        "<?php class C implements \\Lib\\Contract { /** orphan docs */ }",
        "<?php class C implements \\Lib\\Contract {",
        "<?php class C implements \\Lib\\Contract { function broken( }",
        "<?php class C {}",
        "<?php interface I { function run(): void; } class C implements I { function run(): void {} }",
        "<?php interface I { function good(): void; #[Marker] function bad(): void; } class C implements I {}",
        "<?php interface I { function run(int $x): void; } interface J { function run(string $x): void; } class C implements I, J {}",
    ] {
        let mut h = Harness::start(&[("a.php", source), ("lib.php", CONTRACT)]).await;
        assert!(actions(&mut h, source, "class C", json!(["quickfix"])).await.is_empty(), "{source}");
    }
}

#[tokio::test]
async fn implement_methods_obeys_provider_settings() {
    let source = "<?php class C implements \\Lib\\Contract {}";
    let mut config = ServerConfig::default();
    config.configuration.language_server.disable.push(Provider::CodeAction);
    let mut h = Harness::start_with(&[("a.php", source), ("lib.php", CONTRACT)], config, client_capabilities()).await;
    assert!(actions(&mut h, source, "class C", json!(["quickfix"])).await.is_empty());
}

#[tokio::test]
async fn implement_methods_declines_forbidden_enum_methods_and_fallback_defaults() {
    for (source, needle) in [
        (
            "<?php interface I { function good(): void; function __construct(); } enum E implements I { case A; }",
            "enum E",
        ),
        (
            "<?php interface I { function good(): void; function __toString(): string; } enum E implements I { case A; }",
            "enum E",
        ),
        (
            r"<?php namespace { const LIMIT = 1; } namespace Lib { if (false) { define('Lib\LIMIT', 2); } interface I { function run($n = LIMIT); } } namespace App { class C implements \Lib\I {} }",
            "class C",
        ),
    ] {
        let mut h = Harness::start(&[("a.php", source)]).await;
        assert!(actions(&mut h, source, needle, json!(["quickfix"])).await.is_empty(), "{source}");
    }
}
