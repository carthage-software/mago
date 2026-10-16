//! Without a configuration, sources Git ignores are dependencies that never shadow the workspace.

use serde_json::Value;
use serde_json::json;

use super::harness::{Harness, client_capabilities};
use crate::language_server::ServerConfig;

const COPY: &str = ".claude/worktrees/copy";

const FOO: &str = "<?php\n\ndeclare(strict_types=1);\n\nnamespace App;\n\nfinal class Foo\n{\n    public function bar(): int\n    {\n        return $this->baz();\n    }\n\n    public function baz(): int\n    {\n        return 1;\n    }\n}\n";
const BAR: &str = "<?php\n\ndeclare(strict_types=1);\n\nnamespace App;\n\nfinal class Bar\n{\n    public function run(Foo $foo): int\n    {\n        return $foo->bar() + $this->other();\n    }\n\n    public function other(): int\n    {\n        return 2;\n    }\n}\n";
const MAIN: &str = "<?php\n\ndeclare(strict_types=1);\n\nnamespace App;\n\nfunction main(Foo $foo, Bar $bar): int\n{\n    return $foo->bar() + $bar->run($foo);\n}\n";
const BROKEN: &str = "<?php\nfunction broken( {}\n";

async fn files_with_diagnostics(h: &mut Harness) -> Vec<String> {
    let mut files: Vec<_> = h
        .diagnostics()
        .await
        .into_iter()
        .filter(|publication| !publication["diagnostics"].as_array().unwrap().is_empty())
        .map(|publication| publication["uri"].as_str().unwrap().to_owned())
        .collect();
    files.sort_unstable();
    files.dedup();
    files
}

fn position(source: &str, needle: &str) -> Value {
    let offset = source.find(needle).unwrap();
    let line = source[..offset].matches('\n').count();
    let character = offset - source[..offset].rfind('\n').map_or(0, |newline| newline + 1);
    json!({ "line": line, "character": character })
}

async fn definition(h: &mut Harness, name: &str, source: &str, needle: &str) -> Value {
    let params = json!({ "textDocument": { "uri": h.url(name) }, "position": position(source, needle) });
    let result = h.request("textDocument/definition", params).await;
    if result.is_array() { result[0].clone() } else { result }
}

#[tokio::test]
async fn git_ignored_copies_do_not_shadow_workspace_declarations() {
    let copy = |name: &str| format!("{COPY}/{name}");
    let files = [
        (".git/HEAD".to_owned(), "ref: refs/heads/main\n"),
        (".gitignore".to_owned(), "/.claude/worktrees/\n"),
        ("src/Foo.php".to_owned(), FOO),
        ("src/Bar.php".to_owned(), BAR),
        ("src/main.php".to_owned(), MAIN),
        (copy("src/Foo.php"), FOO),
        (copy("src/Bar.php"), BAR),
        (copy("src/main.php"), MAIN),
        (copy("src/Broken.php"), BROKEN),
    ];
    let files: Vec<_> = files.iter().map(|(name, contents)| (name.as_str(), *contents)).collect();
    let mut h = Harness::start(&files).await;
    assert_eq!(files_with_diagnostics(&mut h).await, Vec::<String>::new());

    for (name, source, needle, target, declaration) in [
        ("src/Foo.php", FOO, "baz();", "src/Foo.php", "baz(): int"),
        ("src/Bar.php", BAR, "other();", "src/Bar.php", "other(): int"),
        ("src/Bar.php", BAR, "bar() +", "src/Foo.php", "bar(): int"),
        ("src/main.php", MAIN, "bar() +", "src/Foo.php", "bar(): int"),
        ("src/main.php", MAIN, "run($foo)", "src/Bar.php", "run(Foo"),
        ("src/main.php", MAIN, "Foo $foo", "src/Foo.php", "Foo\n"),
    ] {
        let location = definition(&mut h, name, source, needle).await;
        let expected = position(if target == "src/Foo.php" { FOO } else { BAR }, declaration);
        assert_eq!(location["uri"], h.url(target), "{name} {needle}: {location}");
        assert_eq!(location["range"]["start"], expected, "{name} {needle}: {location}");
    }

    let late = copy("src/Late.php");
    std::fs::write(h.workspace.join(&late), BROKEN).unwrap();
    std::fs::write(h.workspace.join("src/Late.php"), BROKEN).unwrap();
    h.watched(&late, 1).await;
    h.watched("src/Late.php", 1).await;
    assert_eq!(files_with_diagnostics(&mut h).await, [h.url("src/Late.php")]);
}

#[tokio::test]
async fn configured_workspaces_keep_git_ignored_sources() {
    let broken = format!("{COPY}/src/Broken.php");
    for config_file in ["mago.toml", "mago.dist.toml", ""] {
        let mut files = vec![
            (".git/HEAD", "ref: refs/heads/main\n"),
            (".gitignore", "/.claude/worktrees/\n"),
            (broken.as_str(), BROKEN),
        ];
        if !config_file.is_empty() {
            files.push((config_file, ""));
        }
        let mut config = ServerConfig::default();
        config.configuration.config_file_is_explicit = config_file.is_empty();
        let mut h = Harness::start_with(&files, config, client_capabilities()).await;
        assert_eq!(files_with_diagnostics(&mut h).await, [h.url(&broken)], "{config_file}");
    }
}

#[tokio::test]
async fn git_ignored_dependencies_keep_inherited_formatter_directory_excludes() {
    const MESSY: &str = "<?php\n\nfunction messy(){return 1;}\n";
    let mut config = ServerConfig::default();
    config.configuration.source.glob.literal_separator = true;
    config.configuration.formatter.excludes = vec!["generated/*".to_owned()];
    let files = [
        (".git/HEAD", "ref: refs/heads/main\n"),
        (".gitignore", "/generated/\n/kept/\n"),
        ("generated/deep/File.php", MESSY),
        ("kept/deep/File.php", MESSY),
        ("src/messy.php", MESSY),
    ];
    let mut h = Harness::start_with(&files, config, client_capabilities()).await;
    for (name, formatted) in [("src/messy.php", true), ("kept/deep/File.php", true), ("generated/deep/File.php", false)]
    {
        let edits = h
            .request(
                "textDocument/formatting",
                json!({ "textDocument": { "uri": h.url(name) }, "options": { "tabSize": 4, "insertSpaces": true } }),
            )
            .await;
        assert_eq!(!edits.is_null(), formatted, "{name}: {edits}");
    }
}
