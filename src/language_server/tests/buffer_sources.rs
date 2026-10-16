//! Open buffers keep the source type chosen by the workspace configuration.

use serde_json::json;

use super::harness::{Harness, client_capabilities, url};
use super::references::{refusable, rename_or_refusal};
use crate::language_server::ServerConfig;

#[tokio::test]
async fn new_dependency_buffers_keep_their_source_type() {
    for patch in [false, true] {
        let dependency = tempfile::tempdir().unwrap();
        let path = dependency.path().canonicalize().unwrap().join("new.php");
        let uri = url(&path);
        let source = "<?php class Dependency {}";
        let mut config = ServerConfig::default();
        let roots = vec![dependency.path().to_string_lossy().into_owned()];
        if patch {
            config.configuration.source.patches = roots;
        } else {
            config.configuration.source.includes = roots;
        }
        let mut h = Harness::start_with(&[("a.php", "<?php")], config, client_capabilities()).await;
        h.client
            .send_notification(
                "textDocument/didOpen",
                json!({
                    "textDocument": { "uri": uri, "languageId": "php", "version": 1, "text": source }
                }),
            )
            .await;
        for saved in [false, true] {
            if saved {
                std::fs::write(&path, source).unwrap();
                h.client.send_notification("textDocument/didClose", json!({ "textDocument": { "uri": uri } })).await;
            }
            let symbols = h.request("textDocument/documentSymbol", json!({ "textDocument": { "uri": uri } })).await;
            assert!(
                symbols.as_array().is_some_and(|items| items.iter().any(|item| item["name"] == "Dependency")),
                "patch={patch}, saved={saved}: {symbols}"
            );
            // An orphan patch keeps its outline but adds no class metadata.
            if !patch {
                let hover = h.at_uri("textDocument/hover", &uri, 0, 14).await;
                assert!(hover["contents"]["value"].as_str().is_some_and(|value| value.contains("class Dependency")));
            }
            let position = json!({ "textDocument": { "uri": uri }, "position": { "line": 0, "character": 14 } });
            let prepared = refusable(&mut h, "textDocument/prepareRename", position).await;
            assert!(prepared.is_err(), "patch={patch}, saved={saved}: {prepared:?}");
            let params = json!({
                "textDocument": { "uri": uri }, "position": { "line": 0, "character": 14 }, "newName": "Changed"
            });
            let rename = rename_or_refusal(&mut h, params).await;
            assert!(rename.is_err(), "patch={patch}, saved={saved}: {rename:?}");
        }
    }
}

#[tokio::test]
async fn open_buffers_respect_the_owning_workspaces_exclusions() {
    let source = "<?php class Excluded {}";
    let mut h = Harness::start_multi(&[
        ("consumer", &[("a.php", "<?php"), ("mago.toml", "[source]\nincludes = ['../owner']\n")]),
        ("owner", &[("a.php", "<?php"), ("mago.toml", "[source]\nexcludes = ['excluded.php']\n")]),
    ])
    .await;
    let uri = h.url_in("owner", "excluded.php");
    h.client
        .send_notification(
            "textDocument/didOpen",
            json!({
                "textDocument": { "uri": uri, "languageId": "php", "version": 1, "text": source }
            }),
        )
        .await;
    assert!(h.at_uri("textDocument/hover", &uri, 0, 14).await.is_null());
    assert!(h.at_uri("textDocument/prepareRename", &uri, 0, 14).await.is_null());
}

#[tokio::test]
async fn configured_patches_supply_editor_signatures() {
    let library = tempfile::tempdir().unwrap();
    let patch = tempfile::tempdir().unwrap();
    std::fs::write(
        library.path().join("value.php"),
        "<?php function library_value(int $original): int { return $original; }",
    )
    .unwrap();
    std::fs::write(
        patch.path().join("value.php"),
        "<?php function library_value(string $original): string { return $original; }",
    )
    .unwrap();
    let mut config = ServerConfig::default();
    config.configuration.source.includes = vec![library.path().to_string_lossy().into_owned()];
    config.configuration.source.patches = vec![patch.path().to_string_lossy().into_owned()];
    let source = "<?php library_value('value');";
    let mut h = Harness::start_with(&[("a.php", source)], config, client_capabilities()).await;
    let signature = h.at("textDocument/signatureHelp", "a.php", 0, source.find("'value'").unwrap() as u32).await;
    assert!(
        signature["signatures"][0]["label"].as_str().is_some_and(|label| label.contains("string $original")),
        "{signature}"
    );
}
