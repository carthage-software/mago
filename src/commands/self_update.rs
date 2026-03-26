use std::fs;
use std::io::BufRead;
use std::io::Write;
use std::process::ExitCode;

use clap::Parser;
use tempfile::TempDir;
use tracing::debug;
use tracing::info;
use tracing::warn;

use crate::consts::*;
use crate::error::Error;
use crate::updater::error::UpdateError;
use crate::updater::github;
use crate::updater::github::Release;
use crate::updater::github::ReleaseAsset;
use crate::updater::version::is_version_compatible;
use crate::updater::version::is_version_newer;

#[derive(Parser, Debug)]
#[command(
    name = "self-update",
    about = "Check for updates or upgrade Mago to the latest version",
    long_about = r#"
The `self-update` command helps keep Mago up-to-date by checking for and applying the latest updates.

This command ensures you are always using the most recent version of Mago with the latest features and fixes.
"#
)]
pub struct SelfUpdateCommand {
    /// Check for updates but do not install them.
    ///
    /// This option allows you to see if a new version is available without making any changes.
    /// The command will exit with code `0` if you are up-to-date, or `1` if an update is available.
    #[arg(long, short)]
    pub check: bool,

    /// Skip confirmation prompts during updates.
    ///
    /// When this flag is set, the update process will proceed without asking for user confirmation.
    /// Use this option for automated scripts or CI environments where no user interaction is possible.
    #[arg(long)]
    pub no_confirm: bool,

    /// Update to a specific version by providing the version tag.
    ///
    /// This option allows you to specify a particular version of Mago to update to, rather than the latest version.
    /// The version tag should match the format used in the release tags (e.g., `1.0.0-beta.10`).
    /// If the specified version is not found, an error will be returned.
    #[arg(long, value_name = "VERSION_TAG")]
    pub tag: Option<String>,
}

pub fn execute(command: SelfUpdateCommand) -> Result<ExitCode, Error> {
    debug!("OS: {}", std::env::consts::OS);
    debug!("ARCH: {}", std::env::consts::ARCH);
    debug!("TARGET: {}", TARGET);
    debug!("BIN: {}", BIN);
    debug!("ARCHIVE_EXTENSION: {}", ARCHIVE_EXTENSION);

    let release = match &command.tag {
        Some(tag) => {
            info!("Checking version {}... ", tag);
            github::get_release_version(REPO_OWNER, REPO_NAME, tag)?
        }
        None => {
            info!("Checking latest released version... ");
            github::get_latest_release(REPO_OWNER, REPO_NAME)?
        }
    };

    info!("Release found: {} ({})", release.name, release.version);

    if command.tag.is_some() && release.version == VERSION {
        info!("Already up-to-date with version `{}`", VERSION);
        return Ok(ExitCode::SUCCESS);
    }

    if command.tag.is_none() {
        if !is_version_newer(VERSION, &release.version)? {
            info!("Already up-to-date with the latest version `{}`", VERSION);
            return Ok(ExitCode::SUCCESS);
        }

        info!("New release found! {} --> {}", VERSION, release.version);
        if !is_version_compatible(VERSION, &release.version)? {
            warn!("New release is not compatible with the current version.");
        }
    }

    if command.check {
        return Ok(ExitCode::FAILURE);
    }

    let target_asset = get_target_asset_from_release(&release)?;

    debug!("Target asset: {:?}", target_asset.name);
    debug!("Download URL: {:?}", target_asset.download_url);
    info!("The new release will be downloaded/extracted and the existing binary will be replaced.");

    if !command.no_confirm {
        confirm_prompt("Do you want to continue? [Y/n] ")?;
    }

    perform_update(&release, target_asset)?;

    info!("Successfully updated to version `{}`", release.version);

    Ok(ExitCode::SUCCESS)
}

fn perform_update(release: &Release, target_asset: &ReleaseAsset) -> Result<(), UpdateError> {
    let tmp_dir = TempDir::new()?;
    let tmp_archive_path = tmp_dir.path().join(&target_asset.name);
    let mut tmp_archive = fs::File::create(&tmp_archive_path)?;

    info!("Downloading archive...");
    github::download_asset(target_asset, &mut tmp_archive)?;

    debug!("Downloaded archive to: {:?}", tmp_archive_path);

    let binary_path = format!("{BIN}-{}-{TARGET}/{BIN}", release.version);

    info!("Extracting archive...");
    crate::updater::archive::extract_file(&tmp_archive_path, &binary_path, tmp_dir.path())?;

    let new_executable = tmp_dir.path().join(&binary_path);
    debug!("Extracted binary to: {:?}", new_executable);

    info!("Replacing current executable...");
    self_replace::self_replace(&new_executable)?;

    Ok(())
}

fn confirm_prompt(msg: &str) -> Result<(), UpdateError> {
    let mut stdout = std::io::stdout().lock();
    let mut stdin = std::io::stdin().lock();

    stdout.write_all(b"\n")?;
    stdout.write_all(b"> ")?;
    stdout.write_all(msg.as_bytes())?;
    stdout.flush()?;

    let mut s = String::new();
    stdin.read_line(&mut s)?;
    let s = s.trim().to_lowercase();
    if !s.is_empty() && s != "y" {
        return Err(UpdateError::Update("User cancelled the update".to_string()));
    }

    stdout.write_all(b"\n")?;

    Ok(())
}

fn get_target_asset_from_release(release: &Release) -> Result<&ReleaseAsset, UpdateError> {
    release
        .assets
        .iter()
        .find(|asset| asset.name.contains(TARGET) && asset.name.ends_with(ARCHIVE_EXTENSION))
        .ok_or_else(|| {
            let binary_asset_count =
                release.assets.iter().filter(|a| SUPPORTED_TARGETS.iter().any(|t| a.name.contains(t))).count();

            let message = if !SUPPORTED_TARGETS.contains(&TARGET) {
                format!(
                    "No pre-built binary is available for your platform `{TARGET}`. \
                     You can compile Mago from source by cloning the repository and running `cargo install --path .`.",
                )
            } else if binary_asset_count == 0 {
                format!(
                    "No binaries are available for release `{}` yet. \
                     The release was just published and binaries are still being built by CI. \
                     This typically takes 30-40 minutes. Please try again shortly.",
                    release.version,
                )
            } else {
                format!(
                    "The binary for your platform `{TARGET}` is not available in release `{}` yet. \
                     The release has {}/{} platform builds ready. \
                     The remaining builds are likely still in progress - please try again in a few minutes.",
                    release.version,
                    binary_asset_count,
                    SUPPORTED_TARGETS.len(),
                )
            };

            UpdateError::Release(message)
        })
}
