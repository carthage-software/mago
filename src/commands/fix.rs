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

use crate::commands::lint::process_sources;
use crate::config::Configuration;
use crate::error::Error;
use crate::source;
use crate::utils;

#[derive(Parser, Debug)]
#[command(
    name = "fix",
    about = "Fix lint issues identified during the linting process",
    long_about = r#"
Fix lint issues identified during the linting process.

Automatically applies fixes where possible, based on the rules in the `mago.toml` or the default settings.
    "#
)]
pub struct FixCommand {
    #[arg(long, short, help = "Apply fixes that are marked as unsafe, including potentially unsafe fixes")]
    pub r#unsafe: bool,
    #[arg(long, short, help = "Apply fixes that are marked as potentially unsafe")]
    pub potentially_unsafe: bool,
    #[arg(long, short, help = "Run the command without writing any changes to disk")]
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

    let issues = process_sources(&interner, &source_manager, &configuration.linter).await?;
    let (plans, skipped_unsafe, skipped_potentially_unsafe) =
        filter_fix_plans(&interner, issues, command.get_classification());

    let total = plans.len();
    let mut handles = Vec::with_capacity(total);
    for (source, plan) in plans.into_iter() {
        handles.push(tokio::spawn({
            let source_manager = source_manager.clone();
            let interner = interner.clone();

            async move {
                let source = source_manager.load(&source)?;
                let source_content = interner.lookup(&source.content);

                utils::apply_changes(
                    &interner,
                    &source_manager,
                    &source,
                    plan.execute(source_content).get_fixed(),
                    command.dry_run,
                )
            }
        }));
    }

    let progress_bar = create_progress_bar(total, "✨  Fixing", ProgressBarTheme::Magenta);
    let mut changed = 0;
    for handle in handles {
        if handle.await?? {
            changed += 1;
        }

        progress_bar.inc(1);
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
