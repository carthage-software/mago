//! Integration tests for --stdin-input on `analyze`, `lint`, and `guard`.
//!
//! These tests run the mago binary with content piped via stdin and a path argument
//! and verify that the path is used for reporting (and baseline when applicable).

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

/// Strip leading log lines (e.g. " INFO ...") so the remainder is JSON.
fn strip_log_prefix(stdout: &str) -> &str {
    let s = stdout.trim();
    let start = s.find('{').unwrap_or(0);
    s[start..].trim()
}

/// Minimal JSON parsing to read the "issues" array and file "name" from the first issue.
/// Returns None if stdout is empty or not valid JSON (e.g. when no issues and report is empty).
fn parse_issues_json(stdout: &str) -> Option<(usize, Vec<String>)> {
    let s = strip_log_prefix(stdout);
    if s.is_empty() {
        return Some((0, vec![]));
    }
    let json: serde_json::Value = serde_json::from_str(s).ok()?;
    let issues = json.get("issues")?.as_array()?;
    let names: Vec<String> = issues
        .iter()
        .filter_map(|issue| {
            let ann = issue.get("annotations")?.as_array()?.first()?;
            let span = ann.get("span")?;
            let file_id = span.get("file_id")?;
            file_id.get("name").and_then(|n| n.as_str()).map(String::from)
        })
        .collect();
    Some((issues.len(), names))
}

fn mago_bin() -> std::path::PathBuf {
    // Prefer runtime env (set when cargo runs the test), then compile-time (set when cargo builds the test).
    let path = std::env::var("CARGO_BIN_EXE_mago")
        .ok()
        .or_else(|| option_env!("CARGO_BIN_EXE_mago").map(String::from))
        .unwrap_or_else(|| "mago".to_string());
    std::path::PathBuf::from(path)
}

fn run_mago_stdin(
    subcommand: &str,
    workspace: &Path,
    path_arg: &str,
    stdin_content: &str,
    extra_args: &[&str],
) -> (std::process::Output, String) {
    let mut args = vec![
        "--workspace",
        workspace.to_str().unwrap(),
        subcommand,
        path_arg,
        "--stdin-input",
        "--reporting-format",
        "json",
    ];
    args.extend(extra_args);

    let mut child = Command::new(mago_bin())
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn mago");

    child.stdin.as_mut().unwrap().write_all(stdin_content.as_bytes()).unwrap();
    let output = child.wait_with_output().expect("failed to run mago");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    (output, stdout)
}

#[test]
fn test_analyze_stdin_input_uses_path_in_output() {
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path();

    std::fs::create_dir(workspace.join("src")).unwrap();
    std::fs::write(
        workspace.join("mago.toml"),
        r#"
php-version = "8.4"
[source]
paths = ["src"]
[analyzer]
"#,
    )
    .unwrap();
    std::fs::write(workspace.join("src").join("example.php"), "<?php\n").unwrap();

    // PHP that triggers at least one analyzer issue, so the JSON report contains the file path.
    let php = r#"<?php
function f(): int { return "not an int"; }
"#;
    let (_output, stdout) = run_mago_stdin("analyze", workspace, "src/example.php", php, &[]);

    assert!(
        stdout.contains("example.php"),
        "analyze --stdin-input output should contain the file path in report; got: {}",
        stdout
    );
}

#[test]
fn test_lint_stdin_input_uses_path_in_output() {
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
    std::fs::write(workspace.join("src").join("example.php"), "<?php\n").unwrap();

    // PHP that triggers at least one lint issue, so the report contains the file path.
    let php = r#"<?php
  $x=1;
"#;
    let (_output, stdout) = run_mago_stdin("lint", workspace, "src/example.php", php, &[]);

    assert!(
        stdout.contains("example.php"),
        "lint --stdin-input output should contain the file path in report; got: {}",
        stdout
    );
}

#[test]
fn test_guard_stdin_input_uses_path_in_output() {
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path();

    std::fs::create_dir(workspace.join("src")).unwrap();
    std::fs::write(
        workspace.join("mago.toml"),
        r#"
php-version = "8.4"
[source]
paths = ["src"]
[guard]
"#,
    )
    .unwrap();
    std::fs::write(workspace.join("src").join("example.php"), "<?php\n").unwrap();

    // Minimal PHP for guard (the path must appear in the report; guard may or may not report issues).
    let php = r#"<?php
$x = 1;
"#;
    let (output, stdout) = run_mago_stdin("guard", workspace, "src/example.php", php, &[]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "guard --stdin-input should succeed; stdout: {:?}; stderr: {:?}", stdout, stderr);
    assert!(
        stdout.contains("example.php") || stderr.contains("No issues found"),
        "guard --stdin-input should use path or report no issues; got stdout: {:?}, stderr: {:?}",
        stdout,
        stderr
    );
}

/// When the path argument has a leading "./", the reported file name in JSON must be normalized
/// (no "./") so baseline matching works the same as for disk-loaded files.
#[test]
fn test_analyze_stdin_input_normalizes_dot_slash_path() {
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path();

    std::fs::create_dir(workspace.join("src")).unwrap();
    std::fs::write(
        workspace.join("mago.toml"),
        r#"
php-version = "8.4"
[source]
paths = ["src"]
[analyzer]
"#,
    )
    .unwrap();
    std::fs::write(workspace.join("src").join("example.php"), "<?php\n").unwrap();

    let php = r#"<?php
function f(): int { return "not an int"; }
"#;
    // Pass path with leading ./ so we verify it is normalized in the report.
    let (_output, stdout) = run_mago_stdin("analyze", workspace, "./src/example.php", php, &[]);

    let (count, names) = parse_issues_json(&stdout).expect("output should be valid JSON with issues");
    assert!(count >= 1, "expected at least one issue; got: {}", stdout);
    for name in &names {
        assert!(
            !name.starts_with("./"),
            "reported file name must be normalized (no leading ./) for baseline matching; got name: {:?}",
            name
        );
        assert!(
            name == "src/example.php" || name.ends_with("example.php"),
            "reported file name should be workspace-relative; got: {:?}",
            name
        );
    }
}

/// With a baseline file, stdin-input should filter issues the same way as analyzing the file from disk,
/// so that baselines apply correctly when using --stdin-input (e.g. from the IDE).
#[test]
fn test_analyze_stdin_input_baseline_filters_same_as_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path();

    std::fs::create_dir(workspace.join("src")).unwrap();
    let mago_toml = r#"
php-version = "8.4"
[source]
paths = ["src"]
[analyzer]
baseline = "baseline.toml"
"#;
    std::fs::write(workspace.join("mago.toml"), mago_toml).unwrap();

    // PHP that triggers exactly two analyzer issues (so we can baseline them).
    let php_two_issues = r#"<?php
function f($a, $b) {
    return (string) $a;
}
"#;
    std::fs::write(workspace.join("src").join("baseline_test.php"), php_two_issues).unwrap();

    let mago = mago_bin();
    let run = |args: &[&str], stdin: Option<&str>| {
        let mut cmd = Command::new(&mago);
        cmd.current_dir(workspace);
        cmd.args(["--workspace", workspace.to_str().unwrap()]);
        cmd.args(args);
        cmd.args(["--reporting-format", "json"]);
        if let Some(s) = stdin {
            cmd.stdin(Stdio::piped());
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());
            let mut child = cmd.spawn().unwrap();
            child.stdin.as_mut().unwrap().write_all(s.as_bytes()).unwrap();
            child.wait_with_output().unwrap()
        } else {
            cmd.output().unwrap()
        }
    };

    // Generate baseline from current 2 issues.
    let out = run(&["analyze", "src/baseline_test.php", "--baseline", "baseline.toml", "--generate-baseline"], None);
    assert!(out.status.success(), "generate-baseline should succeed: {}", String::from_utf8_lossy(&out.stderr));

    // Add one more issue (missing return type) so we have 3 total; only 1 should be reported after baseline.
    let php_three_issues = r#"<?php
function f($a, $b) {
    return (string) $a;
}
function g() { return 1; }
"#;

    std::fs::write(workspace.join("src").join("baseline_test.php"), php_three_issues).unwrap();

    // Run from disk: baseline should filter some issues; we only care that stdin matches disk.
    let out_disk = run(&["analyze", "src/baseline_test.php"], None);
    let stdout_disk = String::from_utf8_lossy(&out_disk.stdout);
    let (count_disk, _) = parse_issues_json(&stdout_disk).unwrap_or((0, vec![]));

    // Run with stdin and normalized path (src/...): should also report 1 issue.
    let out_stdin = run(&["analyze", "src/baseline_test.php", "--stdin-input"], Some(php_three_issues));
    let stdout_stdin = String::from_utf8_lossy(&out_stdin.stdout);
    let (count_stdin, _) = parse_issues_json(&stdout_stdin).unwrap_or((0, vec![]));

    assert_eq!(
        count_disk, count_stdin,
        "stdin-input should apply baseline like disk: disk reported {} issues, stdin reported {}",
        count_disk, count_stdin
    );

    // Same with path as ./src/... (normalized internally).
    let out_stdin_dot = run(&["analyze", "./src/baseline_test.php", "--stdin-input"], Some(php_three_issues));
    let stdout_stdin_dot = String::from_utf8_lossy(&out_stdin_dot.stdout);
    let (count_stdin_dot, _) = parse_issues_json(&stdout_stdin_dot).unwrap_or((0, vec![]));
    assert_eq!(
        count_stdin_dot, count_disk,
        "stdin with ./path should normalize and apply baseline like disk; disk={}, stdin_dot={}",
        count_disk, count_stdin_dot
    );
}
