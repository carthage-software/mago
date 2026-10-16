use std::fs;
use std::process::Command;

use tempfile::tempdir;

fn analyze_with_config(analyzer_config: &str) -> String {
    let workspace = tempdir().unwrap();
    fs::write(workspace.path().join("mago.toml"), format!("[analyzer]\n{analyzer_config}\n")).unwrap();
    fs::write(
        workspace.path().join("throws.php"),
        "<?php\n/** @throws LogicException */\nfunction documented_throw_not_emitted(): void {}\n",
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_mago"))
        .args(["analyze", "--reporting-format", "json", "throws.php"])
        .current_dir(workspace.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    serde_json::from_str::<serde_json::Value>(&stdout).unwrap_or_else(|error| {
        panic!("invalid analyzer JSON output: {error}; stderr: {}", String::from_utf8_lossy(&output.stderr))
    });
    stdout
}

#[test]
fn too_wide_throw_check_is_off_by_default_when_check_throws_is_enabled() {
    let output = analyze_with_config("check-throws = true");

    assert!(!output.contains("too-wide-throw-type"), "{output}");
}

#[test]
fn too_wide_throw_check_can_be_enabled_without_check_throws() {
    let output = analyze_with_config("find-overly-wide-throw-types = true");

    assert!(output.contains("too-wide-throw-type"), "{output}");
}
