use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::Sequence;

/// Kinds of trivia preserved in a [`Template`](crate::ast::Template).
///
/// Trivia are tokens that carry no syntactic meaning for the parse tree
/// proper but are retained so that the template source can be recovered
/// from the AST.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
pub enum TriviaKind {
    /// Whitespace inside `{% %}` / `{{ }}` / `{# #}` blocks that does not
    /// appear in the significant token stream.
    Whitespace,
    /// A template-level comment: `{# ... #}` (possibly with trim markers).
    Comment,
    /// A `# ...` comment that ends at the next newline, inside an
    /// expression.
    InlineComment,
}

/// A piece of trivia retained on the template root.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Trivia<'arena> {
    pub kind: TriviaKind,
    pub span: Span,
    pub value: &'arena str,
}

impl TriviaKind {
    #[inline]
    #[must_use]
    pub const fn is_comment(self) -> bool {
        matches!(self, TriviaKind::Comment | TriviaKind::InlineComment)
    }

    #[inline]
    #[must_use]
    pub const fn is_whitespace(self) -> bool {
        matches!(self, TriviaKind::Whitespace)
    }
}

impl HasSpan for Trivia<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

/// Iteration helpers over a trivia [`Sequence`].
pub trait TriviaSequenceExt<'arena> {
    fn comments<'borrow>(&'borrow self) -> impl Iterator<Item = &'borrow Trivia<'arena>>
    where
        'arena: 'borrow;
}

impl<'arena> TriviaSequenceExt<'arena> for Sequence<'arena, Trivia<'arena>> {
    #[inline]
    fn comments<'borrow>(&'borrow self) -> impl Iterator<Item = &'borrow Trivia<'arena>>
    where
        'arena: 'borrow,
    {
        self.iter().filter(|t| t.kind.is_comment())
    }
}
