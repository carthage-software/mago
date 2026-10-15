use mago_database::file::File;
use serde_json::json;
use tower_lsp_server::gen_lsp_types::Position;
use tower_lsp_server::gen_lsp_types::SelectionRange;

use super::harness::Harness;
use crate::language_server::ServerConfig;
use crate::language_server::position::offset_at_position;
use crate::language_server::position::position_at_offset;

async fn selections(source: &str, offsets: &[usize]) -> (File, Vec<SelectionRange>) {
    let config = ServerConfig { analyzer: false, linter: false, ..ServerConfig::default() };
    let mut harness = Harness::start_with(&[("a.php", source)], config, json!({})).await;
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let positions: Vec<_> = offsets.iter().map(|offset| position_at_offset(&file, *offset as u32)).collect();
    let result = harness
        .request(
            "textDocument/selectionRange",
            json!({
                "textDocument": { "uri": harness.url("a.php") }, "positions": positions,
            }),
        )
        .await;
    let ranges: Vec<SelectionRange> = serde_json::from_value(result).unwrap();
    assert_eq!(ranges.len(), positions.len());
    for (range, position) in ranges.iter().zip(positions) {
        check_chain(range, position);
    }
    (file, ranges)
}

fn check_chain(mut selection: &SelectionRange, cursor: Position) {
    loop {
        assert!(selection.range.start <= cursor && cursor <= selection.range.end, "{selection:?} at {cursor:?}");
        let Some(parent) = selection.parent.as_deref() else { break };
        assert!(parent.range.start <= selection.range.start && selection.range.end <= parent.range.end);
        assert_ne!(parent.range, selection.range);
        selection = parent;
    }
}

fn texts<'a>(file: &'a File, mut selection: &SelectionRange) -> Vec<&'a str> {
    let mut result = Vec::new();
    loop {
        let start = offset_at_position(file, selection.range.start) as usize;
        let end = offset_at_position(file, selection.range.end) as usize;
        result.push(std::str::from_utf8(&file.contents[start..end]).unwrap());
        let Some(parent) = selection.parent.as_deref() else { break };
        selection = parent;
    }
    result
}

#[tokio::test]
async fn selection_expands_from_method_token_through_call_statement_and_declaration() {
    let source = "<?php\nnamespace App;\nfunction run($service) {\n    return $service->name('x');\n}\n";
    let offset = source.find("->name").unwrap() + 3;
    let (file, ranges) = selections(source, &[offset]).await;
    let chain = texts(&file, &ranges[0]);
    assert_eq!(chain[0], "name");
    let call = chain.iter().position(|text| *text == "$service->name('x')").unwrap();
    let statement = chain.iter().position(|text| *text == "return $service->name('x');").unwrap();
    let block = chain.iter().position(|text| *text == "{\n    return $service->name('x');\n}").unwrap();
    let function = chain.iter().position(|text| text.starts_with("function run(")).unwrap();
    assert!(call < statement && statement < block && block < function);
    assert_eq!(chain.last(), Some(&source));
}

#[tokio::test]
async fn selection_keeps_utf16_tokens_and_contains_boundary_and_eof_cursors() {
    let source = "<?php /*😀*/ function café($service) { $service->naïve('é😀'); $service->next(); }";
    let start = source.find("naïve").unwrap();
    let next = source.find("$service->next").unwrap();
    let offsets = [start + 2, start, start + "naïve".len(), next, source.len(), 0];
    let (file, ranges) = selections(source, &offsets).await;
    assert_eq!(texts(&file, &ranges[0])[0], "naïve");
    assert_eq!(ranges[0].range.start.character, source[..start].encode_utf16().count() as u32);
    assert_eq!(ranges[0].range.end.character - ranges[0].range.start.character, 5);
    let next_chain = texts(&file, &ranges[3]);
    assert!(next_chain.contains(&"$service->next();"));
    assert!(!next_chain.contains(&"$service->naïve('é😀');"));
    assert_eq!(texts(&file, &ranges[4]).last(), Some(&source));
}

#[tokio::test]
async fn selection_handles_empty_files_and_trailing_whitespace() {
    for source in ["", "<?php foo();   \n"] {
        let (file, ranges) = selections(source, &[source.len()]).await;
        assert_eq!(texts(&file, &ranges[0]).last(), Some(&source));
    }
}
