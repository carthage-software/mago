use serde_json::Value;
use tower_lsp_server::gen_lsp_types::TextEdit;

use mago_allocator::LocalArena;
use mago_database::file::File;
use mago_syntax::parser::parse_file_borrowed_with_settings;

use crate::language_server::position::{offset_at_position, position_at_offset};

use super::harness::Harness;

const LIBRARY: &str = r"<?php namespace Build;
class CtorNone {}
class CtorPublic { public function __construct() {} }
class CtorPrivate { private function __construct() {} }
class CtorProtected { protected function __construct() {} }
class CtorPrivateChild extends CtorPrivate {}
class CtorProtectedChild extends CtorProtected {}
class CtorPublicChild extends CtorPrivate { public function __construct() {} }
trait OpenConstructor { public function __construct() {} }
trait HiddenConstructor { private function __construct() {} }
class CtorTraitPrivate { use OpenConstructor { __construct as private; } }
class CtorTraitPublic { use HiddenConstructor { __construct as public; } }
class CtorTraitPrivateChild extends CtorTraitPrivate {}
class CtorTraitPublicChild extends CtorTraitPublic {}
";

async fn check(h: &mut Harness, marked: &str, selected: &str, absent: &[&str], version: i32) {
    let offset = marked.find("<cursor>").unwrap();
    let source = marked.replace("<cursor>", "");
    h.change("a.php", &source, version).await;
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let position = position_at_offset(&file, offset as u32);
    let result = h.at("textDocument/completion", "a.php", position.line, position.character).await;
    let items = result["items"].as_array().unwrap();
    for label in absent {
        assert!(!items.iter().any(|item| item["label"] == *label), "inaccessible {label}, {marked}: {items:?}");
    }
    let item: &Value = items
        .iter()
        .find(|item| item["label"] == selected)
        .unwrap_or_else(|| panic!("missing {selected}, {marked}: {items:?}"));
    assert_ne!(item["insertTextFormat"], 2, "existing constructor arguments must stay unchanged");
    let edit: TextEdit = serde_json::from_value(item["textEdit"].clone()).unwrap();
    let mut applied = source.clone();
    applied.replace_range(
        offset_at_position(&file, edit.range.start) as usize..offset_at_position(&file, edit.range.end) as usize,
        &edit.new_text,
    );
    assert_eq!(applied, source, "{marked}");
    let arena = LocalArena::new();
    assert!(!parse_file_borrowed_with_settings(&arena, &file, Default::default()).has_errors(), "{applied}");
}

#[tokio::test]
async fn constructor_completion_checks_bare_and_qualified_access() {
    let mut h = Harness::start(&[("lib.php", LIBRARY), ("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    for (index, marked) in
        ["<?php namespace Build; new Ctor<cursor>Public();", "<?php new \\Build\\Ctor<cursor>Public();"]
            .iter()
            .enumerate()
    {
        check(
            &mut h,
            marked,
            "CtorPublic",
            &[
                "CtorPrivate",
                "CtorProtected",
                "CtorPrivateChild",
                "CtorProtectedChild",
                "CtorTraitPrivate",
                "CtorTraitPrivateChild",
            ],
            index as i32 + 2,
        )
        .await;
    }
}

#[tokio::test]
async fn constructor_completion_checks_import_alias_access() {
    let mut h = Harness::start(&[("lib.php", LIBRARY), ("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    check(&mut h, "<?php use Build\\CtorPrivate as CtorHidden; use Build\\CtorProtectedChild as CtorFamily; use Build\\CtorPublic as CtorVisible; new Ctor<cursor>Visible();",
        "CtorVisible", &["CtorHidden", "CtorFamily"], 2).await;
}

#[tokio::test]
async fn constructor_completion_keeps_accessible_and_absent_constructors() {
    let mut h = Harness::start(&[("lib.php", LIBRARY), ("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    for (index, (marked, selected)) in [
        ("<?php new \\Build\\Ctor<cursor>None();", "CtorNone"),
        ("<?php new \\Build\\Ctor<cursor>PublicChild();", "CtorPublicChild"),
        ("<?php new \\Build\\Ctor<cursor>TraitPublic();", "CtorTraitPublic"),
        ("<?php new \\Build\\Ctor<cursor>TraitPublicChild();", "CtorTraitPublicChild"),
        ("<?php namespace Build; class CtorOwned { private function __construct() {} static function make() { return new Ctor<cursor>Owned(); } }", "CtorOwned"),
        ("<?php namespace Build; class CtorOwned { use HiddenConstructor; static function make() { return new Ctor<cursor>Owned(); } }", "CtorOwned"),
        ("<?php namespace Build; class CtorOwned extends CtorProtected { static function make() { return new Ctor<cursor>Protected(); } }", "CtorProtected"),
        ("<?php namespace Build; class CtorOwned extends CtorProtected { static function make() { return new Ctor<cursor>ProtectedChild(); } }", "CtorProtectedChild"),
    ].iter().enumerate() {
        check(&mut h, marked, selected, &[], index as i32 + 2).await;
    }
}

#[tokio::test]
async fn constructor_completion_does_not_give_named_functions_class_access() {
    let mut h = Harness::start(&[("lib.php", LIBRARY), ("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    check(&mut h, "<?php namespace Build; class CtorOwned { private function __construct() {} static function make() { function nested() { return new Ctor<cursor>Public(); } } }",
        "CtorPublic", &["CtorOwned", "CtorPrivate", "CtorProtected"], 2).await;
}
