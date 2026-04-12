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
use crate::version_check::VersionCheck;
use crate::version_check::VersionPin;

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
    #[arg(long, value_name = "VERSION_TAG", conflicts_with = "to_project_version")]
    pub tag: Option<String>,

    /// Update to whatever version the project's `mago.toml` pins.
    ///
    /// Reads the `version` field from `mago.toml` and installs the matching release.
    /// Fails if `mago.toml` has no `version` pin; add one (e.g. `version = "1"`) or
    /// use `--tag` explicitly.
    ///
    /// For exact pins (`version = "1.19.3"`) this resolves to that exact release tag.
    /// For non-exact pins (`version = "1"` or `version = "1.19"`) this installs the
    /// latest published release that satisfies the pin, or fails with a clear error
    /// if the latest release is on a different major/minor line.
    #[arg(long, conflicts_with = "tag")]
    pub to_project_version: bool,
}

pub fn execute(command: SelfUpdateCommand, project_version_pin: Option<String>) -> Result<ExitCode, Error> {
    debug!("OS: {}", std::env::consts::OS);
    debug!("ARCH: {}", std::env::consts::ARCH);
    debug!("TARGET: {}", TARGET);
    debug!("BIN: {}", BIN);
    debug!("ARCHIVE_EXTENSION: {}", ARCHIVE_EXTENSION);

    let mut resolved_exact_tag = false;
    let release = if command.to_project_version {
        let pin_string = match project_version_pin.as_deref() {
            Some(pin) => pin,
            None => {
                tracing::error!(
                    "Add a pin to `mago.toml` (e.g. `version = \"1\"` at the top of the file), or pass `--tag <VERSION>` instead."
                );

                return Err(Error::NoPinnedProjectVersion);
            }
        };

        let pin = VersionPin::parse(pin_string)?;

        if pin.is_exact() {
            resolved_exact_tag = true;

            info!("Fetching project version {}... ", pin);

            github::get_release_version(REPO_OWNER, REPO_NAME, pin_string)?
        } else {
            info!("Resolving latest release satisfying project pin `{pin}`...");

            find_latest_release_satisfying(&pin)?
        }
    } else {
        match command.tag {
            Some(tag) => {
                resolved_exact_tag = true;

                info!("Fetching version {}... ", tag);

                github::get_release_version(REPO_OWNER, REPO_NAME, &tag)?
            }
            None => {
                info!("Checking latest released version... ");
                github::get_latest_release(REPO_OWNER, REPO_NAME)?
            }
        }
    };

    info!("Release found: {} ({})", release.name, release.version);

    if resolved_exact_tag && release.version == VERSION {
        info!("Already up-to-date with version `{}`", VERSION);
        return Ok(ExitCode::SUCCESS);
    }

    if !resolved_exact_tag {
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

/// Scans recent releases and returns the version string of the highest one
/// that satisfies `pin`.
fn find_latest_release_satisfying(pin: &VersionPin) -> Result<Release, Error> {
    const MAX_PAGES: u32 = 10;

    let pin_major = parse_version_components(&pin.to_string()).map(|(m, _, _)| m).unwrap_or(0);

    let mut best: Option<(Release, (u64, u64, u64))> = None;
    let mut latest_seen: Option<String> = None;

    for page in 1..=MAX_PAGES {
        let releases = github::list_releases(REPO_OWNER, REPO_NAME, page).map_err(Error::SelfUpdate)?;

        if releases.is_empty() {
            break;
        }

        if latest_seen.is_none() {
            latest_seen = Some(releases[0].version.clone());
        }

        let mut page_touched_pin_era = false;

        for release in releases {
            let Ok(components) = parse_version_components(&release.version) else {
                continue;
            };

            if components.0 >= pin_major {
                page_touched_pin_era = true;
            }

            if !matches!(pin.check(&release.version), Ok(VersionCheck::Match)) {
                continue;
            }

            if best.as_ref().is_none_or(|(_, best_components)| components > *best_components) {
                best = Some((release, components));
            }
        }

        if !page_touched_pin_era {
            break;
        }
    }

    if let Some((version, _)) = best {
        Ok(version)
    } else {
        let latest = latest_seen.unwrap_or_else(|| "unknown".to_owned());
        tracing::error!(
            "Scanned the most recent releases on GitHub but none matched `{pin}`; the most recent one was `{latest}`."
        );
        tracing::error!(
            "If a matching release exists further back in history, pass `--tag <VERSION>` explicitly to install it."
        );
        Err(Error::LatestReleaseDoesNotSatisfyPin(pin.to_string(), latest))
    }
}

/// Parses a `major.minor.patch` string (with optional `-pre`/`+build`
/// suffix) into a `(u64, u64, u64)` tuple suitable for lexicographic
/// ordering of release versions.
fn parse_version_components(version: &str) -> Result<(u64, u64, u64), UpdateError> {
    let core = version.split(['-', '+']).next().unwrap_or(version).trim();
    let mut parts = core.split('.');

    let parse_part = |p: Option<&str>| -> Result<u64, UpdateError> {
        p.ok_or_else(|| UpdateError::Release(format!("release version `{version}` is missing a component")))?
            .parse::<u64>()
            .map_err(|_| UpdateError::Release(format!("release version `{version}` has a non-numeric component")))
    };

    let major = parse_part(parts.next())?;
    let minor = parse_part(parts.next())?;
    let patch = parse_part(parts.next()).unwrap_or(0);

    Ok((major, minor, patch))
}

fn get_target_asset_from_release(release: &Release) -> Result<&ReleaseAsset, UpdateError> {
    if let Some(asset) =
        release.assets.iter().find(|asset| asset.name.contains(TARGET) && asset.name.ends_with(ARCHIVE_EXTENSION))
    {
        return Ok(asset);
    }

    // Emit human-readable guidance via tracing, return a terse summary as
    // the error. Three distinct failure shapes: unsupported target,
    // release with no binaries at all, or release with some binaries but
    // not ours.
    let binary_asset_count =
        release.assets.iter().filter(|a| SUPPORTED_TARGETS.iter().any(|t| a.name.contains(t))).count();

    if !SUPPORTED_TARGETS.contains(&TARGET) {
        tracing::error!("Your platform `{TARGET}` is not in Mago's list of pre-built binary targets.");
        tracing::error!("Compile from source: clone the repository and run `cargo install --path .`.");
        Err(UpdateError::Release(format!("no pre-built binary available for platform `{TARGET}`")))
    } else if binary_asset_count == 0 {
        tracing::error!(
            "Release `{}` was just published and its binaries are still being built by CI.",
            release.version
        );
        tracing::error!("This typically takes 30-40 minutes. Please try again shortly.");
        Err(UpdateError::Release(format!("no binaries published yet for release `{}`", release.version)))
    } else {
        tracing::error!(
            "Release `{}` has {}/{} platform builds ready; the `{TARGET}` build is still in progress.",
            release.version,
            binary_asset_count,
            SUPPORTED_TARGETS.len(),
        );
        tracing::error!("Please try again in a few minutes.");
        Err(UpdateError::Release(format!("binary for `{TARGET}` not yet available in release `{}`", release.version)))
    }
}
