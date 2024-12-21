use std::process::ExitCode;

use clap::Parser;

use mago_feedback::info;
use self_update::backends::github::Update;
use self_update::Status;

use crate::consts::*;
use crate::error::Error;

#[derive(Parser, Debug)]
#[command(
    name = "self-update",
    about = "Check for updates or update Mago to the latest version",
    long_about = r#"The `self-update` command allows you to update Mago to the latest version
or check if updates are available. By default, it fetches the latest release
from GitHub and replaces the current binary with the updated version.

You can also specify additional options like downloading a specific version
or suppressing output."#
)]
pub struct SelfUpdateCommand {
    #[arg(long, short, help = "Check for updates but do not install them")]
    pub check: bool,

    #[arg(long, help = "Do not show progress bars")]
    pub no_progress: bool,

    #[arg(long, help = "Do not ask for confirmation")]
    pub no_confirm: bool,

    #[arg(long, help = "Toggle update output information")]
    pub no_output: bool,

    #[arg(long, help = "Update to a specific version")]
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
