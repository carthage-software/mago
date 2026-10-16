//! File watcher negotiation and workspace registration changes.

use std::collections::BTreeMap;
use std::path::Path;

use globset::GlobBuilder;
use globset::GlobSet;
use globset::GlobSetBuilder;
use serde_json::Value;
use serde_json::json;

use super::harness::{Harness, url};
use crate::config::LoadOptions;
use crate::language_server::ServerConfig;

async fn watchers_matching(h: &mut Harness, path: &Path, watched: bool) -> GlobSet {
    let original = h.workspace.canonicalize().unwrap().join("a.php");
    for attempt in 0..10 {
        h.for_doc("textDocument/foldingRange", "a.php").await;
        let mut registrations = BTreeMap::new();
        for request in h.client.server_requests() {
            match request["method"].as_str() {
                Some("client/registerCapability") => {
                    for registration in request["params"]["registrations"].as_array().unwrap() {
                        assert_eq!(registration["method"], "workspace/didChangeWatchedFiles");
                        assert!(registrations.insert(registration["id"].as_str().unwrap(), registration).is_none());
                    }
                }
                Some("client/unregisterCapability") => {
                    for registration in request["params"]["unregisterations"].as_array().unwrap() {
                        assert!(registrations.remove(registration["id"].as_str().unwrap()).is_some());
                    }
                }
                _ => continue,
            }
            assert!(
                registered_watchers(&registrations).is_match(&original),
                "the original workspace lost watcher coverage after {request}"
            );
        }
        let matcher = registered_watchers(&registrations);
        let root_watched = matcher.is_match(&original);
        if (root_watched && matcher.is_match(path.to_str().unwrap()) == watched) || attempt == 9 {
            assert!(root_watched, "the original workspace must remain watched");
            assert_eq!(matcher.is_match(path.to_str().unwrap()), watched, "watcher coverage for {}", path.display());
            return matcher;
        }
    }
    unreachable!()
}

fn registered_watchers(registrations: &BTreeMap<&str, &Value>) -> GlobSet {
    let mut builder = GlobSetBuilder::new();
    for registration in registrations.values() {
        for watcher in registration["registerOptions"]["watchers"].as_array().unwrap() {
            let pattern = watcher["globPattern"].as_str().expect("relative patterns require client support");
            builder.add(GlobBuilder::new(pattern).literal_separator(true).build().unwrap());
        }
    }
    builder.build().unwrap()
}

async fn watcher_request(h: &mut Harness, method: &str) -> Value {
    loop {
        let message = h.client.read_message(10).await;
        if message.get("id").is_some() {
            assert_eq!(message["method"], method, "{message}");
            return message;
        }
    }
}

async fn reject_watcher_request(h: &mut Harness, request: &Value) {
    h.client.respond_error(request["id"].clone(), json!({ "code": -32603, "message": "rejected by test" })).await;
    loop {
        let message = h.client.read_message(10).await;
        assert!(message.get("id").is_none(), "unexpected request after rejection: {message}");
        if message["method"] == "window/logMessage"
            && message["params"]["message"].as_str().is_some_and(|text| text.contains("update file watchers:"))
        {
            assert_eq!(message["params"]["type"], 1);
            return;
        }
    }
}

async fn start_watchers() -> (Harness, Value) {
    let mut config = ServerConfig::default();
    config.configuration.source.paths = vec!["a.php".into()];
    let mut h = Harness::start_with(
        &[("a.php", "<?php\n")],
        config,
        json!({ "workspace": { "didChangeWatchedFiles": { "dynamicRegistration": true } } }),
    )
    .await;
    let request = watcher_request(&mut h, "client/registerCapability").await;
    h.client.respond(request["id"].clone(), Value::Null).await;
    (h, request["params"]["registrations"][0]["id"].clone())
}

#[cfg(unix)]
async fn accept_watchers(h: &mut Harness, replacement: bool) -> GlobSet {
    let request = watcher_request(h, "client/registerCapability").await;
    let registration = &request["params"]["registrations"][0];
    let matcher = registered_watchers(&BTreeMap::from([(registration["id"].as_str().unwrap(), registration)]));
    h.client.respond(request["id"].clone(), Value::Null).await;
    if replacement {
        let retired = watcher_request(h, "client/unregisterCapability").await;
        h.client.respond(retired["id"].clone(), Value::Null).await;
    }
    matcher
}

#[cfg(unix)]
#[tokio::test]
async fn watchers_discover_targets_missing_at_startup() {
    let workspace = tempfile::tempdir().unwrap();
    let external = tempfile::tempdir().unwrap();
    let root = workspace.path().canonicalize().unwrap();
    let target = external.path().canonicalize().unwrap().join("missing");
    let link = root.join("vendor/package");
    std::fs::create_dir(root.join("vendor")).unwrap();
    std::os::unix::fs::symlink(&target, &link).unwrap();
    let source = "<?php function target(): LaterTarget { return new LaterTarget(); }";
    std::fs::write(root.join("a.php"), source).unwrap();
    std::fs::write(root.join("mago.toml"), "[source]\npaths = ['a.php']\nincludes = ['vendor']\n").unwrap();
    let config = ServerConfig {
        linter: false,
        launch: LoadOptions { workspace: Some(root.clone()), ..LoadOptions::default() },
        ..ServerConfig::default()
    };
    let mut h = Harness::start_with(
        &[],
        config,
        json!({"workspace": {"didChangeWatchedFiles": {"dynamicRegistration": true}}}),
    )
    .await;
    let caller_uri = url(&root.join("a.php"));
    let initial = h.client.await_notification("textDocument/publishDiagnostics").await;
    assert_eq!(initial["params"]["uri"], caller_uri);
    assert!(
        initial["params"]["diagnostics"].as_array().unwrap().iter().any(|issue| issue["code"] == "non-existent-class"),
        "{initial}"
    );
    let watched = accept_watchers(&mut h, false).await;
    assert!(watched.is_match(&target));
    assert!(watched.is_match(target.join("LaterTarget.php")));

    std::fs::create_dir(&target).unwrap();
    std::fs::write(target.join("LaterTarget.php"), "<?php class LaterTarget {}").unwrap();
    h.client
        .send_notification("workspace/didChangeWatchedFiles", json!({"changes": [{"uri": url(&target), "type": 1}]}))
        .await;
    let publications = h.diagnostics().await;
    assert!(
        publications
            .iter()
            .any(|publication| publication["uri"] == caller_uri && publication["diagnostics"] == json!([])),
        "{publications:?}"
    );
    let definition =
        h.at_uri("textDocument/definition", &caller_uri, 0, source.find("LaterTarget").unwrap() as u32).await;
    assert_eq!(definition["uri"], url(&link.join("LaterTarget.php")));
}

#[cfg(unix)]
#[tokio::test]
async fn watchers_follow_empty_directory_symlinks_and_their_current_targets() {
    let workspace = tempfile::tempdir().unwrap();
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    let root = workspace.path().canonicalize().unwrap();
    let first = first.path().canonicalize().unwrap();
    let second = second.path().canonicalize().unwrap();
    let link = root.join("vendor/package");
    std::fs::create_dir_all(root.join("vendor")).unwrap();
    std::os::unix::fs::symlink(&first, &link).unwrap();
    std::fs::write(root.join("a.php"), "<?php").unwrap();
    std::fs::write(root.join("mago.toml"), "[source]\npaths = ['a.php']\nincludes = ['vendor']\n").unwrap();
    let config = ServerConfig {
        linter: false,
        launch: LoadOptions { workspace: Some(root.clone()), ..LoadOptions::default() },
        ..ServerConfig::default()
    };
    let mut h = Harness::start_with(
        &[],
        config,
        json!({"workspace": {"didChangeWatchedFiles": {"dynamicRegistration": true}}}),
    )
    .await;
    let watched = accept_watchers(&mut h, false).await;
    assert!(watched.is_match(root.join("a.php")));
    assert!(watched.is_match(&first));
    assert!(watched.is_match(first.join("New.php")));
    assert!(!watched.is_match(&second));

    std::fs::write(first.join("New.php"), "<?php class Appeared {}").unwrap();
    h.client
        .send_notification(
            "workspace/didChangeWatchedFiles",
            json!({"changes": [{"uri": url(&first.join("New.php")), "type": 1}]}),
        )
        .await;
    let outline =
        h.request("textDocument/documentSymbol", json!({"textDocument": {"uri": url(&link.join("New.php"))}})).await;
    assert_eq!(outline[0]["name"], "Appeared");

    std::fs::remove_file(&link).unwrap();
    std::os::unix::fs::symlink(&second, &link).unwrap();
    h.client
        .send_notification("workspace/didChangeWatchedFiles", json!({"changes": [{"uri": url(&link), "type": 2}]}))
        .await;
    let watched = accept_watchers(&mut h, true).await;
    assert!(watched.is_match(root.join("a.php")));
    assert!(watched.is_match(&second));
    assert!(watched.is_match(second.join("New.php")));
    assert!(!watched.is_match(&first));

    std::fs::remove_file(&link).unwrap();
    h.client
        .send_notification("workspace/didChangeWatchedFiles", json!({"changes": [{"uri": url(&link), "type": 3}]}))
        .await;
    let watched = accept_watchers(&mut h, true).await;
    assert!(watched.is_match(root.join("a.php")));
    assert!(!watched.is_match(&first));
    assert!(!watched.is_match(&second));

    std::os::unix::fs::symlink(&first, &link).unwrap();
    h.client
        .send_notification("workspace/didChangeWatchedFiles", json!({"changes": [{"uri": url(&link), "type": 1}]}))
        .await;
    let watched = accept_watchers(&mut h, true).await;
    assert!(watched.is_match(root.join("a.php")));
    assert!(watched.is_match(first.join("New.php")));
    assert!(!watched.is_match(&second));
}

#[tokio::test]
async fn rejected_watcher_replacement_keeps_the_current_registration() {
    let (mut h, original) = start_watchers().await;
    h.add_folder("first", &[("a.php", "<?php\n")]).await;
    let rejected = watcher_request(&mut h, "client/registerCapability").await;
    let next = &rejected["params"]["registrations"][0]["id"];
    assert_ne!(next, &original);
    reject_watcher_request(&mut h, &rejected).await;

    h.add_folder("second", &[("a.php", "<?php\n")]).await;
    let accepted = watcher_request(&mut h, "client/registerCapability").await;
    assert_eq!(&accepted["params"]["registrations"][0]["id"], next);
    h.client.respond(accepted["id"].clone(), Value::Null).await;
    let retired = watcher_request(&mut h, "client/unregisterCapability").await;
    assert_eq!(retired["params"]["unregisterations"][0]["id"], original);
    h.client.respond(retired["id"].clone(), Value::Null).await;
}

#[cfg(unix)]
#[tokio::test]
async fn shared_empty_symlink_targets_stay_watched_after_either_link_is_removed() {
    let external = tempfile::tempdir().unwrap();
    let target = external.path().canonicalize().unwrap();
    let mut config = ServerConfig::default();
    config.configuration.source.paths = vec!["a.php".into()];
    config.configuration.source.includes = vec!["vendor".into()];
    let mut h = Harness::start_with(
        &[("a.php", "<?php")],
        config,
        json!({"workspace": {"didChangeWatchedFiles": {"dynamicRegistration": true}}}),
    )
    .await;
    for removed in ["first", "second"] {
        let vendor = h.workspace.join("vendor");
        std::fs::create_dir_all(&vendor).unwrap();
        for name in ["first", "second"] {
            std::os::unix::fs::symlink(&target, vendor.join(name)).unwrap();
        }
        h.watched("vendor", 1).await;
        watchers_matching(&mut h, &target.join("Future.php"), true).await;

        std::fs::remove_file(vendor.join(removed)).unwrap();
        h.watched(&format!("vendor/{removed}"), 3).await;
        watchers_matching(&mut h, &target.join("Future.php"), true).await;

        std::fs::remove_dir_all(&vendor).unwrap();
        h.watched("vendor", 3).await;
        watchers_matching(&mut h, &target.join("Future.php"), false).await;
    }
}

#[tokio::test]
async fn failed_watcher_cleanup_retries_before_registering_more_patterns() {
    let (mut h, original) = start_watchers().await;
    h.add_folder("first", &[("a.php", "<?php\n")]).await;
    let accepted = watcher_request(&mut h, "client/registerCapability").await;
    let current = &accepted["params"]["registrations"][0]["id"];
    assert_ne!(current, &original);
    h.client.respond(accepted["id"].clone(), Value::Null).await;
    let retired = watcher_request(&mut h, "client/unregisterCapability").await;
    assert_eq!(retired["params"]["unregisterations"][0]["id"], original);
    reject_watcher_request(&mut h, &retired).await;

    for name in ["second", "third"] {
        h.add_folder(name, &[("a.php", "<?php\n")]).await;
        let retry = watcher_request(&mut h, "client/unregisterCapability").await;
        assert_eq!(retry["params"]["unregisterations"][0]["id"], original);
        if name == "second" {
            reject_watcher_request(&mut h, &retry).await;
        } else {
            h.client.respond(retry["id"].clone(), Value::Null).await;
        }
    }
    let replacement = watcher_request(&mut h, "client/registerCapability").await;
    assert_eq!(replacement["params"]["registrations"][0]["id"], original);
    let registration = &replacement["params"]["registrations"][0];
    let matcher = registered_watchers(&BTreeMap::from([(original.as_str().unwrap(), registration)]));
    for name in ["a.php", "first/a.php", "second/a.php", "third/a.php"] {
        assert!(matcher.is_match(h.workspace.canonicalize().unwrap().join(name)), "missing watcher for {name}");
    }
    h.client.respond(replacement["id"].clone(), Value::Null).await;
    let retired = watcher_request(&mut h, "client/unregisterCapability").await;
    assert_eq!(&retired["params"]["unregisterations"][0]["id"], current);
    h.client.respond(retired["id"].clone(), Value::Null).await;
}

#[tokio::test]
async fn watchers_require_client_registration_support() {
    for supported in [false, true] {
        let capabilities = json!({ "workspace": { "didChangeWatchedFiles": { "dynamicRegistration": supported } } });
        let mut h = Harness::start_with(&[("a.php", "<?php\n")], ServerConfig::default(), capabilities).await;
        h.for_doc("textDocument/foldingRange", "a.php").await;
        if supported {
            // Await the registration task with a protocol barrier, not a fixed delay.
            for _ in 0..10 {
                if !h.client.server_requests().is_empty() {
                    break;
                }
                h.for_doc("textDocument/foldingRange", "a.php").await;
            }
        }
        let registrations: Vec<_> = h
            .client
            .server_requests()
            .iter()
            .filter(|message| message["method"] == "client/registerCapability")
            .collect();
        assert_eq!(registrations.len(), usize::from(supported));
        if supported {
            assert_eq!(registrations[0]["params"]["registrations"][0]["method"], "workspace/didChangeWatchedFiles");
        }
    }
}

#[tokio::test]
async fn watchers_cover_configured_extensions_and_exact_files() {
    let files = [
        ("a.php", "<?php class PhpBefore {}"),
        ("view.phtml", "<?php class TemplateBefore {}"),
        ("entry", "<?php class EntryBefore {}"),
        ("ignored.phtml", "<?php class Ignored {}"),
        ("dependency.inc", "<?php class DependencyBefore {}"),
    ];
    for custom in [false, true] {
        let mut config = ServerConfig::default();
        if custom {
            config.configuration.source.paths =
                vec!["a.php".into(), "view.phtml".into(), "entry".into(), "future".into(), "ignored.phtml".into()];
            config.configuration.source.extensions = vec!["php".into(), "phtml".into(), "inc".into()];
            config.configuration.source.includes = vec!["dependency.inc".into()];
            config.configuration.source.excludes = vec!["ignored.phtml".into()];
        }
        let mut h = Harness::start_with(
            &files,
            config,
            json!({ "workspace": { "didChangeWatchedFiles": { "dynamicRegistration": true } } }),
        )
        .await;
        let workspace = h.workspace.canonicalize().unwrap();
        let matcher = watchers_matching(&mut h, &workspace.join("a.php"), true).await;
        assert_eq!(matcher.is_match(&workspace), !custom);
        assert!(matcher.is_match(workspace.join("view.phtml").to_str().unwrap()));
        assert!(matcher.is_match(workspace.join("entry").to_str().unwrap()));
        assert!(matcher.is_match(workspace.join("future").to_str().unwrap()));
        assert!(matcher.is_match(workspace.join("dependency.inc").to_str().unwrap()));
        // Broad directory globs also cover unknown extensions; source membership filters them.
        assert_eq!(matcher.is_match(workspace.join("unrelated.txt").to_str().unwrap()), !custom);
        for (name, class) in [
            ("a.php", "PhpAfter"),
            ("view.phtml", "TemplateAfter"),
            ("entry", "EntryAfter"),
            ("dependency.inc", "DependencyAfter"),
        ] {
            if !custom && name != "a.php" {
                continue;
            }
            std::fs::write(workspace.join(name), format!("<?php class {class} {{}}")).unwrap();
            h.watched(name, 2).await;
            assert_eq!(h.for_doc("textDocument/documentSymbol", name).await[0]["name"], class);
        }
        if custom {
            assert!(!matcher.is_match(workspace.join("a.php/child.php")));
            assert!(matcher.is_match(workspace.join("future/child.txt")));
            assert!(!matcher.is_match(workspace.join("ignored.phtml").to_str().unwrap()));
            std::fs::write(workspace.join("future"), "<?php class Created {}").unwrap();
            h.watched("future", 1).await;
            assert_eq!(h.for_doc("textDocument/documentSymbol", "future").await[0]["name"], "Created");
            h.watched("ignored.phtml", 2).await;
            assert_eq!(h.for_doc("textDocument/documentSymbol", "ignored.phtml").await, Value::Null);
        }
    }
}

#[tokio::test]
async fn watchers_follow_added_and_removed_workspace_settings() {
    let mut config = ServerConfig::default();
    config.configuration.source.paths = vec!["a.php".into()];
    let mut h = Harness::start_with(
        &[("a.php", "<?php class Root {}")],
        config,
        json!({ "workspace": { "didChangeWatchedFiles": { "dynamicRegistration": true } } }),
    )
    .await;
    let root = h.workspace.canonicalize().unwrap();
    let template = root.join("templates/view.phtml");
    let matcher = watchers_matching(&mut h, &root.join("a.php"), true).await;
    assert!(!matcher.is_match(template.to_str().unwrap()));

    h.add_folder(
        "templates",
        &[("mago.toml", "[source]\nextensions = ['phtml']\n"), ("view.phtml", "<?php class Added {}")],
    )
    .await;
    let matcher = watchers_matching(&mut h, &template, true).await;
    assert!(matcher.is_match(root.join("a.php").to_str().unwrap()));
    assert!(!matcher.is_match(root.join("outside.phtml").to_str().unwrap()));
    std::fs::write(&template, "<?php class Changed {}").unwrap();
    h.watched("templates/view.phtml", 2).await;
    assert_eq!(h.for_doc("textDocument/documentSymbol", "templates/view.phtml").await[0]["name"], "Changed");

    h.client
        .send_notification(
            "workspace/didChangeWorkspaceFolders",
            json!({
                "event": { "added": [], "removed": [{ "uri": url(&root.join("templates")), "name": "templates" }] }
            }),
        )
        .await;
    let matcher = watchers_matching(&mut h, &template, false).await;
    assert!(matcher.is_match(root.join("a.php").to_str().unwrap()));
}
