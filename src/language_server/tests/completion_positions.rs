use serde_json::{Value, json};
use tower_lsp_server::gen_lsp_types::TextEdit;

use mago_database::file::File;

use super::harness::Harness;

use crate::language_server::position::offset_at_position;

fn items(result: &Value) -> &[Value] {
    result.as_array().or_else(|| result["items"].as_array()).map_or(&[], Vec::as_slice)
}

fn check_edit(result: &Value, source: &str, selected: &str, replacement: &str) {
    let item = items(result).iter().find(|item| item["label"] == replacement).unwrap_or_else(|| panic!("{result}"));
    let start = source.rfind(selected).unwrap();
    let before = &source[..start];
    let line = before.bytes().filter(|byte| *byte == b'\n').count();
    let column = before.rsplit('\n').next().unwrap().encode_utf16().count();
    assert_eq!(
        item["textEdit"]["range"],
        json!({
            "start": { "line": line, "character": column },
            "end": { "line": line, "character": column + selected.encode_utf16().count() }
        })
    );
    assert_eq!(item["textEdit"]["newText"], replacement);
    let mut expected = source.to_owned();
    expected.replace_range(start..start + selected.len(), replacement);
    let edit: TextEdit = serde_json::from_value(item["textEdit"].clone()).unwrap();
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let start = offset_at_position(&file, edit.range.start) as usize;
    let end = offset_at_position(&file, edit.range.end) as usize;
    let mut applied = source.to_owned();
    applied.replace_range(start..end, &edit.new_text);
    assert_eq!(applied, expected);
}

#[tokio::test]
async fn completion_positions_reject_lines_outside_the_document() {
    let source = "<?php class Target {} new Tar";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let end = source.encode_utf16().count() as u32;
    let valid = h.at("textDocument/completion", "a.php", 0, end).await;
    check_edit(&valid, source, "Tar", "Target");
    for (line, column) in [(1, 0), (99, 0), (u32::MAX, u32::MAX)] {
        let result = h.at("textDocument/completion", "a.php", line, column).await;
        assert!(items(&result).is_empty(), "invalid line ({line}, {column}): {result}");
    }
}

#[tokio::test]
async fn completion_positions_reject_split_utf16_characters() {
    let source = "<?php class 😀Thing {} new 😀Th;";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let start = source.rfind("😀Th").unwrap();
    let column = source[..start].encode_utf16().count() as u32;
    let valid = h.at("textDocument/completion", "a.php", 0, column + 4).await;
    check_edit(&valid, source, "😀Th", "😀Thing");
    let result = h.at("textDocument/completion", "a.php", 0, column + 1).await;
    assert!(items(&result).is_empty(), "split UTF-16 character: {result}");
}

#[tokio::test]
async fn completion_positions_keep_the_line_end_column_clamp() {
    for newline in ["\n", "\r\n"] {
        let source = "<?php\nclass 😀Thing {}\nnew 😀Th\n;".replace('\n', newline);
        let mut h = Harness::start(&[("a.php", &source)]).await;
        let end = source.lines().nth(2).unwrap().encode_utf16().count() as u32;
        let valid = h.at("textDocument/completion", "a.php", 2, end).await;
        check_edit(&valid, &source, "😀Th", "😀Thing");
        for column in [end + 1, u32::MAX] {
            let result = h.at("textDocument/completion", "a.php", 2, column).await;
            assert_eq!(result, valid, "line-end clamp at {column}");
            check_edit(&result, &source, "😀Th", "😀Thing");
        }
    }
}
