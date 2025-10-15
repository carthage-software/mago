use std::path::PathBuf;
use std::sync::LazyLock;

use num_cpus::get as get_logical_cpus;
use tracing::error;

use mago_php_version::PHPVersion;

/// The current version of mago.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The target triple for the current build.
pub const TARGET: &str = env!("TARGET");

/// The name of the binary.
pub const BIN: &str = env!("CARGO_PKG_NAME");

/// The extension for the archive file for the current platform.
#[cfg(target_os = "windows")]
pub const ARCHIVE_EXTENSION: &str = "zip";
#[cfg(not(target_os = "windows"))]
pub const ARCHIVE_EXTENSION: &str = "tar.gz";

/// The extension for PHP files.
pub const PHP_EXTENSION: &str = "php";

/// The name of the repository owner.
pub const REPO_OWNER: &str = "carthage-software";

/// The name of the repository.
pub const REPO_NAME: &str = "mago";

/// The URL for creating new issues.
pub const ISSUE_URL: &str = "https://github.com/carthage-software/mago/issues/new";

/// The name of the environment variable prefix for mago.
pub const ENVIRONMENT_PREFIX: &str = "MAGO";

/// The name of the configuration file for mago.
pub const CONFIGURATION_FILE: &str = "mago.toml";

/// The name of `composer.json` file.
pub const COMPOSER_JSON_FILE: &str = "composer.json";

/// The minimum stack size for each thread (8 MB).
pub const MINIMUM_STACK_SIZE: usize = 8 * 1024 * 1024;

/// The default stack size for each thread (12 MB).
pub const DEFAULT_STACK_SIZE: usize = 12 * 1024 * 1024;

/// The maximum stack size for each thread (256 MB).
pub const MAXIMUM_STACK_SIZE: usize = 256 * 1024 * 1024;

/// The default php version.
pub const DEFAULT_PHP_VERSION: PHPVersion = PHPVersion::LATEST;

/// The minimum supported PHP version.
pub const MINIMUM_PHP_VERSION: PHPVersion = PHPVersion::PHP72;

/// The maximum supported PHP version.
pub const MAXIMUM_PHP_VERSION: PHPVersion = PHPVersion::NEXT;

/// The number of logical CPUs on the system.
pub static LOGICAL_CPUS: LazyLock<usize> = LazyLock::new(get_logical_cpus);

/// The current working directory.
pub static CURRENT_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    std::env::current_dir().unwrap_or_else(|e| {
        error!("Failed to get the current working directory: {}", e);
        error!("This might occur if the directory has been deleted or if the process lacks the necessary permissions.");
        error!("Please ensure that the directory exists and that you have the required permissions to access it.");
        error!("Need help? Open an issue at {}.", ISSUE_URL);

        std::process::exit(1);
    })
});

pub const PRELUDE_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/prelude.bin"));
