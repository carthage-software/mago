//! Requests inside dependencies outside the editor's workspace roots.

use serde_json::json;
use tempfile::TempDir;

use super::harness::{Harness, client_capabilities, url};
use crate::language_server::ServerConfig;

const SOURCE: &str =
    "<?php\nfunction external_value(): int { return 1; }\nfunction local_call(): int { return external_value(); }\n";

async fn open_external_dependency() -> (Harness, TempDir, String) {
    let dependency = tempfile::tempdir().unwrap();
    let path = dependency.path().join("dependency.php");
    std::fs::write(&path, SOURCE).unwrap();
    let uri = url(&path.canonicalize().unwrap());
    let mut config = ServerConfig::default();
    config.configuration.source.includes = vec![dependency.path().to_string_lossy().into_owned()];
    let mut h = Harness::start_with(&[("a.php", "<?php\nexternal_value();\n")], config, client_capabilities()).await;
    let definition = h.at("textDocument/definition", "a.php", 1, 5).await;
    assert_eq!(definition["uri"], uri);
    h.client
        .send_notification(
            "textDocument/didOpen",
            json!({
                "textDocument": { "uri": uri, "languageId": "php", "version": 1, "text": SOURCE }
            }),
        )
        .await;
    (h, dependency, uri)
}

#[tokio::test]
async fn external_dependency_hover_uses_the_open_buffer() {
    let (mut h, _dependency, uri) = open_external_dependency().await;
    let edited = SOURCE.replace(": int", ": string").replace("return 1;", "return 'buffer';");
    h.client
        .send_notification(
            "textDocument/didChange",
            json!({
                "textDocument": { "uri": uri, "version": 2 }, "contentChanges": [{ "text": edited }]
            }),
        )
        .await;
    let hover = h.at_uri("textDocument/hover", &uri, 1, 12).await;
    assert!(
        hover["contents"]["value"].as_str().is_some_and(|text| text.contains("function external_value(): string")),
        "{hover}"
    );
}

#[tokio::test]
async fn external_dependency_completion_uses_workspace_symbols() {
    let (mut h, _dependency, uri) = open_external_dependency().await;
    let column = SOURCE.lines().nth(2).unwrap().find("external_value").unwrap() + "external_".len();
    let completion = h.at_uri("textDocument/completion", &uri, 2, column as u32).await;
    assert!(
        completion["items"].as_array().is_some_and(|items| items.iter().any(|item| item["label"] == "external_value")),
        "{completion}"
    );
}

#[tokio::test]
async fn external_dependency_references_include_host_calls() {
    let (mut h, _dependency, uri) = open_external_dependency().await;
    let references = h
        .request(
            "textDocument/references",
            json!({
                "textDocument": { "uri": uri }, "position": { "line": 1, "character": 12 },
                "context": { "includeDeclaration": false }
            }),
        )
        .await;
    assert!(
        references
            .as_array()
            .is_some_and(|references| references.iter().any(|reference| reference["uri"] == h.url("a.php"))),
        "{references}"
    );
}

#[tokio::test]
async fn external_includes_do_not_override_an_explicit_workspace() {
    let config = "[source]\nincludes = ['../owner']\n[language-server]\ndisable = ['hover']\n";
    let mut h = Harness::start_multi(&[
        ("consumer", &[("a.php", "<?php external_value();"), ("mago.toml", config)]),
        ("owner", &[("dependency.php", SOURCE)]),
    ])
    .await;
    let uri = h.url_in("owner", "dependency.php");
    let hover = h.at_uri("textDocument/hover", &uri, 1, 12).await;
    assert!(
        hover["contents"]["value"].as_str().is_some_and(|text| text.contains("function external_value(): int")),
        "{hover}"
    );

    h.client
        .send_notification(
            "workspace/didChangeWorkspaceFolders",
            json!({
                "event": { "added": [], "removed": [{ "uri": h.url_in("owner", ""), "name": "owner" }] }
            }),
        )
        .await;
    assert!(h.at_uri("textDocument/hover", &uri, 1, 12).await.is_null());
}
