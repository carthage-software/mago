use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use ureq::Agent;

const DATA_URL: &str = "https://carthage-software.github.io/php-toolchain-benchmarks/latest.json";
const PROJECT_ID: &str = "wordpress";
const PROJECT_LABEL: &str = "WordPress";
const PROJECT_LOC: &str = "7M";

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkSummary {
    pub project_label: &'static str,
    pub project_loc: &'static str,
    pub aggregation_date: String,
    pub mago_version: String,
    pub analyzer: CategorySummary,
    pub linter: CategorySummary,
    pub formatter: CategorySummary,
}

#[derive(Debug, Clone, Serialize)]
pub struct CategorySummary {
    pub mago_seconds: f64,
    pub mago_label: String,
    pub peers: Vec<PeerEntry>,
    pub factor: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PeerEntry {
    pub name: String,
    pub seconds: f64,
    pub label: String,
}

#[derive(Debug, Deserialize)]
struct Document {
    #[serde(rename = "aggregation-date")]
    aggregation_date: String,
    projects: HashMap<String, HashMap<String, HashMap<String, Vec<Run>>>>,
}

#[derive(Debug, Deserialize)]
struct Run {
    mean: Option<f64>,
    #[serde(default)]
    timed_out: bool,
}

pub fn fetch() -> Result<BenchmarkSummary> {
    let agent = Agent::config_builder().timeout_global(Some(Duration::from_secs(15))).build().new_agent();

    let body: Document =
        agent.get(DATA_URL).call().context("failed to fetch benchmark data")?.body_mut().read_json()?;

    let project = body.projects.get(PROJECT_ID).context("benchmark JSON missing wordpress project")?;

    let analyzer = summarize(project, "Cold", "Mago", "analyzer")?;
    let linter = summarize(project, "Linter", "Mago Lint", "linter")?;
    let formatter = summarize(project, "Formatter", "Mago Fmt", "formatter")?;

    let mago_version = analyzer.mago_label.clone();

    Ok(BenchmarkSummary {
        project_label: PROJECT_LABEL,
        project_loc: PROJECT_LOC,
        aggregation_date: body.aggregation_date,
        mago_version,
        analyzer,
        linter,
        formatter,
    })
}

fn summarize(
    project: &HashMap<String, HashMap<String, Vec<Run>>>,
    category: &str,
    mago_prefix: &str,
    category_label: &str,
) -> Result<CategorySummary> {
    let entries = project.get(category).with_context(|| format!("benchmark project missing category {category}"))?;

    let mut latest: BTreeMap<String, (String, f64)> = BTreeMap::new();
    for (full_name, runs) in entries {
        let Some((base_name, version)) = split_name(full_name) else {
            continue;
        };

        let valid_runs: Vec<&Run> = runs.iter().filter(|run| !run.timed_out).collect();
        if valid_runs.is_empty() {
            continue;
        }

        let mean: f64 = valid_runs.iter().filter_map(|run| run.mean).sum::<f64>() / valid_runs.len().max(1) as f64;

        latest
            .entry(base_name.to_string())
            .and_modify(|existing| {
                if compare_versions(version, &existing.0) == Ordering::Greater {
                    *existing = (version.to_string(), mean);
                }
            })
            .or_insert((version.to_string(), mean));
    }

    let mago_entry = latest
        .iter()
        .find(|(name, _)| name.as_str() == mago_prefix)
        .with_context(|| format!("benchmark project missing {mago_prefix} entry for {category_label}"))?;
    let mago_version = mago_entry.1.0.clone();
    let mago_seconds = mago_entry.1.1;

    let mut peers: Vec<PeerEntry> = latest
        .iter()
        .filter(|(name, _)| !name.starts_with("Mago"))
        .map(|(name, (version, seconds))| PeerEntry {
            name: name.clone(),
            seconds: *seconds,
            label: format!("{name} {version}"),
        })
        .collect();

    peers.sort_by(|a, b| b.seconds.partial_cmp(&a.seconds).unwrap_or(Ordering::Equal));

    let slowest_peer = peers.first().map(|peer| peer.seconds).unwrap_or(0.0);
    let factor = if mago_seconds > 0.0 { (slowest_peer / mago_seconds).round() as u64 } else { 0 };

    Ok(CategorySummary { mago_seconds, mago_label: format!("Mago {mago_version}"), peers, factor })
}

fn split_name(full_name: &str) -> Option<(&str, &str)> {
    let bytes = full_name.as_bytes();
    let mut split = None;
    for (idx, byte) in bytes.iter().enumerate().rev() {
        if *byte == b' ' {
            let candidate = &full_name[idx + 1..];
            if candidate.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                split = Some(idx);
                break;
            }
        }
    }

    let idx = split?;
    Some((full_name[..idx].trim(), full_name[idx + 1..].trim()))
}

fn compare_versions(a: &str, b: &str) -> Ordering {
    let parts_a = a.split('.').filter_map(|p| p.parse::<u32>().ok()).collect::<Vec<_>>();
    let parts_b = b.split('.').filter_map(|p| p.parse::<u32>().ok()).collect::<Vec<_>>();
    let len = parts_a.len().max(parts_b.len());
    for i in 0..len {
        let ai = parts_a.get(i).copied().unwrap_or(0);
        let bi = parts_b.get(i).copied().unwrap_or(0);
        match ai.cmp(&bi) {
            Ordering::Equal => continue,
            other => return other,
        }
    }

    Ordering::Equal
}

pub fn format_seconds(seconds: f64) -> String {
    if seconds >= 100.0 {
        format!("{seconds:.0}s")
    } else if seconds >= 10.0 {
        format!("{seconds:.1}s")
    } else {
        format!("{seconds:.2}s")
    }
}
