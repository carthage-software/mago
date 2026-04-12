use std::cmp::min;
use std::io;
use std::io::BufRead;
use std::io::Write;

use serde_json::Value;
use ureq::typestate::WithoutBody;

use crate::updater::error::UpdateError;

/// Returns a GitHub authentication token from the environment, if available.
fn get_github_token() -> Option<String> {
    std::env::var("GITHUB_TOKEN").ok().or_else(|| std::env::var("GH_TOKEN").ok()).filter(|t| !t.is_empty())
}

/// Applies common headers (User-Agent, Accept, and optional Authorization) to a request.
fn github_api_request(url: &str, accept: &str) -> ureq::RequestBuilder<WithoutBody> {
    let req =
        ureq::get(url).header("User-Agent", &format!("mago/{}", env!("CARGO_PKG_VERSION"))).header("Accept", accept);

    if let Some(token) = get_github_token() { req.header("Authorization", &format!("Bearer {token}")) } else { req }
}

#[derive(Debug)]
pub struct Release {
    pub name: String,
    pub version: String,
    pub assets: Vec<ReleaseAsset>,
}

#[derive(Debug)]
pub struct ReleaseAsset {
    pub name: String,
    pub download_url: String,
}

pub fn get_latest_release(owner: &str, repo: &str) -> Result<Release, UpdateError> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/releases/latest");

    let json: Value = github_api_request(&url, "application/vnd.github.v3+json").call()?.body_mut().read_json()?;

    parse_release(&json)
}

pub fn get_release_version(owner: &str, repo: &str, tag: &str) -> Result<Release, UpdateError> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/releases/tags/{tag}");

    let json: Value = github_api_request(&url, "application/vnd.github.v3+json").call()?.body_mut().read_json()?;

    parse_release(&json)
}

/// Fetches one page of non-draft releases (newest first) from the GitHub
/// Releases API.
pub fn list_releases(owner: &str, repo: &str, page: u32) -> Result<Vec<Release>, UpdateError> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/releases?per_page=100&page={page}");

    let json: Value = github_api_request(&url, "application/vnd.github.v3+json").call()?.body_mut().read_json()?;

    json.as_array()
        .ok_or_else(|| UpdateError::Release("releases endpoint did not return an array".into()))?
        .iter()
        .filter(|entry| !entry["draft"].as_bool().unwrap_or(false))
        .map(parse_release)
        .collect()
}

pub fn download_asset(asset: &ReleaseAsset, dest: &mut impl Write) -> Result<(), UpdateError> {
    let response = github_api_request(&asset.download_url, "application/octet-stream").call()?;

    let size = response
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);

    let bar = if size > 0 {
        let pb = indicatif::ProgressBar::new(size);
        pb.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("[{elapsed_precise}] [{bar:40}] {bytes}/{total_bytes} ({eta})")
                .expect("invalid progress template")
                .progress_chars("=>-"),
        );
        Some(pb)
    } else {
        None
    };

    let mut src = io::BufReader::new(response.into_body().into_reader());
    let mut downloaded: u64 = 0;

    loop {
        let n = {
            let buf = src.fill_buf()?;
            dest.write_all(buf)?;
            buf.len()
        };

        if n == 0 {
            break;
        }

        src.consume(n);
        downloaded = min(downloaded + n as u64, size);

        if let Some(ref bar) = bar {
            bar.set_position(downloaded);
        }
    }

    if let Some(bar) = bar {
        bar.finish_with_message("Done");
    }

    Ok(())
}

fn parse_release(json: &Value) -> Result<Release, UpdateError> {
    let tag = json["tag_name"].as_str().ok_or_else(|| UpdateError::Release("release missing `tag_name`".into()))?;

    let name = json["name"].as_str().unwrap_or(tag);

    let assets = json["assets"]
        .as_array()
        .ok_or_else(|| UpdateError::Release("release missing `assets`".into()))?
        .iter()
        .map(|asset| {
            let name = asset["name"].as_str().ok_or_else(|| UpdateError::Release("asset missing `name`".into()))?;
            let download_url =
                asset["url"].as_str().ok_or_else(|| UpdateError::Release("asset missing `url`".into()))?;

            Ok(ReleaseAsset { name: name.to_owned(), download_url: download_url.to_owned() })
        })
        .collect::<Result<Vec<_>, UpdateError>>()?;

    Ok(Release { name: name.to_owned(), version: tag.trim_start_matches('v').to_owned(), assets })
}
