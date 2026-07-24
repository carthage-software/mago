//! Integration tests for the `--stats` flag on `mago lint`.
//!
//! `--stats` is a shorthand for `--reporting-format code-count`: it reports a
//! summary of issue counts per code, ordered from highest to lowest, instead
//! of individual issues. These tests run the mago binary against a temporary
//! workspace and verify the output shape, the ordering, the equivalence with
//! `--reporting-format code-count`, the exit-code semantics, and the clap
//! conflicts.

use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

fn mago_bin() -> PathBuf {
    // Prefer runtime env (set when cargo runs the test), then compile-time (set when cargo builds the test).
    let path = std::env::var("CARGO_BIN_EXE_mago")
        .ok()
        .or_else(|| option_env!("CARGO_BIN_EXE_mago").map(String::from))
        .unwrap_or_else(|| "mago".to_string());

    PathBuf::from(path)
}

/// Returns true if the mago binary can actually execute on this host.
/// This is false when cross-compiling (e.g., building aarch64 on x86_64).
fn can_run_mago() -> bool {
    Command::new(mago_bin())
        .arg("--help")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

/// Creates a workspace with a single PHP file that triggers 3 `constant-condition`
/// issues (help) and 1 `strict-types` issue (warning).
fn setup_workspace() -> tempfile::TempDir {
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path();

    std::fs::create_dir(workspace.join("src")).unwrap();
    std::fs::write(
        workspace.join("mago.toml"),
        r#"
php-version = "8.4"
[source]
paths = ["src"]
[linter]
"#,
    )
    .unwrap();

    let php = r#"<?php

if (true) {
    echo 'a';
}

if (false) {
    echo 'b';
}

if (true) {
    echo 'c';
}
"#;
    std::fs::write(workspace.join("src").join("example.php"), php).unwrap();

    temp_dir
}

fn run_lint(workspace: &Path, extra_args: &[&str]) -> std::process::Output {
    Command::new(mago_bin())
        .args(["--workspace", workspace.to_str().unwrap(), "lint"])
        .args(extra_args)
        .current_dir(workspace)
        .output()
        .expect("failed to run mago")
}

/// Parses `level[code]: count` lines from stats output into (label, count) pairs.
fn parse_count_lines(stdout: &str) -> Vec<(String, usize)> {
    stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let (label, count) = line.rsplit_once(": ").unwrap_or_else(|| panic!("unexpected stats line: {line:?}"));
            let count = count.trim().parse::<usize>().unwrap_or_else(|_| panic!("unexpected count in line: {line:?}"));

            (label.to_string(), count)
        })
        .collect()
}

#[test]
fn test_lint_stats_reports_counts_ordered_descending() {
    if !can_run_mago() {
        return;
    }

    let temp_dir = setup_workspace();
    let output = run_lint(temp_dir.path(), &["--stats"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Only help/warning issues are present, which is below the default fail level (error).
    assert!(
        output.status.success(),
        "lint --stats should succeed when no issue reaches the fail level; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let entries = parse_count_lines(&stdout);
    assert!(entries.len() >= 2, "expected at least two issue codes in stats output; got: {stdout:?}");
    assert!(
        entries.iter().any(|(label, count)| label.contains("[constant-condition]") && *count == 3),
        "expected 3 constant-condition issues in stats output; got: {stdout:?}"
    );
    assert!(
        entries.windows(2).all(|pair| pair[0].1 >= pair[1].1),
        "stats output should be ordered from highest to lowest count; got: {stdout:?}"
    );
}

#[test]
fn test_lint_stats_matches_code_count_format() {
    if !can_run_mago() {
        return;
    }

    let temp_dir = setup_workspace();
    let stats_output = run_lint(temp_dir.path(), &["--stats"]);
    let format_output = run_lint(temp_dir.path(), &["--reporting-format", "code-count"]);

    assert_eq!(
        String::from_utf8_lossy(&stats_output.stdout),
        String::from_utf8_lossy(&format_output.stdout),
        "--stats should produce the same output as --reporting-format code-count"
    );
    assert_eq!(stats_output.status.code(), format_output.status.code());
}

#[test]
fn test_lint_stats_honors_minimum_fail_level() {
    if !can_run_mago() {
        return;
    }

    let temp_dir = setup_workspace();
    let output = run_lint(temp_dir.path(), &["--stats", "--minimum-fail-level", "help"]);

    assert!(
        !output.status.success(),
        "lint --stats should fail when issues reach the minimum fail level; stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn test_lint_stats_conflicts_with_explicit_reporting_format_and_fix() {
    if !can_run_mago() {
        return;
    }

    let temp_dir = setup_workspace();

    for conflicting_args in [&["--stats", "--reporting-format", "json"][..], &["--stats", "--fix"][..]] {
        let output = run_lint(temp_dir.path(), conflicting_args);
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert_eq!(
            output.status.code(),
            Some(2),
            "lint {} should be rejected by clap; stderr: {stderr}",
            conflicting_args.join(" ")
        );
        assert!(stderr.contains("--stats"), "conflict error should mention --stats; got: {stderr}");
    }
}
