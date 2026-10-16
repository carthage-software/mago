use serde_json::{Value, json};

use super::harness::Harness;
use super::references::{apply, rename_or_refusal};

async fn query(h: &mut Harness, method: &str, line: u32, character: u32) -> Value {
    let mut params = json!({
        "textDocument": { "uri": h.url("a.php") },
        "position": { "line": line, "character": character }
    });
    if method == "rename" {
        params["newName"] = json!("Renamed");
        return rename_or_refusal(h, params).await.unwrap();
    }
    h.request(&format!("textDocument/{method}"), params).await
}

async fn reject_invalid_positions(method: &str) {
    // The final expression lacks its semicolon, as in an unfinished editor buffer.
    let source = "<?php $😀item = 1;\n$😀item";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let inside = query(&mut h, method, 1, 3).await;
    assert!(!inside.is_null(), "valid position: {method}: {inside}");
    assert_eq!(query(&mut h, method, 1, 7).await, inside, "valid EOF caret: {method}");
    for (line, character) in [(1, 2), (2, 0), (99, 0), (u32::MAX, u32::MAX)] {
        let result = query(&mut h, method, line, character).await;
        assert!(result.is_null(), "{method} at ({line}, {character}): {result}");
    }
}

#[tokio::test]
async fn rename_positions_prepare_rejects_split_surrogates_and_unknown_lines() {
    reject_invalid_positions("prepareRename").await;
}

#[tokio::test]
async fn rename_positions_rename_rejects_split_surrogates_and_unknown_lines() {
    reject_invalid_positions("rename").await;
}

#[tokio::test]
async fn rename_positions_keep_right_edges_and_line_end_column_clamping() {
    for newline in ["\n", "\r\n"] {
        let source = "<?php\nclass 😀Thing {}\n$😀item = new 😀Thing\n();\n$😀item\n;".replace('\n', newline);
        let mut h = Harness::start(&[("a.php", &source)]).await;
        for (line, name) in [(2, "😀Thing"), (4, "$😀item")] {
            let end = source.lines().nth(line as usize).unwrap().encode_utf16().count() as u32;
            for method in ["prepareRename", "rename"] {
                let inside = query(&mut h, method, line, end - 1).await;
                assert!(!inside.is_null(), "{method}: {name}: {inside}");
                for column in [end, end + 1, u32::MAX] {
                    assert_eq!(query(&mut h, method, line, column).await, inside, "{method}: {name}: {column}");
                }
                if method == "prepareRename" {
                    assert_eq!(inside["placeholder"], name.trim_start_matches('$'));
                } else {
                    let replacement = if name.starts_with('$') { "$Renamed" } else { "Renamed" };
                    assert_eq!(apply(&source, &inside["changes"][h.url("a.php")]), source.replace(name, replacement));
                }
            }
        }
    }
}
