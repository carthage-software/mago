/// Settings for the lexer.
#[derive(Debug, Clone, Copy)]
pub struct LexerSettings {
    /// Whether to enable short open tags (`<?`).
    ///
    /// When enabled (default), `<?` is treated as a short open tag that switches to PHP scripting mode.
    /// When disabled, only `<?php` and `<?=` are recognized as valid PHP open tags.
    ///
    /// This corresponds to PHP's `short_open_tag` ini directive.
    pub enable_short_tags: bool,
}

/// Settings for the parser.
#[derive(Debug, Clone, Copy, Default)]
pub struct ParserSettings {
    /// Settings for the lexer.
    pub lexer: LexerSettings,
    /// Keep unfinished bodies and lists at EOF for editor queries.
    /// Missing delimiters still produce parse errors.
    pub recover_incomplete: bool,
}

impl Default for LexerSettings {
    fn default() -> Self {
        Self { enable_short_tags: true }
    }
}
