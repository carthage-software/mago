use mago_feedback::create_progress_bar;
use mago_feedback::remove_progress_bar;
use mago_feedback::ProgressBarTheme;
use mago_fixer::FixPlan;
use mago_fixer::SafetyClassification;
use mago_interner::ThreadedInterner;
use mago_linter::plugin::best_practices::BestPracticesPlugin;
use mago_linter::plugin::comment::CommentPlugin;
use mago_linter::plugin::consistency::ConsistencyPlugin;
use mago_linter::plugin::naming::NamingPlugin;
use mago_linter::plugin::phpunit::PHPUnitPlugin;
use mago_linter::plugin::redundancy::RedundancyPlugin;
use mago_linter::plugin::safety::SafetyPlugin;
use mago_linter::plugin::strictness::StrictnessPlugin;
use mago_linter::plugin::symfony::SymfonyPlugin;
use mago_linter::settings::RuleSettings;
use mago_linter::settings::Settings;
use mago_linter::Linter;
use mago_reporting::Issue;
use mago_reporting::IssueCollection;
use mago_reporting::Level;
use mago_semantics::Semantics;
use mago_source::error::SourceError;
use mago_source::SourceIdentifier;
use mago_source::SourceManager;

use crate::linter::config::LinterConfiguration;
use crate::linter::config::LinterLevel;
use crate::utils;

pub mod config;

#[derive(Debug)]
pub struct LintService {
    configuration: LinterConfiguration,
    interner: ThreadedInterner,
    source_manager: SourceManager,
}

#[derive(Debug)]
pub struct LinterFixResult {
    pub skipped_unsafe: usize,
    pub skipped_potentially_unsafe: usize,
    pub changed: usize,
}

impl LintService {
    pub fn new(configuration: LinterConfiguration, interner: ThreadedInterner, source_manager: SourceManager) -> Self {
        Self { configuration, interner, source_manager }
    }

    /// Runs the linting process and returns a collection of issues.
    pub async fn run(&self) -> Result<IssueCollection, SourceError> {
        // Initialize the linter
        let linter = self.initialize_linter();

        // Process sources concurrently
        self.process_sources(linter, self.source_manager.user_defined_source_ids().collect()).await
    }

    /// Runs the linting process and returns a collection of issues.
    pub async fn fix(
        &self,
        r#unsafe: bool,
        potentially_unsafe: bool,
        dry_run: bool,
    ) -> Result<LinterFixResult, SourceError> {
        let classification = if r#unsafe {
            SafetyClassification::Unsafe
        } else if potentially_unsafe {
            SafetyClassification::PotentiallyUnsafe
        } else {
            SafetyClassification::Safe
        };

        let mut skipped_unsafe = 0;
        let mut skipped_potentially_unsafe = 0;
        let fix_plans = self
            .run()
            .await?
            .to_fix_plans()
            .into_iter()
            .filter_map(|(source, plan)| {
                if plan.is_empty() {
                    return None;
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
                                    self.interner.lookup(&source.0)
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
                                    self.interner.lookup(&source.0)
                                );
                            }
                        }
                        SafetyClassification::Safe => {
                            operations.push(operation);
                        }
                    }
                }

                if operations.is_empty() {
                    None
                } else {
                    Some((source, FixPlan::from_operations(operations)))
                }
            })
            .collect::<Vec<_>>();

        let fix_pb = create_progress_bar(fix_plans.len(), "✨  Fixing", ProgressBarTheme::Magenta);
        let mut handles = vec![];
        for (source, plan) in fix_plans.into_iter() {
            handles.push(tokio::spawn({
                let source_manager = self.source_manager.clone();
                let interner = self.interner.clone();
                let fix_pb = fix_pb.clone();

                async move {
                    let source = source_manager.load(source)?;
                    let source_content = interner.lookup(&source.content);
                    let result = utils::apply_changes(
                        &interner,
                        &source_manager,
                        &source,
                        plan.execute(source_content).get_fixed(),
                        dry_run,
                    );

                    fix_pb.inc(1);

                    result
                }
            }));
        }

        let mut changed = 0;
        for handle in handles {
            if handle.await.expect("failed to fix sources, this should never happen.")? {
                changed += 1;
            }
        }

        Ok(LinterFixResult { skipped_unsafe, skipped_potentially_unsafe, changed })
    }

    #[inline]
    async fn process_sources(
        &self,
        linter: Linter,
        source_ids: Vec<SourceIdentifier>,
    ) -> Result<IssueCollection, SourceError> {
        let mut handles = Vec::with_capacity(source_ids.len());

        let source_pb = create_progress_bar(source_ids.len(), "📂  Loading", ProgressBarTheme::Red);
        let semantics_pb = create_progress_bar(source_ids.len(), "🔬  Building", ProgressBarTheme::Blue);
        let lint_pb = create_progress_bar(source_ids.len(), "🧹  Linting", ProgressBarTheme::Cyan);

        for source_id in source_ids.into_iter() {
            handles.push(tokio::spawn({
                let interner = self.interner.clone();
                let manager = self.source_manager.clone();
                let linter = linter.clone();
                let source_pb = source_pb.clone();
                let semantics_pb = semantics_pb.clone();
                let lint_pb = lint_pb.clone();

                async move {
                    // Step 1: load the source
                    let source = manager.load(source_id)?;
                    source_pb.inc(1);

                    // Step 2: build semantics
                    let semantics = Semantics::build(&interner, source);
                    semantics_pb.inc(1);

                    // Step 3: Collect issues
                    let mut issues = linter.lint(&semantics);
                    issues.extend(semantics.issues);
                    if let Some(error) = &semantics.parse_error {
                        issues.push(Into::<Issue>::into(error));
                    }
                    lint_pb.inc(1);

                    Result::<_, SourceError>::Ok(issues)
                }
            }));
        }

        let mut results = Vec::with_capacity(handles.len());
        for handle in handles {
            results.push(handle.await.expect("failed to collect issues. this should never happen.")?);
        }

        remove_progress_bar(source_pb);
        remove_progress_bar(semantics_pb);
        remove_progress_bar(lint_pb);

        Ok(IssueCollection::from(results.into_iter().flatten()))
    }

    #[inline]
    fn initialize_linter(&self) -> Linter {
        let mut settings = Settings::new();

        if let Some(level) = self.configuration.level {
            settings = match level {
                LinterLevel::Off => settings.off(),
                LinterLevel::Help => settings.with_level(Level::Help),
                LinterLevel::Note => settings.with_level(Level::Note),
                LinterLevel::Warning => settings.with_level(Level::Warning),
                LinterLevel::Error => settings.with_level(Level::Error),
            };
        }

        if let Some(default_plugins) = self.configuration.default_plugins {
            settings = settings.with_default_plugins(default_plugins);
        }

        settings = settings.with_plugins(self.configuration.plugins.clone());

        for rule in &self.configuration.rules {
            let rule_settings = match rule.level {
                Some(linter_level) => match linter_level {
                    LinterLevel::Off => RuleSettings::disabled(),
                    LinterLevel::Help => RuleSettings::from_level(Some(Level::Help)),
                    LinterLevel::Note => RuleSettings::from_level(Some(Level::Note)),
                    LinterLevel::Warning => RuleSettings::from_level(Some(Level::Warning)),
                    LinterLevel::Error => RuleSettings::from_level(Some(Level::Error)),
                },
                None => RuleSettings::enabled(),
            };

            settings = settings.with_rule(rule.name.clone(), rule_settings.with_options(rule.options.clone()));
        }

        let mut linter = Linter::new(settings, self.interner.clone());

        linter.add_plugin(BestPracticesPlugin);
        linter.add_plugin(CommentPlugin);
        linter.add_plugin(ConsistencyPlugin);
        linter.add_plugin(NamingPlugin);
        linter.add_plugin(RedundancyPlugin);
        linter.add_plugin(SafetyPlugin);
        linter.add_plugin(StrictnessPlugin);
        linter.add_plugin(SymfonyPlugin);
        linter.add_plugin(PHPUnitPlugin);

        linter
    }
}
