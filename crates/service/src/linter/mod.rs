use fennec_config::linter::LinterLevel;
use fennec_config::Configuration;
use fennec_interner::ThreadedInterner;
use fennec_linter::plugin::best_practices::BestPracticesPlugin;
use fennec_linter::plugin::comment::CommentPlugin;
use fennec_linter::plugin::consistency::ConsistencyPlugin;
use fennec_linter::plugin::naming::NamingPlugin;
use fennec_linter::plugin::redundancy::RedundancyPlugin;
use fennec_linter::plugin::safety::SafetyPlugin;
use fennec_linter::plugin::strictness::StrictnessPlugin;
use fennec_linter::plugin::symfony::SymfonyPlugin;
use fennec_linter::settings::RuleSettings;
use fennec_linter::settings::Settings;
use fennec_linter::Linter;
use fennec_reporting::Issue;
use fennec_reporting::IssueCollection;
use fennec_reporting::Level;
use fennec_semantics::Semantics;
use fennec_source::error::SourceError;
use fennec_source::SourceIdentifier;
use fennec_source::SourceManager;

use crate::linter::result::LintResult;

pub mod result;

#[derive(Debug)]
pub struct LintService {
    configuration: Configuration,
    interner: ThreadedInterner,
    source_manager: SourceManager,
}

impl LintService {
    pub fn new(configuration: Configuration, interner: ThreadedInterner, source_manager: SourceManager) -> Self {
        Self { configuration, interner, source_manager }
    }

    /// Runs the linting process and returns a stream of issues.
    pub async fn run(&self) -> Result<LintResult, SourceError> {
        let source_ids: Vec<_> = self.source_manager.source_ids().collect();

        // Initialize the linter
        let linter = self.initialize_linter();

        // Filter source ids based on linter settings
        let filter_source_ids = self.filter_semantics(&linter, source_ids);

        // Process sources concurrently
        self.process_sources(linter, filter_source_ids).await
    }

    #[inline]
    async fn process_sources(
        &self,
        linter: Linter,
        source_ids: Vec<SourceIdentifier>,
    ) -> Result<LintResult, SourceError> {
        let mut handles = Vec::with_capacity(source_ids.len());
        for source_id in source_ids {
            let interner = self.interner.clone();
            let manager = self.source_manager.clone();
            let linter = linter.clone();

            handles.push(tokio::spawn(async move {
                let source = manager.load(source_id).await?;
                let semantics = Semantics::build(&interner, source);
                let mut issues = linter.lint(&semantics);
                issues.extend(semantics.issues);
                if let Some(error) = &semantics.parse_error {
                    issues.push(Into::<Issue>::into(error));
                }

                Result::<_, SourceError>::Ok(issues)
            }));
        }

        let mut results = Vec::with_capacity(handles.len());
        for handle in handles {
            results.push(handle.await.expect("failed to collect issues. this should never happen.")?);
        }

        Ok(LintResult::new(IssueCollection::from(results.into_iter().flatten())))
    }

    #[inline]
    fn initialize_linter(&self) -> Linter {
        let mut settings = Settings::new();

        if let Some(level) = self.configuration.linter.level {
            settings = match level {
                LinterLevel::Off => settings.off(),
                LinterLevel::Help => settings.with_level(Level::Help),
                LinterLevel::Note => settings.with_level(Level::Note),
                LinterLevel::Warning => settings.with_level(Level::Warning),
                LinterLevel::Error => settings.with_level(Level::Error),
            };
        }

        if let Some(external) = self.configuration.linter.external {
            settings = settings.with_external(external);
        }

        if let Some(default_plugins) = self.configuration.linter.default_plugins {
            settings = settings.with_default_plugins(default_plugins);
        }

        settings = settings.with_plugins(self.configuration.linter.plugins.clone());

        for rule in &self.configuration.linter.rules {
            let mut rule_settings = match rule.level {
                Some(linter_level) => match linter_level {
                    LinterLevel::Off => RuleSettings::disabled(),
                    LinterLevel::Help => RuleSettings::from_level(Some(Level::Help)),
                    LinterLevel::Note => RuleSettings::from_level(Some(Level::Note)),
                    LinterLevel::Warning => RuleSettings::from_level(Some(Level::Warning)),
                    LinterLevel::Error => RuleSettings::from_level(Some(Level::Error)),
                },
                None => RuleSettings::enabled(),
            };

            if let Some(options) = rule.options.clone() {
                rule_settings = rule_settings.with_options(options.clone());
            }

            settings = settings.with_rule(rule.name.clone(), rule_settings);
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

        linter
    }

    #[inline]
    fn filter_semantics(&self, linter: &Linter, source_ids: Vec<SourceIdentifier>) -> Vec<SourceIdentifier> {
        if !linter.settings.external {
            source_ids.into_iter().filter(|s| !s.is_external()).collect()
        } else {
            source_ids
        }
    }
}
