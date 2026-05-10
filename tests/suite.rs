#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::use_debug,
    clippy::panic_in_result_fn,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc
)]

use std::env;
use std::fmt::Write as _;
use std::path::Path;
use std::process::Command;

use libtest_mimic::Failed;
use libtest_mimic::Trial;
use serde::Deserialize;

#[derive(Deserialize)]
struct VerifyOutput {
    new_issues: Vec<ChangeEntry>,
    removed_issues: Vec<ChangeEntry>,
}

#[derive(Deserialize)]
struct ChangeEntry {
    file: String,
    code: String,
    start_line: u32,
    end_line: u32,
}

fn invoke(bin: &Path, cmd: &str, dir: &Path, name: &str, extra_args: &[&str]) -> std::process::Output {
    Command::new(bin)
        .args(["--no-version-check", "--colors", "never", "--threads", "4", cmd])
        .args(extra_args)
        .current_dir(dir)
        .output()
        .unwrap_or_else(|e| panic!("[{name}] failed to spawn `mago {cmd}`: {e}"))
}

fn generate_baseline(bin: &Path, cmd: &str, dir: &Path, name: &str) {
    let output = invoke(bin, cmd, dir, name, &["--generate-baseline"]);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("[{name}] `mago {cmd} --generate-baseline` failed:\n{stderr}");
    }
}

fn verify_baseline(bin: &Path, cmd: &str, dir: &Path, name: &str) -> Result<(), String> {
    let output = invoke(bin, cmd, dir, name, &["--reporting-format", "json", "--verify-baseline"]);
    if output.status.success() {
        return Ok(());
    }

    let stdout = output.stdout.trim_ascii();
    if stdout.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("  [{cmd}] baseline verification failed:\n{stderr}"));
    }

    let verify: VerifyOutput = serde_json::from_slice(stdout).unwrap_or_else(|e| {
        let raw = String::from_utf8_lossy(stdout);
        panic!("[{name}] failed to parse `mago {cmd}` verify output: {e}\nstdout: {raw}");
    });

    let mut msg = String::new();
    for e in &verify.new_issues {
        let _ =
            writeln!(msg, "  [{cmd}] {}: unexpected `{}` at lines {}..{}", e.file, e.code, e.start_line, e.end_line);
    }
    for e in &verify.removed_issues {
        let _ = writeln!(
            msg,
            "  [{cmd}] {}: expected `{}` at lines {}..{}, not found",
            e.file, e.code, e.start_line, e.end_line
        );
    }

    Err(msg)
}

fn run(bin: &Path, dir: &Path) -> Result<(), String> {
    let name = dir.file_name().unwrap().to_str().unwrap();

    if !dir.join("mago.toml").exists() {
        panic!("[{name}] missing required mago.toml");
    }

    if env::var("MAGO_UPDATE_SNAPSHOTS").is_ok() {
        generate_baseline(bin, "analyze", dir, name);
        generate_baseline(bin, "lint", dir, name);
        return Ok(());
    }

    let analyze1 = verify_baseline(bin, "analyze", dir, name);
    let lint1 = verify_baseline(bin, "lint", dir, name);

    // If either first run fails it's a correctness issue, not stability — report and stop.
    if analyze1.is_err() || lint1.is_err() {
        let mut msg = String::new();
        if let Err(e) = analyze1 {
            msg.push_str(&e);
        }
        if let Err(e) = lint1 {
            msg.push_str(&e);
        }
        return Err(msg);
    }

    // Both first runs passed — run again to catch non-determinism.
    let mut msg = String::new();
    if let Err(e) = verify_baseline(bin, "analyze", dir, name) {
        msg.push_str(&e);
    }
    if let Err(e) = verify_baseline(bin, "lint", dir, name) {
        msg.push_str(&e);
    }
    if msg.is_empty() { Ok(()) } else { Err(msg) }
}

fn main() {
    let args = libtest_mimic::Arguments::from_args();
    let cases_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("cases");
    let bin = Path::new(env!("CARGO_BIN_EXE_mago"));

    let mut dirs: Vec<_> = std::fs::read_dir(&cases_dir)
        .unwrap_or_else(|e| panic!("cannot read cases dir {}: {e}", cases_dir.display()))
        .filter_map(|e| {
            let e = e.unwrap();
            e.file_type().unwrap().is_dir().then(|| e.path())
        })
        .collect();
    dirs.sort();

    let trials: Vec<_> = dirs
        .into_iter()
        .map(|dir| {
            let bin = bin.to_owned();
            let name = dir.file_name().unwrap().to_str().unwrap().to_owned();
            Trial::test(name, move || run(&bin, &dir).map_err(Failed::from))
        })
        .collect();

    libtest_mimic::run(&args, trials).exit();
}
