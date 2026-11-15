use serde_json::{Value, json};
use tower_lsp_server::gen_lsp_types::TextEdit;

use mago_allocator::LocalArena;
use mago_database::file::File;
use mago_php_version::PHPVersion;
use mago_syntax::parser::parse_file_borrowed_with_settings;

use crate::language_server::ServerConfig;
use crate::language_server::position::{offset_at_position, position_at_offset};

use super::harness::Harness;

const DECLARATIONS: &str = r"<?php namespace NativeTypes;
interface A {}
interface B {}
class TypeFailure extends \RuntimeException {}
class TypeChildFailure extends TypeFailure {}
interface TypeThrowable extends \Throwable {}
class TypePlain {}
trait TypeTrait {}
enum TypeEnum { case One; }
";

async fn complete(h: &mut Harness, marked: &str, version: i32) -> (String, Vec<Value>) {
    let offset = marked.find("<cursor>").unwrap();
    let source = marked.replace("<cursor>", "");
    h.change("a.php", &source, version).await;
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let position = position_at_offset(&file, offset as u32);
    let result = h.at("textDocument/completion", "a.php", position.line, position.character).await;
    (source, result["items"].as_array().unwrap().clone())
}

fn has(items: &[Value], label: &str) -> bool {
    items.iter().any(|item| item["label"] == label)
}

fn apply(source: &str, items: &[Value], label: &str) -> String {
    let item = items.iter().find(|item| item["label"] == label).unwrap_or_else(|| panic!("missing {label}: {items:?}"));
    assert_ne!(item["insertTextFormat"], 2, "native types must use plain text: {item}");
    let edit: TextEdit = serde_json::from_value(item["textEdit"].clone()).unwrap();
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let mut applied = source.to_owned();
    applied.replace_range(
        offset_at_position(&file, edit.range.start) as usize..offset_at_position(&file, edit.range.end) as usize,
        &edit.new_text,
    );
    applied
}

#[tokio::test]
async fn native_type_completion_applies_plain_edits_in_declarations() {
    let mut h = Harness::start(&[("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    for (index, (marked, selected, label)) in [
        ("<?php function accept(in<cursor> $value) {}", "in", "int"),
        ("<?php function accept(str<cursor>ing $value) {}", "string", "string"),
        ("<?php function accept($first, bo<cursor> $value) {}", "bo", "bool"),
        ("<?php $f = function (flo<cursor> $value) {};", "flo", "float"),
        ("<?php $f = fn (arr<cursor> $value) => $value;", "arr", "array"),
        ("<?php class Subject { public ite<cursor> $values; }", "ite", "iterable"),
        ("<?php class Subject { private const str<cursor>ing VALUE = ''; }", "string", "string"),
        ("<?php class Subject { public const int|str<cursor> VALUE = ''; }", "str", "string"),
        ("<?php class Subject { function __construct(public obj<cursor> $value) {} }", "obj", "object"),
        ("<?php function read(): mi<cursor> { throw new Exception(); }", "mi", "mixed"),
        ("<?php function done(): voi<cursor> {}", "voi", "void"),
        ("<?php function stop(): nev<cursor> { throw new Exception(); }", "nev", "never"),
        ("<?php class Subject { function copy(): sta<cursor> { return $this; } }", "sta", "static"),
    ]
    .iter()
    .enumerate()
    {
        let (source, items) = complete(&mut h, marked, index as i32 + 2).await;
        let applied = apply(&source, &items, label);
        assert_eq!(items.iter().find(|item| item["label"] == *label).unwrap()["kind"], 14);
        let cursor = marked.find("<cursor>").unwrap();
        let token_start = source[..cursor].rfind(|ch: char| !ch.is_ascii_alphanumeric()).unwrap() + 1;
        let mut expected = source.clone();
        expected.replace_range(token_start..token_start + selected.len(), label);
        assert_eq!(applied, expected, "{marked}");
        let file = File::ephemeral(b"a.php".as_slice().into(), applied.as_bytes().to_vec().into());
        let arena = LocalArena::new();
        assert!(!parse_file_borrowed_with_settings(&arena, &file, Default::default()).has_errors(), "{applied}");
    }
}

#[tokio::test]
async fn constant_completion_keeps_types_and_initializer_expressions_apart() {
    let mut h = Harness::start(&[("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    for (index, (marked, present, absent)) in [
        (
            "<?php class Subject { private const <cursor> }",
            &["private const", "int"][..],
            &["strlen", "PHP_VERSION", "die", "exit", "clone"][..],
        ),
        ("<?php interface Subject { public const Ex<cursor> VALUE = null; }", &["Exception"][..], &["exit"][..]),
        (
            "<?php class Subject { private const int <cursor> }",
            &[][..],
            &["int", "Exception", "strlen", "PHP_VERSION"][..],
        ),
        (
            "<?php class Subject { private const (Countable&Stringable) <cursor> }",
            &[][..],
            &["int", "Exception", "strlen", "PHP_VERSION"][..],
        ),
        (
            "<?php class Subject { private const string VALUE = PHP_VERS<cursor>; }",
            &["PHP_VERSION"][..],
            &["string", "private const"][..],
        ),
        ("<?php const VALUE = [PHP_VERS<cursor>];", &["PHP_VERSION"][..], &["int", "const"][..]),
        ("<?php const FIRST = 1, SECOND = PHP_VERS<cursor>;", &["PHP_VERSION"][..], &["int", "const"][..]),
        ("<?php use const PHP_VERS<cursor>;", &["PHP_VERSION"][..], &["int", "strlen", "Exception"][..]),
    ]
    .iter()
    .enumerate()
    {
        let (_, items) = complete(&mut h, marked, index as i32 + 2).await;
        for label in *present {
            assert!(has(&items, label), "{marked}: missing {label}: {items:?}");
        }
        for label in *absent {
            assert!(!has(&items, label), "{marked}: unexpected {label}: {items:?}");
        }
    }
}

#[tokio::test]
async fn native_type_completion_keeps_unfinished_declarations() {
    let mut h = Harness::start(&[("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    for (index, (marked, label)) in [
        ("<?php function accept(in<cursor>", "int"),
        ("<?php function accept($first, ?str<cursor>", "string"),
        ("<?php function make(): voi<cursor>", "void"),
        ("<?php class Subject { public arr<cursor>", "array"),
    ]
    .iter()
    .enumerate()
    {
        let (source, items) = complete(&mut h, marked, index as i32 + 2).await;
        let start = source.rfind(|ch: char| !ch.is_ascii_alphanumeric()).unwrap() + 1;
        assert_eq!(apply(&source, &items, label), source[..start].to_owned() + label);
    }
}

#[tokio::test]
async fn native_type_completion_limits_keywords_to_the_declaration_kind() {
    let mut h = Harness::start(&[("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    for (index, (marked, present, absent)) in [
        (
            "<?php function accept(<cursor> $value) {}",
            &["int", "callable"][..],
            &["void", "never", "static", "self", "parent"][..],
        ),
        (
            "<?php class Subject { public <cursor> $value; }",
            &["int", "self"][..],
            &["callable", "void", "never", "static", "parent"][..],
        ),
        (
            "<?php class Subject { private const <cursor> }",
            &["int", "self"][..],
            &["callable", "void", "never", "static", "parent", "strlen", "die", "exit", "clone", "PHP_VERSION"][..],
        ),
        (
            "<?php class Subject { function __construct(public <cursor> $value) {} }",
            &["int"][..],
            &["callable", "void", "never", "static"][..],
        ),
        (
            "<?php function make(): <cursor> { throw new Exception(); }",
            &["callable", "void", "never"][..],
            &["static", "self", "parent"][..],
        ),
        (
            "<?php class Base {} class Subject extends Base { function make(): <cursor> { throw new Exception(); } }",
            &["self", "parent", "static"][..],
            &[][..],
        ),
        (
            "<?php class Subject { function work() { function nested(): <cursor> {} } }",
            &["void"][..],
            &["static", "self", "parent"][..],
        ),
    ]
    .iter()
    .enumerate()
    {
        let (_, items) = complete(&mut h, marked, index as i32 + 2).await;
        for label in *present {
            assert!(has(&items, label), "missing {label}, {marked}: {items:?}");
        }
        for label in *absent {
            assert!(!has(&items, label), "invalid {label}, {marked}: {items:?}");
        }
    }
}

#[tokio::test]
async fn native_type_completion_obeys_php_version() {
    for php in [
        PHPVersion::PHP70,
        PHPVersion::PHP71,
        PHPVersion::PHP74,
        PHPVersion::PHP80,
        PHPVersion::PHP81,
        PHPVersion::PHP82,
        PHPVersion::PHP83,
    ] {
        let mut config = ServerConfig::default();
        config.configuration.php_version = php;
        let mut h = Harness::start_with(&[("a.php", "<?php")], config, json!({})).await;
        h.open("a.php", "<?php").await;
        for (index, (marked, label, since)) in [
            ("<?php function accept(in<cursor> $v) {}", "int", PHPVersion::PHP70),
            ("<?php function accept(ite<cursor> $v) {}", "iterable", PHPVersion::PHP71),
            ("<?php function accept(obj<cursor> $v) {}", "object", PHPVersion::PHP72),
            ("<?php class S { public in<cursor> $v; }", "int", PHPVersion::PHP74),
            ("<?php class S { private const in<cursor> VALUE = 1; }", "int", PHPVersion::PHP83),
            ("<?php class S { private const Ex<cursor> VALUE = null; }", "Exception", PHPVersion::PHP83),
            ("<?php function accept(mi<cursor> $v) {}", "mixed", PHPVersion::PHP80),
            ("<?php function stop(): nev<cursor> { throw new Exception(); }", "never", PHPVersion::PHP81),
            ("<?php function accept(fal<cursor> $v) {}", "false", PHPVersion::PHP82),
            ("<?php function accept(tru<cursor> $v) {}", "true", PHPVersion::PHP82),
            ("<?php function accept(nu<cursor> $v) {}", "null", PHPVersion::PHP82),
            ("<?php function accept(?fal<cursor> $v) {}", "false", PHPVersion::PHP82),
            ("<?php function accept(null|fal<cursor> $v) {}", "false", PHPVersion::PHP82),
            ("<?php function accept(false|nu<cursor> $v) {}", "null", PHPVersion::PHP82),
            ("<?php function accept(string|fal<cursor> $v) {}", "false", PHPVersion::PHP80),
            ("<?php function accept(string|nu<cursor> $v) {}", "null", PHPVersion::PHP80),
            ("<?php function accept(string|null|fal<cursor> $v) {}", "false", PHPVersion::PHP80),
            ("<?php function done(): voi<cursor> {}", "void", PHPVersion::PHP71),
            ("<?php class S { function copy(): sta<cursor> { return $this; } }", "static", PHPVersion::PHP80),
        ]
        .iter()
        .enumerate()
        {
            let (_, items) = complete(&mut h, marked, index as i32 + 2).await;
            assert_eq!(has(&items, label), php >= *since, "PHP {php}, {marked}: {items:?}");
        }
    }
}

#[tokio::test]
async fn native_type_completion_respects_nullable_union_and_intersection_members() {
    let mut h = Harness::start(&[("lib.php", DECLARATIONS), ("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    for (index, (marked, present, absent)) in [
        (
            "<?php function accept(?<cursor> $value) {}",
            &["int", "string"][..],
            &["void", "never", "mixed", "null", "static"][..],
        ),
        (
            "<?php function accept(int|<cursor> $value) {}",
            &["string", "false", "null"][..],
            &["int", "void", "never", "mixed", "static"][..],
        ),
        (
            "<?php function accept(<cursor>|bool $value) {}",
            &["string"][..],
            &["bool", "false", "true", "void", "never", "mixed"][..],
        ),
        (
            "<?php namespace NativeTypes; function accept(\\NativeTypes\\A&<cursor> $value) {}",
            &["B"][..],
            &["int", "string", "object", "callable", "self", "static", "mixed"][..],
        ),
        (
            "<?php function accept((\\NativeTypes\\A&\\NativeTypes\\B)|<cursor> $value) {}",
            &["string"][..],
            &["void", "never", "mixed", "static"][..],
        ),
    ]
    .iter()
    .enumerate()
    {
        let (source, items) = complete(&mut h, marked, index as i32 + 2).await;
        for label in *present {
            assert!(has(&items, label), "missing {label}, {marked}: {items:?}");
            let applied = apply(&source, &items, label);
            assert_eq!(applied, marked.replace("<cursor>", label));
            let file = File::ephemeral(b"a.php".as_slice().into(), applied.as_bytes().to_vec().into());
            let arena = LocalArena::new();
            assert!(!parse_file_borrowed_with_settings(&arena, &file, Default::default()).has_errors(), "{applied}");
        }
        for label in *absent {
            assert!(!has(&items, label), "invalid {label}, {marked}: {items:?}");
        }
    }
}

#[tokio::test]
async fn native_type_completion_limits_catch_to_throwable_names_and_aliases() {
    let mut h = Harness::start(&[("lib.php", DECLARATIONS), ("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    for (index, marked) in [
        "<?php try {} catch (\\NativeTypes\\Type<cursor> $e) {}",
        "<?php try {} catch (\\Error|\\NativeTypes\\Type<cursor> $e) {}",
        "<?php namespace App; use NativeTypes\\TypeFailure as TypeCrash; use NativeTypes\\TypePlain as TypeWrong; try {} catch (Type<cursor> $e) {}",
    ]
    .iter()
    .enumerate()
    {
        let (source, items) = complete(&mut h, marked, index as i32 + 2).await;
        for label in ["TypePlain", "TypeWrong", "TypeEnum", "TypeTrait"] {
            assert!(!has(&items, label), "catch offered {label}, {marked}: {items:?}");
        }
        let label = if marked.contains("TypeCrash") { "TypeCrash" } else { "TypeFailure" };
        let applied = apply(&source, &items, label);
        assert_eq!(applied, marked.replace("Type<cursor>", label));
        assert!(has(&items, "TypeChildFailure") && has(&items, "TypeThrowable"), "{items:?}");
    }
}

#[tokio::test]
async fn native_type_completion_does_not_offer_keywords_as_qualified_names_or_values() {
    let mut h = Harness::start(&[("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    for (index, marked) in [
        "<?php function accept(\\str<cursor> $value) {}",
        "<?php function accept($value = str<cursor>len('x')) {}",
        "<?php consume(str<cursor>len('x'));",
        "<?php $value = new str<cursor>();",
        "<?php $result = $value instanceof str<cursor>;",
        "<?php use str<cursor>;",
    ]
    .iter()
    .enumerate()
    {
        let (_, items) = complete(&mut h, marked, index as i32 + 2).await;
        assert!(!has(&items, "string"), "{marked}: {items:?}");
        if marked.contains("len('x')") {
            assert!(has(&items, "strlen"), "{marked}: {items:?}");
        }
    }
}

#[tokio::test]
async fn native_type_completion_keeps_relative_scope_for_closures_and_traits() {
    let mut h = Harness::start(&[("a.php", "<?php")]).await;
    h.open("a.php", "<?php").await;
    for (index, (marked, label, expected)) in [
        ("<?php $f = function (): se<cursor>lf { throw new Exception(); };", "self", true),
        ("<?php $f = static fn (): sta<cursor>tic => throw new Exception();", "static", true),
        ("<?php trait T { function make(): par<cursor>ent { throw new Exception(); } }", "parent", true),
        ("<?php $f = fn (): voi<cursor>d => 1;", "void", false),
        ("<?php class S { function outer() { function nested(): sta<cursor>", "static", false),
        ("<?php class S { function outer() { function nested(se<cursor>", "self", false),
    ]
    .iter()
    .enumerate()
    {
        let (_, items) = complete(&mut h, marked, index as i32 + 2).await;
        assert_eq!(has(&items, label), *expected, "{marked}: {items:?}");
    }
}
