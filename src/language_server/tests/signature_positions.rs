use super::harness::Harness;

#[tokio::test]
async fn signature_positions_reject_lines_outside_the_document() {
    let source = "<?php function sample(int $value): void {} sample(";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let end = source.encode_utf16().count() as u32;
    let valid = h.at("textDocument/signatureHelp", "a.php", 0, end).await;
    assert_eq!(valid["signatures"][0]["label"], "function sample(int $value): void", "{valid}");
    for (line, column) in [(1, 0), (99, 0), (u32::MAX, u32::MAX)] {
        let result = h.at("textDocument/signatureHelp", "a.php", line, column).await;
        assert!(result.is_null(), "invalid line ({line}, {column}): {result}");
    }
}

#[tokio::test]
async fn signature_positions_reject_split_utf16_characters() {
    let source = "<?php function sample(string $value): void {} sample('😀');";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let start = source.find('😀').unwrap();
    let column = source[..start].encode_utf16().count() as u32;
    for valid_column in [column, column + 2] {
        let valid = h.at("textDocument/signatureHelp", "a.php", 0, valid_column).await;
        assert_eq!(valid["signatures"][0]["label"], "function sample(string $value): void", "{valid}");
    }
    let result = h.at("textDocument/signatureHelp", "a.php", 0, column + 1).await;
    assert!(result.is_null(), "split UTF-16 character: {result}");
}

#[tokio::test]
async fn signature_positions_keep_the_line_end_column_clamp() {
    for newline in ["\n", "\r\n"] {
        let source =
            "<?php\nfunction sample(string $value, int $count = 1): void {}\nsample('😀',\n1);".replace('\n', newline);
        let mut h = Harness::start(&[("a.php", &source)]).await;
        let end = source.lines().nth(2).unwrap().encode_utf16().count() as u32;
        let valid = h.at("textDocument/signatureHelp", "a.php", 2, end).await;
        assert_eq!(valid["signatures"][0]["label"], "function sample(string $value, int $count = 1): void", "{valid}",);
        assert_eq!(valid["activeParameter"], 1, "{valid}");
        for column in [end + 1, u32::MAX] {
            let result = h.at("textDocument/signatureHelp", "a.php", 2, column).await;
            assert_eq!(result, valid, "line-end clamp at {column}");
        }
    }
}
