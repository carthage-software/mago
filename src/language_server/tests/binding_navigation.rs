use serde_json::Value;
use serde_json::json;
use tower_lsp_server::gen_lsp_types::Range;

use mago_database::file::File;

use super::harness::Harness;
use super::references::{apply, rename_at};
use crate::language_server::position::position_at_offset;

fn range(source: &str, offset: usize, len: usize) -> Range {
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    Range { start: position_at_offset(&file, offset as u32), end: position_at_offset(&file, (offset + len) as u32) }
}

async fn at(h: &mut Harness, source: &str, offset: usize, method: &str) -> Value {
    let position = range(source, offset, 0).start;
    h.at(method, "a.php", position.line, position.character).await
}

#[tokio::test]
async fn nested_captures_share_a_binding_and_parameters_shadow_it() {
    let source = "<?php function outer() { $item = 1; $capture = function () use ($item) { return fn() => $item; }; $shadow = fn($item) => $item; return $item; } function other() { $item = 2; return $item; }";
    let offsets: Vec<_> = source.match_indices("$item").map(|(offset, _)| offset).collect();
    let mut h = Harness::start(&[("a.php", source)]).await;
    let expected: Vec<_> = [0, 1, 2, 5].map(|index| json!(range(source, offsets[index], 5))).into();
    for index in [0, 1, 2, 5] {
        let highlights = at(&mut h, source, offsets[index], "textDocument/documentHighlight").await;
        assert_eq!(
            highlights.as_array().unwrap().iter().map(|item| item["range"].clone()).collect::<Vec<_>>(),
            expected,
        );
        let definition = at(&mut h, source, offsets[index], "textDocument/definition").await;
        assert_eq!(definition["range"], json!(range(source, offsets[0], 5)), "{definition}");
    }
    let shadow = at(&mut h, source, offsets[4], "textDocument/documentHighlight").await;
    assert_eq!(shadow.as_array().unwrap().len(), 2);
    let edits = rename_at(&mut h, "a.php", source, "$item = 1", "value").await.unwrap();
    let mut expected = source.to_owned();
    for index in [5, 2, 1, 0] {
        expected.replace_range(offsets[index]..offsets[index] + 5, "$value");
    }
    assert_eq!(apply(source, &edits["changes"][h.url("a.php")]), expected);
}

#[tokio::test]
async fn trait_and_enum_methods_keep_separate_local_bindings() {
    let source = "<?php trait Work { function run() { $value = 1; return $value; } } enum Choice { case One; function run() { $value = 2; return $value; } } class Host { use Work; function own() { $value = 3; return $value; } }";
    let offsets: Vec<_> = source.match_indices("$value").map(|(offset, _)| offset).collect();
    let mut h = Harness::start(&[("a.php", source)]).await;
    for pair in offsets.as_chunks::<2>().0 {
        let definition = at(&mut h, source, pair[1], "textDocument/definition").await;
        assert_eq!(definition["range"], json!(range(source, pair[0], 6)), "{definition}");
        let highlights = at(&mut h, source, pair[1], "textDocument/documentHighlight").await;
        assert_eq!(
            highlights.as_array().unwrap().iter().map(|item| item["range"].clone()).collect::<Vec<_>>(),
            pair.iter().map(|offset| json!(range(source, *offset, 6))).collect::<Vec<_>>(),
        );
    }
}

#[tokio::test]
async fn unfinished_closure_keeps_its_capture_binding() {
    let source = "<?php function outer() { $item = 1; $capture = function () use ($item) { return $item";
    let offsets: Vec<_> = source.match_indices("$item").map(|(offset, _)| offset).collect();
    let mut h = Harness::start(&[("a.php", source)]).await;
    let definition = at(&mut h, source, offsets[2], "textDocument/definition").await;
    assert_eq!(definition["range"], json!(range(source, offsets[0], 5)), "{definition}");
    let highlights = at(&mut h, source, offsets[2], "textDocument/documentHighlight").await;
    assert_eq!(
        highlights.as_array().unwrap().iter().map(|item| item["range"].clone()).collect::<Vec<_>>(),
        offsets.iter().map(|offset| json!(range(source, *offset, 5))).collect::<Vec<_>>(),
    );
}

async fn check_destructured_binding(body: &str) {
    let source =
        format!("<?php function run() {{ {body} return $item; }} function other() {{ $item = 99; return $item; }}");
    let offsets: Vec<_> = source.match_indices("$item").map(|(offset, _)| offset).collect();
    assert_eq!(offsets.len(), 4);
    let mut h = Harness::start(&[("a.php", &source)]).await;
    let definition = at(&mut h, &source, offsets[1], "textDocument/definition").await;
    assert_eq!(definition["range"], json!(range(&source, offsets[0], 5)), "{body}: {definition}");
    let highlights = at(&mut h, &source, offsets[0], "textDocument/documentHighlight").await;
    assert_eq!(
        highlights.as_array().unwrap().iter().map(|item| item["range"].clone()).collect::<Vec<_>>(),
        offsets[..2].iter().map(|offset| json!(range(&source, *offset, 5))).collect::<Vec<_>>(),
        "{body}",
    );
    let position = range(&source, offsets[1], 0).start;
    let references = h
        .request(
            "textDocument/references",
            json!({
                "textDocument": { "uri": h.url("a.php") }, "position": position,
                "context": { "includeDeclaration": false }
            }),
        )
        .await;
    assert_eq!(references.as_array().unwrap().len(), 1, "{body}: {references}");
    assert_eq!(references[0]["range"], json!(range(&source, offsets[1], 5)));
    let edits = rename_at(&mut h, "a.php", &source, "$item", "renamed").await.unwrap();
    assert_eq!(apply(&source, &edits["changes"][h.url("a.php")]), source.replacen("$item", "$renamed", 2), "{body}",);
}

#[tokio::test]
async fn array_and_list_destructuring_mark_nested_declarations() {
    for body in [
        "[$skip, ['key' => $item]] = [1, ['key' => 2]];",
        "list($skip, list($item)) = [1, [2]];",
        "$values = [1]; [&$item] = $values;",
    ] {
        check_destructured_binding(body).await;
    }
}

#[tokio::test]
async fn foreach_destructuring_marks_nested_declarations() {
    for body in
        ["foreach ([[1, [2]]] as [$skip, [$item]]) {}", "foreach ([[1, [2]]] as $key => list($skip, list($item))) {}"]
    {
        check_destructured_binding(body).await;
    }
}

#[tokio::test]
async fn destructuring_keys_and_member_receivers_are_reads() {
    let source = "<?php function run() { [$key => $item, 1 => $box->value] = [1, 2]; return $item; }";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for name in ["$key", "$box"] {
        let definition = at(&mut h, source, source.find(name).unwrap(), "textDocument/definition").await;
        assert!(definition.is_null(), "{name}: {definition}");
    }
}
