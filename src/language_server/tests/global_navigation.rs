use serde_json::Value;
use serde_json::json;
use tower_lsp_server::gen_lsp_types::Range;

use mago_database::file::File;

use super::harness::Harness;
use crate::language_server::position::offset_at_position;
use crate::language_server::position::position_at_offset;

async fn at(h: &mut Harness, method: &str, path: &str, source: &str, needle: &str) -> Value {
    let file = File::ephemeral(path.as_bytes().to_vec().into(), source.as_bytes().to_vec().into());
    let position = position_at_offset(&file, source.find(needle).unwrap() as u32);
    if method == "textDocument/references" {
        h.request(
            method,
            json!({"textDocument":{"uri":h.url(path)},"position":position,"context":{"includeDeclaration":true}}),
        )
        .await
    } else {
        h.at(method, path, position.line, position.character).await
    }
}

fn selected<'a>(source: &'a str, range: &Value) -> &'a str {
    let file = File::ephemeral(b"test.php".as_slice().into(), source.as_bytes().to_vec().into());
    let range: Range = serde_json::from_value(range.clone()).unwrap();
    &source[offset_at_position(&file, range.start) as usize..offset_at_position(&file, range.end) as usize]
}

#[tokio::test]
async fn unqualified_functions_and_constants_fall_back_without_class_fallback() {
    let declarations = "<?php function shared(): int { return 1; } const FLAG = 1; const flag = 2; class Shared {}";
    let source = "<?php namespace App; /*😀*/ SHARED(); echo FLAG, flag; new Shared();";
    let mut h = Harness::start(&[("a.php", declarations), ("b.php", source)]).await;
    for (needle, name, hover_text) in
        [("SHARED()", "shared", "function shared("), ("FLAG,", "FLAG", "const FLAG"), ("flag;", "flag", "const flag")]
    {
        let definition = at(&mut h, "textDocument/definition", "b.php", source, needle).await;
        assert_eq!(definition["uri"], h.url("a.php"));
        assert_eq!(selected(declarations, &definition["range"]), name);
        let hover = at(&mut h, "textDocument/hover", "b.php", source, needle).await;
        assert!(hover["contents"]["value"].as_str().unwrap().contains(hover_text));
        assert_eq!(selected(source, &hover["range"]), needle.trim_end_matches(['(', ')', ',', ';']));
        let uses = at(&mut h, "textDocument/references", "b.php", source, needle).await;
        assert_eq!(uses.as_array().unwrap().len(), 2);
        assert_eq!(uses, at(&mut h, "textDocument/references", "a.php", declarations, name).await);
    }
    assert!(at(&mut h, "textDocument/definition", "b.php", source, "Shared()").await.is_null());
    assert!(at(&mut h, "textDocument/hover", "b.php", source, "Shared()").await.is_null());
    let classes = at(&mut h, "textDocument/references", "a.php", declarations, "Shared {}").await;
    assert_eq!(classes.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn imports_and_qualified_names_do_not_borrow_global_targets() {
    let declarations = "<?php function shared(): void {} const FLAG = 1;";
    let source = "<?php namespace App; use function Missing\\shared as imported; use const Missing\\FLAG as IMPORTED; function caller(): void { shared(); echo FLAG; imported(); Missing\\shared(); namespace\\shared(); echo IMPORTED, Missing\\FLAG, namespace\\FLAG; }";
    let mut h = Harness::start(&[("a.php", declarations), ("b.php", source)]).await;
    for needle in
        ["imported();", "Missing\\shared();", "namespace\\shared();", "IMPORTED,", "Missing\\FLAG,", "namespace\\FLAG;"]
    {
        assert!(at(&mut h, "textDocument/definition", "b.php", source, needle).await.is_null(), "{needle}");
        assert!(at(&mut h, "textDocument/hover", "b.php", source, needle).await.is_null(), "{needle}");
    }
    for (needle, use_text) in [("shared()", "shared"), ("FLAG =", "FLAG")] {
        let references = at(&mut h, "textDocument/references", "a.php", declarations, needle).await;
        assert_eq!(references.as_array().unwrap().len(), 2, "{references}");
        let reference =
            references.as_array().unwrap().iter().find(|reference| reference["uri"] == h.url("b.php")).unwrap();
        assert_eq!(selected(source, &reference["range"]), use_text);
    }
}

#[tokio::test]
async fn namespaced_declarations_shadow_globals_and_explicit_names_keep_their_target() {
    let global = "<?php function shared(): int { return 1; } const FLAG = 1;";
    let local = "<?php namespace App; function shared(): string { return ''; } const FLAG = 2;";
    let source = "<?php namespace App; shared(); echo FLAG; \\shared(); echo \\FLAG;";
    let mut h = Harness::start(&[("a.php", global), ("b.php", local), ("c.php", source)]).await;
    for (needle, expected, text) in [
        ("shared();", "b.php", "App\n\nfunction shared("),
        ("FLAG;", "b.php", "app\n\nconst FLAG"),
        ("\\shared();", "a.php", "function shared("),
        ("\\FLAG;", "a.php", "const FLAG"),
    ] {
        let definition = at(&mut h, "textDocument/definition", "c.php", source, needle).await;
        assert_eq!(definition["uri"], h.url(expected));
        let hover = at(&mut h, "textDocument/hover", "c.php", source, needle).await;
        assert!(hover["contents"]["value"].as_str().unwrap().contains(text), "{needle}: {hover}");
        assert_eq!(at(&mut h, "textDocument/references", "c.php", source, needle).await.as_array().unwrap().len(), 2);
    }
}

#[tokio::test]
async fn imported_globals_keep_constant_case_and_namespace_case_rules() {
    let declarations = "<?php namespace Lib; function work(): void {} const Flag = 1; const FLAG = 2;";
    let source = "<?php namespace App; use function Lib\\work as run; use const Lib\\Flag as VALUE; run(); echo VALUE, \\LIB\\Flag, \\lib\\FLAG;";
    let mut h = Harness::start(&[("a.php", declarations), ("b.php", source)]).await;
    for (needle, name, text, count) in [
        ("run();", "work", "Lib\n\nfunction work(", 3),
        ("VALUE,", "Flag", "lib\n\nconst Flag", 4),
        ("\\lib\\FLAG;", "FLAG", "lib\n\nconst FLAG", 2),
    ] {
        let definition = at(&mut h, "textDocument/definition", "b.php", source, needle).await;
        assert_eq!(selected(declarations, &definition["range"]), name);
        let hover = at(&mut h, "textDocument/hover", "b.php", source, needle).await;
        assert!(hover["contents"]["value"].as_str().unwrap().contains(text), "{needle}: {hover}");
        assert_eq!(
            at(&mut h, "textDocument/references", "b.php", source, needle).await.as_array().unwrap().len(),
            count
        );
    }
}

#[tokio::test]
async fn fallback_references_change_when_a_namespaced_declaration_appears() {
    let global = "<?php function shared(): void {} const FLAG = 1;";
    let initial = "<?php namespace App; shared(); echo FLAG;";
    let mut h = Harness::start(&[("a.php", global), ("b.php", initial)]).await;
    for needle in ["shared()", "FLAG ="] {
        assert_eq!(at(&mut h, "textDocument/references", "a.php", global, needle).await.as_array().unwrap().len(), 2);
    }
    h.open("b.php", "<?php namespace App; function shared(): void {} const FLAG = 2; shared(); echo FLAG;").await;
    for needle in ["shared()", "FLAG ="] {
        assert_eq!(at(&mut h, "textDocument/references", "a.php", global, needle).await.as_array().unwrap().len(), 1);
    }
    h.close("b.php").await;
    for needle in ["shared()", "FLAG ="] {
        assert_eq!(at(&mut h, "textDocument/references", "a.php", global, needle).await.as_array().unwrap().len(), 2);
    }
}
