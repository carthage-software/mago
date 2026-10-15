use serde_json::json;
use tower_lsp_server::gen_lsp_types::Range;

use mago_database::file::File;

use super::harness::Harness;
use crate::language_server::position::offset_at_position;
use crate::language_server::position::position_at_offset;

async fn check_uses(h: &mut Harness, declarations: &str, source: &str, name: &str, target: &str, hover_text: &str) {
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let library = File::ephemeral(b"lib.php".as_slice().into(), declarations.as_bytes().to_vec().into());
    let target_start = declarations.find(target).unwrap();
    for (offset, _) in source.match_indices(name) {
        let position = position_at_offset(&file, offset as u32);
        let definition = h.at("textDocument/definition", "a.php", position.line, position.character).await;
        assert_eq!(definition["uri"], h.url("lib.php"), "{name}: {definition}");
        let range: Range = serde_json::from_value(definition["range"].clone()).unwrap();
        assert_eq!(offset_at_position(&library, range.start) as usize, target_start, "{name}: {definition}");
        let hover = h.at("textDocument/hover", "a.php", position.line, position.character).await;
        assert!(hover["contents"]["value"].as_str().is_some_and(|text| text.contains(hover_text)), "{name}: {hover}");
    }
    let position = position_at_offset(&library, target_start as u32);
    let references = h
        .request(
            "textDocument/references",
            json!({
                "textDocument":{"uri":h.url("lib.php")},"position":position,"context":{"includeDeclaration":false}
            }),
        )
        .await;
    assert_eq!(references.as_array().unwrap().len(), source.match_indices(name).count(), "{name}: {references}");
}

#[tokio::test]
async fn repeated_enum_receivers_keep_case_navigation() {
    let declarations = "<?php enum Status: string { case Ready = 'ready'; }";
    let source = "<?php echo Status::Ready->name, Status::Ready->name, Status::Ready->value, Status::Ready->value;";
    let mut h = Harness::start(&[("lib.php", declarations), ("a.php", source)]).await;
    check_uses(&mut h, declarations, source, "Ready", "Ready =", "Status\n\ncase Ready").await;
}

#[tokio::test]
async fn repeated_static_property_receivers_keep_method_navigation() {
    let declarations = "<?php class Base { public static int $value = 1; } class Box extends Base {} class Provider { public function box(): Box { return new Box(); } }";
    let source = "<?php $provider = new Provider(); echo $provider->box()::$value, $provider->box()::$value;";
    let mut h = Harness::start(&[("lib.php", declarations), ("a.php", source)]).await;
    check_uses(&mut h, declarations, source, "box()", "box():", "Provider\n\npublic function box(): Box").await;
    check_uses(&mut h, declarations, source, "$value", "$value =", "Base\n\npublic static int $value").await;
}

#[tokio::test]
async fn repeated_trait_self_returns_keep_static_property_navigation() {
    let declarations = "<?php trait Factory { public function make(): self { return $this; } } class Base { public static int $value = 1; } class Box extends Base { use Factory; }";
    let source = "<?php $box = new Box(); echo $box->make()::$value, $box->make()::$value;";
    let mut h = Harness::start(&[("lib.php", declarations), ("a.php", source)]).await;
    check_uses(&mut h, declarations, source, "$value", "$value =", "Base\n\npublic static int $value").await;
}
