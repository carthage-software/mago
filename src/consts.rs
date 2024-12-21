use std::path::PathBuf;
use std::sync::LazyLock;

use num_cpus::get as get_logical_cpus;

use mago_feedback::error;

/// The current version of mago.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The target triple for the current build.
pub const TARGET: &str = env!("TARGET");

/// The name of the binary.
pub const BIN: &str = env!("CARGO_PKG_NAME");

/// The name of the repository owner.
pub const REPO_OWNER: &str = "carthage-software";

/// The name of the repository.
pub const REPO_NAME: &str = "mago";

/// The URL for creating new issues.
pub const ISSUE_URL: &str = "https://github.com/carthage-software/mago/issues/new";

/// The name of the environment variable prefix for mago.
pub const ENVIRONMENT_PREFIX: &str = "MAGO";

/// The name of the configuration file for mago.
pub const CONFIGURATION_FILE: &str = "mago";

/// The minimum stack size for each thread.
pub const MINIMUM_STACK_SIZE: usize = 8 * 1024 * 1024;

/// The default stack size for each thread.
pub const DEFAULT_STACK_SIZE: usize = 36 * 1024 * 1024;

/// The maximum stack size for each thread.
pub const MAXIMUM_STACK_SIZE: usize = 256 * 1024 * 1024;

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

include!("../stubs/map.rs");
