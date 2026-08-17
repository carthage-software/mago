#![allow(clippy::expect_used, clippy::missing_panics_doc, clippy::unwrap_used)]

use std::borrow::Cow;
use std::num::NonZeroUsize;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use mago_allocator::LocalArena;
use mago_database::file::File;
use mago_extension::WorkerCommand;
use mago_extension::WorkerPool;
use mago_extension::WorkerPoolOptions;
use mago_linter::Linter;
use mago_linter::external::ExternalLinter;
use mago_linter::settings::Settings;
use mago_names::resolver::NameResolver;
use mago_php_version::PHPVersion;
use mago_syntax::parser::parse_file;
use mago_text_edit::Safety;

#[test]
fn external_linter_issue_preserves_suggested_edits() -> Result<(), Box<dyn std::error::Error>> {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"));
    let sdk_available = repository.join("vendor/autoload.php").is_file()
        && Command::new("php").arg("--version").output().is_ok_and(|output| output.status.success());
    assert!(
        sdk_available || std::env::var_os("MAGO_REQUIRE_PHP_SDK_TESTS").is_none(),
        "PHP and vendor dependencies are required for the external linter SDK test"
    );
    if !sdk_available {
        return Ok(());
    }

    let command = WorkerCommand::new("php")
        .with_argument(repository.join("composer/tests/Sdk/worker.php"))
        .with_current_directory(repository);
    let pool = WorkerPool::spawn(command, NonZeroUsize::MIN, WorkerPoolOptions::default())?;
    let external = ExternalLinter::initialize([Arc::new(pool)], PHPVersion::PHP85)?;

    let source = b"<?php\n\nPsl\\Iter\\any([1], static fn(int $value): bool => $value > 0);\n";
    let file = File::ephemeral(Cow::Borrowed(b"src/example.php"), Cow::Borrowed(source));
    let arena = LocalArena::new();
    let program = parse_file(&arena, &file);
    let resolved_names = NameResolver::new(&arena).resolve(program);
    let settings = Settings { php_version: PHPVersion::PHP85, ..Settings::default() };
    let only = vec!["mago-sdk-test/prefer-array-any".to_owned()];
    let linter = Linter::new(&arena, &settings, Some(&only), false);

    let issues = linter.lint_with_external(&file, program, &resolved_names, &external)?;
    let issue = issues
        .iter()
        .find(|issue| issue.code.as_deref() == Some("mago-sdk-test/prefer-array-any"))
        .expect("the PHP rule should report its issue");
    let edits = issue.edits.get(&file.id).expect("the PHP rule should attach one edit batch");
    assert_eq!(edits.len(), 1);
    assert_eq!(edits[0].new_text, b"array_any");
    assert_eq!(edits[0].safety, Safety::Safe);
    assert_eq!(&source[edits[0].range.start as usize..edits[0].range.end as usize], b"Psl\\Iter\\any");

    Ok(())
}
