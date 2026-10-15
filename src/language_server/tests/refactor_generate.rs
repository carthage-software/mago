use mago_allocator::LocalArena;
use mago_database::file::File;
use mago_php_version::PHPVersion;
use mago_span::HasSpan;
use mago_syntax::cst::{Modifier, Node};
use mago_syntax::parser::parse_file;
use serde_json::{Value, json};

use super::harness::{Harness, client_capabilities};
use super::references::apply;
use crate::config::language_server::Provider;
use crate::language_server::ServerConfig;

async fn actions(h: &mut Harness, source: &str, needle: &str, only: Value) -> Vec<Value> {
    let before = &source[..source.find(needle).unwrap()];
    let position = json!({ "line": before.bytes().filter(|byte| *byte == b'\n').count(),
        "character": before.rsplit('\n').next().unwrap().encode_utf16().count() });
    let result = h
        .request(
            "textDocument/codeAction",
            json!({
                "textDocument": { "uri": h.url("a.php") }, "range": { "start": position, "end": position },
                "context": { "diagnostics": [], "only": only }
            }),
        )
        .await;
    result
        .as_array()
        .into_iter()
        .flatten()
        .filter(|row| row["title"].as_str().is_some_and(|title| title.starts_with("Generate method ")))
        .cloned()
        .collect()
}

fn applied_parameters(source: &str, action: &Value, uri: &str, name: &[u8], is_static: bool) -> Vec<String> {
    applied_signature(source, action, uri, name, is_static).into_iter().map(|(name, _)| name).collect()
}

fn applied_signature(
    source: &str,
    action: &Value,
    uri: &str,
    name: &[u8],
    is_static: bool,
) -> Vec<(String, Option<String>)> {
    let edits = &action["edit"]["changes"][uri];
    assert_eq!(edits.as_array().unwrap().len(), 1);
    assert_eq!(edits[0]["range"]["start"], edits[0]["range"]["end"]);
    let changed = apply(source, edits);
    let inserted = edits[0]["newText"].as_str().unwrap();
    assert_eq!(changed.replace(inserted, ""), source);
    assert!(inserted.contains("throw new \\LogicException('Not implemented.');"));
    let arena = LocalArena::new();
    let file = File::ephemeral(b"a.php".as_slice().into(), changed.as_bytes().to_vec().into());
    let program = parse_file(&arena, &file);
    assert!(!program.has_errors(), "{changed}");
    let mut nodes = vec![Node::Program(program)];
    while let Some(node) = nodes.pop() {
        if let Node::Method(method) = node
            && method.name.value == name
        {
            assert_eq!(method.is_static(), is_static);
            assert!(method.modifiers.iter().any(Modifier::is_private));
            assert!(method.return_type_hint.is_none());
            return method
                .parameter_list
                .parameters
                .iter()
                .map(|parameter| {
                    assert!(parameter.default_value.is_none() && parameter.ellipsis.is_none());
                    (
                        String::from_utf8(parameter.variable.name[1..].to_vec()).unwrap(),
                        parameter
                            .hint
                            .as_ref()
                            .map(|hint| changed[hint.start_offset() as usize..hint.end_offset() as usize].to_owned()),
                    )
                })
                .collect();
        }
        node.visit_children(|child| nodes.push(child));
    }
    panic!("generated method missing: {changed}");
}

#[tokio::test]
async fn generate_method_preserves_named_argument_binding_and_positional_slots() {
    let source = "<?php namespace App; class C { function run() { $this->Build(first(), second(), arg1: 3, arg3: 4, same: 5); } }";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let result = actions(&mut h, source, "Build(", json!(["quickfix"])).await;
    assert_eq!(result.len(), 1);
    assert_eq!(result[0]["kind"], "quickfix");
    assert_eq!(
        applied_parameters(source, &result[0], &h.url("a.php"), b"Build", false),
        ["arg2", "arg4", "arg1", "arg3", "same"]
    );
    assert!(actions(&mut h, source, "first()", json!(["quickfix"])).await.is_empty());
    assert!(actions(&mut h, source, "Build(", json!(["refactor"])).await.is_empty());
}

#[tokio::test]
async fn generate_method_uses_variable_names_with_safe_fallbacks() {
    for (arguments, expected) in [
        ("$message, ($count), $café", vec!["message", "count", "café"]),
        ("$arg1, $arg1, $label, 1, label: 2, arg3: 3", vec!["arg1", "arg2", "arg4", "arg5", "label", "arg3"]),
        (
            concat!(
                "$this, $GLOBALS, $_SERVER, $_GET, $_POST, $_FILES, $_COOKIE, $_SESSION, $_REQUEST, $_ENV, ",
                "'hello', 42, $value->field, $value . '!', $$value",
            ),
            vec![
                "arg1", "arg2", "arg3", "arg4", "arg5", "arg6", "arg7", "arg8", "arg9", "arg10", "arg11", "arg12",
                "arg13", "arg14", "arg15",
            ],
        ),
    ] {
        let source = format!("<?php class C {{ function run() {{ $this->deliver({arguments}); }} }}");
        let mut h = Harness::start(&[("a.php", &source)]).await;
        let result = actions(&mut h, &source, "deliver(", json!(["quickfix"])).await;
        assert_eq!(result.len(), 1, "{arguments}");
        assert_eq!(
            applied_parameters(&source, &result[0], &h.url("a.php"), b"deliver", false),
            expected,
            "{arguments}"
        );
    }
}

#[tokio::test]
async fn generate_method_distinguishes_static_calls_and_preserves_crlf() {
    let source = "<?php\r\nnamespace App {\r\n    class C {\r\n        public static function run() { $😀 = 1; self::create(label: 'x'); }\r\n    }\r\n}\r\n";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let result = actions(&mut h, source, "create(", json!(["quickfix"])).await;
    assert_eq!(result.len(), 1);
    assert_eq!(applied_parameters(source, &result[0], &h.url("a.php"), b"create", true), ["label"]);
    let inserted = result[0]["edit"]["changes"][h.url("a.php")][0]["newText"].as_str().unwrap();
    assert!(inserted.contains("        private static function create(string $label)"));
    assert!(!inserted.replace("\r\n", "").contains('\n'));
}

#[tokio::test]
async fn generate_method_widens_literals_and_uses_inferred_variable_types() {
    let source = r#"<?php
namespace App;
class Payload {}
class GenerateCheck {
    public function send(string $message, ?payload $payload, int|string $id, object $object, Payload|string|null $choice): void {
        self::deliver(($message), $payload, $id, $object, $choice, priority: 2, text: 'hello', yes: true,
            no: false, ratio: -1.5, count: +42, huge: 9223372036854775808, empty: null);
    }
}
"#;
    let mut h = Harness::start(&[("a.php", source)]).await;
    let result = actions(&mut h, source, "deliver(", json!(["quickfix"])).await;
    assert_eq!(result.len(), 1);
    let signature = applied_signature(source, &result[0], &h.url("a.php"), b"deliver", true);
    assert_eq!(
        signature,
        [
            ("message", Some("string")),
            ("payload", Some("?\\App\\Payload")),
            ("id", Some("int|string")),
            ("object", Some("object")),
            ("choice", Some("\\App\\Payload|string|null")),
            ("priority", Some("int")),
            ("text", Some("string")),
            ("yes", Some("bool")),
            ("no", Some("bool")),
            ("ratio", Some("float")),
            ("count", Some("int")),
            ("huge", Some("float")),
            ("empty", None),
        ]
        .map(|(name, hint)| (name.to_owned(), hint.map(str::to_owned)))
    );
}

#[tokio::test]
async fn generate_method_leaves_unsupported_unions_and_unrecorded_expressions_unhinted() {
    let source = r#"<?php
namespace App;
class Payload { public string $message = ''; }
class GenerateCheck {
    /**
     * @param resource|int $resource
     * @param array{value: int} $shape
     */
    public function send($resource, array $shape, Payload $payload, $unknown): void {
        $anonymous = new class {};
        $this->deliver($resource, $shape, $unknown, $anonymous, ($payload)->message,
            $payload->message, $payload, [1], 18446744073709551616);
    }
}
"#;
    let mut h = Harness::start(&[("a.php", source)]).await;
    let result = actions(&mut h, source, "deliver(", json!(["quickfix"])).await;
    assert_eq!(result.len(), 1);
    let signature = applied_signature(source, &result[0], &h.url("a.php"), b"deliver", false);
    assert_eq!(
        signature.into_iter().map(|(_, hint)| hint).collect::<Vec<_>>(),
        [None, None, None, None, None, None, Some("\\App\\Payload".to_owned()), None, Some("float".to_owned())]
    );
}

#[tokio::test]
async fn generate_method_checks_target_php_version_for_native_hints() {
    let source = r#"<?php class C {
    /**
     * @param string|null $nullable
     * @param int|string $union
     * @param object $object
     */
    function run($nullable, $union, $object) { $this->deliver($nullable, $union, $object, 42); }
}"#;
    for (version, expected) in [
        (PHPVersion::PHP70, [None, None, None, Some("int")]),
        (PHPVersion::PHP71, [Some("?string"), None, None, Some("int")]),
        (PHPVersion::PHP72, [Some("?string"), None, Some("object"), Some("int")]),
        (PHPVersion::PHP80, [Some("?string"), Some("int|string"), Some("object"), Some("int")]),
    ] {
        let mut config = ServerConfig::default();
        config.configuration.php_version = version;
        let mut h = Harness::start_with(&[("a.php", source)], config, client_capabilities()).await;
        let result = actions(&mut h, source, "deliver(", json!(["quickfix"])).await;
        assert_eq!(result.len(), 1, "{version}");
        let signature = applied_signature(source, &result[0], &h.url("a.php"), b"deliver", false);
        assert_eq!(
            signature.into_iter().map(|(_, hint)| hint).collect::<Vec<_>>(),
            expected.map(|hint| hint.map(str::to_owned)),
            "{version}"
        );
    }
}

#[tokio::test]
async fn generate_method_uses_current_unsaved_argument_types() {
    let source = "<?php class C { function run(int $value) { $this->deliver($value); } }";
    let mut h = Harness::start(&[("a.php", source)]).await;
    h.open("a.php", source).await;
    h.diagnostics().await;
    let result = actions(&mut h, source, "deliver(", json!(["quickfix"])).await;
    assert_eq!(result.len(), 1);
    assert_eq!(
        applied_signature(source, &result[0], &h.url("a.php"), b"deliver", false),
        [("value".to_owned(), Some("int".to_owned()))]
    );
    let changed = source.replace("int $value", "string $value");
    h.change("a.php", &changed, 2).await;
    h.diagnostics().await;
    let result = actions(&mut h, &changed, "deliver(", json!(["quickfix"])).await;
    assert_eq!(result.len(), 1);
    assert_eq!(
        applied_signature(&changed, &result[0], &h.url("a.php"), b"deliver", false),
        [("value".to_owned(), Some("string".to_owned()))]
    );
    assert_eq!(std::fs::read_to_string(h.workspace.join("a.php")).unwrap(), source);
}

#[tokio::test]
async fn generate_method_declines_ambiguous_or_invalid_calls() {
    for source in [
        "<?php class C { function run() { $this->missing(...$args); } }",
        "<?php class C { function run() { $this->missing(...); } }",
        "<?php class C { function run() { $this->missing(x: 1, x: 2); } }",
        "<?php class C { function run() { $this->missing(x: 1, 2); } }",
        "<?php class C { function run() { $this->missing(this: 1); } }",
        "<?php class C { function run() { $this?->missing(); } }",
        "<?php class C { function run() { $other->missing(); } }",
        "<?php class C { function run() { static::missing(); } }",
        "<?php class C { function run() { parent::missing(); } }",
        "<?php class C { function run() { $this->{'missing'}(); } }",
        "<?php class C { static function run() { $this->missing(); } }",
        "<?php class C { function run() { $f = fn() => $this->missing(); } }",
        "<?php class C { function run() { $f = function() { $this->missing(); }; } }",
        "<?php trait C { function run() { $this->missing(); } }",
        "<?php $c = new class { function run() { $this->missing(); } };",
        "<?php class Base { private function missing() {} } class C extends Base { function run() { $this->missing(); } }",
        "<?php class C { function missing() {} function run() { $this->MISSING(); } }",
        "<?php class C { function __call($name, $args) {} function run() { $this->missing(); } }",
        "<?php class C { static function __callStatic($name, $args) {} function run() { self::missing(); } }",
        "<?php /** @method void missing() */ class C { function run() { $this->missing(); } }",
        "<?php class C { function run() { $this->missing(); } } class Child extends C { function missing() {} }",
        "<?php class C extends Unknown { function run() { $this->missing(); } }",
        "<?php class C { function run() { $this->missing(); } /** orphan */ }",
        "<?php class C { function run() { $this->missing(); }",
    ] {
        let mut h = Harness::start(&[("a.php", source)]).await;
        let offset = ["->missing", "::missing", "->MISSING"]
            .iter()
            .find_map(|prefix| source.find(prefix).map(|offset| offset + 2))
            .unwrap_or_else(|| source.find("missing").unwrap());
        assert!(actions(&mut h, source, &source[offset..], json!(["quickfix"])).await.is_empty(), "{source}");
    }
    for name in [
        "__construct",
        "__DESTRUCT",
        "__clone",
        "__CALL",
        "__callStatic",
        "__get",
        "__set",
        "__isset",
        "__unset",
        "__sleep",
        "__wakeup",
        "__serialize",
        "__unserialize",
        "__toString",
        "__invoke",
        "__set_state",
        "__debugInfo",
    ] {
        let source = format!("<?php class C {{ function run() {{ $this->{name}(); }} }}");
        let mut h = Harness::start(&[("a.php", &source)]).await;
        assert!(actions(&mut h, &source, name, json!(["quickfix"])).await.is_empty(), "{source}");
    }
}

#[tokio::test]
async fn generate_method_obeys_provider_settings() {
    let source = "<?php class C { function run() { $this->missing(); } }";
    let mut config = ServerConfig::default();
    config.configuration.language_server.disable.push(Provider::CodeAction);
    let mut h = Harness::start_with(&[("a.php", source)], config, client_capabilities()).await;
    assert!(actions(&mut h, source, "missing", json!(["quickfix"])).await.is_empty());
}
