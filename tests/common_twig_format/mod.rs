#![allow(dead_code)]

use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::process::Output;
use std::process::Stdio;

pub const BEFORE: &[u8] = include_bytes!("../fixtures/format_twig/before.twig");
pub const AFTER: &[u8] = include_bytes!("../fixtures/format_twig/after.twig");
pub const PHP: &[u8] = include_bytes!("../fixtures/format_twig/unchanged.php");
pub const PHP_BEFORE: &[u8] = include_bytes!("../fixtures/format_twig/php_before.php");
pub const INVALID: &[u8] = include_bytes!("../fixtures/format_twig/invalid.twig");
pub const CONFIG: &str = include_str!("../fixtures/format_twig/mago.toml");

pub fn workspace(configuration: &str) -> tempfile::TempDir {
    let directory = tempfile::tempdir().unwrap();
    std::fs::create_dir(directory.path().join("src")).unwrap();
    std::fs::write(directory.path().join("mago.toml"), configuration).unwrap();
    std::fs::write(directory.path().join("src/template.html.twig"), BEFORE).unwrap();
    std::fs::write(directory.path().join("src/unchanged.php"), PHP_BEFORE).unwrap();
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

pub fn stdin(workspace: &Path, filepath: Option<&str>, content: &[u8]) -> Output {
    stdin_for(workspace, "twig:fmt", filepath, content)
}

pub fn stdin_for(workspace: &Path, tool: &str, filepath: Option<&str>, content: &[u8]) -> Output {
    let mut command = command(workspace);
    command.args([tool, "--stdin-input"]);
    if let Some(filepath) = filepath {
        command.args(["--stdin-filepath", filepath]);
    }
    let mut child = command.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    child.stdin.take().unwrap().write_all(content).unwrap();
    child.wait_with_output().unwrap()
}

pub fn contents(workspace: &Path) -> Vec<u8> {
    std::fs::read(workspace.join("src/template.html.twig")).unwrap()
}

pub fn git(workspace: &Path, arguments: &[&str]) -> Output {
    let output = Command::new("git").args(arguments).current_dir(workspace).output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    output
}

pub fn json(workspace: &Path, arguments: &[&str]) -> serde_json::Value {
    let output = run(workspace, arguments);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    serde_json::from_slice(&output.stdout).unwrap()
}
