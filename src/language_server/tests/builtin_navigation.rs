use serde_json::json;
use tower_lsp_server::gen_lsp_types::Location;

use mago_database::file::File;

use super::harness::{Harness, client_capabilities};
use super::references::{READ_ONLY_STUBS, refusable};
use crate::language_server::ServerConfig;
use crate::language_server::position::{offset_at_position, position_at_offset};
use crate::language_server::uri::UriFilePath;

#[tokio::test]
async fn builtin_definitions_open_source_and_support_followup_queries() {
    let source = "<?php strlen('x');";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let result = h.at("textDocument/definition", "a.php", 0, 8).await;
    let location: Location = serde_json::from_value(result).expect("builtin source location");
    let path = location.uri.to_file_path().unwrap().into_owned();
    assert!(!path.starts_with(&h.workspace));
    let components: Vec<_> = path.components().map(|component| component.as_os_str()).collect();
    assert!(
        components.windows(2).any(|pair| pair[0] == "mago-language-server" && pair[1] == crate::consts::VERSION),
        "{}",
        path.display()
    );
    let contents = std::fs::read(&path).unwrap();
    let file = File::ephemeral(b"stub.php".as_slice().into(), contents.clone().into());
    let start = offset_at_position(&file, location.range.start) as usize;
    let end = offset_at_position(&file, location.range.end) as usize;
    assert_eq!(&contents[start..end], b"strlen");
    assert!(std::fs::metadata(&path).unwrap().permissions().readonly());
    let params = json!({"textDocument": {"uri": location.uri}, "position": location.range.start});
    h.client.send_notification("textDocument/didOpen", json!({"textDocument": {
        "uri": location.uri, "languageId": "php", "version": 1, "text": String::from_utf8(contents.clone()).unwrap()
    }})).await;
    h.client
        .send_notification(
            "textDocument/didChange",
            json!({
                "textDocument": {"uri": location.uri, "version": 2}, "contentChanges": [{"text": "<?php"}]
            }),
        )
        .await;
    let hover = h.request("textDocument/hover", params.clone()).await;
    assert!(hover["contents"]["value"].as_str().unwrap().contains("function strlen(string $string):"));
    let references = h
        .request(
            "textDocument/references",
            json!({
                "textDocument": {"uri": location.uri}, "position": location.range.start,
                "context": {"includeDeclaration": false}
            }),
        )
        .await;
    assert!(references.as_array().unwrap().iter().any(|reference| reference["uri"] == h.url("a.php")));
    assert_eq!(refusable(&mut h, "textDocument/prepareRename", params).await, Err(READ_ONLY_STUBS.into()));
    assert!(
        h.request(
            "textDocument/formatting",
            json!({
                "textDocument": {"uri": location.uri}, "options": {"tabSize": 4, "insertSpaces": true}
            })
        )
        .await
        .is_null()
    );
    assert_eq!(std::fs::read(path).unwrap(), contents);
}

#[tokio::test]
async fn builtin_hierarchies_keep_edges_to_prelude_declarations() {
    let source = "<?php class Failure extends RuntimeException {} function size(): int { return strlen('x'); }";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
    let position = position_at_offset(&file, source.find("Failure").unwrap() as u32);
    let prepared = h.at("textDocument/prepareTypeHierarchy", "a.php", position.line, position.character).await;
    let parents = h.request("typeHierarchy/supertypes", json!({"item": prepared[0]})).await;
    assert_eq!(parents[0]["name"], "RuntimeException");
    let grandparents = h.request("typeHierarchy/supertypes", json!({"item": parents[0]})).await;
    assert_eq!(grandparents[0]["name"], "Exception");
    let position = position_at_offset(&file, source.find("size()").unwrap() as u32);
    let prepared = h.at("textDocument/prepareCallHierarchy", "a.php", position.line, position.character).await;
    let calls = h.request("callHierarchy/outgoingCalls", json!({"item": prepared[0]})).await;
    assert_eq!(calls[0]["to"]["name"], "strlen");
    let callers = h.request("callHierarchy/incomingCalls", json!({"item": calls[0]["to"]})).await;
    assert_eq!(callers[0]["from"]["name"], "size");
}

#[tokio::test]
async fn builtin_sources_keep_workspace_routing_after_server_restart() {
    let mut first = Harness::start(&[("a.php", "<?php strlen('x');")]).await;
    let result = first.at("textDocument/definition", "a.php", 0, 8).await;
    let location: Location = serde_json::from_value(result).unwrap();
    let host_uri = first.url("a.php");
    first.request("shutdown", json!(null)).await;
    first.client.send_notification("exit", json!(null)).await;

    let config = ServerConfig { workspace_override: Some(first.workspace.clone()), ..Default::default() };
    let mut restarted = Harness::start_with(&[], config, client_capabilities()).await;
    let params = json!({ "textDocument": { "uri": location.uri }, "position": location.range.start });
    let hover = restarted.request("textDocument/hover", params.clone()).await;
    assert!(hover["contents"]["value"].as_str().unwrap().contains("function strlen(string $string):"));
    let references = restarted
        .request(
            "textDocument/references",
            json!({
                "textDocument": { "uri": location.uri }, "position": location.range.start,
                "context": { "includeDeclaration": false }
            }),
        )
        .await;
    assert!(references.as_array().unwrap().iter().any(|reference| reference["uri"] == host_uri));
    let definition = restarted
        .request(
            "textDocument/definition",
            json!({
                "textDocument": { "uri": host_uri }, "position": { "line": 0, "character": 8 }
            }),
        )
        .await;
    assert_eq!(definition["uri"], location.uri.as_str());
}

#[tokio::test]
async fn restarting_from_a_builtin_root_recovers_the_project_without_analyzing_stubs() {
    let mut first = Harness::start(&[("a.php", "<?php strlen('x');")]).await;
    let result = first.at("textDocument/definition", "a.php", 0, 8).await;
    let location: Location = serde_json::from_value(result).unwrap();
    let path = location.uri.to_file_path().unwrap().into_owned();
    let contents = std::fs::read_to_string(&path).unwrap();
    let host_uri = first.url("a.php");
    first.request("shutdown", json!(null)).await;
    first.client.send_notification("exit", json!(null)).await;

    for root in [path.parent().unwrap(), path.as_path()] {
        let mut restarted =
            Harness::start_with_root(&[], ServerConfig::default(), client_capabilities(), Some(root)).await;
        restarted
            .client
            .send_notification(
                "textDocument/didOpen",
                json!({"textDocument": {
                    "uri": location.uri, "languageId": "php", "version": 1, "text": contents
                }}),
            )
            .await;
        let diagnostics = restarted.diagnostics().await;
        let stub_diagnostics: Vec<_> = diagnostics
            .iter()
            .filter(|publication| publication["uri"] == location.uri.as_str())
            .flat_map(|publication| publication["diagnostics"].as_array().unwrap())
            .collect();
        assert!(
            stub_diagnostics.is_empty(),
            "restarting from {} produced {} builtin diagnostics; first three: {:?}",
            root.display(),
            stub_diagnostics.len(),
            &stub_diagnostics[..stub_diagnostics.len().min(3)],
        );
        let params = json!({ "textDocument": { "uri": location.uri }, "position": location.range.start });
        let hover = restarted.request("textDocument/hover", params.clone()).await;
        assert!(hover["contents"]["value"].as_str().unwrap().contains("function strlen(string $string):"));
        let references = restarted
            .request(
                "textDocument/references",
                json!({
                    "textDocument": { "uri": location.uri }, "position": location.range.start,
                    "context": { "includeDeclaration": false }
                }),
            )
            .await;
        assert!(references.as_array().unwrap().iter().any(|reference| reference["uri"] == host_uri), "{references}");
        assert_eq!(refusable(&mut restarted, "textDocument/prepareRename", params).await, Err(READ_ONLY_STUBS.into()));
        assert!(
            restarted
                .request(
                    "textDocument/formatting",
                    json!({
                        "textDocument": { "uri": location.uri }, "options": { "tabSize": 4, "insertSpaces": true }
                    })
                )
                .await
                .is_null()
        );
        assert_eq!(std::fs::read_to_string(&path).unwrap(), contents);
    }
}
