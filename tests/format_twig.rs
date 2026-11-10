mod common_twig_format;

use common_twig_format::*;

#[test]
fn formats_only_twig_inputs_and_checks_without_writing() {
    let directory = workspace(CONFIG);
    let root = directory.path();
    for arguments in [["twig:fmt", "--check"], ["twig:fmt", "--dry-run"]] {
        let output = run(root, &arguments);
        assert_eq!(output.status.code(), Some(1), "{}", String::from_utf8_lossy(&output.stderr));
        assert_eq!(contents(root), BEFORE);
    }
    assert!(run(root, &["twig:fmt"]).status.success());
    assert_eq!(contents(root), AFTER);
    assert_eq!(std::fs::read(root.join("src/unchanged.php")).unwrap(), PHP_BEFORE);
    assert!(run(root, &["twig:fmt", "--check"]).status.success());
}

#[test]
fn formats_explicit_files_and_directories() {
    for path in ["src/template.html.twig", "src"] {
        let directory = workspace(CONFIG);
        let output = run(directory.path(), &["twig:fmt", path]);
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        assert_eq!(contents(directory.path()), AFTER);
    }
}

#[test]
fn command_selects_twig_for_explicit_files_with_any_suffix() {
    for name in ["buffer", "buffer.php", "buffer.TWIG"] {
        let directory = workspace(CONFIG);
        let path = directory.path().join(name);
        std::fs::write(&path, include_bytes!("fixtures/format_twig/language_boundary.twig")).unwrap();
        let output = run(directory.path(), &["twig:fmt", name]);
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        assert_eq!(std::fs::read(path).unwrap(), include_bytes!("fixtures/format_twig/language_boundary.after.twig"));
    }
}

#[test]
fn stdin_always_uses_twig_regardless_of_buffer_name() {
    let directory = workspace(CONFIG);
    for path in
        [None, Some("src/template.html.twig"), Some("src/unsaved"), Some("src/buffer.php"), Some("src/UPPER.TWIG")]
    {
        let output = stdin(directory.path(), path, BEFORE);
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        assert_eq!(output.stdout, AFTER);
    }
    assert_eq!(contents(directory.path()), BEFORE);
}

#[test]
fn php_formatter_uses_php_regardless_of_buffer_name() {
    let directory = workspace(CONFIG);
    for path in [None, Some("src/template.html.twig"), Some("src/buffer.php")] {
        let output = stdin_for(directory.path(), "fmt", path, PHP_BEFORE);
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        assert_eq!(output.stdout, PHP);
    }
    assert_eq!(stdin_for(directory.path(), "fmt", None, BEFORE).stdout, BEFORE);
}

#[test]
fn disabling_twig_does_not_disable_php() {
    let directory = workspace(include_str!("fixtures/format_twig/disabled.toml"));
    let root = directory.path();
    std::fs::write(root.join("src/template.html.twig"), INVALID).unwrap();
    assert!(run(root, &["twig:fmt", "--check"]).status.success());
    assert!(run(root, &["twig:fmt", "src/template.html.twig"]).status.success());
    assert_eq!(stdin(root, Some("src/template.html.twig"), INVALID).stdout, INVALID);
    assert_eq!(contents(root), INVALID);
    assert!(run(root, &["fmt"]).status.success());
    assert_eq!(std::fs::read(root.join("src/unchanged.php")).unwrap(), PHP);
}

#[test]
fn honors_source_and_twig_excludes() {
    let directory = workspace(include_str!("fixtures/format_twig/excludes.toml"));
    for path in ["src/source-excluded.twig", "src/format-excluded.twig"] {
        std::fs::write(directory.path().join(path), BEFORE).unwrap();
        assert_eq!(stdin(directory.path(), Some(path), BEFORE).stdout, BEFORE);
    }
    assert!(run(directory.path(), &["twig:fmt"]).status.success());
    assert_eq!(contents(directory.path()), AFTER);
    for path in ["src/source-excluded.twig", "src/format-excluded.twig"] {
        assert_eq!(std::fs::read(directory.path().join(path)).unwrap(), BEFORE);
    }
}

#[test]
fn php_formatter_excludes_do_not_apply_to_twig() {
    let directory = workspace("[source]\npaths=['src']\n[formatter]\nexcludes=['src']\n");
    assert!(run(directory.path(), &["twig:fmt"]).status.success());
    assert_eq!(contents(directory.path()), AFTER);
    assert_eq!(std::fs::read(directory.path().join("src/unchanged.php")).unwrap(), PHP_BEFORE);
}

#[test]
fn invalid_templates_remain_unchanged() {
    let directory = workspace(CONFIG);
    std::fs::write(directory.path().join("src/template.html.twig"), INVALID).unwrap();
    let output = run(directory.path(), &["twig:fmt"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("template.html.twig"));
    assert_eq!(contents(directory.path()), INVALID);
    let output = stdin(directory.path(), None, INVALID);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
}

#[test]
fn extreme_nesting_reports_an_error_without_aborting() {
    let directory = workspace(CONFIG);
    for source in [
        format!("{{{{ {}x{} }}}}", "(".repeat(5000), ")".repeat(5000)),
        format!("{}x{}", "{% if x %}".repeat(5000), "{% endif %}".repeat(5000)),
    ] {
        let output = stdin(directory.path(), None, source.as_bytes());
        assert_eq!(output.status.code(), Some(2), "{}", String::from_utf8_lossy(&output.stderr));
        assert!(output.stdout.is_empty());
    }
}

#[test]
fn staged_tools_keep_languages_and_working_copies_separate() {
    let directory = workspace(CONFIG);
    let root = directory.path();
    git(root, &["init", "--quiet"]);
    git(root, &["config", "core.autocrlf", "false"]);
    git(root, &["add", "src/template.html.twig", "src/unchanged.php"]);
    assert!(run(root, &["twig:fmt", "--staged"]).status.success());
    assert_eq!(git(root, &["show", ":src/template.html.twig"]).stdout, AFTER);
    assert_eq!(git(root, &["show", ":src/unchanged.php"]).stdout, PHP_BEFORE);
    assert_eq!(contents(root), BEFORE);
    assert!(run(root, &["fmt", "--staged"]).status.success());
    assert_eq!(git(root, &["show", ":src/unchanged.php"]).stdout, PHP);
    assert_eq!(std::fs::read(root.join("src/unchanged.php")).unwrap(), PHP_BEFORE);
    std::fs::write(root.join("src/template.html.twig"), INVALID).unwrap();
    git(root, &["add", "src/template.html.twig"]);
    assert!(!run(root, &["twig:fmt", "--staged"]).status.success());
    assert_eq!(git(root, &["show", ":src/template.html.twig"]).stdout, INVALID);
}

#[test]
fn staged_mode_honors_disabled_twig_and_excludes() {
    for (configuration, path) in [
        (include_str!("fixtures/format_twig/disabled.toml"), "src/template.html.twig"),
        (include_str!("fixtures/format_twig/excludes.toml"), "src/format-excluded.twig"),
        (include_str!("fixtures/format_twig/excludes.toml"), "src/source-excluded.twig"),
    ] {
        let directory = workspace(configuration);
        let root = directory.path();
        std::fs::write(root.join(path), INVALID).unwrap();
        git(root, &["init", "--quiet"]);
        git(root, &["add", path]);
        assert!(run(root, &["twig:fmt", "--staged"]).status.success());
        assert_eq!(git(root, &["show", &format!(":{path}")]).stdout, INVALID);
    }
}

#[test]
fn php_commands_and_file_listing_do_not_discover_twig() {
    let directory = workspace(CONFIG);
    let root = directory.path();
    std::fs::write(root.join("src/template.html.twig"), INVALID).unwrap();
    for command in ["analyze", "lint", "guard", "fmt"] {
        let output = run(root, &[command]);
        assert!(output.status.success(), "{command}: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert_eq!(contents(root), INVALID);
    for tool in ["formatter", "linter", "analyzer", "guard"] {
        let output = run(root, &["list-files", "--command", tool, "-0"]);
        assert!(output.status.success());
        assert!(!String::from_utf8_lossy(&output.stdout).contains("template.html.twig"));
    }
}

#[test]
fn fix_does_not_run_the_twig_formatter() {
    let directory = workspace(CONFIG);
    let root = directory.path();
    let output = run(root, &["fix", "--no-guard", "--no-analyze", "--no-lint"]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(contents(root), BEFORE);
    assert_eq!(std::fs::read(root.join("src/unchanged.php")).unwrap(), PHP);
}
