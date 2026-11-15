//! Configuration, Git ignore, and `restart-on` changes reload a workspace from scratch.

use std::time::Duration;

use serde_json::Value;
use serde_json::json;

use super::harness::{Harness, client_capabilities, url};
use crate::config::LoadOptions;
use crate::language_server::ServerConfig;

const SOURCE: &str = "<?php function shown(): void {}\n";
const BROKEN: &str = "<?php\nfunction broken( {}\n";

async fn restarted(h: &mut Harness) -> String {
    loop {
        let message = h.client.await_notification("window/logMessage").await;
        let text = message["params"]["message"].as_str().unwrap();
        if text.starts_with("mago-server: restarted") {
            return text.to_owned();
        }
    }
}

/// Log messages sent before the server handles a barrier sent now.
async fn logs(h: &mut Harness) -> Vec<String> {
    const MARKER: &str = "mago/testBarrier";
    h.client.send_notification(MARKER, json!({ "type": 4, "message": MARKER })).await;
    let mut logs = Vec::new();
    loop {
        let message = h.client.await_notification("window/logMessage").await;
        let text = message["params"]["message"].as_str().unwrap().to_owned();
        if text == MARKER {
            return logs;
        }
        logs.push(text);
    }
}

async fn changed(h: &mut Harness, name: &str, contents: &str, kind: i32) {
    std::fs::write(h.workspace.join(name), contents).unwrap();
    h.watched(name, kind).await;
}

async fn hover(h: &mut Harness) -> Value {
    h.at("textDocument/hover", "a.php", 0, 15).await
}

async fn files_with_diagnostics(h: &mut Harness) -> Vec<String> {
    let mut files: Vec<_> = h
        .diagnostics()
        .await
        .into_iter()
        .filter(|publication| !publication["diagnostics"].as_array().unwrap().is_empty())
        .map(|publication| publication["uri"].as_str().unwrap().to_owned())
        .collect();
    files.sort_unstable();
    files.dedup();
    files
}

async fn symbols(h: &mut Harness, query: &str) -> Vec<String> {
    let symbols = h.request("workspace/symbol", json!({ "query": query })).await;
    symbols.as_array().unwrap().iter().map(|symbol| symbol["name"].as_str().unwrap().to_owned()).collect()
}

#[tokio::test]
async fn configuration_layers_reload_the_workspace() {
    let mut h =
        Harness::start(&[("mago.toml", "extends = \"base.toml\"\n"), ("base.toml", ""), ("a.php", SOURCE)]).await;
    assert!(!hover(&mut h).await.is_null());

    changed(&mut h, "base.toml", "[language-server]\ndisable = [\"hover\"]\n", 2).await;
    assert!(
        restarted(&mut h)
            .await
            .ends_with(&format!("after {} changed", h.workspace.canonicalize().unwrap().join("base.toml").display()))
    );
    assert!(hover(&mut h).await.is_null());

    changed(&mut h, "mago.toml", "", 2).await;
    restarted(&mut h).await;
    assert!(!hover(&mut h).await.is_null());
}

#[tokio::test]
async fn new_project_configurations_replace_the_inherited_one() {
    let mut h = Harness::start(&[("a.php", SOURCE)]).await;
    assert!(!hover(&mut h).await.is_null());

    changed(&mut h, "mago.dist.toml", "[language-server]\ndisable = [\"hover\"]\n", 1).await;
    restarted(&mut h).await;
    assert!(hover(&mut h).await.is_null());
}

#[tokio::test]
async fn launch_configuration_files_are_read_again() {
    let external = tempfile::tempdir().unwrap();
    let file = external.path().canonicalize().unwrap().join("launch.toml");
    std::fs::write(&file, "").unwrap();
    let launch = LoadOptions { file: Some(file.clone()), ..LoadOptions::default() };
    let config = ServerConfig { configuration: launch.load().unwrap(), launch, ..ServerConfig::default() };
    let mut h = Harness::start_with(&[("a.php", SOURCE)], config, client_capabilities()).await;
    assert!(!hover(&mut h).await.is_null());

    std::fs::write(&file, "[language-server]\ndisable = [\"hover\"]\n").unwrap();
    h.client
        .send_notification("workspace/didChangeWatchedFiles", json!({ "changes": [{ "uri": url(&file), "type": 2 }] }))
        .await;
    restarted(&mut h).await;
    assert!(hover(&mut h).await.is_null());
}

#[tokio::test]
async fn git_ignore_changes_reload_which_sources_are_dependencies() {
    let mut h = Harness::start(&[
        (".git/HEAD", "ref: refs/heads/main\n"),
        (".gitignore", "/ignored/\n"),
        ("ignored/Broken.php", BROKEN),
        ("a.php", SOURCE),
    ])
    .await;
    let broken = h.url("ignored/Broken.php");
    assert!(!files_with_diagnostics(&mut h).await.contains(&broken));

    changed(&mut h, ".gitignore", "", 2).await;
    restarted(&mut h).await;
    assert!(files_with_diagnostics(&mut h).await.contains(&broken));
}

#[tokio::test]
async fn restart_on_paths_reload_the_workspace_once_changes_settle() {
    let mut h = Harness::start(&[
        ("mago.toml", "[language-server]\nrestart-on = [\"composer.lock\", \"vendor/\"]\n"),
        ("a.php", SOURCE),
    ])
    .await;

    for _ in 0..5 {
        changed(&mut h, "composer.lock", "{}", 2).await;
    }
    std::fs::create_dir_all(h.workspace.join("vendor/acme")).unwrap();
    changed(&mut h, "vendor/acme/Package.php", "<?php class Package {}", 1).await;
    restarted(&mut h).await;
    assert_eq!(symbols(&mut h, "Package").await, ["Package"]);

    changed(&mut h, "a.php", SOURCE, 2).await;
    tokio::time::sleep(Duration::from_millis(300)).await;
    let logs = logs(&mut h).await;
    assert!(logs.iter().all(|text| !text.contains("restarted")), "{logs:?}");
}

#[tokio::test]
async fn restarts_keep_open_buffers() {
    let mut h = Harness::start(&[("mago.toml", ""), ("a.php", "<?php class Disk {}\n")]).await;
    h.open("a.php", "<?php class Buffer {}\n").await;
    changed(&mut h, "mago.toml", "[linter]\n", 2).await;
    restarted(&mut h).await;
    assert_eq!(symbols(&mut h, "Buffer").await, ["Buffer"]);
    assert!(symbols(&mut h, "Disk").await.is_empty());
}

#[tokio::test]
async fn failed_restarts_keep_the_workspace_and_apply_the_changes_they_held() {
    let mut h = Harness::start(&[("mago.toml", "[language-server]\ndisable = [\"hover\"]\n"), ("a.php", SOURCE)]).await;
    changed(&mut h, "mago.toml", "[language-server]\nunknown = true\n", 2).await;
    changed(&mut h, "b.php", "<?php class Later {}\n", 1).await;
    let message = h.client.await_notification("window/showMessage").await;
    assert_eq!(message["params"]["type"], 1);
    assert!(message["params"]["message"].as_str().unwrap().contains("cannot restart"), "{message}");
    h.diagnostics().await;
    assert!(hover(&mut h).await.is_null());
    assert_eq!(symbols(&mut h, "Later").await, ["Later"]);
}

#[tokio::test]
async fn buffers_opened_while_excluded_join_the_workspace_that_admits_them() {
    let mut h = Harness::start(&[
        ("mago.toml", "[source]\nexcludes = [\"excluded.php\"]\n"),
        ("excluded.php", "<?php class OnDisk {}\n"),
    ])
    .await;
    h.open("excluded.php", "<?php class Opened {}\n").await;
    assert!(symbols(&mut h, "Opened").await.is_empty());

    changed(&mut h, "mago.toml", "", 2).await;
    restarted(&mut h).await;
    assert_eq!(symbols(&mut h, "Opened").await, ["Opened"]);
    assert!(symbols(&mut h, "OnDisk").await.is_empty());

    h.change("excluded.php", "<?php class Edited {}\n", 2).await;
    assert_eq!(symbols(&mut h, "Edited").await, ["Edited"]);
}
