use std::process::ExitCode;

use clap::Parser;
use self_update::backends::github::Update;
use self_update::Status;

use mago_feedback::info;

use crate::consts::*;
use crate::error::Error;

#[derive(Parser, Debug)]
#[command(
    name = "self-update",
    about = "check for updates or upgrade Mago to the latest version",
    long_about = r#"
The `self-update` command helps keep Mago up-to-date by checking for and applying the latest updates.

This command ensures you are always using the most recent version of Mago with the latest features and fixes.
"#
)]
pub struct SelfUpdateCommand {
    /// Check for updates but do not install them.
    #[arg(long, short, help = "check for updates without installing them")]
    pub check: bool,

    /// Disable progress bars during the update process.
    #[arg(long, help = "disable progress bars")]
    pub no_progress: bool,

    /// Skip confirmation prompts during updates.
    #[arg(long, help = "skip confirmation prompts")]
    pub no_confirm: bool,

    /// Suppress update output information.
    #[arg(long, help = "suppress output information during the update process")]
    pub no_output: bool,

    /// Update to a specific version by providing the version tag.
    #[arg(long, help = "update to a specific version", value_name = "VERSION_TAG")]
    pub tag: Option<String>,
}

pub fn execute(command: SelfUpdateCommand) -> Result<ExitCode, Error> {
    info!("Current version: {}", VERSION);

    let mut status_builder = Update::configure();
    status_builder
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .target(TARGET)
        .bin_name(BIN)
        .current_version(VERSION)
        .bin_path_in_archive("{{ bin }}-{{ version }}-{{ target }}/{{ bin }}")
        .show_download_progress(!command.no_progress)
        .show_output(!command.no_output)
        .no_confirm(command.no_confirm);

    if let Some(tag) = command.tag {
        status_builder.target_version_tag(&tag);
    }

    let status = status_builder.build()?;

    if command.check {
        let release = status.get_latest_release()?;

        info!("Latest release: {}", release.version);
        info!("Date: {}", release.date);

        return Ok(ExitCode::SUCCESS);
    }

    let status = status.update()?;

    match status {
        Status::UpToDate(latest_version) => {
            info!("Already up-to-date with the latest version {}", latest_version);
        }
        Status::Updated(latest_version) => {
            info!("Updated to version {}", latest_version);
        }
    }

    Ok(ExitCode::SUCCESS)
}
