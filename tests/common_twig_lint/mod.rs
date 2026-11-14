use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::process::Output;
use std::process::Stdio;

pub fn workspace(configuration: &str) -> tempfile::TempDir {
    let directory = tempfile::tempdir().unwrap();
    std::fs::create_dir(directory.path().join("src")).unwrap();
    std::fs::write(directory.path().join("modern.toml"), include_bytes!("../fixtures/lint_twig/modern.toml")).unwrap();
    std::fs::write(
        directory.path().join("mago.toml"),
        format!("extends='modern.toml'\n[source]\npaths = ['src']\n{configuration}"),
    )
    .unwrap();
    std::fs::write(directory.path().join("src/template.twig"), include_bytes!("../fixtures/lint_twig/before.twig"))
        .unwrap();
    std::fs::write(directory.path().join("src/main.php"), b"<?php\nfunction main(): void {}\n").unwrap();
    directory
}

pub fn command(workspace: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mago"));
    command.args(["--no-version-check", "--colors", "never", "--threads", "2"]).current_dir(workspace);
    command
}

pub fn run(workspace: &Path, arguments: &[&str]) -> Output {
    command(workspace).args(arguments).output().unwrap()
}

pub fn issues(workspace: &Path, arguments: &[&str]) -> serde_json::Value {
    let output = command(workspace).args(["twig:lint", "--reporting-format", "json"]).args(arguments).output().unwrap();
    serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap_or_else(|error| {
        panic!("{error}\n{}\n{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr))
    })["issues"]
        .clone()
}

pub fn stdin(workspace: &Path, path: &str, content: &[u8]) -> Output {
    let mut child = command(workspace)
        .args(["twig:lint", "--stdin-input", path, "--only", "twig-no-dump", "--reporting-format", "json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(content).unwrap();
    child.wait_with_output().unwrap()
}
