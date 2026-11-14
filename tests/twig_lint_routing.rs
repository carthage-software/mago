mod common_twig_lint;

use common_twig_lint::*;

#[test]
fn explicit_files_and_substitutions_are_twig_regardless_of_extension() {
    let directory = workspace("");
    let root = directory.path();
    let original = root.join("src/template.twig");
    let before = std::fs::read(&original).unwrap();
    for name in ["buffer", "buffer.php", "buffer.TWIG"] {
        let replacement = root.join(name);
        std::fs::write(&replacement, b"<?php broken( {{ dump(value) }}").unwrap();
        let result = issues(root, &[name, "--only", "twig-no-dump"]);
        assert_eq!(result.as_array().unwrap().len(), 1);
        assert_eq!(result[0]["code"], "twig-no-dump");
        assert_eq!(result[0]["annotations"][0]["span"]["file_id"]["name"], name);
        let mapping = format!("{}={}", original.display(), replacement.display());
        let result = issues(root, &["src/template.twig", "--only", "twig-no-dump", "--substitute", &mapping]);
        assert_eq!(result.as_array().unwrap().len(), 1);
        assert_eq!(result[0]["code"], "twig-no-dump");
        assert_eq!(result[0]["annotations"][0]["span"]["file_id"]["name"], name);
        assert_eq!(std::fs::read(&original).unwrap(), before);
        let output = stdin(root, name, b"<?php broken( {{ dump(value) }}");
        assert!(output.status.success());
        let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(result["issues"].as_array().unwrap().len(), 1);
        assert_eq!(result["issues"][0]["code"], "twig-no-dump");
    }
}

#[test]
fn php_settings_do_not_change_twig_reporting_or_discovery() {
    let directory = workspace("[linter]\nexcludes=['src']\nbaseline='missing.toml'\nminimum-fail-level='help'\n");
    let output =
        run(directory.path(), &["twig:lint", "--only", "twig-named-argument-separator", "--reporting-format", "json"]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["issues"].as_array().unwrap().len(), 2);
}

#[test]
fn php_rule_lists_config_and_file_lists_keep_their_own_scope() {
    let directory = workspace("");
    let root = directory.path();
    let output = run(root, &["lint", "--only", "strict-types,twig-no-dump", "--list-rules", "--json"]);
    assert!(output.status.success());
    let rules: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(rules.as_array().unwrap().len(), 1);
    assert_eq!(rules[0]["code"], "strict-types");
    assert!(!run(root, &["lint", "--explain", "twig-no-dump"]).status.success());
    let output = run(root, &["config", "--show", "linter"]);
    let config: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(config.get("twig").is_none());
    assert!(config.get("twig-linter").is_none());
    let output = run(root, &["list-files", "--command", "linter"]);
    assert!(output.status.success());
    let listing = String::from_utf8_lossy(&output.stdout);
    assert!(listing.contains("src/main.php"));
    assert!(!listing.contains("template.twig"));
}

#[test]
fn fix_does_not_run_twig_lint_rules() {
    let directory = workspace("[twig.linter.rules.end-block-name]\nenabled=true\n");
    let root = directory.path();
    let path = root.join("src/template.twig");
    let before = std::fs::read(&path).unwrap();
    let output = run(root, &["fix", "src/template.twig", "--no-guard", "--no-analyze", "--no-fmt"]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(std::fs::read(path).unwrap(), before);
    assert_eq!(issues(root, &["--only", "twig-named-argument-separator"]).as_array().unwrap().len(), 2);
}

#[test]
fn php_lint_does_not_discover_twig_or_use_twig_configuration() {
    let directory = workspace("[twig.linter]\nminimum-fail-level='note'\n");
    let root = directory.path();
    std::fs::write(root.join("src/template.twig"), b"<?php broken( {{ dump(value) }}").unwrap();
    let output = run(root, &["lint", "--only", "strict-types", "--reporting-format", "json"]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["issues"].as_array().unwrap().len(), 1);
    assert_eq!(result["issues"][0]["code"], "strict-types");
    assert_eq!(result["issues"][0]["annotations"][0]["span"]["file_id"]["name"], "src/main.php");
}

#[test]
fn format_after_fix_is_rejected_before_any_edit() {
    let directory = workspace("");
    let root = directory.path();
    let path = root.join("buffer.php");
    let source = b"<?php broken( {{ range(low = 1, high = 3)|join(',') }}";
    std::fs::write(&path, source).unwrap();
    let output = run(root, &["twig:lint", "buffer.php", "--fix", "--format-after-fix"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Twig formatting must run as a separate step"));
    assert_eq!(std::fs::read(&path).unwrap(), source);
}

#[test]
fn safe_fixes_validate_twig_without_parsing_literal_php_text() {
    let directory = workspace("");
    let root = directory.path();
    let path = root.join("buffer.php");
    let source = b"<?php broken( {{ range(low = 1, high = 3)|join(',') }}";
    let expected = b"<?php broken( {{ range(low : 1, high : 3)|join(',') }}";
    std::fs::write(&path, source).unwrap();
    let output = run(
        root,
        &["twig:lint", "buffer.php", "--only", "twig-named-argument-separator", "--fix", "--fail-on-remaining"],
    );
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(std::fs::read(&path).unwrap(), expected);
}

#[test]
fn configured_baseline_is_rooted_in_workspace_when_cwd_is_elsewhere() {
    let directory = workspace("[twig.linter]\nbaseline='twig.baseline.toml'\n");
    let outside = tempfile::tempdir().unwrap();
    let root = directory.path();
    let output = command(root)
        .current_dir(outside.path())
        .arg("--workspace")
        .arg(root)
        .args(["twig:lint", "--only", "twig-named-argument-separator", "--generate-baseline"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(root.join("twig.baseline.toml").is_file());
    assert!(!outside.path().join("twig.baseline.toml").exists());
    let output = command(root)
        .current_dir(outside.path())
        .arg("--workspace")
        .arg(root)
        .args(["twig:lint", "--only", "twig-named-argument-separator", "--reporting-format", "json"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(result["issues"].as_array().unwrap().is_empty());
    let output = command(root)
        .current_dir(outside.path())
        .arg("--workspace")
        .arg(root)
        .args([
            "twig:lint",
            "--only",
            "twig-named-argument-separator",
            "--ignore-baseline",
            "--reporting-format",
            "json",
        ])
        .output()
        .unwrap();
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["issues"].as_array().unwrap().len(), 2);
}
