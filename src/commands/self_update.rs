use std::env::consts::ARCH;
use std::env::consts::OS;
use std::fs;
use std::io::BufRead;
use std::io::Write;
use std::process::ExitCode;

use clap::Parser;
use self_update::Download;
use self_update::Extract;
use self_update::backends::github::Update;
use self_update::errors::Error as SelfUpdateError;
use self_update::self_replace;
use self_update::update::Release;
use self_update::update::ReleaseAsset;
use self_update::update::ReleaseUpdate;
use self_update::update::UpdateStatus;
use self_update::version::bump_is_compatible;
use self_update::version::bump_is_greater;
use tempfile::TempDir;
use tracing::debug;
use tracing::info;
use tracing::warn;

use crate::consts::*;
use crate::error::Error;

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
    let mut status_builder = Update::configure();
    status_builder
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .target(TARGET)
        .bin_name(BIN)
        .current_version(VERSION)
        .bin_path_in_archive("{{ bin }}-{{ version }}-{{ target }}/{{ bin }}")
        .no_confirm(command.no_confirm);

    if let Some(tag) = command.tag {
        status_builder.target_version_tag(&tag);
    }

    let release_update = status_builder.build()?;

    debug!("OS: {}", OS);
    debug!("ARCH: {}", ARCH);
    debug!("TARGET: {}", TARGET);
    debug!("BIN: {}", BIN);
    debug!("ARCHIVE_EXTENSION: {}", ARCHIVE_EXTENSION);
    debug!("CURRENT EXECUTABLE: {:?}", release_update.bin_install_path());

    if command.check {
        return Ok(match release_update.target_version() {
            None => {
                info!("Checking latest released version... ");
                let latest_release = release_update.get_latest_release()?;
                info!("Latest release: {}", latest_release.version);
                if !bump_is_greater(VERSION, &latest_release.version)? {
                    info!("Already up-to-date with the latest version `{}`", VERSION);

                    ExitCode::SUCCESS
                } else {
                    info!("New release found! {} --> {}", VERSION, latest_release.version);
                    if !bump_is_compatible(VERSION, &latest_release.version)? {
                        warn!("New release is not compatible with the current version.");
                    }

                    ExitCode::FAILURE
                }
            }
            Some(ref version) => {
                info!("Checking version {}... ", version);
                let version_release = release_update.get_release_version(version)?;
                info!("Release found for version: {}", version);
                if version_release.version == VERSION {
                    info!("Already up-to-date with the latest version `{}`", VERSION);

                    ExitCode::SUCCESS
                } else {
                    ExitCode::FAILURE
                }
            }
        });
    }

    let status = perform_update(release_update)?;

    match status {
        UpdateStatus::UpToDate => {
            info!("Already up-to-date with the latest version `{}`", VERSION);
        }
        UpdateStatus::Updated(release) => {
            info!("Successfully updated to version `{}`", release.version);
        }
    }

    Ok(ExitCode::SUCCESS)
}

fn perform_update(release_update: Box<dyn ReleaseUpdate>) -> Result<UpdateStatus, Error> {
    info!("Starting the update process for Mago. Current version: `{}`. Target platform: `{}`.", VERSION, TARGET);

    let release = match release_update.target_version() {
        None => {
            info!("Checking latest released version... ");
            let latest_release = release_update.get_latest_release()?;
            info!("Latest release: {}", latest_release.version);
            if !bump_is_greater(VERSION, &latest_release.version)? {
                return Ok(UpdateStatus::UpToDate);
            }

            info!("New release found! {} --> {}", VERSION, latest_release.version);
            if !bump_is_compatible(VERSION, &latest_release.version)? {
                warn!("New release is not compatible with the current version.");
            }

            latest_release
        }
        Some(ref version) => {
            info!("Checking version {}... ", version);
            let version_release = release_update.get_release_version(version)?;
            info!("Release found for version: {}", version);
            if version_release.version == VERSION {
                return Ok(UpdateStatus::UpToDate);
            }

            version_release
        }
    };

    let target_asset = get_target_asset_from_release(&release)?;

    debug!("Mago release status:");
    debug!("- New release asset name: {:?}", target_asset.name);
    debug!("- New release asset download URL: {:?}", target_asset.download_url);
    info!("The new release will be downloaded/extracted and the existing binary will be replaced.");
    if !release_update.no_confirm() {
        confirm_prompt("Do you want to continue? [Y/n] ")?;
    }

    let tmp_archive_dir = TempDir::new().map_err(SelfUpdateError::from)?;
    let tmp_archive_path = tmp_archive_dir.path().join(&target_asset.name);
    let mut tmp_archive = fs::File::create(&tmp_archive_path).map_err(SelfUpdateError::from)?;

    info!("Downloading archive...");
    let mut download = Download::from_url(&target_asset.download_url);
    let mut headers = release_update.api_headers(&release_update.auth_token())?;
    headers.insert("Accept", "application/octet-stream".parse().unwrap());
    download.set_headers(headers);
    download.download_to(&mut tmp_archive)?;

    debug!("Downloaded archive to: {:?}", tmp_archive_path);

    let binary_path = release_update
        .bin_path_in_archive()
        .replace("{{ version }}", &release.version)
        .replace("{{ target }}", TARGET)
        .replace("{{ bin }}", BIN);

    info!("Extracting archive...");
    Extract::from_source(&tmp_archive_path).extract_file(tmp_archive_dir.path(), &binary_path)?;

    let new_executable = tmp_archive_dir.path().join(binary_path);
    debug!("Extracted binary to: {:?}", new_executable);
    info!("Replacing current executable...");
    self_replace::self_replace(new_executable).map_err(SelfUpdateError::from)?;
    info!("Update complete!");

    Ok(UpdateStatus::Updated(release))
}

fn confirm_prompt(msg: &str) -> Result<(), Error> {
    let mut stdout = std::io::stdout().lock();
    let mut stdin = std::io::stdin().lock();

    stdout.write_all("\n".as_bytes()).map_err(SelfUpdateError::from)?;
    stdout.write_all(b"> ").map_err(SelfUpdateError::from)?;
    stdout.write_all(msg.as_bytes()).map_err(SelfUpdateError::from)?;
    stdout.flush().map_err(SelfUpdateError::from)?;

    let mut s = String::new();
    stdin.read_line(&mut s).map_err(SelfUpdateError::from)?;
    let s = s.trim().to_lowercase();
    if !s.is_empty() && s != "y" {
        return Err(Error::SelfUpdate(SelfUpdateError::Update("User cancelled the update".to_string())));
    }

    stdout.write_all("\n".as_bytes()).map_err(SelfUpdateError::from)?;

    Ok(())
}

fn get_target_asset_from_release(release: &Release) -> Result<&ReleaseAsset, Error> {
    release
        .assets
        .iter()
        .find(|asset| asset.name.contains(TARGET) && asset.name.ends_with(ARCHIVE_EXTENSION))
        .ok_or_else(|| {
            Error::SelfUpdate(SelfUpdateError::Release("No asset found for the current platform.".to_string()))
        })
}
