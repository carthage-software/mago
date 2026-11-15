use super::harness::Harness;
use super::references::{apply, rename_at};

#[tokio::test]
async fn class_rename_phpdoc_updates_types_without_changing_keys_literals_or_prose() {
    let source = r#"<?php
namespace Lib;
class OldName {}
/**
 * OldName appears in this description.
 * @param array{OldName: list<OldName>, label: 'OldName'} $items
 * @return class-string<OldName>
 */
function inspect($items): string { return OldName::class; }
"#;
    let mut h = Harness::start(&[("a.php", source)]).await;
    let edit = rename_at(&mut h, "a.php", source, "OldName {}", "NewName").await.unwrap();
    assert!(edit["changes"].is_object(), "{edit}");
    assert_eq!(
        apply(source, &edit["changes"][h.url("a.php")]),
        source
            .replace("class OldName", "class NewName")
            .replace("list<OldName>", "list<NewName>")
            .replace("class-string<OldName>", "class-string<NewName>")
            .replace("OldName::class", "NewName::class")
    );
}

#[tokio::test]
async fn class_rename_phpdoc_updates_doc_only_qualified_references_at_absolute_spans() {
    let declaration = "<?php namespace Lib; class OldName {}";
    let source = r#"<?php
namespace App;
use Lib as Types;
$label = 'é😀'; /** @var \Lib\OldName|null $value */ $value = null;
/** @return list<Types\OldName> */
function items(): array { return []; }
namespace Other;
/** @return OldName|null */
function unrelated() { return null; }
"#;
    let mut h = Harness::start(&[("a.php", declaration), ("b.php", source)]).await;
    let edit = rename_at(&mut h, "a.php", declaration, "OldName", "NewName").await.unwrap();
    assert_eq!(edit["changes"].as_object().unwrap().len(), 2, "{edit}");
    assert_eq!(apply(declaration, &edit["changes"][h.url("a.php")]), declaration.replace("OldName", "NewName"));
    assert_eq!(
        apply(source, &edit["changes"][h.url("b.php")]),
        source.replace("\\Lib\\OldName", "\\Lib\\NewName").replace("Types\\OldName", "Types\\NewName")
    );
}

#[tokio::test]
async fn class_rename_phpdoc_updates_implicit_imports_and_their_types_together() {
    let declaration = "<?php namespace Lib; class OldName {}";
    let source = "<?php namespace App; use Lib\\OldName; /** @var OldName $value */ $value = new OldName();";
    let mut h = Harness::start(&[("a.php", declaration), ("b.php", source)]).await;
    let edit = rename_at(&mut h, "a.php", declaration, "OldName", "NewName").await.unwrap();
    assert!(edit["changes"].is_object(), "{edit}");
    assert_eq!(apply(declaration, &edit["changes"][h.url("a.php")]), declaration.replace("OldName", "NewName"));
    assert_eq!(apply(source, &edit["changes"][h.url("b.php")]), source.replace("OldName", "NewName"));
}

#[tokio::test]
async fn class_rename_phpdoc_preserves_explicit_aliases() {
    let declaration = "<?php namespace Lib; class OldName {}";
    for alias in ["Stable", "OldName"] {
        let source = format!(
            "<?php namespace App; use Lib\\OldName as {alias}; /** @var {alias} $value */ $value = new {alias}();"
        );
        let mut h = Harness::start(&[("a.php", declaration), ("b.php", &source)]).await;
        let edit = rename_at(&mut h, "a.php", declaration, "OldName", "NewName").await.unwrap();
        assert_eq!(edit["changes"].as_object().unwrap().len(), 2, "{edit}");
        assert_eq!(apply(declaration, &edit["changes"][h.url("a.php")]), declaration.replace("OldName", "NewName"));
        assert_eq!(apply(&source, &edit["changes"][h.url("b.php")]), source.replace("Lib\\OldName", "Lib\\NewName"));
    }
}

#[tokio::test]
async fn class_rename_phpdoc_updates_import_type_source_classes() {
    let declaration = "<?php namespace Lib; /** @phpstan-type Payload array{value: int} */ class OldName {}";
    let source = r#"<?php
namespace App;
/** @phpstan-import-type Payload from \Lib\OldName as LocalPayload */
class Consumer {
    /** @return LocalPayload */
    public function value(): array { return ['value' => 1]; }
}
"#;
    let mut h = Harness::start(&[("a.php", declaration), ("b.php", source)]).await;
    let edit = rename_at(&mut h, "a.php", declaration, "OldName", "NewName").await.unwrap();
    assert_eq!(edit["changes"].as_object().unwrap().len(), 2, "{edit}");
    assert_eq!(apply(declaration, &edit["changes"][h.url("a.php")]), declaration.replace("OldName", "NewName"));
    assert_eq!(apply(source, &edit["changes"][h.url("b.php")]), source.replace("\\Lib\\OldName", "\\Lib\\NewName"));
}

#[tokio::test]
async fn class_rename_phpdoc_preserves_template_and_type_alias_bindings() {
    let declaration = "<?php namespace Lib; class OldName {}";
    for body in [
        "/** @template OldName */ class Box { /** @param OldName $value */ public function take($value): void {} }",
        "/**\n * @template OldName\n * @param OldName $value\n */ function take($value): void {}",
        "/** @phpstan-type OldName int */ class Box { /** @return OldName */ public function value(): int { return 1; } }",
        "/** @phpstan-type Value int */ class Source {} /** @phpstan-import-type Value from Source as OldName */ class Box { /** @return OldName */ public function value(): int { return 1; } }",
    ] {
        let source = format!("<?php namespace Lib; {body}");
        let mut h = Harness::start(&[("a.php", declaration), ("b.php", &source)]).await;
        let edit = rename_at(&mut h, "a.php", declaration, "OldName", "NewName").await.unwrap();
        assert_eq!(edit["changes"].as_object().unwrap().len(), 1, "{body}: {edit}");
        assert_eq!(apply(declaration, &edit["changes"][h.url("a.php")]), declaration.replace("OldName", "NewName"));
    }
}

#[tokio::test]
async fn class_rename_phpdoc_declines_retargeting_a_namespace_prefix() {
    let declaration = "<?php namespace Lib; class OldName {}";
    let child = "<?php namespace Lib\\OldName; class Child {}";
    let source = "<?php namespace App; use Lib\\OldName as Prefix; /** @return Prefix\\Child|null */ function child() { return null; }";
    let mut h = Harness::start(&[("a.php", declaration), ("b.php", source), ("child.php", child)]).await;
    assert!(rename_at(&mut h, "a.php", declaration, "OldName", "NewName").await.is_err());
}

#[tokio::test]
async fn class_rename_phpdoc_declines_capture_by_a_template_or_type_alias() {
    let declaration = "<?php namespace Lib; class OldName {}";
    for body in [
        "/**\n * @template NewName\n * @param OldName $value\n */ function take($value): void {}",
        "/** @phpstan-type NewName int */ class Box { /** @param OldName $value */ public function take($value): void {} }",
    ] {
        let source = format!("<?php namespace Lib; {body}");
        let mut h = Harness::start(&[("a.php", declaration), ("b.php", &source)]).await;
        assert!(rename_at(&mut h, "a.php", declaration, "OldName", "NewName").await.is_err(), "{body}");
    }
}

#[tokio::test]
async fn class_rename_phpdoc_declines_capture_of_a_proposed_class_name() {
    let declaration = "<?php namespace Lib; class OldName {}";
    let source = "<?php namespace App; /** @return \\Lib\\NewName|null */ function value() { return null; }";
    let mut h = Harness::start(&[("a.php", declaration), ("b.php", source)]).await;
    assert!(rename_at(&mut h, "a.php", declaration, "OldName", "NewName").await.is_err());
}

#[tokio::test]
async fn class_rename_phpdoc_keeps_ambiguous_template_bounds_safe() {
    let declaration = "<?php class OldName {}";
    for template in ["OldName", "Unrelated"] {
        let source = format!("<?php /** @template {template} of {template} */ class Box {{}}");
        let mut h = Harness::start(&[("a.php", declaration), ("b.php", &source)]).await;
        let edit = rename_at(&mut h, "a.php", declaration, "OldName", "NewName").await;
        if template == "OldName" {
            assert!(edit.is_err(), "{edit:?}");
        } else {
            let edit = edit.unwrap();
            assert_eq!(edit["changes"].as_object().unwrap().len(), 1, "{edit}");
            assert_eq!(apply(declaration, &edit["changes"][h.url("a.php")]), declaration.replace("OldName", "NewName"));
        }
    }
}

#[tokio::test]
async fn class_rename_phpdoc_declines_ambiguous_magic_method_templates() {
    let declaration = "<?php namespace Lib; class OldName {}";
    for template in ["OldName", "NewName", "Unrelated"] {
        let source = format!("<?php namespace Lib; /** @method OldName make<{template}>() */ class Consumer {{}}");
        let mut h = Harness::start(&[("a.php", declaration), ("b.php", &source)]).await;
        let edit = rename_at(&mut h, "a.php", declaration, "OldName", "NewName").await;
        if template != "Unrelated" {
            assert!(edit.is_err(), "{edit:?}");
        } else {
            let edit = edit.unwrap();
            assert_eq!(edit["changes"].as_object().unwrap().len(), 2, "{edit}");
            assert_eq!(apply(declaration, &edit["changes"][h.url("a.php")]), declaration.replace("OldName", "NewName"));
            assert_eq!(
                apply(&source, &edit["changes"][h.url("b.php")]),
                source.replace("@method OldName", "@method NewName")
            );
        }
    }
}

#[tokio::test]
async fn class_rename_phpdoc_keeps_template_types_separate_from_class_names() {
    let declaration = "<?php namespace Lib; /** @phpstan-type Payload int */ class OldName { const VALUE = 1; }";
    let source = r#"<?php
namespace App;
use Lib\OldName;
/**
 * @phpstan-import-type Payload from OldName as LocalPayload
 * @template OldName
 * @method OldName::VALUE magic()
 */
class Container {
    /** @return OldName */
    public function template_value() {}
    /** @return OldName::VALUE */
    public function constant_value() {}
    /** @return LocalPayload */
    public function payload() {}
}
/** @return OldName */
function sibling() {}
"#;
    let mut h = Harness::start(&[("a.php", declaration), ("b.php", source)]).await;
    let edit = rename_at(&mut h, "a.php", declaration, "OldName", "NewName").await.unwrap();
    assert_eq!(edit["changes"].as_object().unwrap().len(), 2, "{edit}");
    assert_eq!(apply(declaration, &edit["changes"][h.url("a.php")]), declaration.replace("OldName", "NewName"));
    assert_eq!(
        apply(source, &edit["changes"][h.url("b.php")]),
        source
            .replace("Lib\\OldName", "Lib\\NewName")
            .replace("from OldName", "from NewName")
            .replace("OldName::VALUE", "NewName::VALUE")
            .replace("@return OldName */\nfunction sibling", "@return NewName */\nfunction sibling")
    );
}

#[tokio::test]
async fn class_rename_phpdoc_respects_plain_alias_contexts_across_classes_and_namespaces() {
    assert_alias_contexts("/** @phpstan-type OldName int */ class Origin {}", false).await;
}

#[tokio::test]
async fn class_rename_phpdoc_respects_imported_alias_contexts_across_classes_and_namespaces() {
    assert_alias_contexts(
        "/** @phpstan-type Value int */ class Source {} /** @phpstan-import-type Value from Source as OldName */ class Origin {}",
        true,
    ).await;
}

async fn assert_alias_contexts(definition: &str, imported: bool) {
    let declaration = "<?php namespace Lib; class OldName {}";
    for namespace in ["", "namespace Second;"] {
        let source = format!(
            r#"<?php
namespace First;
{definition}
{namespace}
use Lib\OldName;
/** @property OldName $virtual */
class Consumer {{
    /** @var OldName */
    public $value;
    /** @return OldName */
    public function value() {{ return 1; }}
}}
/** @var OldName|null $value */
$value = null;
/** @return OldName */
function global_value() {{ return 1; }}
"#
        );
        let mut h = Harness::start(&[("a.php", declaration), ("b.php", &source)]).await;
        let edit = rename_at(&mut h, "a.php", declaration, "OldName", "NewName").await.unwrap();
        let mut expected = source
            .replace("Lib\\OldName", "Lib\\NewName")
            .replace("@property OldName", "@property NewName")
            .replace("@var OldName", "@var NewName");
        if !imported {
            expected = expected
                .replace("@return OldName */\nfunction global_value", "@return NewName */\nfunction global_value");
        }
        assert_eq!(edit["changes"].as_object().unwrap().len(), 2, "{source}: {edit}");
        assert_eq!(apply(declaration, &edit["changes"][h.url("a.php")]), declaration.replace("OldName", "NewName"));
        assert_eq!(apply(&source, &edit["changes"][h.url("b.php")]), expected, "{source}");
    }
}
