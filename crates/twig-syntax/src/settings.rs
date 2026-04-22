/// Settings for the Twig lexer.
///
/// Reserved for future behavioural knobs.
#[derive(Debug, Clone, Copy, Default)]
pub struct LexerSettings {}

/// Settings for the Twig parser.
#[derive(Debug, Clone, Copy, Default)]
pub struct ParserSettings {
    /// Settings for the lexer.
    pub lexer: LexerSettings,
}
