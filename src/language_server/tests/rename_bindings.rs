use serde_json::json;

use super::harness::Harness;
use super::references::{apply, prepare_at, rename_at};
use crate::language_server::ServerConfig;

#[tokio::test]
async fn local_rename_ignores_bindings_in_independent_child_scopes() {
    for child in [
        "$callback = function ($value) { return $value; };",
        "$callback = fn($value) => $value;",
        "$callback = fn($value) => fn() => $value;",
        "function child($value) { return $value; }",
    ] {
        let source = format!("<?php function f() {{ $item = 1; {child} return $item; }}");
        let mut h = Harness::start(&[("a.php", &source)]).await;
        let edit = rename_at(&mut h, "a.php", &source, "$item", "value").await.unwrap();
        assert!(edit["changes"].is_object(), "{source}: {edit}");
        assert_eq!(apply(&source, &edit["changes"][h.url("a.php")]), source.replace("$item", "$value"));
    }
}

#[tokio::test]
async fn local_rename_rejects_collisions_where_child_scopes_capture_the_binding() {
    for child in [
        "$callback = function ($value) use ($item) { return $item + $value; };",
        "$callback = fn($value) => $item + $value;",
        "$callback = fn($value) => fn() => $item + $value;",
    ] {
        let source = format!("<?php function f() {{ $item = 1; {child} return $item; }}");
        let mut h = Harness::start(&[("a.php", &source)]).await;
        assert!(rename_at(&mut h, "a.php", &source, "$item", "value").await.is_err(), "{source}");
    }
}

#[tokio::test]
async fn global_binding_rename_ignores_locals_in_a_parent_function() {
    let source = "<?php $item = 1; function outer() { $value = 2; function inner() { global $item; return $item; } return $value; }";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let edit = rename_at(&mut h, "a.php", source, "$item", "value").await.unwrap();
    assert!(edit["changes"].is_object(), "{edit}");
    assert_eq!(apply(source, &edit["changes"][h.url("a.php")]), source.replace("$item", "$value"));
}

#[tokio::test]
async fn alias_rename_updates_phpdoc_names_and_declines_new_name_capture() {
    let library = "<?php namespace Lib; class Thing {}";
    for annotation in ["Old", "Old\\Child", "Renamed", "Renamed\\Child"] {
        let source =
            format!("<?php namespace App; use Lib\\Thing as Old; /** @var {annotation} $value */ $value = new Old();");
        let mut h = Harness::start(&[("a.php", &source), ("lib.php", library)]).await;
        let edit = rename_at(&mut h, "a.php", &source, "Old;", "Renamed").await;
        if annotation.starts_with("Old") {
            assert_eq!(apply(&source, &edit.unwrap()["changes"][h.url("a.php")]), source.replace("Old", "Renamed"));
        } else {
            assert!(edit.is_err(), "{source}");
        }
    }
}

#[tokio::test]
async fn alias_rename_keeps_unrelated_phpdoc_names() {
    let library = "<?php namespace Lib; class Thing {}";
    for annotation in ["Other\\Old", "\\Old", "Other\\Renamed", "\\Renamed"] {
        let source =
            format!("<?php namespace App; use Lib\\Thing as Old; /** @var {annotation} $value */ $value = new Old();");
        let mut h = Harness::start(&[("a.php", &source), ("lib.php", library)]).await;
        let edit = rename_at(&mut h, "a.php", &source, "Old;", "Renamed").await.unwrap();
        assert!(edit["changes"].is_object(), "{source}: {edit}");
        assert_eq!(
            apply(&source, &edit["changes"][h.url("a.php")]),
            source.replace("as Old;", "as Renamed;").replace("new Old()", "new Renamed()")
        );
    }
}

#[tokio::test]
async fn prepare_rename_declines_builtin_targets_from_host_references() {
    let source = "<?php strlen('x'); new DateTime();";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for name in ["strlen", "DateTime"] {
        let position = source.find(name).unwrap() as u32;
        assert!(prepare_at(&mut h, "a.php", 0, position).await.is_err(), "{name}");
        assert!(rename_at(&mut h, "a.php", source, name, "Renamed").await.is_err(), "{name}");
    }
}

#[tokio::test]
async fn builtin_import_aliases_remain_renameable() {
    let source = "<?php namespace App; use DateTime as Clock; use function strlen as length; new Clock(); length('x');";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for (name, replacement) in [("Clock", "Calendar"), ("length", "size")] {
        let position = source.find(name).unwrap() as u32;
        let prepared = h.at("textDocument/prepareRename", "a.php", 0, position).await;
        assert_eq!(prepared["placeholder"], name);
        let edit = rename_at(&mut h, "a.php", source, name, replacement).await.unwrap();
        assert_eq!(edit["changes"].as_object().unwrap().len(), 1);
        assert_eq!(apply(source, &edit["changes"][h.url("a.php")]), source.replace(name, replacement));
    }
}

#[tokio::test]
async fn prepare_and_direct_rename_decline_vendor_declarations() {
    let vendor = tempfile::tempdir().unwrap();
    std::fs::write(
        vendor.path().join("library.php"),
        "<?php class VendorThing {} function vendor_run(): void {} const VENDOR_TOKEN = 1;",
    )
    .unwrap();
    let mut config = ServerConfig::default();
    config.configuration.source.includes = vec![vendor.path().to_str().unwrap().to_owned()];
    let source = "<?php new VendorThing(); vendor_run(); echo VENDOR_TOKEN;";
    let mut h = Harness::start_with(&[("a.php", source)], config, json!({})).await;
    for name in ["VendorThing", "vendor_run", "VENDOR_TOKEN"] {
        let position = source.find(name).unwrap() as u32;
        assert!(prepare_at(&mut h, "a.php", 0, position).await.is_err(), "{name}");
        assert!(rename_at(&mut h, "a.php", source, name, "Renamed").await.is_err(), "{name}");
    }
}

#[tokio::test]
async fn alias_rename_phpdoc_edits_use_workspace_short_tag_settings() {
    let library = "<?php namespace Lib; class Thing {}";
    let source = "<?php namespace App; use Lib\\Thing as Old; new Old(); ?>\n<? /** @var Old $value */ ?>";
    for enabled in [false, true] {
        let mut config = ServerConfig::default();
        config.configuration.parser.enable_short_tags = enabled;
        let mut h = Harness::start_with(&[("a.php", source), ("lib.php", library)], config, json!({})).await;
        let position = source.find("Old;").unwrap() as u32;
        assert!(!h.at("textDocument/prepareRename", "a.php", 0, position).await.is_null());
        let edit = rename_at(&mut h, "a.php", source, "Old;", "Renamed").await.unwrap();
        let expected = if enabled {
            source.replace("Old", "Renamed")
        } else {
            source.replace("as Old;", "as Renamed;").replace("new Old()", "new Renamed()")
        };
        assert_eq!(apply(source, &edit["changes"][h.url("a.php")]), expected);
    }
}

#[tokio::test]
async fn local_rename_updates_braced_string_interpolation() {
    let source = r#"<?php class Pool { public static array $members = []; } function f(string $name): string { $total = 1; $items = [1]; return "sum ${total} {$total} $total ${items[0]}" . Pool::$$name; }"#;
    let mut h = Harness::start(&[("a.php", source)]).await;
    for needle in ["$total =", "total}", "$total ${", "$total}"] {
        let edit = rename_at(&mut h, "a.php", source, needle, "sum").await.unwrap();
        assert_eq!(
            apply(source, &edit["changes"][h.url("a.php")]),
            source.replace("$total", "$sum").replace("${total}", "${sum}"),
            "{needle}"
        );
    }
    let edit = rename_at(&mut h, "a.php", source, "items[", "values").await.unwrap();
    assert_eq!(
        apply(source, &edit["changes"][h.url("a.php")]),
        source.replace("$items", "$values").replace("${items[", "${values["),
    );
}
