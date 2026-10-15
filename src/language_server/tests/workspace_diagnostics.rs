//! Diagnostic ownership follows the most specific workspace root.

use serde_json::{Value, json};

use super::harness::{Harness, client_capabilities};
use crate::language_server::ServerConfig;

const IGNORE_FUNCTION: &str = "[analyzer]\nignore = ['non-existent-function']\n";

async fn publications(h: &mut Harness, uri: &str) -> Vec<Value> {
    h.diagnostics().await.into_iter().filter(|message| message["uri"] == uri).collect()
}

fn assert_publication(messages: &[Value], expected: &[&str]) {
    assert_eq!(messages.len(), 1, "{messages:?}");
    assert!(messages[0]["version"].is_null(), "{messages:?}");
    let mut codes: Vec<_> = messages[0]["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|diagnostic| diagnostic["code"].as_str())
        .filter(|code| code.starts_with("non-existent-"))
        .collect();
    codes.sort_unstable();
    assert_eq!(codes, expected, "{messages:?}");
    if expected.is_empty() {
        assert_eq!(messages[0]["diagnostics"], json!([]), "{messages:?}");
    }
}

#[tokio::test]
async fn overlapping_roots_publish_only_the_initial_owners_diagnostics() {
    let mut h = Harness::start_multi(&[
        ("parent", &[("anchor.php", "<?php")]),
        (
            "parent/nested",
            &[("a.php", "<?php\nmissing_function();\necho MISSING_CONSTANT;\n"), ("mago.toml", IGNORE_FUNCTION)],
        ),
    ])
    .await;
    let uri = h.url_in("parent/nested", "a.php");
    let initial = publications(&mut h, &uri).await;
    assert_publication(&initial, &["non-existent-constant"]);
    assert!(publications(&mut h, &uri).await.is_empty());
}

#[tokio::test]
async fn workspace_changes_restore_cached_diagnostics_or_clear_the_removed_owner() {
    for parent_ignores in [false, true] {
        let config = ServerConfig { linter: false, ..ServerConfig::default() };
        let mut h = Harness::start_with(
            &[("anchor.php", "<?php"), ("mago.toml", if parent_ignores { IGNORE_FUNCTION } else { "" })],
            config,
            client_capabilities(),
        )
        .await;
        let nested = h.workspace.join("nested");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(nested.join("a.php"), "<?php\nmissing_function();\n").unwrap();
        let uri = h.url("nested/a.php");
        h.watched("nested/a.php", 1).await;
        let initial = publications(&mut h, &uri).await;
        if parent_ignores {
            assert!(initial.is_empty(), "{initial:?}");
        } else {
            assert_publication(&initial, &["non-existent-function"]);
        }

        h.add_folder("nested", &[("mago.toml", if parent_ignores { "" } else { IGNORE_FUNCTION })]).await;
        let added = publications(&mut h, &uri).await;
        assert_publication(&added, if parent_ignores { &["non-existent-function"] } else { &[] });

        let folder_uri = h.url("nested");
        h.client
            .send_notification(
                "workspace/didChangeWorkspaceFolders",
                json!({ "event": { "added": [], "removed": [{ "uri": folder_uri, "name": "nested" }] } }),
            )
            .await;
        let removed = publications(&mut h, &uri).await;
        assert_publication(&removed, if parent_ignores { &[] } else { &["non-existent-function"] });
        assert!(publications(&mut h, &uri).await.is_empty());
    }
}
