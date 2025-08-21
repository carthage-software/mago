use strum::Display;

use crate::token::DocumentKind;

/// Represents the different stages of the `halt_compiler` keyword.
///
/// The `halt_compiler` keyword is followed by `"("`, `")"`, and `";"` or `"?>"`.
///
/// This enum is used to track the current stage of the `halt_compiler` keyword
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, Display)]
pub enum HaltStage {
    LookingForLeftParenthesis,
    LookingForRightParenthesis,
    LookingForTerminator,
    End,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, Display)]
pub enum Interpolation {
    None,
    Until(u32),
}

/// Represents the different modes the lexer can be in while tokenizing PHP code.
///
/// The lexer operates in various modes to handle different contexts within the code,
/// such as inline text, PHP scripts, strings, and heredoc/nowdoc documents.
/// This allows the lexer to correctly tokenize the input based on the current context.
///
/// The lifetime `'a` is associated with the `DocumentString` variant to tie the label's lifetime
/// to the input being lexed.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, Display)]
pub enum LexerMode<'a> {
    /// **Inline Mode**
    ///
    /// Used when tokenizing content outside of PHP tags.
    ///
    /// In this mode, the lexer treats all content as inline text until it encounters a PHP opening tag.
    /// It collects characters and emits them as `InlineText` tokens.
    Inline,

    /// **Script Mode**
    ///
    /// Used when tokenizing standard PHP code.
    ///
    /// In this mode, the lexer tokenizes PHP code according to PHP syntax rules,
    /// emitting appropriate tokens for keywords, identifiers, operators, etc.
    Script,

    /// **Double Quote String Mode**
    ///
    /// Used when tokenizing double-quoted strings (`"..."`).
    ///
    /// This mode handles string interpolation, escape sequences, and special characters within double-quoted strings.
    /// It allows variable and expression interpolation inside the string.
    DoubleQuoteString(Interpolation),

    /// **Shell Execute String Mode**
    ///
    /// Used when tokenizing shell execution strings enclosed in backticks (`` `...` ``).
    ///
    /// Similar to double-quoted strings, this mode handles interpolation and escape sequences within backticks.
    /// It executes the content as a shell command when evaluated.
    ShellExecuteString(Interpolation),

    /// **Document String Mode**
    ///
    /// Used when tokenizing heredoc and nowdoc strings.
    ///
    /// - `DocumentKind` specifies whether it's a heredoc or nowdoc (e.g., `Heredoc`, `Nowdoc`).
    /// - `&'a [u8]` is a reference to the label used to terminate the document string.
    /// - `Interpolation` specifies whether interpolation is allowed and, if so, where it ends.
    ///
    /// In this mode, the lexer reads until it encounters the closing label that matches the provided label,
    /// typically at the start of a line. Heredocs allow interpolation, whereas nowdocs do not.
    DocumentString(DocumentKind, &'a [u8], Interpolation),

    /// **Halt Mode**
    ///
    /// Used after the `halt_compiler` keyword is encountered, followed by `"("`, `")"`, and `";"`.
    ///
    /// In this mode, the lexer reads all the remaining input and returns it as inline text,
    /// even if it contains PHP opening tags like `<?php`. The lexer does not switch back to `Inline` mode from `Halt` mode.
    ///
    /// **Behavior:**
    ///
    /// - Upon entering `Halt` mode, the lexer consumes all remaining input.
    /// - The consumed content is emitted as a single `InlineText` token.
    /// - The lexer remains in `Halt` mode until EOF is reached.
    Halt(HaltStage),
}
