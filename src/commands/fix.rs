use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

use mago_fixer::FixPlan;
use mago_fixer::SafetyClassification;
use mago_interner::ThreadedInterner;
use mago_reporting::IssueCollection;
use mago_source::SourceIdentifier;

use crate::commands::lint::lint_check;
use crate::config::Configuration;
use crate::error::Error;
use crate::source;
use crate::utils;
use crate::utils::progress::create_progress_bar;
use crate::utils::progress::remove_progress_bar;
use crate::utils::progress::ProgressBarTheme;

#[derive(Parser, Debug)]
#[command(
    name = "fix",
    about = "Apply fixes for lint issues identified during linting",
    long_about = r#"
The `fix` command automatically applies fixes for issues identified during the linting process.

This command streamlines the process of addressing lint issues, improving code quality and consistency.
"#
)]
pub struct FixCommand {
    /// Lint specific files or directories, overriding the source configuration.
    #[arg(help = "Lint specific files or directories, overriding the source configuration")]
    pub path: Vec<PathBuf>,

    #[arg(short, long, help = "Do not load default plugins, only load the ones specified in the configuration.")]
    pub no_default_plugins: bool,

    #[arg(short, long, help = "Specify plugins to load, overriding the configuration.")]
    pub plugins: Vec<String>,

    /// Apply fixes that are marked as unsafe, including potentially unsafe fixes.
    #[arg(
        long,
        short = 'u',
        help = "Apply fixes marked as unsafe, including those with potentially destructive changes"
    )]
    pub r#unsafe: bool,

    /// Apply fixes that are marked as potentially unsafe.
    #[arg(long, short = 'p', help = "Apply fixes marked as potentially unsafe, which may require manual review")]
    pub potentially_unsafe: bool,

    /// Run the command without writing any changes to disk.
    #[arg(long, short = 'd', help = "Preview the fixes without applying them, showing what changes would be made")]
    pub dry_run: bool,
}

impl FixCommand {
    pub const fn get_classification(&self) -> SafetyClassification {
        if self.r#unsafe {
            SafetyClassification::Unsafe
        } else if self.potentially_unsafe {
            SafetyClassification::PotentiallyUnsafe
        } else {
            SafetyClassification::Safe
        }
    }
}

pub async fn execute(command: FixCommand, mut configuration: Configuration) -> Result<ExitCode, Error> {
    // Initialize the interner for managing identifiers.
    let interner = ThreadedInterner::new();

    // Determine the safety classification for the fixes.
    let classification = command.get_classification();

    if command.no_default_plugins {
        configuration.linter.default_plugins = Some(false);
    }

    if !command.plugins.is_empty() {
        configuration.linter.plugins = command.plugins;
    }

    // Load sources
    let source_manager = if !command.path.is_empty() {
        source::from_paths(&interner, &configuration.source, command.path, true).await?
    } else {
        source::load(&interner, &configuration.source, true, true).await?
    };

    let issues = lint_check(&interner, &source_manager, &configuration).await?;
    let (plans, skipped_unsafe, skipped_potentially_unsafe) = filter_fix_plans(&interner, issues, classification);

    let total = plans.len();
    let progress_bar = create_progress_bar(total, "✨  Fixing", ProgressBarTheme::Cyan);
    let mut handles = Vec::with_capacity(total);
    for (source, plan) in plans.into_iter() {
        handles.push(tokio::spawn({
            let source_manager = source_manager.clone();
            let interner = interner.clone();
            let progress_bar = progress_bar.clone();

            async move {
                let source = source_manager.load(&source)?;
                let source_content = interner.lookup(&source.content);
                let result = utils::apply_changes(
                    &interner,
                    &source_manager,
                    &source,
                    plan.execute(source_content).get_fixed(),
                    command.dry_run,
                );

                progress_bar.inc(1);

                result
            }
        }));
    }

    let mut changed = 0;
    for handle in handles {
        if handle.await?? {
            changed += 1;
        }
    }

    remove_progress_bar(progress_bar);

    if skipped_unsafe > 0 {
        tracing::warn!(
            "Skipped {} fixes because they were marked as unsafe. To apply those fixes, use the `--unsafe` flag.",
            skipped_unsafe
        );
    }

    if skipped_potentially_unsafe > 0 {
        tracing::warn!(
            "Skipped {} fixes because they were marked as potentially unsafe. To apply those fixes, use the `--potentially-unsafe` flag.",
            skipped_potentially_unsafe
        );
    }

    if changed == 0 {
        tracing::info!("No fixes were applied");

        return Ok(ExitCode::SUCCESS);
    }

    Ok(if command.dry_run {
        tracing::info!("Found {} fixes that can be applied", changed);

        ExitCode::FAILURE
    } else {
        tracing::info!("Applied {} fixes successfully", changed);

        ExitCode::SUCCESS
    })
}

fn filter_fix_plans(
    interner: &ThreadedInterner,
    issues: IssueCollection,
    classification: SafetyClassification,
) -> (Vec<(SourceIdentifier, FixPlan)>, usize, usize) {
    let mut skipped_unsafe = 0;
    let mut skipped_potentially_unsafe = 0;

    let mut results = vec![];
    for (source, plan) in issues.to_fix_plans() {
        if plan.is_empty() {
            continue;
        }

        let mut operations = vec![];
        for operation in plan.take_operations() {
            match operation.get_safety_classification() {
                SafetyClassification::Unsafe => {
                    if classification == SafetyClassification::Unsafe {
                        operations.push(operation);
                    } else {
                        skipped_unsafe += 1;

                        tracing::warn!(
                            "Skipping a fix for `{}` because it contains unsafe changes.",
                            interner.lookup(&source.0)
                        );
                    }
                }
                SafetyClassification::PotentiallyUnsafe => {
                    if classification == SafetyClassification::Unsafe
                        || classification == SafetyClassification::PotentiallyUnsafe
                    {
                        operations.push(operation);
                    } else {
                        skipped_potentially_unsafe += 1;

                        tracing::warn!(
                            "Skipping a fix for `{}` because it contains potentially unsafe changes.",
                            interner.lookup(&source.0)
                        );
                    }
                }
                SafetyClassification::Safe => {
                    operations.push(operation);
                }
            }
        }

        if !operations.is_empty() {
            results.push((source, FixPlan::from_operations(operations)));
        }
    }

    (results, skipped_unsafe, skipped_potentially_unsafe)
}
