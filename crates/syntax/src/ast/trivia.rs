use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::Sequence;

/// Represents the kind of trivia.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
pub enum TriviaKind {
    WhiteSpace,
    SingleLineComment,
    MultiLineComment,
    HashComment,
    DocBlockComment,
}

/// Represents a trivia.
///
/// A trivia is a piece of information that is not part of the syntax tree,
/// such as comments and white spaces.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Trivia<'arena> {
    pub kind: TriviaKind,
    pub span: Span,
    pub value: &'arena str,
}

impl TriviaKind {
    /// Returns `true` if the trivia kind is a comment.
    #[inline]
    pub const fn is_comment(&self) -> bool {
        matches!(
            self,
            TriviaKind::SingleLineComment
                | TriviaKind::MultiLineComment
                | TriviaKind::HashComment
                | TriviaKind::DocBlockComment
        )
    }

    #[inline]
    pub const fn is_docblock(&self) -> bool {
        matches!(self, TriviaKind::DocBlockComment)
    }

    #[inline]
    pub const fn is_block_comment(&self) -> bool {
        matches!(self, TriviaKind::MultiLineComment | TriviaKind::DocBlockComment)
    }

    #[inline]
    pub const fn is_single_line_comment(&self) -> bool {
        matches!(self, TriviaKind::HashComment | TriviaKind::SingleLineComment)
    }
}

impl HasSpan for Trivia<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<'arena> Sequence<'arena, Trivia<'arena>> {
    /// Returns an iterator over the comments in the sequence.
    pub fn comments(&self) -> impl Iterator<Item = &Trivia<'arena>> {
        self.iter().filter(|trivia| trivia.kind.is_comment())
    }
}
