use serde_json::{Value, json};

use mago_database::file::File;
use mago_php_version::PHPVersion;

use super::harness::{Harness, client_capabilities};
use super::references::apply;
use crate::language_server::ServerConfig;
use crate::language_server::position::position_at_offset;

async fn actions(h: &mut Harness, source: &str, needle: &str, only: Value) -> Vec<Value> {
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let start = source.rfind(needle).unwrap() as u32;
    h.request("textDocument/codeAction", json!({
        "textDocument":{"uri":h.url("a.php")},
        "range":{"start":position_at_offset(&file, start),"end":position_at_offset(&file, start + needle.len() as u32)},
        "context":{"diagnostics":[],"only":only}
    })).await.as_array().unwrap().iter().filter(|action| {
        action["title"].as_str().is_some_and(|title| title.starts_with("Extract to self::"))
    }).cloned().collect()
}

#[tokio::test]
async fn extract_constant_preserves_crlf_utf16_and_method_documentation() {
    let source = "<?php\r\nclass C {\r\n\t/** Method docs. */\r\n\t#[Deprecated]\r\n\tpublic function f() { echo '😀'; return 'é'; }\r\n}\r\n";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let result = actions(&mut h, source, "'é'", json!(["refactor.extract"])).await;
    assert_eq!(result.len(), 1, "{result:?}");
    assert_eq!(result[0]["kind"], "refactor.extract");
    assert_eq!(result[0]["title"], "Extract to self::EXTRACTED");
    assert_eq!(
        apply(source, &result[0]["edit"]["changes"][h.url("a.php")]),
        "<?php\r\nclass C {\r\n\t/** Method docs. */\r\n\t#[Deprecated]\r\n\tpublic function f() { echo '😀'; return self::EXTRACTED; }\r\n\r\n\tprivate const string EXTRACTED = 'é';\r\n}\r\n"
    );
    for only in [json!(["quickfix"]), json!(["refactor.rewrite"]), json!([]), json!(["refac"])] {
        assert!(actions(&mut h, source, "'é'", only).await.is_empty());
    }
    assert!(actions(&mut h, source, "é", json!(["refactor.extract"])).await.is_empty());
}

#[tokio::test]
async fn extract_constant_types_literals_without_changing_values() {
    for (value, hint) in [
        ("(-42)", "int"),
        ("+2.5", "float"),
        ("true", "bool"),
        ("false", "bool"),
        ("null", "null"),
        ("9223372036854775808", "float"),
        ("18446744073709551616", "float"),
        ("[1, 'key' => [true, null, -2.5], -1 => false]", "array"),
        ("array('key' => [1, 2],)", "array"),
    ] {
        let source = format!("<?php class C {{ function f() {{ return {value}; }} }}");
        let mut h = Harness::start(&[("a.php", &source)]).await;
        let result = actions(&mut h, &source, value, json!(["refactor"])).await;
        assert_eq!(result.len(), 1, "{source}: {result:?}");
        assert_eq!(
            apply(&source, &result[0]["edit"]["changes"][h.url("a.php")]),
            format!(
                "<?php class C {{ function f() {{ return self::EXTRACTED; }}\n\n    private const {hint} EXTRACTED = {value};\n}}"
            )
        );
    }
}

#[tokio::test]
async fn extract_constant_obeys_php_version_and_keeps_existing_blank_line() {
    let source = "<?php\nfinal class C {\n    public function answer(): int { return 42; }\n\n}\n";
    for version in [PHPVersion::PHP71, PHPVersion::PHP82, PHPVersion::PHP83, PHPVersion::PHP84] {
        let mut config = ServerConfig::default();
        config.configuration.php_version = version;
        let mut h = Harness::start_with(&[("a.php", source)], config, client_capabilities()).await;
        h.open("a.php", source).await;
        let result = actions(&mut h, source, "42", json!(["refactor.extract"])).await;
        assert_eq!(result.len(), 1, "{version}: {result:?}");
        let edits = &result[0]["edit"]["changes"][h.url("a.php")];
        assert_eq!(edits.as_array().unwrap().len(), 2);
        let changed = apply(source, edits);
        let hint = if version >= PHPVersion::PHP83 { "int " } else { "" };
        assert_eq!(
            changed,
            format!(
                "<?php\nfinal class C {{\n    public function answer(): int {{ return self::EXTRACTED; }}\n\n    private const {hint}EXTRACTED = 42;\n}}\n"
            )
        );
        h.diagnostics().await;
        h.change("a.php", &changed, 2).await;
        let publications = h.diagnostics().await;
        let publication = publications.iter().rev().find(|item| item["uri"] == h.url("a.php")).unwrap();
        assert!(
            publication["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .all(|diagnostic| { diagnostic["code"].as_str() != Some("missing-constant-type") }),
            "{publication}"
        );
    }
}

#[tokio::test]
async fn extract_constant_avoids_own_inherited_trait_interface_and_case_names() {
    let declarations = "<?php interface I { const EXTRACTED = 1; } trait T { const EXTRACTED_2 = 2; } class Base { const EXTRACTED_3 = 3; }";
    let source = "<?php class C extends Base implements I { use T; private const EXTRACTED_4 = 4; function f() { echo self::EXTRACTED_5; return 42; } }";
    let mut h = Harness::start(&[("a.php", source), ("base.php", declarations)]).await;
    let result = actions(&mut h, source, "42", json!(["refactor.extract"])).await;
    assert_eq!(result.len(), 1, "{result:?}");
    assert_eq!(result[0]["title"], "Extract to self::EXTRACTED_6");
    assert_eq!(
        apply(source, &result[0]["edit"]["changes"][h.url("a.php")]),
        "<?php class C extends Base implements I { use T; private const EXTRACTED_4 = 4; function f() { echo self::EXTRACTED_5; return self::EXTRACTED_6; }\n\n    private const int EXTRACTED_6 = 42;\n}"
    );
    let source = "<?php enum E { case EXTRACTED; function f() { return 42; } }";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let result = actions(&mut h, source, "42", json!(["refactor.extract"])).await;
    assert_eq!(result.len(), 1, "{result:?}");
    assert_eq!(
        apply(source, &result[0]["edit"]["changes"][h.url("a.php")]),
        "<?php enum E { case EXTRACTED; function f() { return self::EXTRACTED_2; }\n\n    private const int EXTRACTED_2 = 42;\n}"
    );
}

#[tokio::test]
async fn extract_constant_declines_unsafe_values_and_non_method_scopes() {
    for value in [
        "1 + 2",
        "__CLASS__",
        "PHP_VERSION",
        "self::VALUE",
        "$x",
        "make()",
        "new C()",
        "\"value $x\"",
        "[&$x]",
        "[...[]]",
        "[false => 1]",
        "[1.5 => 2]",
        "[work()]",
    ] {
        let source = format!("<?php class C {{ function f($x) {{ return {value}; }} }}");
        let mut h = Harness::start(&[("a.php", &source)]).await;
        assert!(actions(&mut h, &source, value, json!(["refactor.extract"])).await.is_empty(), "{source}");
    }
    for source in [
        "<?php function f() { return 42; }",
        "<?php trait T { function f() { return 42; } }",
        "<?php $c = new class { function f() { return 42; } };",
        "<?php class C { function f() { return function () { return 42; }; } }",
        "<?php class C { function f() { return fn () => 42; } }",
        "<?php class C { function f() { function nested() { return 42; } } }",
        "<?php class C { function f() { interface I { function g($x = 42); } } }",
        "<?php class C { function f($x = 42) {} }",
        "<?php class C { public $x = 42; }",
        "<?php class C extends Missing { function f() { return 42; } }",
        "<?php class C { function f() { return 42; } /** orphan */ }",
        "<?php class C { function f() { return 42; } } echo __LINE__;",
        "<?php class C { function f() { return 42; }",
    ] {
        let mut h = Harness::start(&[("a.php", source)]).await;
        assert!(actions(&mut h, source, "42", json!(["refactor.extract"])).await.is_empty(), "{source}");
    }
}

#[tokio::test]
async fn extract_constant_declines_directives_and_loop_levels() {
    for body in ["declare(ticks=1) { echo 'x'; }", "while (true) { break 1; }", "while (true) { continue 1; }"] {
        let source = format!("<?php class C {{ function f() {{ {body} }} }}");
        let mut h = Harness::start(&[("a.php", &source)]).await;
        assert!(actions(&mut h, &source, "1", json!(["refactor.extract"])).await.is_empty(), "{source}");
    }
}
