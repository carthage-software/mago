//! Long-lived linter context shared with [`crate::file_analysis`].
//!
//! The actual lint pass lives in [`crate::file_analysis::build`] so the
//! parse + resolve work is shared with the rest of the per-file derived
//! data. This module just owns the rule registry + settings.

use std::sync::Arc;

use mago_linter::registry::RuleRegistry;
use mago_linter::settings::Settings as LinterSettings;
use mago_syntax::settings::ParserSettings;

#[derive(Debug)]
pub struct LinterContext {
    pub settings: LinterSettings,
    pub parser_settings: ParserSettings,
    pub registry: Arc<RuleRegistry>,
}

impl LinterContext {
    #[must_use]
    pub fn new(settings: LinterSettings, parser_settings: ParserSettings) -> Self {
        let registry = Arc::new(RuleRegistry::build(&settings, None, false));
        Self { settings, parser_settings, registry }
    }
}
