/// The mode the lexer is currently in.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum LexerMode {
    /// Outside of any tag, lexing raw template text.
    Data,
    /// Inside `{% ... %}`.
    Block,
    /// Inside `{{ ... }}`.
    Variable,
    /// Inside an interpolating `"..."` string.
    DoubleQuoted,
    /// Inside `#{ ... }` interpolation within a double-quoted string.
    Interpolation,
    /// Inside a `{% verbatim %}` or `{% raw %}` tag - lexed as a strict
    /// linear sequence of tokens that no other state machine touches.
    Verbatim(VerbatimKind, VerbatimStage),
}

/// Which keyword opened the current verbatim region.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum VerbatimKind {
    /// `{% verbatim %} ... {% endverbatim %}`.
    Verbatim,
    /// `{% raw %} ... {% endraw %}`.
    Raw,
}

/// The current stage of a verbatim/raw tag's lexing pipeline.  Each stage
/// emits exactly one significant token (with optional preceding whitespace
/// trivia tokens) and transitions to the next stage.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum VerbatimStage {
    /// Just after the opening `{%` was emitted.  Next significant token is
    /// the keyword Name (`verbatim` or `raw`).
    Entering,
    /// Just after the keyword Name was emitted.  Next significant token is
    /// the closing `%}` of the opening tag.
    Emitted,
    /// Just after the opening tag's `%}` was emitted.  Next emission is the
    /// raw body covering everything up to the closing `{%`.
    Body,
    /// Body has been emitted (or skipped if empty).  Next emission is the
    /// closing tag's `{%`.
    EndingOpened,
    /// Just after the closing `{%` was emitted.  Next significant token is
    /// the end keyword Name (`endverbatim` or `endraw`).
    EndOpened,
    /// Just after the end keyword Name was emitted.  Next significant
    /// token is the final `%}`.  Returns to [`LexerMode::Data`] after.
    Ended,
}

impl VerbatimKind {
    /// The opening keyword bytes (`b"verbatim"` or `b"raw"`).
    #[inline]
    pub(crate) const fn open_keyword(self) -> &'static [u8] {
        match self {
            VerbatimKind::Verbatim => b"verbatim",
            VerbatimKind::Raw => b"raw",
        }
    }

    /// The closing keyword bytes (`b"endverbatim"` or `b"endraw"`).
    #[inline]
    pub(crate) const fn end_keyword(self) -> &'static [u8] {
        match self {
            VerbatimKind::Verbatim => b"endverbatim",
            VerbatimKind::Raw => b"endraw",
        }
    }
}

impl LexerMode {
    /// Human-readable name for diagnostics about unclosed tags.
    #[inline]
    pub(crate) const fn tag_name(self) -> &'static str {
        match self {
            LexerMode::Block => "block tag",
            LexerMode::Variable => "variable tag",
            LexerMode::DoubleQuoted => "string",
            LexerMode::Interpolation => "interpolation",
            LexerMode::Data => "tag",
            LexerMode::Verbatim(_, _) => "verbatim tag",
        }
    }
}
