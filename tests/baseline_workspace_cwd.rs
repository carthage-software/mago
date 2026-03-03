//! Integration test: the baseline file is found when running with --config and --workspace
//! from a current working directory that is not the workspace.
//!
//! Previously, the baseline path from config (e.g. `baseline = "mago-analysis-baseline.toml"`)
//! was resolved relative to the process CWD, so running from e.g. `/` with
//! `--config=/opt/project/mago.toml --workspace=/opt/project` failed to find the baseline.

use std::path::Path;
use std::process::Command;

fn mago_bin() -> std::path::PathBuf {
    let path = std::env::var("CARGO_BIN_EXE_mago")
        .ok()
        .or_else(|| option_env!("CARGO_BIN_EXE_mago").map(String::from))
        .unwrap_or_else(|| "mago".to_string());
    std::path::PathBuf::from(path)
}

/// Strip log lines so the remainder is JSON; return (issue_count, stderr_contains_filtered).
fn parse_analyze_output(stdout: &[u8], stderr: &[u8]) -> (usize, bool) {
    let stdout_str = std::str::from_utf8(stdout).unwrap_or("");
    let stderr_str = std::str::from_utf8(stderr).unwrap_or("");
    let start = stdout_str.find('{').unwrap_or(0);
    let json_str = stdout_str[start..].trim();
    let count = if json_str.is_empty() {
        0
    } else {
        let json: serde_json::Value = serde_json::from_str(json_str).unwrap_or(serde_json::Value::Null);
        json.get("issues").and_then(|a| a.as_array()).map(|a| a.len()).unwrap_or(0)
    };
    let filtered = stderr_str.contains("Filtered out");
    (count, filtered)
}

#[test]
fn test_analyze_baseline_found_when_run_from_different_cwd() {
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path();

    std::fs::create_dir(workspace.join("src")).unwrap();

    let mago_toml = r#"
php-version = "8.4"
[source]
paths = ["src"]
[analyzer]
baseline = "mago-analysis-baseline.toml"
"#;
    std::fs::write(workspace.join("mago.toml"), mago_toml).unwrap();

    // PHP that triggers one analyzer issue (undefined function)
    let php = "<?php\ndoesNotExist();\n";
    std::fs::write(workspace.join("src").join("baseline.php"), php).unwrap();

    // Minimal loose baseline that suppresses that one issue
    let baseline_toml = r#"
variant = "loose"
[[issues]]
file = "src/baseline.php"
code = "non-existent-function"
message = "Function `doesNotExist` could not be found."
count = 1
"#;
    std::fs::write(workspace.join("mago-analysis-baseline.toml"), baseline_toml).unwrap();

    let config_path = workspace.join("mago.toml");
    let workspace_str = workspace.to_str().unwrap();
    let config_str = config_path.to_str().unwrap();

    // Run from workspace: baseline should be found, 0 issues reported
    let out_from_workspace = Command::new(mago_bin())
        .current_dir(workspace)
        .args([
            "--config",
            config_str,
            "--workspace",
            workspace_str,
            "analyze",
            "src/baseline.php",
            "--reporting-format",
            "json",
        ])
        .output()
        .unwrap();
    let (count_ws, filtered_ws) = parse_analyze_output(&out_from_workspace.stdout, &out_from_workspace.stderr);
    assert!(
        out_from_workspace.status.success(),
        "analyze from workspace should succeed; stderr: {}",
        String::from_utf8_lossy(&out_from_workspace.stderr)
    );
    assert_eq!(count_ws, 0, "from workspace: baseline should filter the issue");
    assert!(filtered_ws, "from workspace: stderr should mention Filtered out");

    // Run from a different CWD (parent of workspace): baseline should still be found
    let other_cwd = temp_dir.path().parent().unwrap_or(Path::new("/tmp"));
    let out_from_other = Command::new(mago_bin())
        .current_dir(other_cwd)
        .args([
            "--config",
            config_str,
            "--workspace",
            workspace_str,
            "analyze",
            "src/baseline.php",
            "--reporting-format",
            "json",
        ])
        .output()
        .unwrap();
    let (count_other, filtered_other) = parse_analyze_output(&out_from_other.stdout, &out_from_other.stderr);
    assert!(
        out_from_other.status.success(),
        "analyze from different CWD should succeed; stderr: {}",
        String::from_utf8_lossy(&out_from_other.stderr)
    );
    assert_eq!(
        count_other, 0,
        "from different CWD: baseline should still filter the issue (got {} issues)",
        count_other
    );
    assert!(
        filtered_other,
        "from different CWD: stderr should mention Filtered out; stderr: {}",
        String::from_utf8_lossy(&out_from_other.stderr)
    );
}
