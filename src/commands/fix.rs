use std::process::ExitCode;

use clap::Parser;

use mago_feedback::create_progress_bar;
use mago_feedback::remove_progress_bar;
use mago_feedback::ProgressBarTheme;
use mago_fixer::FixPlan;
use mago_fixer::SafetyClassification;
use mago_interner::ThreadedInterner;
use mago_reporting::IssueCollection;
use mago_source::SourceIdentifier;

use crate::commands::lint::lint_sources;
use crate::config::Configuration;
use crate::error::Error;
use crate::source;
use crate::utils;

#[derive(Parser, Debug)]
#[command(
    name = "fix",
    about = "apply fixes for lint issues identified during linting",
    long_about = r#"
The `fix` command automatically applies fixes for issues identified during the linting process.

This command streamlines the process of addressing lint issues, improving code quality and consistency.
"#
)]
pub struct FixCommand {
    /// Apply fixes that are marked as unsafe, including potentially unsafe fixes.
    #[arg(
        long,
        short = 'u',
        help = "apply fixes marked as unsafe, including those with potentially destructive changes"
    )]
    pub r#unsafe: bool,

    /// Apply fixes that are marked as potentially unsafe.
    #[arg(long, short = 'p', help = "apply fixes marked as potentially unsafe, which may require manual review")]
    pub potentially_unsafe: bool,

    /// Run the command without writing any changes to disk.
    #[arg(long, short = 'd', help = "preview the fixes without applying them, showing what changes would be made")]
    pub dry_run: bool,
}

impl FixCommand {
    pub fn get_classification(&self) -> SafetyClassification {
        if self.r#unsafe {
            SafetyClassification::Unsafe
        } else if self.potentially_unsafe {
            SafetyClassification::PotentiallyUnsafe
        } else {
            SafetyClassification::Safe
        }
    }
}

pub async fn execute(command: FixCommand, configuration: Configuration) -> Result<ExitCode, Error> {
    // Initialize the interner for managing identifiers.
    let interner = ThreadedInterner::new();
    // Load sources
    let source_manager = source::load(&interner, &configuration.source, true).await?;

    let issues = lint_sources(&interner, &source_manager, &configuration.linter).await?;
    let (plans, skipped_unsafe, skipped_potentially_unsafe) =
        filter_fix_plans(&interner, issues, command.get_classification());

    let total = plans.len();
    let progress_bar = create_progress_bar(total, "✨  Fixing", ProgressBarTheme::Magenta);
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
        mago_feedback::warn!(
            "Skipped {} fixes because they were marked as unsafe. To apply those fixes, use the `--unsafe` flag.",
            skipped_unsafe
        );
    }

    if skipped_potentially_unsafe > 0 {
        mago_feedback::warn!(
            "Skipped {} fixes because they were marked as potentially unsafe. To apply those fixes, use the `--potentially-unsafe` flag.",
            skipped_potentially_unsafe
        );
    }

    if changed == 0 {
        mago_feedback::info!("No fixes were applied");

        return Ok(ExitCode::SUCCESS);
    }

    Ok(if command.dry_run {
        mago_feedback::info!("Found {} fixes that can be applied", changed);

        ExitCode::FAILURE
    } else {
        mago_feedback::info!("Applied {} fixes successfully", changed);

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

                        mago_feedback::warn!(
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

                        mago_feedback::warn!(
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
