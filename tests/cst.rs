use std::path::Path;
use std::process::Command;
use std::process::Output;

fn run(file: &str, arguments: &[&str]) -> Output {
    let workspace = tempfile::tempdir().unwrap();
    std::fs::write(workspace.path().join("mago.toml"), "").unwrap();

    Command::new(env!("CARGO_BIN_EXE_mago"))
        .args(["--no-version-check", "--colors", "never", "cst"])
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cst").join(file))
        .args(["--reporting-target", "stderr", "--reporting-format", "short"])
        .args(arguments)
        .env("MAGO_LOG", "off")
        .current_dir(workspace.path())
        .output()
        .unwrap()
}

#[test]
fn valid_input_succeeds() {
    for arguments in [&[][..], &["--json"], &["--names"], &["--tokens"]] {
        let output = run("valid.php", arguments);
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        assert!(!output.stdout.is_empty());
    }
}

#[test]
fn parse_errors_fail_without_hiding_output() {
    for (arguments, expected) in [(&[][..], "Program"), (&["--json"], "\"program\""), (&["--names"], "Resolved Names")]
    {
        let output = run("invalid.php", arguments);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(1), "{stderr}");
        assert!(stderr.contains("invalid.php"), "{stderr}");
        assert!(String::from_utf8_lossy(&output.stdout).contains(expected));
    }
}
