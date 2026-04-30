//! Workspace lifecycle: open / change / close + on-disk
//! create/change/delete via `workspace/didChangeWatchedFiles`.

use serde_json::json;

use super::harness::Harness;

const HOLDER: &str = "<?php\nclass Holder {\n    public int $value = 0;\n}\n";
const CONSUMER: &str = "<?php\nfunction pull(Holder $h): int {\n    return $h->value;\n}\n";

async fn workspace_symbol_names(h: &mut Harness, query: &str) -> Vec<String> {
    let result = h.request("workspace/symbol", json!({ "query": query })).await;
    result.as_array().cloned().unwrap_or_default().iter().filter_map(|s| s["name"].as_str().map(String::from)).collect()
}

#[tokio::test]
async fn buffer_changes_reflect_immediately() {
    let mut h = Harness::start(&[("h.php", HOLDER)]).await;
    h.open("h.php", HOLDER).await;
    h.change("h.php", "<?php\nclass Vault {}\n", 2).await;

    let names = workspace_symbol_names(&mut h, "Vault").await;
    assert!(names.iter().any(|n| n.ends_with("Vault")), "got {names:?}");
}

#[tokio::test]
async fn close_restores_disk_content() {
    let mut h = Harness::start(&[("h.php", HOLDER)]).await;
    h.open("h.php", HOLDER).await;
    h.change("h.php", "<?php\nclass Vault {}\n", 2).await;

    let dropped = workspace_symbol_names(&mut h, "Holder").await;
    assert!(!dropped.iter().any(|n| n.ends_with("Holder")), "expected Holder gone, got {dropped:?}");

    h.close("h.php").await;
    let restored = workspace_symbol_names(&mut h, "Holder").await;
    assert!(restored.iter().any(|n| n.ends_with("Holder")), "got {restored:?}");
}

#[tokio::test]
async fn external_create_picked_up() {
    let mut h = Harness::start(&[("seed.php", HOLDER)]).await;
    std::fs::write(h.workspace.join("late.php"), "<?php\nclass Late { public string $name = ''; }\n").unwrap();
    h.watched("late.php", 1).await;

    let names = workspace_symbol_names(&mut h, "Late").await;
    assert!(names.iter().any(|n| n.ends_with("Late")), "got {names:?}");
}

#[tokio::test]
async fn external_delete_invalidates() {
    let mut h = Harness::start(&[("h.php", HOLDER), ("c.php", CONSUMER)]).await;
    h.open("c.php", CONSUMER).await;
    let resolved = h.at("textDocument/definition", "c.php", 1, 16).await;
    assert!(!resolved.is_null());

    std::fs::remove_file(h.workspace.join("h.php")).unwrap();
    h.watched("h.php", 3).await;

    let result = h.at("textDocument/definition", "c.php", 1, 16).await;
    assert!(result.is_null(), "expected null, got {result}");
}

#[tokio::test]
async fn external_change_re_reads() {
    let mut h = Harness::start(&[("h.php", HOLDER)]).await;
    let updated = "<?php\nclass Alpha {}\nclass Beta {}\n";
    std::fs::write(h.workspace.join("h.php"), updated).unwrap();
    h.watched("h.php", 2).await;

    let names = workspace_symbol_names(&mut h, "").await;
    assert!(names.iter().any(|n| n.ends_with("Alpha")) && names.iter().any(|n| n.ends_with("Beta")), "got {names:?}");
}

#[tokio::test]
async fn change_without_open_is_noop() {
    let mut h = Harness::start(&[("a.php", HOLDER)]).await;
    h.change("a.php", "<?php\nclass Other {}\n", 1).await;

    let result = h.for_doc("textDocument/foldingRange", "a.php").await;
    assert!(!result.is_null());
}
