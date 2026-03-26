use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_syntax::settings::LexerSettings;
use mago_syntax::settings::ParserSettings;

/// Configuration for the PHP parser.
///
/// Controls how PHP code is parsed, including lexer-level settings
/// that affect tokenization behavior.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct ParserConfiguration {
    /// Whether to enable PHP short open tags (`<?`).
    ///
    /// When enabled (default), `<?` is recognized as a PHP open tag,
    /// equivalent to PHP's `short_open_tag` ini directive being on.
    ///
    /// When disabled, only `<?php` and `<?=` are recognized as PHP open tags.
    /// The sequence `<?` will be treated as inline text.
    ///
    /// This setting is useful for projects that need to parse PHP files
    /// that may contain XML or other content using `<?` sequences.
    #[serde(default = "default_enable_short_tags")]
    pub enable_short_tags: bool,
}

fn default_enable_short_tags() -> bool {
    true
}

impl Default for ParserConfiguration {
    fn default() -> Self {
        Self { enable_short_tags: default_enable_short_tags() }
    }
}

impl ParserConfiguration {
    /// Converts this configuration into `ParserSettings` for use with the parser.
    #[must_use]
    pub fn to_settings(&self) -> ParserSettings {
        ParserSettings { lexer: LexerSettings { enable_short_tags: self.enable_short_tags } }
    }
}
