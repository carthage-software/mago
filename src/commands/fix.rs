use std::collections::BTreeMap;
use std::collections::HashSet;
use std::hash::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::ColorChoice;
use clap::Parser;
use strum::Display;

use crate::commands::analyze::AnalyzeCommand;
use crate::commands::args::baseline_reporting::BaselineReportingArgs;
use crate::commands::args::reporting::ReportingArgs;
use crate::commands::format::FormatCommand;
use crate::commands::guard::GuardCommand;
use crate::commands::lint::LintCommand;
use crate::commands::outcome::CommandOutcome;
use crate::config::Configuration;
use crate::error::Error;

#[derive(Clone, Copy, Debug, Display)]
#[strum(serialize_all = "lowercase")]
enum FixTool {
    Guard,
    Analyze,
    Lint,
    Format,
}

#[derive(Parser, Debug)]
pub struct FixCommand {
    /// Fix these files or directories instead of the configured source paths.
    pub path: Vec<PathBuf>,

    /// Skip the guard.
    #[arg(long)]
    pub no_guard: bool,

    /// Skip the analyzer.
    #[arg(long)]
    pub no_analyze: bool,

    /// Skip the linter.
    #[arg(long)]
    pub no_lint: bool,

    /// Skip the formatter.
    #[arg(long)]
    pub no_fmt: bool,

    /// Allow potentially unsafe fixes as well as safe fixes.
    #[arg(long)]
    pub potentially_unsafe: bool,

    /// Allow all fixes, including unsafe fixes. Review the changes carefully.
    #[arg(long)]
    pub r#unsafe: bool,

    /// Ignore each tool's baseline when applying fixes.
    #[arg(long)]
    pub ignore_baseline: bool,

    /// Exit with code 1 if issues remain after all available fixes have been applied.
    #[arg(long)]
    pub fail_on_remaining: bool,

    /// Stop with an error if fixes do not settle within this many passes (1-256).
    #[arg(long, default_value_t = 10, value_parser = clap::value_parser!(u32).range(1..=256))]
    pub max_passes: u32,
}

impl FixCommand {
    pub fn execute(self, configuration: Configuration, color_choice: ColorChoice) -> Result<ExitCode, Error> {
        if self.no_guard && self.no_analyze && self.no_lint && self.no_fmt {
            tracing::info!("All tools are disabled; nothing to fix.");
            return Ok(ExitCode::SUCCESS);
        }

        let baseline_reporting = BaselineReportingArgs {
            ignore_baseline: self.ignore_baseline,
            reporting: ReportingArgs {
                fix: true,
                r#unsafe: self.r#unsafe,
                potentially_unsafe: self.potentially_unsafe,
                fail_on_remaining: self.fail_on_remaining,
                ..ReportingArgs::default()
            },
            ..BaselineReportingArgs::default()
        };

        run_until_stable(self.max_passes, |pass| {
            let mut outcome = CommandOutcome::from(ExitCode::SUCCESS);
            for (tool, disabled) in [
                (FixTool::Guard, self.no_guard),
                (FixTool::Analyze, self.no_analyze),
                (FixTool::Lint, self.no_lint),
                (FixTool::Format, self.no_fmt),
            ] {
                if disabled {
                    continue;
                }

                tracing::info!("Running {tool} (fix pass {pass}).");

                let configuration = configuration.clone();
                let path = self.path.clone();
                let baseline_reporting = baseline_reporting.clone();
                let result = match tool {
                    FixTool::Guard => GuardCommand { path, baseline_reporting, ..GuardCommand::default() }
                        .execute(configuration, color_choice)?,
                    FixTool::Analyze => AnalyzeCommand { path, baseline_reporting, ..AnalyzeCommand::default() }
                        .execute(configuration, color_choice)?,
                    FixTool::Lint => LintCommand { path, baseline_reporting, ..LintCommand::default() }
                        .execute(configuration, color_choice)?,
                    FixTool::Format => {
                        FormatCommand { path, ..FormatCommand::default() }.execute(configuration, color_choice)?
                    }
                };

                if result.exit_code != ExitCode::SUCCESS {
                    outcome.exit_code = ExitCode::FAILURE;
                }

                outcome.changed_files.extend(result.changed_files);
            }

            Ok(outcome)
        })
    }
}

fn run_until_stable(
    max_passes: u32,
    mut run_pass: impl FnMut(u32) -> Result<CommandOutcome, Error>,
) -> Result<ExitCode, Error> {
    let mut files = BTreeMap::new();
    let mut seen = HashSet::new();

    for pass in 1..=max_passes {
        let outcome = run_pass(pass)?;
        if outcome.changed_files.is_empty() {
            tracing::info!("No more fixes available at the selected safety level after {pass} pass(es).");
            return Ok(outcome.exit_code);
        }

        files.extend(outcome.changed_files);
        let mut hasher = DefaultHasher::new();
        files.hash(&mut hasher);
        if !seen.insert(hasher.finish()) {
            tracing::error!("Fixes keep returning to an earlier file state. Check for conflicting rules or settings.");
            return Ok(ExitCode::FAILURE);
        }
    }

    if max_passes == 256 {
        tracing::error!("Fixes did not settle after 256 passes. Please report a bug in Mago.");
    } else {
        tracing::error!("Fixes did not settle after {max_passes} passes. Increase --max-passes (up to 256).");
        if max_passes > 16 {
            tracing::warn!(
                "Running this many passes may indicate a bug in Mago or a conflicting rule. Please report a bug in Mago."
            );
        }
    }

    Ok(ExitCode::FAILURE)
}

#[cfg(test)]
mod tests {
    use mago_database::file::FileId;

    use super::*;

    fn changed(content: u64) -> CommandOutcome {
        CommandOutcome {
            exit_code: ExitCode::SUCCESS,
            changed_files: BTreeMap::from([(FileId::new(b"test.php"), content)]),
        }
    }

    #[test]
    fn defaults_to_ten_passes() {
        assert_eq!(FixCommand::parse_from(["fix"]).max_passes, 10);
    }

    #[test]
    fn continues_until_a_whole_pass_makes_no_changes() {
        let mut passes = 0;
        let result = run_until_stable(10, |pass| {
            passes = pass;
            Ok(if pass < 3 { changed(u64::from(pass)) } else { ExitCode::SUCCESS.into() })
        });

        assert_eq!(result.unwrap(), ExitCode::SUCCESS);
        assert_eq!(passes, 3);
    }

    #[test]
    fn reports_only_the_final_pass_failure_status() {
        let result = run_until_stable(10, |pass| {
            Ok(if pass == 1 {
                CommandOutcome { exit_code: ExitCode::FAILURE, ..changed(1) }
            } else {
                ExitCode::SUCCESS.into()
            })
        });

        assert_eq!(result.unwrap(), ExitCode::SUCCESS);
        assert_eq!(run_until_stable(10, |_| Ok(ExitCode::FAILURE.into())).unwrap(), ExitCode::FAILURE);
    }

    #[test]
    fn detects_repeated_states_without_treating_them_as_stable() {
        for period in [1, 2] {
            let mut passes = 0;
            let result = run_until_stable(10, |pass| {
                passes = pass;
                Ok(changed(u64::from(pass % period)))
            });

            assert_eq!(result.unwrap(), ExitCode::FAILURE);
            assert_eq!(passes, period + 1);
        }
    }

    #[test]
    fn stops_at_the_pass_limit() {
        for limit in [3, 10, 256] {
            let mut passes = 0;
            let result = run_until_stable(limit, |pass| {
                passes = pass;
                Ok(changed(u64::from(pass)))
            });

            assert_eq!(result.unwrap(), ExitCode::FAILURE);
            assert_eq!(passes, limit);
        }
    }
}
