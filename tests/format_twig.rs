use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::process::Output;
use std::process::Stdio;

const BEFORE: &[u8] = include_bytes!("fixtures/format_twig/before.twig");
const AFTER: &[u8] = include_bytes!("fixtures/format_twig/after.twig");
const PHP: &[u8] = include_bytes!("fixtures/format_twig/unchanged.php");
const INVALID: &[u8] = include_bytes!("fixtures/format_twig/invalid.twig");

fn workspace(configuration: &str) -> tempfile::TempDir {
    let directory = tempfile::tempdir().unwrap();
    std::fs::create_dir(directory.path().join("src")).unwrap();
    std::fs::write(directory.path().join("mago.toml"), configuration).unwrap();
    std::fs::write(directory.path().join("src/template.html.twig"), BEFORE).unwrap();
    std::fs::write(directory.path().join("src/unchanged.php"), PHP).unwrap();
    directory
}

fn command(workspace: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mago"));
    command.args(["--no-version-check", "--colors", "never", "--threads", "2"]).current_dir(workspace);
    command
}

fn run(workspace: &Path, arguments: &[&str]) -> Output {
    command(workspace).args(arguments).output().unwrap()
}

fn stdin(workspace: &Path, filepath: Option<&str>, content: &[u8]) -> Output {
    let mut command = command(workspace);
    command.args(["fmt", "--stdin-input"]);
    if let Some(filepath) = filepath {
        command.args(["--stdin-filepath", filepath]);
    }

    let mut child = command.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    child.stdin.take().unwrap().write_all(content).unwrap();
    child.wait_with_output().unwrap()
}

fn contents(workspace: &Path) -> Vec<u8> {
    std::fs::read(workspace.join("src/template.html.twig")).unwrap()
}

#[test]
fn formats_mixed_workspaces_and_checks_without_writing() {
    let directory = workspace(include_str!("fixtures/format_twig/mago.toml"));
    let root = directory.path();
    for arguments in [["fmt", "--check"], ["fmt", "--dry-run"]] {
        let output = run(root, &arguments);
        assert_eq!(output.status.code(), Some(1), "{}", String::from_utf8_lossy(&output.stderr));
        assert_eq!(contents(root), BEFORE);
    }

    let output = run(root, &["fmt"]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(contents(root), AFTER);
    assert_eq!(std::fs::read(root.join("src/unchanged.php")).unwrap(), PHP);
    assert!(run(root, &["fmt", "--check"]).status.success());
}

#[test]
fn formats_explicit_twig_files_and_directories() {
    for path in ["src/template.html.twig", "src"] {
        let directory = workspace(include_str!("fixtures/format_twig/mago.toml"));
        let output = run(directory.path(), &["fmt", path]);
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        assert_eq!(contents(directory.path()), AFTER);
    }
}

#[test]
fn stdin_uses_filepath_to_select_twig() {
    let directory = workspace(include_str!("fixtures/format_twig/mago.toml"));
    for path in [directory.path().join("src/template.html.twig"), "src/unsaved.twig".into(), "src/UPPER.TWIG".into()] {
        let output = stdin(directory.path(), Some(path.to_str().unwrap()), BEFORE);
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        assert_eq!(output.stdout, AFTER);
    }

    for path in [None, Some("twig"), Some("src/twig.php")] {
        let output = stdin(directory.path(), path, PHP);
        assert!(output.status.success());
        assert_eq!(output.stdout, PHP);
    }

    assert_eq!(contents(directory.path()), BEFORE);
}

#[test]
fn can_disable_twig_without_disabling_php() {
    let directory = workspace(include_str!("fixtures/format_twig/disabled.toml"));
    std::fs::write(directory.path().join("src/template.html.twig"), INVALID).unwrap();
    assert!(run(directory.path(), &["fmt", "--check"]).status.success());
    assert!(run(directory.path(), &["fmt", "src/template.html.twig"]).status.success());
    assert_eq!(contents(directory.path()), INVALID);
    let output = stdin(directory.path(), Some("src/template.html.twig"), INVALID);
    assert!(output.status.success());
    assert_eq!(output.stdout, INVALID);
}

#[test]
fn honors_source_and_formatter_excludes() {
    let directory = workspace(include_str!("fixtures/format_twig/excludes.toml"));
    for path in ["src/source-excluded.twig", "src/format-excluded.twig"] {
        std::fs::write(directory.path().join(path), BEFORE).unwrap();
        let output = stdin(directory.path(), Some(path), BEFORE);
        assert!(output.status.success());
        assert_eq!(output.stdout, BEFORE);
    }

    assert!(run(directory.path(), &["fmt"]).status.success());
    assert_eq!(contents(directory.path()), AFTER);
    for path in ["src/source-excluded.twig", "src/format-excluded.twig"] {
        assert_eq!(std::fs::read(directory.path().join(path)).unwrap(), BEFORE);
    }
}

#[test]
fn rejects_invalid_twig_without_writing_it() {
    let directory = workspace(include_str!("fixtures/format_twig/mago.toml"));
    std::fs::write(directory.path().join("src/template.html.twig"), INVALID).unwrap();
    let output = run(directory.path(), &["fmt"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("template.html.twig"));
    assert_eq!(contents(directory.path()), INVALID);
    let output = stdin(directory.path(), Some("src/template.html.twig"), INVALID);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
}

#[test]
fn extreme_nesting_returns_an_error_without_aborting() {
    let directory = workspace(include_str!("fixtures/format_twig/mago.toml"));
    for source in [
        format!("{{{{ {}x{} }}}}", "(".repeat(5000), ")".repeat(5000)),
        format!("{}x{}", "{% if x %}".repeat(5000), "{% endif %}".repeat(5000)),
    ] {
        let output = stdin(directory.path(), Some("src/template.html.twig"), source.as_bytes());
        assert_eq!(output.status.code(), Some(2), "{}", String::from_utf8_lossy(&output.stderr));
        assert!(output.stdout.is_empty());
    }
}

#[test]
fn twig_settings_are_separate_and_visible_in_configuration() {
    let directory = workspace(include_str!("fixtures/format_twig/settings.toml"));
    let output = run(directory.path(), &["config", "--show", "formatter"]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let config: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(config["print-width"], 20);
    assert_eq!(config["twig"]["print-width"], 80);
    assert_eq!(config["twig"]["enabled"], true);
    assert_eq!(config["twig"]["tab-width"], 2);
    let output = stdin(directory.path(), Some("src/template.html.twig"), BEFORE);
    assert!(output.status.success());
    assert_eq!(output.stdout, include_bytes!("fixtures/format_twig/two_spaces.twig"));
    let schema = run(directory.path(), &["config", "--schema", "--show", "formatter"]);
    assert!(schema.status.success());
    let schema: serde_json::Value = serde_json::from_slice(&schema.stdout).unwrap();
    let mut twig = &schema["properties"]["twig"];
    if let Some(reference) = twig["$ref"].as_str() {
        twig = schema.pointer(reference.strip_prefix('#').unwrap()).unwrap();
    }

    for name in ["enabled", "print-width", "tab-width", "use-tabs", "end-of-line"] {
        assert!(twig["properties"][name].is_object(), "Missing Twig setting in schema: {name}");
    }
}

#[test]
fn rejects_unknown_twig_settings() {
    let directory = workspace(include_str!("fixtures/format_twig/unknown_setting.toml"));
    let output = run(directory.path(), &["fmt"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unknown"));
}

#[test]
fn inherits_twig_settings_through_configuration_layers() {
    let directory = workspace(include_str!("fixtures/format_twig/inherits.toml"));
    std::fs::write(directory.path().join("base.toml"), include_str!("fixtures/format_twig/settings.toml")).unwrap();
    let output = run(directory.path(), &["config", "--show", "formatter"]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let config: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(config["twig"]["print-width"], 60);
    assert_eq!(config["twig"]["tab-width"], 2);
    assert_eq!(config["print-width"], 20);
    let output = stdin(directory.path(), Some("src/template.html.twig"), BEFORE);
    assert!(output.status.success());
    assert_eq!(output.stdout, include_bytes!("fixtures/format_twig/two_spaces.twig"));
}

fn git(workspace: &Path, arguments: &[&str]) -> Output {
    let output = Command::new("git").args(arguments).current_dir(workspace).output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    output
}

#[test]
fn formats_staged_twig_without_changing_the_worktree() {
    let directory = workspace(include_str!("fixtures/format_twig/mago.toml"));
    let root = directory.path();
    git(root, &["init", "--quiet"]);
    git(root, &["config", "core.autocrlf", "false"]);
    git(root, &["add", "src/template.html.twig"]);
    let output = run(root, &["fmt", "--staged"]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(git(root, &["show", ":src/template.html.twig"]).stdout, AFTER);
    assert_eq!(contents(root), BEFORE);

    std::fs::write(root.join("src/template.html.twig"), INVALID).unwrap();
    git(root, &["add", "src/template.html.twig"]);
    assert!(!run(root, &["fmt", "--staged"]).status.success());
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
        let output = run(root, &["fmt", "--staged"]);
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        assert_eq!(git(root, &["show", &format!(":{path}")]).stdout, INVALID);
    }
}

#[test]
fn php_tools_do_not_discover_twig_files() {
    let directory = workspace(include_str!("fixtures/format_twig/mago.toml"));
    std::fs::write(directory.path().join("src/template.html.twig"), INVALID).unwrap();
    for command in ["analyze", "lint", "guard"] {
        let output = run(directory.path(), &[command]);
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    }
}

#[test]
fn formatter_file_listing_includes_enabled_twig_files() {
    for (configuration, enabled) in [
        (include_str!("fixtures/format_twig/mago.toml"), true),
        (include_str!("fixtures/format_twig/disabled.toml"), false),
    ] {
        let directory = workspace(configuration);
        let listed = run(directory.path(), &["list-files", "--command", "formatter", "-0"]);
        assert!(listed.status.success(), "{}", String::from_utf8_lossy(&listed.stderr));
        assert_eq!(String::from_utf8_lossy(&listed.stdout).contains("template.html.twig"), enabled);

        for arguments in [
            vec!["list-files"],
            vec!["list-files", "--command", "linter"],
            vec!["list-files", "--command", "analyzer"],
            vec!["list-files", "--command", "guard"],
        ] {
            let listed = run(directory.path(), &arguments);
            assert!(listed.status.success());
            assert!(!String::from_utf8_lossy(&listed.stdout).contains("template.html.twig"));
        }
    }
}

#[test]
fn formatter_file_listing_honors_excludes() {
    let directory = workspace(include_str!("fixtures/format_twig/excludes.toml"));
    for path in ["src/source-excluded.twig", "src/format-excluded.twig"] {
        std::fs::write(directory.path().join(path), BEFORE).unwrap();
    }

    let listed = run(directory.path(), &["list-files", "--command", "formatter"]);
    assert!(listed.status.success(), "{}", String::from_utf8_lossy(&listed.stderr));
    let paths = String::from_utf8_lossy(&listed.stdout);
    assert!(paths.contains("template.html.twig"));
    assert!(!paths.contains("source-excluded.twig"));
    assert!(!paths.contains("format-excluded.twig"));
}

#[test]
fn fix_uses_twig_formatter() {
    let directory = workspace(include_str!("fixtures/format_twig/mago.toml"));
    let output = run(directory.path(), &["fix", "--no-guard", "--no-analyze", "--no-lint"]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(contents(directory.path()), AFTER);
}
