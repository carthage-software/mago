use serde_json::{Value, json};

use super::harness::Harness;

const CLASS_QUERIES: &[&str] = &[
    "hover",
    "definition",
    "references",
    "documentHighlight",
    "typeDefinition",
    "implementation",
    "prepareTypeHierarchy",
];

async fn query(h: &mut Harness, method: &str, line: u32, character: u32) -> Value {
    let mut params = json!({
        "textDocument": { "uri": h.url("a.php") },
        "position": { "line": line, "character": character },
    });
    if method == "references" {
        params["context"] = json!({ "includeDeclaration": true });
    }
    h.request(&format!("textDocument/{method}"), params).await
}

fn has_result(value: &Value) -> bool {
    !value.is_null() && !value.as_array().is_some_and(Vec::is_empty)
}

#[tokio::test]
async fn symbol_positions_reject_lines_outside_the_document() {
    let source = "<?php interface 😀Contract {} class Item implements 😀Contract {} new 😀Contract";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let end = source.encode_utf16().count() as u32;
    for method in CLASS_QUERIES {
        let valid = query(&mut h, method, 0, end).await;
        assert!(has_result(&valid), "valid {method}: {valid}");
        for (line, column) in [(1, 0), (99, 0), (u32::MAX, u32::MAX)] {
            let result = query(&mut h, method, line, column).await;
            assert!(result.is_null(), "{method} on invalid line ({line}, {column}): {result}");
        }
    }
}

#[tokio::test]
async fn symbol_positions_reject_split_utf16_characters() {
    let source =
        "<?php\ninterface 😀Contract {}\nclass Item implements 😀Contract {}\nfunction 😀make(): void {}\n😀make();";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for (method, line, column) in CLASS_QUERIES.iter().map(|method| (*method, 1, "interface ".len() as u32)).chain([(
        "prepareCallHierarchy",
        3,
        "function ".len() as u32,
    )]) {
        let valid = query(&mut h, method, line, column + 2).await;
        assert!(has_result(&valid), "valid {method}: {valid}");
        let result = query(&mut h, method, line, column + 1).await;
        assert!(result.is_null(), "{method} on a split UTF-16 character: {result}");
    }
}

#[tokio::test]
async fn symbol_positions_keep_the_line_end_column_clamp() {
    for newline in ["\n", "\r\n"] {
        let source = "<?php\ninterface 😀Contract {}\nclass Item implements 😀Contract\n{}\nfunction 😀make(): void {}\n😀make\n();"
            .replace('\n', newline);
        let mut h = Harness::start(&[("a.php", &source)]).await;
        for (method, line) in CLASS_QUERIES.iter().map(|method| (*method, 2)).chain([("prepareCallHierarchy", 5)]) {
            let end = source.lines().nth(line as usize).unwrap().encode_utf16().count() as u32;
            let valid = query(&mut h, method, line, end).await;
            assert!(has_result(&valid), "valid {method}: {valid}");
            for column in [end + 1, u32::MAX] {
                let result = query(&mut h, method, line, column).await;
                assert_eq!(result, valid, "{method} line-end clamp at {column}");
            }
        }
    }
}
