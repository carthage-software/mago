use mago_database::file::File;
use serde_json::{Value, json};

use super::harness::{Harness, client_capabilities};
use crate::language_server::ServerConfig;
use crate::language_server::position::{position_at_offset, range_at_offsets};

async fn hover(harness: &mut Harness, file: &File, offset: usize) -> Value {
    let position = position_at_offset(file, offset as u32);
    harness.at("textDocument/hover", "a.php", position.line, position.character).await
}

#[tokio::test]
async fn literal_hover_matches_analyzed_variable_types_and_utf16_ranges() {
    let source = "<?php $text = 'é😀'; $count = 0x2A; $large = 9223372036854775808; $ratio = 1.5; $yes = TRUE; $no = false; $empty = null; echo $text, $count, $large, $ratio, $yes, $no, $empty;";
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().into());
    let mut harness = Harness::start(&[("a.php", source)]).await;
    for (literal, variable, ty) in [
        ("'é😀'", "$text", "string"),
        ("0x2A", "$count", "int"),
        ("9223372036854775808", "$large", "float"),
        ("1.5", "$ratio", "float"),
        ("TRUE", "$yes", "bool"),
        ("false", "$no", "bool"),
        ("null", "$empty", "null"),
    ] {
        let start = source.find(literal).unwrap();
        let result = hover(&mut harness, &file, start + 1).await;
        let expected = if ty == "null" { literal.to_owned() } else { format!("{ty}\n{literal}") };
        assert_eq!(result["contents"]["value"], expected, "{result}");
        assert_eq!(result["range"], json!(range_at_offsets(&file, start as u32, (start + literal.len()) as u32)));
        let variable_hover = hover(&mut harness, &file, source.rfind(variable).unwrap() + 1).await;
        assert_eq!(variable_hover["contents"]["value"].as_str().unwrap().split(' ').next(), Some(ty));
    }
}

#[tokio::test]
async fn literal_hover_without_analyzer_stays_inside_literal_expressions() {
    let source = "<?php\n// 'comment' 1234 true\nfunction f(true $value): false { return false; }\n$value = ['oops', 42, 1.5, true, false, null, 1 + 2, \"hello $value\"];";
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().into());
    let config = ServerConfig { analyzer: false, linter: false, ..Default::default() };
    let mut harness = Harness::start_with(&[("a.php", source)], config, client_capabilities()).await;
    for (literal, expected) in
        [("'oops'", "string\n'oops'"), ("42", "int\n42"), ("1.5", "float\n1.5"), ("null", "null")]
    {
        let result = hover(&mut harness, &file, source.find(literal).unwrap() + 1).await;
        assert_eq!(result["contents"]["value"], expected, "{result}");
    }
    for marker in ["comment", "1234", "true $value", ": false", "+", "hello"] {
        let offset = source.find(marker).unwrap();
        let offset = if marker == ": false" { offset + 2 } else { offset };
        assert!(hover(&mut harness, &file, offset).await.is_null(), "{marker}");
    }
    let end = source.find("'oops'").unwrap() + "'oops'".len();
    assert!(hover(&mut harness, &file, end).await.is_null());
}

#[tokio::test]
async fn literal_hover_bounds_values_and_keeps_multiline_code_fences_safe() {
    let source = "<?php $value = '```line\né😀';";
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().into());
    let mut harness = Harness::start_with(
        &[("a.php", source)],
        ServerConfig { analyzer: false, linter: false, ..Default::default() },
        json!({"textDocument":{"hover":{"contentFormat":["markdown"]}}}),
    )
    .await;
    let result = hover(&mut harness, &file, source.find("line").unwrap()).await;
    assert_eq!(result["contents"]["value"], "````php\nstring\n'```line\né😀'\n````");
    assert_eq!(result["range"]["end"], json!({"line": 1, "character": 4}));

    let large = format!("<?php $value = '{}😀{}';", "é".repeat(255), "long value".repeat(1000));
    harness.open("a.php", source).await;
    harness.change("a.php", &large, 2).await;
    let file = File::ephemeral(b"a.php".as_slice().into(), large.as_bytes().to_vec().into());
    let result = hover(&mut harness, &file, large.find('é').unwrap()).await;
    let preview = result["contents"]["value"].as_str().unwrap();
    assert!(preview.contains("The value exceeds the preview limit."), "{result}");
    assert!(preview.len() < 600 && !preview.contains("\\xF0") && !preview.contains('\u{FFFD}'), "{result}");
    assert_eq!(result["range"]["end"]["character"], large[..large.len() - 1].encode_utf16().count());
}
