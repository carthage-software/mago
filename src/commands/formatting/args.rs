use std::path::PathBuf;

use clap::Parser;

/// File selection and output modes shared by formatter commands.
#[derive(Parser, Debug, Default)]
pub struct FormatArgs {
    /// Files or directories to format instead of the configured source paths.
    #[arg()]
    pub path: Vec<PathBuf>,

    /// Print a diff without writing files.
    #[arg(long, short = 'd', conflicts_with_all = ["check", "stdin_input"], alias = "diff")]
    pub dry_run: bool,

    /// Fail if files need formatting, without writing them.
    #[arg(long, short = 'c', conflicts_with_all = ["dry_run", "stdin_input"])]
    pub check: bool,

    /// Read stdin and write the formatted bytes to stdout.
    #[arg(long, short = 'i', conflicts_with_all = ["dry_run", "check", "path", "staged"])]
    pub stdin_input: bool,

    /// Use this buffer path for exclusions and diagnostics, not language selection.
    #[arg(long, requires = "stdin_input", value_name = "PATH")]
    pub stdin_filepath: Option<PathBuf>,

    /// Format matching staged files in the Git index without changing working copies.
    #[arg(long, short = 's', conflicts_with_all = ["dry_run", "check", "stdin_input", "path"])]
    pub staged: bool,
}
