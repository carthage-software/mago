use std::path::Path;
use std::process::Command;
use std::process::Output;

const BEFORE: &str = include_str!("fixtures/fix/multipass.before.php");
const AFTER: &str = include_str!("fixtures/fix/multipass.after.php");

fn workspace(code: &str) -> tempfile::TempDir {
    let directory = tempfile::tempdir().unwrap();
    std::fs::create_dir(directory.path().join("src")).unwrap();
    std::fs::write(directory.path().join("mago.toml"), include_str!("fixtures/fix/mago.toml")).unwrap();
    std::fs::write(directory.path().join("src/test.php"), code).unwrap();
    directory
}

fn run(workspace: &Path, command: &str, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mago"))
        .args(["--no-version-check", "--colors", "never", "--threads", "2", command])
        .args(arguments)
        .env("MAGO_LOG", "info")
        .current_dir(workspace)
        .output()
        .unwrap()
}

fn contents(workspace: &Path) -> String {
    std::fs::read_to_string(workspace.join("src/test.php")).unwrap()
}

#[test]
fn fixes_dependent_rules_in_order_and_is_idempotent() {
    let directory = workspace(BEFORE);
    let output = run(directory.path(), "fix", &[]);
    let log = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{log}");
    assert_eq!(contents(directory.path()), AFTER);
    let steps: Vec<_> = log.lines().filter(|line| line.contains("Running ")).collect();
    assert_eq!(steps.len(), 12, "{log}");
    for pass in steps.as_chunks::<4>().0 {
        for (line, tool) in pass.iter().zip(["guard", "analyze", "lint", "format"]) {
            assert!(line.contains(&format!("Running {tool}")), "{line}");
        }
    }

    let output = run(directory.path(), "fix", &[]);
    assert!(output.status.success());
    assert_eq!(contents(directory.path()), AFTER);
    assert!(!String::from_utf8_lossy(&output.stderr).contains("fix pass 2"));
}

#[test]
fn can_disable_any_or_all_tools() {
    let directory = workspace(BEFORE);
    let output = run(directory.path(), "fix", &["--no-guard", "--no-analyze", "--no-lint", "--no-fmt"]);
    assert!(output.status.success());
    assert_eq!(contents(directory.path()), BEFORE);
    assert!(!String::from_utf8_lossy(&output.stderr).contains("Running "));

    let output = run(directory.path(), "fix", &["--no-guard", "--no-analyze", "--no-lint"]);
    assert!(output.status.success());
    assert!(contents(directory.path()).contains("is_null($value)"));
    assert_ne!(contents(directory.path()), BEFORE);

    std::fs::write(directory.path().join("src/test.php"), BEFORE).unwrap();
    let output = run(directory.path(), "fix", &["--no-guard", "--no-analyze", "--no-fmt"]);
    assert!(output.status.success());
    assert_eq!(contents(directory.path()), BEFORE.replace("is_null($value)", "$value === null"));
}

#[test]
fn uses_the_selected_safety_level() {
    let code = include_str!("fixtures/fix/safety.php");
    let directory = workspace(code);
    let output = run(directory.path(), "fix", &["--no-guard", "--no-analyze", "--no-fmt"]);
    assert!(output.status.success());
    assert_eq!(contents(directory.path()), code);

    for flag in ["--potentially-unsafe", "--unsafe"] {
        std::fs::write(directory.path().join("src/test.php"), code).unwrap();
        let output = run(directory.path(), "fix", &["--no-guard", "--no-analyze", "--no-fmt", flag]);
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        assert!(contents(directory.path()).contains("declare(strict_types=1);"));
    }
}

#[test]
fn applies_analyzer_fixes_before_formatting() {
    let directory = workspace(include_str!("fixtures/fix/analyzer.php"));
    let output = run(directory.path(), "fix", &["--no-guard", "--no-lint"]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(contents(directory.path()).contains("return $value;"));
    assert!(!contents(directory.path()).contains("=== true"));
}

#[test]
fn unfixable_issues_do_not_stop_other_fixes() {
    for fail_on_remaining in [false, true] {
        let directory = workspace(include_str!("fixtures/fix/unfixable.php"));
        let arguments = if fail_on_remaining { vec!["--fail-on-remaining"] } else { vec![] };
        let output = run(directory.path(), "fix", &arguments);
        assert_eq!(output.status.success(), !fail_on_remaining, "{}", String::from_utf8_lossy(&output.stderr));
        assert!(contents(directory.path()).contains("return $value === null;"));
        assert!(contents(directory.path()).contains("missing_function();"));
    }
}

#[test]
fn limits_fixes_to_requested_paths() {
    let directory = workspace(BEFORE);
    std::fs::write(directory.path().join("src/other.php"), BEFORE).unwrap();
    let output = run(directory.path(), "fix", &["src/test.php", "--no-guard", "--no-analyze"]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(contents(directory.path()), AFTER);
    assert_eq!(std::fs::read_to_string(directory.path().join("src/other.php")).unwrap(), BEFORE);
}

#[test]
fn refuses_to_claim_success_before_fixes_settle() {
    let directory = workspace(BEFORE);
    let output = run(directory.path(), "fix", &["--no-guard", "--no-analyze", "--max-passes", "1"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("did not settle"));

    for limit in ["0", "257"] {
        let output = run(directory.path(), "fix", &["--max-passes", limit]);
        assert_eq!(output.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&output.stderr).contains("1..=256"));
    }

    let output = run(directory.path(), "fix", &["--no-guard", "--no-analyze", "--max-passes", "256"]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(contents(directory.path()), AFTER);
}

#[test]
fn reports_formatter_parse_failures() {
    let code = include_str!("fixtures/fix/invalid.php");
    let directory = workspace(code);
    let output = run(directory.path(), "fix", &["--no-guard", "--no-analyze", "--no-lint"]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(contents(directory.path()), code);
}

#[test]
fn respects_tool_exclusions() {
    let directory = workspace(BEFORE);
    std::fs::write(directory.path().join("mago.toml"), include_str!("fixtures/fix/exclusions.toml")).unwrap();
    for name in ["lint_excluded.php", "format_excluded.php", "source_excluded.php"] {
        std::fs::write(directory.path().join("src").join(name), BEFORE).unwrap();
    }

    let output = run(directory.path(), "fix", &["--no-guard", "--no-analyze"]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(contents(directory.path()), AFTER);
    assert_eq!(
        std::fs::read_to_string(directory.path().join("src/lint_excluded.php")).unwrap(),
        AFTER.replace("$value === null", "is_null($value)")
    );
    assert_eq!(
        std::fs::read_to_string(directory.path().join("src/format_excluded.php")).unwrap(),
        BEFORE.replace("is_null($value)", "$value === null")
    );
    assert_eq!(std::fs::read_to_string(directory.path().join("src/source_excluded.php")).unwrap(), BEFORE);
}

#[test]
fn respects_baselines_unless_asked_to_ignore_them() {
    let directory = workspace(BEFORE);
    std::fs::write(directory.path().join("mago.toml"), include_str!("fixtures/fix/baseline.toml")).unwrap();
    let output = run(directory.path(), "lint", &["--generate-baseline"]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));

    let output = run(directory.path(), "fix", &["--no-guard", "--no-analyze", "--no-fmt"]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(contents(directory.path()), BEFORE);

    let output = run(directory.path(), "fix", &["--no-guard", "--no-analyze", "--ignore-baseline"]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(contents(directory.path()), AFTER);
}
