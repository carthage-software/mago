//! Long-lived linter context shared with [`crate::file_analysis`].
//!
//! The actual lint pass lives in [`crate::file_analysis::build`] so the
//! parse + resolve work is shared with the rest of the per-file derived
//! data. This module just owns the rule registry + settings.

use std::sync::Arc;

use mago_database::file::File;
use mago_database::membership::PathExclusions;
use mago_linter::registry::RuleRegistry;
use mago_linter::settings::Settings as LinterSettings;
use mago_syntax::settings::ParserSettings;

#[derive(Debug)]
pub struct LinterContext {
    pub settings: LinterSettings,
    pub parser_settings: ParserSettings,
    pub registry: Option<Arc<RuleRegistry>>,
    pub excludes: PathExclusions,
}

impl LinterContext {
    #[must_use]
    pub fn new(settings: LinterSettings, parser_settings: ParserSettings, enabled: bool) -> Self {
        let registry = enabled.then(|| Arc::new(RuleRegistry::build(&settings, None, false)));
        Self { settings, parser_settings, registry, excludes: PathExclusions::default() }
    }

    #[must_use]
    pub fn with_excludes(self, excludes: PathExclusions) -> Self {
        Self { excludes, ..self }
    }

    /// The rules to run on `file`, if it is a host file the linter covers.
    #[must_use]
    pub fn registry_for(&self, file: &File) -> Option<&Arc<RuleRegistry>> {
        self.registry.as_ref().filter(|_| file.file_type.is_host() && !self.excludes.excludes_file(file))
    }
}
