use serde_json::Value;
use serde_json::json;
use tower_lsp_server::gen_lsp_types::TextEdit;

use mago_database::file::File;

use super::harness::Harness;
use crate::language_server::ServerConfig;
use crate::language_server::capabilities::server_capabilities;
use crate::language_server::position::offset_at_position;

async fn request(h: &mut Harness, start: (u32, u32), end: (u32, u32)) -> Value {
    h.request(
        "textDocument/rangeFormatting",
        json!({
            "textDocument": { "uri": h.url("a.php") },
            "range": {
                "start": { "line": start.0, "character": start.1 },
                "end": { "line": end.0, "character": end.1 }
            },
            "options": { "tabSize": 4, "insertSpaces": true }
        }),
    )
    .await
}

#[tokio::test]
async fn range_formatting_uses_live_text_and_utf16_bounds() {
    let mut h = Harness::start(&[("a.php", "<?php $old=1;")]).await;
    let source = "<?php\n$before=  1;\n$😀= [\"yes\" ,2 ];\n$after=  3;\n";
    h.open("a.php", source).await;
    let edits: Vec<TextEdit> = serde_json::from_value(request(&mut h, (2, 7), (2, 10)).await).unwrap();
    assert_eq!(edits.len(), 1);
    let edit = &edits[0];
    assert_eq!(edit.range.start.line, 2);
    assert_eq!(edit.range.start.character, 0);
    assert_eq!(edit.range.end.line, 2);
    assert_eq!(edit.range.end.character, source.lines().nth(2).unwrap().encode_utf16().count() as u32);
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let start = offset_at_position(&file, edit.range.start) as usize;
    let end = offset_at_position(&file, edit.range.end) as usize;
    let result = format!("{}{}{}", &source[..start], edit.new_text, &source[end..]);
    assert_eq!(result, "<?php\n$before=  1;\n$😀 = ['yes', 2];\n$after=  3;\n");
    h.change("a.php", &result, 2).await;
    assert!(request(&mut h, (2, 7), (2, 10)).await.is_null());
}

#[tokio::test]
async fn range_formatting_rejects_invalid_ranges_and_syntax() {
    let mut h = Harness::start(&[("a.php", "<?php\n$x=1;\n")]).await;
    for (start, end) in [((1, 4), (1, 0)), ((0, 0), (99, 0))] {
        assert!(request(&mut h, start, end).await.is_null(), "invalid range must return no edit");
    }
    h.open("a.php", "<?php\n$x=;\n").await;
    assert!(request(&mut h, (1, 0), (1, 2)).await.is_null());
}

#[tokio::test]
async fn no_formatter_disables_range_formatting() {
    let config = ServerConfig { formatter: false, ..ServerConfig::default() };
    assert!(server_capabilities(&config).document_range_formatting_provider.is_none());
    let mut h = Harness::start_with(&[("a.php", "<?php\n$x=1;\n")], config, json!({})).await;
    assert!(request(&mut h, (1, 0), (1, 2)).await.is_null());
}

#[tokio::test]
async fn range_formatting_clamps_columns_but_rejects_split_characters() {
    let mut h = Harness::start(&[("a.php", "<?php\n$😀=1;\r\n")]).await;
    let exact = request(&mut h, (1, 0), (1, 6)).await;
    assert_eq!(exact.as_array().unwrap().len(), 1);
    assert_eq!(request(&mut h, (1, 0), (1, 999)).await, exact);
    assert!(request(&mut h, (1, 2), (1, 5)).await.is_null());
}
