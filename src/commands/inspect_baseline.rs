//! The `inspect-baseline` command: inspect and visualize a baseline file.
//!
//! Large baselines (thousands of entries across thousands of lines) are tedious to read by
//! hand. This command summarizes a baseline so it is easy to see, at a glance, which issue
//! codes and which files dominate it - and to drill into a single code or a single file.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::ExitCode;

use mago_reporting::baseline::Baseline;
use mago_reporting::osc8_hyperlink;

use clap::ColorChoice;
use clap::Parser;
use colored::Colorize;

use crate::baseline::unserialize_baseline;
use crate::config::Configuration;
use crate::error::Error;
use crate::utils::configure_colors;
use crate::utils::should_use_colors;

/// The dimension to group the baseline by.
#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum InspectionGrouping {
    /// Group by file, listing the issue codes under each file.
    File,
    /// Group by issue code, listing the files under each code.
    Code,
}

/// Inspect and visualize the issues recorded in a baseline file.
///
/// With no flags it prints a summary of how many times each issue code appears, sorted by
/// frequency - the quickest way to see what dominates the baseline. The grouping flags expand
/// that into a tree, and a `CODE_OR_FILE` argument drills into a single code (listing the files
/// that contain it) or a single file (listing the codes it contains).
#[derive(Parser, Debug)]
pub struct InspectBaselineCommand {
    /// Path to the baseline file to inspect.
    ///
    /// When omitted, the baseline configured in `mago.toml` is used - but only when a single one
    /// is configured. If several sections configure different baselines, pass the path explicitly.
    #[arg(value_name = "BASELINE")]
    pub baseline: Option<PathBuf>,

    /// Drill into a single issue code or file.
    ///
    /// When this matches an issue code present in the baseline, the files containing it are
    /// listed. Otherwise it is treated as a file path (matched as a suffix), and the issue
    /// codes recorded for that file are listed.
    #[arg(value_name = "CODE_OR_FILE")]
    pub filter: Option<String>,

    /// Group the whole baseline by `code` or by `file`.
    ///
    /// Ignored when a `CODE_OR_FILE` argument is given (that drills into a single entry instead).
    #[arg(long, value_name = "GROUPING")]
    pub group: Option<InspectionGrouping>,

    /// Limit how many entries are shown, both top-level groups and members within each group.
    #[arg(long, value_name = "N")]
    pub limit: Option<usize>,
}

impl InspectBaselineCommand {
    /// Executes the `inspect-baseline` command.
    pub fn execute(self, configuration: Configuration, color_choice: ColorChoice) -> Result<ExitCode, Error> {
        configure_colors(color_choice);

        let baseline_path = match self.baseline.clone() {
            Some(path) => path,
            None => match resolve_configured_baseline(&configuration) {
                ConfiguredBaseline::Single(path) => path,
                ConfiguredBaseline::None => {
                    eprintln!(
                        "No baseline file given or configured. Pass a baseline path: `mago inspect-baseline <BASELINE>`."
                    );

                    return Ok(ExitCode::FAILURE);
                }
                ConfiguredBaseline::Ambiguous(paths) => {
                    eprintln!("Several baselines are configured in `mago.toml`; pass the one to inspect explicitly:");
                    for path in paths {
                        eprintln!("  mago inspect-baseline {}", path.display());
                    }

                    return Ok(ExitCode::FAILURE);
                }
            },
        };

        let (baseline, _needs_warning) = unserialize_baseline(&baseline_path)?;
        let entries = flatten_baseline(&baseline);

        if entries.is_empty() {
            println!("The baseline at `{}` contains no issues.", baseline_path.display());

            return Ok(ExitCode::SUCCESS);
        }

        // Only emit OSC 8 hyperlinks for an interactive terminal. The template comes from the
        // configured editor URL, falling back to `file://` so paths open in most editors/terminals.
        let links = LinkContext {
            workspace: configuration.source.workspace.clone(),
            template: should_use_colors(color_choice)
                .then(|| configuration.editor_url.clone().unwrap_or_else(|| "file://%file%".to_string())),
        };

        match (&self.filter, self.group) {
            (Some(filter), _) => self.print_filtered(&entries, filter, &links),
            (None, Some(InspectionGrouping::File)) => {
                self.print_groups(group_by(&entries, |entry| entry.file, |entry| entry.code), "file", &links);
            }
            (None, Some(InspectionGrouping::Code)) => {
                self.print_groups(group_by(&entries, |entry| entry.code, |entry| entry.file), "code", &links);
            }
            (None, None) => self.print_code_summary(&entries),
        }

        Ok(ExitCode::SUCCESS)
    }

    /// Prints the default summary: a ranked bar chart of issue codes, most frequent first.
    fn print_code_summary(&self, entries: &[FlatEntry<'_>]) {
        let mut totals: BTreeMap<&str, u32> = BTreeMap::new();
        for entry in entries {
            *totals.entry(entry.code).or_insert(0) += entry.count;
        }

        let total: u32 = entries.iter().map(|entry| entry.count).sum();

        let mut ranked: Vec<(&str, u32)> = totals.into_iter().collect();
        sort_by_count_then_name(&mut ranked, |&(name, _)| name, |&(_, count)| count);

        print_heading(total, ranked.len(), "code");
        // Codes are not files, so they are never hyperlinked.
        print_bars(&ranked, self.limit, None);
    }

    /// Prints a ranked bar chart per group (e.g. a code with the files under it).
    fn print_groups(&self, groups: Vec<Group<'_>>, dimension: &str, links: &LinkContext) {
        let total: u32 = groups.iter().map(|group| group.total).sum();
        print_heading(total, groups.len(), dimension);

        // When grouping by file, the group headers are files (link those, not the members);
        // when grouping by code, the members are files (link those, not the headers).
        let groups_are_files = dimension == "file";
        let member_links = (!groups_are_files).then_some(links);

        let shown = self.limit.map_or(groups.len(), |limit| limit.min(groups.len()));
        for group in groups.iter().take(shown) {
            let styled = group.name.cyan().bold().to_string();
            let header = if groups_are_files { links.hyperlink(group.name, &styled) } else { Cow::Borrowed(&*styled) };
            println!("{header} {}", format!("({})", group.total).dimmed());
            print_bars(&group.members, self.limit, member_links);
            println!();
        }

        let remaining = groups.len() - shown;
        if remaining > 0 {
            println!("{}", format!("… and {remaining} more {dimension}(s)").dimmed());
        }
    }

    /// Prints the result of drilling into a single code or file as a ranked bar chart.
    fn print_filtered(&self, entries: &[FlatEntry<'_>], filter: &str, links: &LinkContext) {
        let is_code = entries.iter().any(|entry| entry.code == filter);

        let members: Vec<(&str, u32)> = if is_code {
            collect_members(entries.iter().filter(|entry| entry.code == filter), |entry| entry.file)
        } else {
            collect_members(
                entries.iter().filter(|entry| entry.file == filter || entry.file.ends_with(filter)),
                |entry| entry.code,
            )
        };

        if members.is_empty() {
            println!("{}", format!("No baseline entries match `{filter}`.").yellow());

            return;
        }

        // Drilling into a code lists files (link members); drilling into a file lists codes
        // (link the header instead).
        let dimension = if is_code { "file" } else { "code" };
        let total: u32 = members.iter().map(|&(_, count)| count).sum();

        let styled = filter.cyan().bold().to_string();
        let header = if is_code { Cow::Borrowed(&*styled) } else { links.hyperlink(filter, &styled) };
        println!("{header}");

        print_heading(total, members.len(), dimension);
        print_bars(&members, self.limit, is_code.then_some(links));
    }
}

/// Carries what's needed to turn a baseline file path into an OSC 8 hyperlink.
struct LinkContext {
    workspace: PathBuf,
    template: Option<String>,
}

impl LinkContext {
    /// Wraps `display` in an OSC 8 hyperlink pointing at `file` (resolved against the workspace),
    /// or borrows `display` unchanged when hyperlinks are disabled - so the common, link-less
    /// path allocates nothing.
    fn hyperlink<'display>(&self, file: &str, display: &'display str) -> Cow<'display, str> {
        match &self.template {
            Some(template) => {
                let absolute = self.workspace.join(file);

                Cow::Owned(osc8_hyperlink(template, &absolute.display().to_string(), 1, 1, display))
            }
            None => Cow::Borrowed(display),
        }
    }
}

/// Width, in cells, of a full-length bar.
const BAR_WIDTH: usize = 30;

/// Prints the heading shown above a chart, e.g. `42 issues across 7 codes`.
fn print_heading(total: u32, groups: usize, dimension: &str) {
    let suffix = if groups == 1 { "" } else { "s" };
    println!("{} issues across {} {dimension}{suffix}\n", total.to_string().bold(), groups.to_string().bold());
}

/// Prints `rows` (already sorted by descending count) as a colored horizontal bar chart,
/// honoring `limit`. Bars are scaled to the largest count and tinted by magnitude - red for
/// the heavy hitters, green for the long tail - so the worst offenders pop out.
///
/// When `file_links` is `Some`, each row label is a file path and is rendered as an OSC 8
/// hyperlink so it can be opened from the terminal.
fn print_bars(rows: &[(&str, u32)], limit: Option<usize>, file_links: Option<&LinkContext>) {
    let shown = limit.map_or(rows.len(), |limit| limit.min(rows.len()));
    let visible = &rows[..shown];

    let max = visible.iter().map(|&(_, count)| count).max().unwrap_or(1).max(1);
    let count_width = visible.iter().map(|&(_, count)| count.to_string().len()).max().unwrap_or(1);

    for &(name, count) in visible {
        let fraction = f64::from(count) / f64::from(max);
        let filled = ((fraction * BAR_WIDTH as f64).round() as usize).clamp(1, BAR_WIDTH);

        let blocks = "█".repeat(filled);
        let bar = if fraction >= 0.66 {
            blocks.red()
        } else if fraction >= 0.33 {
            blocks.yellow()
        } else {
            blocks.green()
        };
        let padding = " ".repeat(BAR_WIDTH - filled);
        let count_label = format!("{count:>count_width$}").bold();
        let label = match file_links {
            Some(links) => links.hyperlink(name, name),
            None => Cow::Borrowed(name),
        };

        println!("  {count_label}  {bar}{padding}  {label}");
    }

    let remaining = rows.len() - shown;
    if remaining > 0 {
        println!("  {}", format!("… and {remaining} more").dimmed());
    }
}

/// A single (file, code) pair from the baseline with its occurrence count.
///
/// Both names borrow from the owned [`Baseline`], so flattening allocates no strings.
struct FlatEntry<'baseline> {
    file: &'baseline str,
    code: &'baseline str,
    count: u32,
}

/// A top-level group (a code or a file) with the members nested under it. All names borrow
/// from the baseline.
struct Group<'baseline> {
    name: &'baseline str,
    total: u32,
    members: Vec<(&'baseline str, u32)>,
}

/// The outcome of looking for a baseline in the configuration.
enum ConfiguredBaseline {
    /// No section configures a baseline.
    None,
    /// Exactly one distinct baseline is configured.
    Single(PathBuf),
    /// Several sections configure different baselines; the caller must disambiguate.
    Ambiguous(Vec<PathBuf>),
}

/// Collects the distinct baselines configured across the analyzer, linter, and guard sections.
///
/// No section takes precedence over another: a single distinct path is used, but several
/// different paths are reported as ambiguous so the user picks one explicitly.
fn resolve_configured_baseline(configuration: &Configuration) -> ConfiguredBaseline {
    let candidates = [&configuration.analyzer.baseline, &configuration.linter.baseline, &configuration.guard.baseline];

    let mut distinct: Vec<PathBuf> = Vec::new();
    for path in candidates.into_iter().flatten() {
        if !distinct.contains(path) {
            distinct.push(path.clone());
        }
    }

    match distinct.len() {
        0 => ConfiguredBaseline::None,
        1 => ConfiguredBaseline::Single(distinct.remove(0)),
        _ => ConfiguredBaseline::Ambiguous(distinct),
    }
}

/// Flattens either baseline variant into a uniform list of (file, code, count) entries that
/// borrow their names from `baseline`.
fn flatten_baseline(baseline: &Baseline) -> Vec<FlatEntry<'_>> {
    match baseline {
        Baseline::Loose(loose) => loose
            .issues
            .iter()
            .map(|issue| FlatEntry { file: &issue.file, code: &issue.code, count: issue.count })
            .collect(),
        Baseline::Strict(strict) => {
            let mut entries = Vec::new();
            for (file, entry) in &strict.entries {
                let mut per_code: BTreeMap<&str, u32> = BTreeMap::new();
                for issue in &entry.issues {
                    *per_code.entry(issue.code.as_str()).or_insert(0) += 1;
                }

                for (code, count) in per_code {
                    entries.push(FlatEntry { file, code, count });
                }
            }

            entries
        }
    }
}

/// Groups `entries` by one dimension (the `key`), collecting the other dimension (the `member`)
/// under each group. Groups and members are both sorted by descending count, then by name.
fn group_by<'baseline>(
    entries: &[FlatEntry<'baseline>],
    key: impl Fn(&FlatEntry<'baseline>) -> &'baseline str,
    member: impl Fn(&FlatEntry<'baseline>) -> &'baseline str,
) -> Vec<Group<'baseline>> {
    let mut grouped: BTreeMap<&'baseline str, BTreeMap<&'baseline str, u32>> = BTreeMap::new();
    for entry in entries {
        *grouped.entry(key(entry)).or_default().entry(member(entry)).or_insert(0) += entry.count;
    }

    let mut groups: Vec<Group<'baseline>> = grouped
        .into_iter()
        .map(|(name, members)| {
            let total = members.values().sum();
            let mut members: Vec<(&'baseline str, u32)> = members.into_iter().collect();
            sort_by_count_then_name(&mut members, |&(name, _)| name, |&(_, count)| count);

            Group { name, total, members }
        })
        .collect();

    sort_by_count_then_name(&mut groups, |group| group.name, |group| group.total);

    groups
}

/// Collects and ranks the members produced by `select` from a set of entries.
fn collect_members<'baseline, 'entry>(
    entries: impl Iterator<Item = &'entry FlatEntry<'baseline>>,
    select: impl Fn(&FlatEntry<'baseline>) -> &'baseline str,
) -> Vec<(&'baseline str, u32)>
where
    'baseline: 'entry,
{
    let mut totals: BTreeMap<&'baseline str, u32> = BTreeMap::new();
    for entry in entries {
        *totals.entry(select(entry)).or_insert(0) += entry.count;
    }

    let mut members: Vec<(&'baseline str, u32)> = totals.into_iter().collect();
    sort_by_count_then_name(&mut members, |&(name, _)| name, |&(_, count)| count);

    members
}

/// Sorts a list by descending count, breaking ties by ascending name.
fn sort_by_count_then_name<T>(items: &mut [T], name: impl Fn(&T) -> &str, count: impl Fn(&T) -> u32) {
    items.sort_by(|a, b| count(b).cmp(&count(a)).then_with(|| name(a).cmp(name(b))));
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use mago_reporting::baseline::BaselineVariant;
    use mago_reporting::baseline::LooseBaseline;
    use mago_reporting::baseline::LooseBaselineIssue;
    use mago_reporting::baseline::StrictBaseline;
    use mago_reporting::baseline::StrictBaselineEntry;
    use mago_reporting::baseline::StrictBaselineIssue;

    use super::*;

    fn loose(issues: Vec<(&str, &str, u32)>) -> Baseline {
        Baseline::Loose(LooseBaseline {
            variant: BaselineVariant::Loose,
            issues: issues
                .into_iter()
                .map(|(file, code, count)| LooseBaselineIssue {
                    file: file.to_string(),
                    code: code.to_string(),
                    message: String::new(),
                    count,
                })
                .collect(),
        })
    }

    #[test]
    fn flatten_loose_keeps_counts() {
        let baseline = loose(vec![("a.php", "mixed-assignment", 3), ("b.php", "mixed-argument", 1)]);
        let entries = flatten_baseline(&baseline);

        assert_eq!(entries.len(), 2);
        assert_eq!(entries.iter().map(|entry| entry.count).sum::<u32>(), 4);
    }

    #[test]
    fn flatten_strict_aggregates_per_code() {
        let mut strict = StrictBaseline::new();
        strict.entries.insert(
            Cow::Borrowed("a.php"),
            StrictBaselineEntry {
                issues: vec![
                    StrictBaselineIssue { code: "mixed-assignment".to_string(), start_line: 1, end_line: 1 },
                    StrictBaselineIssue { code: "mixed-assignment".to_string(), start_line: 5, end_line: 5 },
                    StrictBaselineIssue { code: "mixed-argument".to_string(), start_line: 9, end_line: 9 },
                ],
            },
        );

        let baseline = Baseline::Strict(strict);
        let entries = flatten_baseline(&baseline);

        let assignment = entries.iter().find(|entry| entry.code == "mixed-assignment");
        assert_eq!(assignment.map(|entry| entry.count), Some(2));
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn group_by_code_ranks_by_count() {
        let baseline = loose(vec![
            ("a.php", "mixed-argument", 1),
            ("a.php", "mixed-assignment", 3),
            ("b.php", "mixed-assignment", 2),
        ]);
        let entries = flatten_baseline(&baseline);
        let groups = group_by(&entries, |entry| entry.code, |entry| entry.file);

        assert_eq!(groups[0].name, "mixed-assignment");
        assert_eq!(groups[0].total, 5);
        assert_eq!(groups[0].members[0].0, "a.php");
        assert_eq!(groups[1].name, "mixed-argument");
    }
}
