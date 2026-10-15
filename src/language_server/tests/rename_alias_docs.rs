use super::harness::Harness;
use super::references::{apply, rename_at};

#[tokio::test]
async fn builtin_alias_rename_updates_phpdoc_and_keeps_fully_qualified_uses() {
    let source = "<?php namespace MagoEditorQa; use DateTimeImmutable as Clock; /** @param Clock $date */ function dates(Clock $date): Clock { return new Clock(); } function untouched(\\DateTimeImmutable $date): \\DateTimeImmutable { return $date; }";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for needle in ["Clock;", "Clock();"] {
        let offset = source.find(needle).unwrap();
        let prepared = h.at("textDocument/prepareRename", "a.php", 0, offset as u32).await;
        assert_eq!(prepared["placeholder"], "Clock", "{prepared}");
        let edit = rename_at(&mut h, "a.php", source, needle, "Calendar").await.unwrap();
        assert_eq!(edit["changes"].as_object().unwrap().len(), 1);
        assert_eq!(apply(source, &edit["changes"][h.url("a.php")]), source.replace("Clock", "Calendar"));
    }
}

#[tokio::test]
async fn alias_rename_phpdoc_ignores_prose_shape_keys_and_string_types() {
    let declaration = "<?php namespace Lib; class Thing {}";
    let source = r#"<?php
use Lib\Thing as Old;
/**
 * Old is a word in this description.
 * @param array{Old: 'Old', NewName: 'NewName'} $value
 * @var \Lib\Thing $other
 */
function take($value): void {}
new Old();
"#;
    let mut h = Harness::start(&[("a.php", source), ("lib.php", declaration)]).await;
    let edit = rename_at(&mut h, "a.php", source, "Old;", "NewName").await.unwrap();
    assert_eq!(edit["changes"].as_object().unwrap().len(), 1, "{edit}");
    assert_eq!(
        apply(source, &edit["changes"][h.url("a.php")]),
        source.replace("as Old;", "as NewName;").replace("new Old()", "new NewName()")
    );
}

#[tokio::test]
async fn alias_rename_phpdoc_updates_nested_types_and_class_constants() {
    let declaration = "<?php namespace Lib; class Thing { const VALUE = 1; }";
    for annotation in [
        "@var list<Old>",
        "@var callable(Old): class-string<Old>",
        "@var array{items: list<Old>, child: Old\\Child}",
        "@var array{Old::VALUE: int}",
        "@method void handle(int $value = Old::VALUE)",
        "@phpstan-import-type Payload from Old",
    ] {
        let source = format!("<?php use Lib\\Thing as Old; /** {annotation} */ class Consumer {{}} new Old();");
        let mut h = Harness::start(&[("a.php", &source), ("lib.php", declaration)]).await;
        let edit = rename_at(&mut h, "a.php", &source, "Old;", "NewName").await.unwrap();
        assert_eq!(apply(&source, &edit["changes"][h.url("a.php")]), source.replace("Old", "NewName"));
    }
}

#[tokio::test]
async fn alias_rename_phpdoc_preserves_template_bindings_and_namespace_blocks() {
    let source = r#"<?php
namespace App {
    use DateTimeImmutable as Clock;
    /**
     * @template Clock
     * @param Clock $value
     */
    function identity($value) { return $value; }
    /** @var Clock $value */
    $value = new Clock();
}
namespace Other {
    use DateTimeImmutable as Clock;
    /** @var Clock $value */
    $value = new Clock();
}
"#;
    let mut h = Harness::start(&[("a.php", source)]).await;
    let edit = rename_at(&mut h, "a.php", source, "Clock;", "Calendar").await.unwrap();
    let expected = source.replacen("as Clock;", "as Calendar;", 1).replacen("@var Clock", "@var Calendar", 1).replacen(
        "new Clock()",
        "new Calendar()",
        1,
    );
    assert_eq!(apply(source, &edit["changes"][h.url("a.php")]), expected);
}

#[tokio::test]
async fn alias_rename_phpdoc_declines_template_type_alias_and_keyword_capture() {
    for (annotation, replacement) in [
        ("@template Calendar\n * @param Clock $value", "Calendar"),
        ("@phpstan-type Calendar int\n * @var Clock", "Calendar"),
        ("@var callable(Clock): class-string<Calendar>", "Calendar"),
        ("@var Clock", "scalar"),
    ] {
        let source =
            format!("<?php use DateTimeImmutable as Clock; /**\n * {annotation}\n */ class Consumer {{}} new Clock();");
        let mut h = Harness::start(&[("a.php", &source)]).await;
        assert!(rename_at(&mut h, "a.php", &source, "Clock;", replacement).await.is_err(), "{annotation}");
    }
}

#[tokio::test]
async fn alias_rename_phpdoc_declines_affected_malformed_or_custom_tags() {
    let declaration = "<?php namespace Lib; class Thing {}";
    for annotation in ["@var array{value: Old", "@var array{value: NewName", "@custom Old", "@custom NewName\\Child"] {
        let source = format!("<?php use Lib\\Thing as Old; /** {annotation} */ $value = new Old();");
        let mut h = Harness::start(&[("a.php", &source), ("lib.php", declaration)]).await;
        assert!(rename_at(&mut h, "a.php", &source, "Old;", "NewName").await.is_err(), "{annotation}");
    }
}

#[tokio::test]
async fn alias_rename_phpdoc_allows_unrelated_malformed_or_custom_tags() {
    let declaration = "<?php namespace Lib; class Thing {}";
    for annotation in [
        "@var array{",
        "@var array{value: Other",
        "@custom metadata",
        "@custom Other\\Old",
        "@custom \\Old",
        "@custom Older",
    ] {
        let source = format!("<?php use Lib\\Thing as Old; /** {annotation} */ $value = new Old();");
        let mut h = Harness::start(&[("a.php", &source), ("lib.php", declaration)]).await;
        let edit = rename_at(&mut h, "a.php", &source, "Old;", "NewName").await.unwrap();
        assert_eq!(edit["changes"].as_object().unwrap().len(), 1, "{annotation}: {edit}");
        assert_eq!(
            apply(&source, &edit["changes"][h.url("a.php")]),
            source.replace("as Old;", "as NewName;").replace("new Old()", "new NewName()")
        );
    }
}
