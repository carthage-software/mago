use clap::Parser;

use fennec_fixer::SafetyClassification;
use fennec_interner::ThreadedInterner;
use fennec_service::config::Configuration;
use fennec_service::linter::LintService;
use fennec_service::source::SourceService;
use fennec_source::error::SourceError;

use crate::utils::bail;

#[derive(Parser, Debug)]
#[command(
    name = "fix",
    about = "Fix lint issues identified during the linting process",
    long_about = r#"
Fix lint issues identified during the linting process.

Automatically applies fixes where possible, based on the rules in the `fennec.toml` or the default settings.
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
    pub fn get_safety_classification(&self) -> SafetyClassification {
        if self.r#unsafe {
            SafetyClassification::Unsafe
        } else if self.potentially_unsafe {
            SafetyClassification::PotentiallyUnsafe
        } else {
            SafetyClassification::Safe
        }
    }
}

pub async fn execute(command: FixCommand, configuration: Configuration) -> i32 {
    let interner = ThreadedInterner::new();

    let source_service = SourceService::new(interner.clone(), configuration.source);
    let source_manager = source_service.load().await.unwrap_or_else(bail);

    let service = LintService::new(configuration.linter, interner.clone(), source_manager.clone());

    let classification = command.get_safety_classification();

    let fix_plans = service.run().await.unwrap_or_else(bail).to_fix_plans().into_iter().filter_map(|(source, plan)| {
        let plan = plan.to_minimum_safety_classification(classification);

        if plan.is_empty() {
            None
        } else {
            Some((source, plan))
        }
    });

    let mut handles = vec![];
    for (source, plan) in fix_plans.into_iter() {
        handles.push(tokio::spawn({
            let source_manager = source_manager.clone();
            let interner = interner.clone();

            async move {
                let source = source_manager.load(source).await.unwrap_or_else(bail);
                let source_name = interner.lookup(&source.identifier.value());
                let source_content = interner.lookup(&source.content);

                fennec_feedback::info!("fixing issues in `{}` ( {} fix operations )", source_name, plan.len());

                let code = plan.execute(source_content);

                if command.dry_run {
                    // todo, print the diff in a pretty way
                    println!("TOO LAZY TO PRETTY PRINT: {:#?}", code);
                } else {
                    source_manager.write(source.identifier, code.get_fixed())?
                }

                Ok::<(), SourceError>(())
            }
        }));
    }

    for handle in handles {
        handle.await.unwrap_or_else(bail).unwrap_or_else(bail);
    }

    0
}
