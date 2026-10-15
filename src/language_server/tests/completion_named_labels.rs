use serde_json::Value;
use tower_lsp_server::gen_lsp_types::TextEdit;

use mago_database::file::File;

use super::harness::Harness;
use crate::language_server::position::{offset_at_position, position_at_offset};

const LIBRARY: &str = r"<?php
function labelTarget(string $prefix = '', int $precision = 0): void {}
function prefixValue(): string { return ''; }
const PREFIX_VALUE = '';
class PrefixA { public function send(string $prefix = '', int $precision = 0): void {} }
class PrefixB { public function send(string $prefix = '', int $priority = 0): void {} }
class PrefixC { public function send(string $prefix = '', bool $pretty = false): void {} }
class NoPrefix { public function send(string $other = ''): void {} }
";

async fn complete(marked: &str) -> (String, Vec<Value>) {
    let offset = marked.find("<cursor>").unwrap();
    let source = marked.replace("<cursor>", "");
    let mut h = Harness::start(&[("lib.php", LIBRARY), ("a.php", &source)]).await;
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let position = position_at_offset(&file, offset as u32);
    let result = h.at("textDocument/completion", "a.php", position.line, position.character).await;
    (source, result["items"].as_array().unwrap().clone())
}

fn apply(source: &str, item: &Value, value: &str) -> String {
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let edit: TextEdit = serde_json::from_value(item["textEdit"].clone()).unwrap();
    let mut applied = source.to_owned();
    applied.replace_range(
        offset_at_position(&file, edit.range.start) as usize..offset_at_position(&file, edit.range.end) as usize,
        &(edit.new_text + value),
    );
    applied
}

async fn check_existing_label(marked: &str) {
    let (source, items) = complete(marked).await;
    let prefix = items.iter().find(|item| item["label"] == "prefix:").unwrap_or_else(|| panic!("{items:?}"));
    assert!(items.iter().all(|item| item["label"] == "prefix:" || item["label"] == "precision:"), "{items:?}");
    assert_eq!(prefix["textEdit"]["newText"], "prefix");
    assert_eq!(apply(&source, prefix, ""), source);
}

#[tokio::test]
async fn named_label_completion_excludes_expressions_inside_label() {
    check_existing_label("<?php labelTarget(pre<cursor>fix: 'x');").await;
}

#[tokio::test]
async fn named_label_completion_excludes_expressions_at_label_end() {
    check_existing_label("<?php labelTarget(prefix<cursor>: 'x');").await;
}

#[tokio::test]
async fn bare_argument_completion_keeps_expressions_and_adds_a_colon() {
    for marked in ["<?php labelTarget(pr<cursor>);", "<?php labelTarget(<cursor>);"] {
        let (source, items) = complete(marked).await;
        let prefix = items.iter().find(|item| item["label"] == "prefix:").unwrap_or_else(|| panic!("{items:?}"));
        assert!(items.iter().any(|item| !item["label"].as_str().unwrap().ends_with(':')), "{items:?}");
        if marked.contains("pr<cursor>") {
            assert!(items.iter().any(|item| item["label"] == "prefixValue"), "{items:?}");
        }
        assert_eq!(prefix["textEdit"]["newText"], "prefix: ");
        assert_eq!(apply(&source, prefix, "'x'"), "<?php labelTarget(prefix: 'x');");
    }
}

#[tokio::test]
async fn named_argument_value_completion_keeps_expressions() {
    let (_, items) = complete("<?php labelTarget(prefix: pre<cursor>);").await;
    assert!(items.iter().any(|item| item["label"] == "prefixValue"), "{items:?}");
    assert!(items.iter().all(|item| !item["label"].as_str().unwrap().ends_with(':')), "{items:?}");
}

#[tokio::test]
async fn named_label_completion_stays_empty_for_unknown_or_uncertain_targets() {
    for marked in [
        "<?php unknownTarget(prefix<cursor>: 'x');",
        "<?php function test($object) { $object->unknown(prefix<cursor>: 'x'); }",
        "<?php function test(PrefixA|Missing $object) { $object->send(prefix<cursor>: 'x'); }",
    ] {
        let (_, items) = complete(marked).await;
        assert!(items.is_empty(), "{marked}: {items:?}");
    }
}

#[tokio::test]
async fn named_label_completion_requires_names_shared_by_all_union_targets() {
    let (_, items) =
        complete("<?php function test(PrefixA|PrefixB|PrefixC $object) { $object->send(pre<cursor>fix: 'x'); }").await;
    assert_eq!(items.iter().map(|item| item["label"].as_str().unwrap()).collect::<Vec<_>>(), ["prefix:"]);
    let (_, items) =
        complete("<?php function test(PrefixA|NoPrefix $object) { $object->send(pre<cursor>fix: 'x'); }").await;
    assert!(items.is_empty(), "{items:?}");
}

#[tokio::test]
async fn named_label_completion_excludes_parameters_bound_elsewhere() {
    for (marked, expected) in [
        ("<?php labelTarget('x', pre<cursor>cision: 3);", "precision"),
        ("<?php labelTarget(prefix: 'x', pre<cursor>cision: 3);", "precision"),
        ("<?php labelTarget(pre<cursor>fix: 'x', precision: 3);", "prefix"),
    ] {
        let (source, items) = complete(marked).await;
        assert_eq!(items.len(), 1, "{marked}: {items:?}");
        assert_eq!(items[0]["label"], format!("{expected}:"));
        assert_eq!(apply(&source, &items[0], ""), source);
    }
}

#[tokio::test]
async fn named_label_completion_stays_empty_after_unpacking() {
    let (_, items) =
        complete("<?php function test(array $values) { labelTarget(...$values, pre<cursor>fix: 'x'); }").await;
    assert!(items.is_empty(), "{items:?}");
}
