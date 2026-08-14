#![allow(clippy::expect_used, clippy::missing_panics_doc)]

use std::collections::HashSet;
use std::num::NonZeroUsize;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use mago_extension::WorkerCommand;
use mago_extension::WorkerPool;
use mago_extension::WorkerPoolOptions;
use mago_linter::external::ExternalLinter;
use mago_php_version::PHPVersion;

#[test]
fn worker_reducers_merge_every_process_on_the_last_survivor() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var_os("MAGO_TRACE_PHP_SDK_TESTS").is_some() {
        let _subscriber = tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .with_test_writer()
            .try_init();
    }

    let repository = Path::new(env!("CARGO_MANIFEST_DIR"));
    let sdk_available = repository.join("vendor/autoload.php").is_file()
        && Command::new("php").arg("--version").output().is_ok_and(|output| output.status.success());
    assert!(
        sdk_available || std::env::var_os("MAGO_REQUIRE_PHP_SDK_TESTS").is_none(),
        "PHP and vendor dependencies are required for the external worker-reduction test"
    );
    if !sdk_available {
        return Ok(());
    }

    let temporary = tempfile::tempdir()?;
    let command = WorkerCommand::new("php")
        .with_argument(repository.join("composer/tests/Sdk/Fixtures/worker-reducer.php"))
        .with_current_directory(repository)
        .with_environment("MAGO_REDUCTION_AUDIT_DIRECTORY", temporary.path());
    let pool = Arc::new(WorkerPool::spawn(
        command,
        NonZeroUsize::new(3).expect("three is non-zero"),
        WorkerPoolOptions::default(),
    )?);
    let linter = ExternalLinter::initialize([Arc::clone(&pool)], PHPVersion::PHP85)?;
    drop(linter);
    pool.shutdown();

    let alpha = read_audit(&temporary.path().join("alpha.txt"), "alpha")?;
    let beta = read_audit(&temporary.path().join("beta.txt"), "beta")?;
    assert_eq!(alpha.worker_ids, beta.worker_ids, "logical extensions must observe the same worker order");
    assert_eq!(alpha.worker_ids.len(), 3);
    assert_eq!(alpha.worker_ids.iter().collect::<HashSet<_>>().len(), 3);
    assert!(alpha.worker_ids.contains(&alpha.leader), "the survivor's own contribution must be included");
    assert!(beta.worker_ids.contains(&beta.leader), "the survivor's own contribution must be included");
    assert_eq!(alpha.leader, beta.leader, "all reducers must run on the same surviving process");
    assert!(pool.is_empty());

    Ok(())
}

#[test]
fn adaptive_reduction_includes_only_workers_that_were_spawned() -> Result<(), Box<dyn std::error::Error>> {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"));
    let sdk_available = repository.join("vendor/autoload.php").is_file()
        && Command::new("php").arg("--version").output().is_ok_and(|output| output.status.success());
    assert!(
        sdk_available || std::env::var_os("MAGO_REQUIRE_PHP_SDK_TESTS").is_none(),
        "PHP and vendor dependencies are required for the external worker-reduction test"
    );
    if !sdk_available {
        return Ok(());
    }

    let temporary = tempfile::tempdir()?;
    let command = WorkerCommand::new("php")
        .with_argument(repository.join("composer/tests/Sdk/Fixtures/worker-reducer.php"))
        .with_current_directory(repository)
        .with_environment("MAGO_REDUCTION_AUDIT_DIRECTORY", temporary.path());
    let pool = Arc::new(WorkerPool::spawn_adaptive(
        command,
        NonZeroUsize::new(5).expect("five is non-zero"),
        WorkerPoolOptions::default(),
    )?);
    let linter = ExternalLinter::initialize([Arc::clone(&pool)], PHPVersion::PHP85)?;
    drop(linter);
    pool.shutdown();

    let alpha = read_audit(&temporary.path().join("alpha.txt"), "alpha")?;
    let beta = read_audit(&temporary.path().join("beta.txt"), "beta")?;
    assert_eq!(alpha.worker_ids.len(), 2, "unspawned adaptive capacity must not contribute empty data");
    assert_eq!(alpha.worker_ids, beta.worker_ids);
    assert_eq!(alpha.leader, beta.leader);

    Ok(())
}

struct Audit {
    leader: String,
    worker_ids: Vec<String>,
}

fn read_audit(path: &Path, identifier: &str) -> Result<Audit, Box<dyn std::error::Error>> {
    let contents = std::fs::read_to_string(path)?;
    let lines = contents.lines().collect::<Vec<_>>();
    assert_eq!(lines.first(), Some(&identifier));
    assert_eq!(lines.get(2), Some(&"active"));
    let worker_ids = lines[3..]
        .iter()
        .map(|payload| {
            payload
                .strip_prefix(identifier)
                .and_then(|payload| payload.strip_prefix(':'))
                .expect("payload must retain its extension identity")
                .to_string()
        })
        .collect();

    Ok(Audit { leader: lines[1].to_string(), worker_ids })
}
