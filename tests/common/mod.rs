#![allow(dead_code)]

use std::borrow::Cow;
use std::path::PathBuf;
use std::process::Command;

use mago_database::DatabaseConfiguration;
use mago_database::GlobSettings;

pub fn php_sdk_is_available(repository: &std::path::Path, test: &str) -> bool {
    let available = repository.join("vendor/autoload.php").is_file()
        && Command::new("php").arg("--version").output().is_ok_and(|output| output.status.success());
    assert!(
        available || std::env::var_os("MAGO_REQUIRE_PHP_SDK_TESTS").is_none(),
        "PHP and vendor dependencies are required for {test}"
    );
    available
}

pub fn database_configuration(
    workspace: impl Into<PathBuf>,
    paths: Vec<Cow<'static, [u8]>>,
) -> DatabaseConfiguration<'static> {
    DatabaseConfiguration {
        workspace: Cow::Owned(workspace.into()),
        paths,
        includes: Vec::new(),
        patches: Vec::new(),
        excludes: Vec::new(),
        extensions: vec![Cow::Borrowed(b"php")],
        glob: GlobSettings::default(),
    }
}
