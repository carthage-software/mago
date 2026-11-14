mod common_twig_lint;

use common_twig_lint::*;

#[test]
fn discovers_only_twig_without_running_php_rules() {
    let directory = workspace("");
    let issues = issues(directory.path(), &["--only", "strict-types,twig-named-argument-separator"]);
    let issues = issues.as_array().unwrap();
    assert_eq!(issues.len(), 2);
    assert_eq!(issues.iter().filter(|issue| issue["code"] == "strict-types").count(), 0);
    assert_eq!(issues.iter().filter(|issue| issue["code"] == "twig-named-argument-separator").count(), 2);
    for path in ["src", "src/template.twig"] {
        assert_eq!(
            self::issues(directory.path(), &[path, "--only", "twig-named-argument-separator"])
                .as_array()
                .unwrap()
                .len(),
            2
        );
    }
}

#[test]
fn disabled_command_never_parses_any_selected_source() {
    let directory = workspace("[twig.linter]\nenabled = false\n");
    std::fs::write(directory.path().join("src/template.twig"), b"{% if %}<?php broken(").unwrap();
    assert!(issues(directory.path(), &["src/template.twig"]).as_array().unwrap().is_empty());
    assert!(issues(directory.path(), &["--only", "strict-types"]).as_array().unwrap().is_empty());
    std::fs::write(
        directory.path().join("mago.toml"),
        "[source]\npaths = ['src']\nextensions = ['php', 'twig']\n[twig.linter]\nenabled = false\n",
    )
    .unwrap();
    assert!(issues(directory.path(), &["src/template.twig"]).as_array().unwrap().is_empty());
}

#[test]
fn stdin_uses_its_path_for_twig_dispatch_and_reporting() {
    let directory = workspace("");
    for path in ["src/unsaved.twig", "src/UPPER.TWIG", "src/café.中文.twig", "src/unsaved", "src/unsaved.php"] {
        let output = stdin(directory.path(), path, b"<?php broken( {{ dump(user) }}");
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        let issues: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(issues["issues"].as_array().unwrap().len(), 1);
        assert_eq!(issues["issues"][0]["code"], "twig-no-dump");
        assert_eq!(issues["issues"][0]["annotations"][0]["span"]["file_id"]["name"], path);
    }
    let output = stdin(directory.path(), "src/bytes.twig", b"  \xff{{ dump(user) }}\xfe ");
    let issues: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(issues["issues"].as_array().unwrap().len(), 1);
    assert_eq!(issues["issues"][0]["code"], "twig-no-dump");
    assert_eq!(issues["issues"][0]["annotations"][0]["span"]["start"]["offset"], 6);
}

#[test]
fn rule_listing_explanation_and_statistics_are_twig_only() {
    let directory = workspace("");
    let output = run(directory.path(), &["twig:lint", "--only", "twig-no-raw,strict-types", "--list-rules", "--json"]);
    assert!(output.status.success());
    let rules: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(rules.as_array().unwrap().len(), 1);
    assert!(rules.as_array().unwrap().iter().any(|rule| rule["code"] == "twig-no-raw"));
    let output = run(directory.path(), &["twig:lint", "--explain", "twig-no-raw"]);
    assert!(output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .replace('│', "")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .contains("off by default")
    );
    let output = run(directory.path(), &["twig:lint", "--only", "twig-named-argument-separator", "--stats"]);
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("twig-named-argument-separator]: 2"));
    let output =
        run(directory.path(), &["twig:lint", "--only", "twig-named-argument-separator", "--reporting-format", "sarif"]);
    assert!(output.status.success());
    let sarif: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let results = sarif["runs"][0]["results"].as_array().unwrap();
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|issue| issue["ruleId"] == "twig-named-argument-separator"));
}

#[test]
fn twig_exclusions_do_not_change_php_discovery_and_rule_exclusions_apply() {
    let directory = workspace("[twig.linter]\nexcludes = ['src/template.twig']\n");
    assert_eq!(
        issues(directory.path(), &["--only", "strict-types,twig-named-argument-separator"]).as_array().unwrap().len(),
        0
    );
    let output = run(directory.path(), &["list-files", "--command", "linter"]);
    assert!(output.status.success());
    assert!(!String::from_utf8_lossy(&output.stdout).contains("template.twig"));
    std::fs::write(
        directory.path().join("mago.toml"),
        "extends='modern.toml'\n[source]\npaths=['src']\n[twig.linter.rules.named-argument-separator]\nexclude=['src/*.twig']\n",
    )
    .unwrap();
    assert!(issues(directory.path(), &["--only", "twig-named-argument-separator"]).as_array().unwrap().is_empty());
}

#[test]
fn native_baselines_suppress_twig_issues_and_can_be_ignored() {
    let directory = workspace("");
    let root = directory.path();
    let output = run(
        root,
        &[
            "twig:lint",
            "--only",
            "twig-named-argument-separator",
            "--baseline",
            "lint.baseline.toml",
            "--generate-baseline",
        ],
    );
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(root.join("lint.baseline.toml").is_file());
    assert!(
        issues(root, &["--only", "twig-named-argument-separator", "--baseline", "lint.baseline.toml"])
            .as_array()
            .unwrap()
            .is_empty()
    );
    std::fs::write(
        root.join("mago.toml"),
        "extends='modern.toml'\n[source]\npaths=['src']\n[twig.linter]\nbaseline='lint.baseline.toml'\n",
    )
    .unwrap();
    assert_eq!(
        issues(root, &["--only", "twig-named-argument-separator", "--ignore-baseline"]).as_array().unwrap().len(),
        2
    );
}

#[test]
fn safe_fixes_and_formatter_settle_without_adding_twig_to_fix() {
    let directory = workspace(
        "[twig.linter.rules.end-block-name]\nenabled=true\n[twig.linter.rules.single-quotes]\nenabled=true\n",
    );
    let root = directory.path();
    let output = run(root, &["twig:lint", "src/template.twig", "--fix"]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let fixed = std::fs::read(root.join("src/template.twig")).unwrap();
    assert!(fixed.windows(b"low : 1".len()).any(|bytes| bytes == b"low : 1"));
    assert!(fixed.windows(b"endblock body".len()).any(|bytes| bytes == b"endblock body"));
    assert!(fixed.windows(b"join(',')".len()).any(|bytes| bytes == b"join(',')"));
    let output = run(root, &["fix", "src/template.twig", "--no-guard", "--no-analyze", "--fail-on-remaining"]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let stable = std::fs::read(root.join("src/template.twig")).unwrap();
    assert!(run(root, &["fix", "src/template.twig", "--no-guard", "--no-analyze"]).status.success());
    assert_eq!(std::fs::read(root.join("src/template.twig")).unwrap(), stable);
}

#[test]
fn invalid_twig_reports_its_own_syntax_code_without_fixes() {
    let directory = workspace("");
    let source = b"A\n{% if %}{{ dump(value) }}";
    std::fs::write(directory.path().join("src/template.twig"), source).unwrap();
    let result = issues(directory.path(), &["src/template.twig"]);
    assert!(result.as_array().unwrap().iter().all(|issue| issue["code"] == "twig-syntax-error"));
    assert!(
        !run(directory.path(), &["twig:lint", "src/template.twig", "--fix", "--fail-on-remaining"]).status.success()
    );
    assert_eq!(std::fs::read(directory.path().join("src/template.twig")).unwrap(), source);
}

#[test]
fn substitutions_lint_the_twig_replacement_without_changing_the_original() {
    let directory = workspace("");
    let root = directory.path();
    let original = root.join("src/template.twig");
    let replacement = root.join("replacement.twig");
    let before = std::fs::read(&original).unwrap();
    std::fs::write(&replacement, b"{{ dump(value) }}").unwrap();
    let mapping = format!("{}={}", original.display(), replacement.display());
    let result = issues(root, &["--only", "twig-no-dump", "--substitute", &mapping]);
    assert_eq!(result.as_array().unwrap().len(), 1);
    assert_eq!(result[0]["code"], "twig-no-dump");
    assert_eq!(result[0]["annotations"][0]["span"]["file_id"]["name"], "replacement.twig");
    assert_eq!(std::fs::read(original).unwrap(), before);
}

#[test]
fn staged_lint_fixes_only_the_staged_twig_file() {
    let directory = workspace("");
    let root = directory.path();
    std::fs::write(root.join("src/unstaged.twig"), b"{% if %}").unwrap();
    std::fs::write(root.join("src/main.php"), b"<?php function broken( {").unwrap();
    std::fs::write(root.join("src/skipped.TWIG"), b"{% if %}").unwrap();
    for arguments in [
        vec!["init", "--quiet"],
        vec!["config", "core.autocrlf", "false"],
        vec!["add", "src/template.twig", "src/main.php", "src/skipped.TWIG"],
    ] {
        assert!(std::process::Command::new("git").args(arguments).current_dir(root).status().unwrap().success());
    }
    let output = run(
        root,
        &["twig:lint", "--staged", "--only", "twig-named-argument-separator", "--fix", "--fail-on-remaining"],
    );
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let staged =
        std::process::Command::new("git").args(["show", ":src/template.twig"]).current_dir(root).output().unwrap();
    assert!(staged.status.success());
    assert_eq!(staged.stdout, std::fs::read(root.join("src/template.twig")).unwrap());
    assert!(!staged.stdout.windows(b"low =".len()).any(|bytes| bytes == b"low ="));
    assert_eq!(std::fs::read(root.join("src/unstaged.twig")).unwrap(), b"{% if %}");
}
