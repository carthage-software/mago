use serde_json::Value;
use serde_json::json;
use tempfile::TempDir;
use tokio::task::JoinHandle;

use super::client::LspClient;
use super::harness::{client_capabilities, url};
use crate::config::Configuration;
use crate::language_server::ServerConfig;
use crate::language_server::capabilities::server_capabilities;

const PROVIDERS: &[(&str, &str)] = &[
    ("formatting", "documentFormattingProvider"),
    ("range-formatting", "documentRangeFormattingProvider"),
    ("hover", "hoverProvider"),
    ("definition", "definitionProvider"),
    ("type-definition", "typeDefinitionProvider"),
    ("document-highlight", "documentHighlightProvider"),
    ("implementation", "implementationProvider"),
    ("type-hierarchy", "typeHierarchyProvider"),
    ("call-hierarchy", "callHierarchyProvider"),
    ("references", "referencesProvider"),
    ("rename", "renameProvider"),
    ("completion", "completionProvider"),
    ("folding-range", "foldingRangeProvider"),
    ("selection-range", "selectionRangeProvider"),
    ("code-action", "codeActionProvider"),
    ("workspace-symbol", "workspaceSymbolProvider"),
    ("document-symbol", "documentSymbolProvider"),
    ("signature-help", "signatureHelpProvider"),
    ("inlay-hint", "inlayHintProvider"),
    ("document-link", "documentLinkProvider"),
    ("code-lens", "codeLensProvider"),
];
const PHP: &str = "<?php\nclass Greeter{}\nfunction greet(string $name): string { return $name; }\n$g = new Greeter();\ngreet('hi');\nclass Child extends Greeter {}\n";

fn disabled_config() -> String {
    let names: Vec<_> = PROVIDERS.iter().map(|(name, _)| *name).collect();
    format!("[language-server]\ndisable = {}\n", serde_json::to_string(&names).unwrap())
}

#[test]
fn provider_config_and_schema() {
    let config: Configuration = toml::from_str(&disabled_config()).unwrap();
    let names: Vec<_> = PROVIDERS.iter().map(|(name, _)| *name).collect();
    assert_eq!(config.to_filtered_value()["language-server"]["disable"], json!(names));
    let schema = serde_json::to_value(schemars::schema_for!(Configuration)).unwrap();
    assert!(!schema["properties"]["language-server"].is_null());
    assert_eq!(schema["$defs"]["Provider"]["enum"], json!(names));
    assert!(toml::from_str::<Configuration>("[language-server]\ndisable = ['hovre']").is_err());
    assert!(toml::from_str::<Configuration>("[language-server]\ndisabled = ['hover']").is_err());
    let defaults: Configuration = toml::from_str("").unwrap();
    assert!(defaults.language_server.disable.is_empty());
    assert!(!defaults.language_server.hide_zero_reference_code_lenses);
    let hidden: Configuration = toml::from_str("[language-server]\nhide-zero-reference-code-lenses = true\n").unwrap();
    assert!(hidden.language_server.hide_zero_reference_code_lenses);
    assert_eq!(hidden.to_filtered_value()["language-server"]["hide-zero-reference-code-lenses"], true);
    let section = &schema["$defs"]["LanguageServerConfiguration"];
    assert_eq!(section["description"], "Language server settings for this workspace.");
    assert_eq!(schema["properties"]["language-server"]["description"], section["description"]);
    let setting = &section["properties"]["hide-zero-reference-code-lenses"];
    assert_eq!(setting["type"], "boolean");
    assert_eq!(setting["default"], false);

    let mut server = ServerConfig::default();
    let enabled = serde_json::to_value(server_capabilities(&server)).unwrap();
    server.configuration = config;
    let disabled = serde_json::to_value(server_capabilities(&server)).unwrap();
    assert!(enabled["experimental"].is_null());
    assert!(disabled["experimental"].is_null());
    for (_, capability) in PROVIDERS {
        assert!(!enabled[*capability].is_null(), "{capability} must stay enabled by default");
        assert!(disabled[*capability].is_null(), "{capability} must be absent when disabled");
    }
}

async fn request(client: &mut LspClient, method: &str, params: Value) -> Value {
    let id = client.send_request(method, params).await;
    let response = client.await_response(id, 60).await;
    assert!(response.get("error").is_none(), "{method} failed: {response}");
    response.get("result").expect("expected a result").clone()
}

async fn start(
    files: &[(&str, &str)],
    folders: &[&str],
    config: ServerConfig,
) -> (TempDir, LspClient, Value, JoinHandle<()>) {
    let dir = tempfile::tempdir().unwrap();
    for (name, contents) in files {
        let path = dir.path().join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }
    let root = dir.path().canonicalize().unwrap();
    let folders: Vec<_> = folders.iter().map(|name| json!({ "uri": url(&root.join(name)), "name": name })).collect();
    let (client_stream, server_stream) = tokio::io::duplex(256 * 1024);
    let server = tokio::spawn(async move {
        let (reader, writer) = tokio::io::split(server_stream);
        let _ = crate::language_server::serve(reader, writer, config).await;
    });
    let mut client = LspClient::new(client_stream);
    let initialized = request(
        &mut client,
        "initialize",
        json!({ "workspaceFolders": folders, "capabilities": client_capabilities() }),
    )
    .await;
    assert!(initialized["capabilities"]["experimental"].is_null(), "private capability marker must not reach clients");
    client.send_notification("initialized", json!({})).await;
    (dir, client, initialized["capabilities"].clone(), server)
}

fn document_requests(uri: &str) -> Vec<(&'static str, Value)> {
    let document = json!({ "textDocument": { "uri": uri } });
    let position = json!({ "textDocument": { "uri": uri }, "position": { "line": 3, "character": 12 } });
    let range = json!({ "start": { "line": 0, "character": 0 }, "end": { "line": 5, "character": 0 } });
    vec![
        ("textDocument/hover", position.clone()),
        ("textDocument/definition", position.clone()),
        ("textDocument/typeDefinition", position.clone()),
        ("textDocument/documentHighlight", position.clone()),
        ("textDocument/implementation", position.clone()),
        ("textDocument/prepareTypeHierarchy", position.clone()),
        ("textDocument/completion", position.clone()),
        ("textDocument/prepareRename", position.clone()),
        (
            "textDocument/rename",
            json!({ "newName": "Speaker", "textDocument": document["textDocument"], "position": position["position"] }),
        ),
        (
            "textDocument/references",
            json!({ "context": { "includeDeclaration": true }, "textDocument": document["textDocument"], "position": position["position"] }),
        ),
        (
            "textDocument/formatting",
            json!({ "textDocument": document["textDocument"], "options": { "tabSize": 4, "insertSpaces": true } }),
        ),
        ("textDocument/foldingRange", document.clone()),
        ("textDocument/documentSymbol", document.clone()),
        (
            "textDocument/rangeFormatting",
            json!({ "textDocument": document["textDocument"], "range": range, "options": { "tabSize": 4, "insertSpaces": true } }),
        ),
        (
            "textDocument/selectionRange",
            json!({ "textDocument": document["textDocument"], "positions": [position["position"]] }),
        ),
        (
            "textDocument/codeAction",
            json!({ "textDocument": document["textDocument"], "range": range, "context": { "diagnostics": [] } }),
        ),
        (
            "textDocument/signatureHelp",
            json!({ "textDocument": document["textDocument"], "position": { "line": 4, "character": 8 } }),
        ),
        ("textDocument/inlayHint", json!({ "textDocument": document["textDocument"], "range": range })),
        ("textDocument/documentLink", document.clone()),
        ("textDocument/codeLens", document),
    ]
}

#[tokio::test]
async fn workspace_config_disables_advertisement_and_requests() {
    let (dir, mut client, capabilities, server) =
        start(&[("a.php", PHP), ("mago.toml", &disabled_config())], &[""], ServerConfig::default()).await;
    for (_, capability) in PROVIDERS {
        assert!(capabilities[*capability].is_null(), "{capability} must be absent");
    }
    let uri = url(&dir.path().canonicalize().unwrap().join("a.php"));
    for (method, params) in document_requests(&uri) {
        assert!(request(&mut client, method, params).await.is_null(), "{method} must be disabled");
    }
    assert!(request(&mut client, "workspace/symbol", json!({ "query": "Greeter" })).await.is_null());
    server.abort();
}

#[tokio::test]
async fn providers_follow_each_workspace_and_advertise_the_union() {
    let (dir, mut client, capabilities, server) = start(
        &[("off/a.php", PHP), ("off/mago.toml", &disabled_config()), ("on/a.php", PHP)],
        &["off", "on"],
        ServerConfig::default(),
    )
    .await;
    for (_, capability) in PROVIDERS {
        assert!(!capabilities[*capability].is_null(), "{capability} must be available in the enabled workspace");
    }
    for (folder, disabled) in [("off", true), ("on", false)] {
        let uri = url(&dir.path().canonicalize().unwrap().join(folder).join("a.php"));
        for (method, params) in document_requests(&uri) {
            assert_eq!(request(&mut client, method, params).await.is_null(), disabled, "{folder}: {method}");
        }
    }
    let symbols = request(&mut client, "workspace/symbol", json!({ "query": "Greeter" })).await;
    let symbols = symbols.as_array().unwrap();
    assert_eq!(symbols.len(), 1);
    assert_eq!(symbols[0]["location"]["uri"], url(&dir.path().canonicalize().unwrap().join("on/a.php")));
    server.abort();
}

#[tokio::test]
async fn rootless_session_uses_launch_settings_and_accepts_new_folders() {
    let config = ServerConfig { configuration: toml::from_str(&disabled_config()).unwrap(), ..ServerConfig::default() };
    let (dir, mut client, capabilities, server) =
        start(&[("a.php", PHP), ("mago.toml", "[language-server]\ndisable = []\n")], &[], config).await;
    for (_, capability) in PROVIDERS {
        assert!(capabilities[*capability].is_null(), "{capability} must use the launch settings");
    }
    client.send_notification("workspace/didChangeWorkspaceFolders", json!({
        "event": { "added": [{ "uri": url(&dir.path().canonicalize().unwrap()), "name": "added" }], "removed": [] }
    })).await;
    let symbols = request(&mut client, "workspace/symbol", json!({ "query": "Greeter" })).await;
    assert_eq!(symbols.as_array().unwrap().len(), 1);
    server.abort();
}

async fn change_folders(client: &mut LspClient, root: &std::path::Path, added: &[&str], removed: &[&str]) {
    let folders = |names: &[&str]| {
        names.iter().map(|name| json!({ "uri": url(&root.join(name)), "name": name })).collect::<Vec<_>>()
    };
    client
        .send_notification(
            "workspace/didChangeWorkspaceFolders",
            json!({ "event": { "added": folders(added), "removed": folders(removed) } }),
        )
        .await;
}

async fn provider_messages(client: &mut LspClient) -> Vec<Value> {
    request(client, "workspace/symbol", json!({ "query": "Greeter" })).await;
    client
        .take_pending_notifications()
        .into_iter()
        .filter(|message| message["method"] == "window/showMessage")
        .map(|message| message["params"].clone())
        .collect()
}

#[tokio::test]
async fn folder_provider_changes_warn_once_after_the_whole_batch() {
    for disabled in [false, true] {
        let off = "[language-server]\ndisable = ['hover']\n";
        let (initial, changed) = if disabled { (off, "") } else { ("", off) };
        let (dir, mut client, capabilities, server) = start(
            &[
                ("first/a.php", PHP),
                ("first/mago.toml", initial),
                ("same/a.php", PHP),
                ("same/mago.toml", initial),
                ("changed/a.php", PHP),
                ("changed/mago.toml", changed),
            ],
            &["first"],
            ServerConfig::default(),
        )
        .await;
        assert_eq!(capabilities["hoverProvider"].is_null(), disabled);
        let root = dir.path().canonicalize().unwrap();
        change_folders(&mut client, &root, &["same"], &["first"]).await;
        assert!(provider_messages(&mut client).await.is_empty(), "unchanged final capabilities need no restart");

        change_folders(&mut client, &root, &["changed"], &["same"]).await;
        let messages = provider_messages(&mut client).await;
        assert_eq!(messages.len(), 1, "{messages:?}");
        assert_eq!(messages[0]["type"], 2);
        assert_eq!(
            messages[0]["message"],
            "The enabled Mago features changed. Restart the Mago language server to update the editor."
        );
        let uri = url(&root.join("changed/a.php"));
        let hover = request(
            &mut client,
            "textDocument/hover",
            json!({ "textDocument": { "uri": uri }, "position": { "line": 3, "character": 12 } }),
        )
        .await;
        assert_eq!(hover.is_null(), !disabled, "requests still obey the current workspace");

        change_folders(&mut client, &root, &["same"], &["changed"]).await;
        change_folders(&mut client, &root, &["changed"], &["same"]).await;
        assert!(provider_messages(&mut client).await.is_empty(), "send at most one restart notice per session");
        server.abort();
    }
}

#[tokio::test]
async fn folder_provider_changes_ignore_features_unavailable_in_this_session() {
    let config = ServerConfig { formatter: false, ..ServerConfig::default() };
    let (dir, mut client, _, server) = start(
        &[
            ("first/a.php", PHP),
            ("first/mago.toml", "[language-server]\ndisable = ['formatting']\n"),
            ("added/a.php", PHP),
            ("added/mago.toml", ""),
        ],
        &["first"],
        config,
    )
    .await;
    change_folders(&mut client, &dir.path().canonicalize().unwrap(), &["added"], &[]).await;
    assert!(provider_messages(&mut client).await.is_empty());
    server.abort();
}
