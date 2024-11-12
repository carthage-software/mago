use serde::Deserialize;
use serde::Serialize;
use strum::Display;

use fennec_interner::StringIdentifier;
use fennec_span::HasSpan;
use fennec_span::Span;

/// Represents the kind of trivia.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
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
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Trivia {
    pub kind: TriviaKind,
    pub span: Span,
    pub value: StringIdentifier,
}

impl TriviaKind {
    /// Returns `true` if the trivia kind is a comment.
    pub fn is_comment(&self) -> bool {
        matches!(
            self,
            TriviaKind::SingleLineComment
                | TriviaKind::MultiLineComment
                | TriviaKind::HashComment
                | TriviaKind::DocBlockComment
        )
    }

    pub fn is_block_comment(&self) -> bool {
        matches!(self, TriviaKind::MultiLineComment | TriviaKind::DocBlockComment)
    }

    pub fn is_single_line_comment(&self) -> bool {
        matches!(self, TriviaKind::HashComment | TriviaKind::SingleLineComment)
    }
}

impl HasSpan for Trivia {
    fn span(&self) -> Span {
        self.span
    }
}
