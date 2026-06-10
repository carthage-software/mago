#![expect(clippy::module_inception)]

use std::fmt::Debug;

use mago_database::file::FileId;
use mago_span::HasSpan;
use mago_span::Position;
use mago_span::Span;

pub use crate::cst::cst::*;
pub use crate::cst::node::*;
pub use crate::cst::sequence::Sequence;
pub use crate::cst::sequence::TokenSeparatedSequence;
pub use crate::cst::sequence::TokenSeparatedSequenceExt;
pub use crate::cst::trivia::Trivia;
pub use crate::cst::trivia::TriviaKind;
pub use crate::cst::trivia::TriviaSequenceExt;
use crate::error::ParseError;

pub mod cst;
pub mod node;
pub mod sequence;
pub mod trivia;

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Program<'arena> {
    pub file_id: FileId,
    pub source_text: &'arena [u8],
    pub trivia: Sequence<'arena, Trivia<'arena>>,
    pub statements: Sequence<'arena, Statement<'arena>>,
    pub errors: &'arena [ParseError],
}

impl Program<'_> {
    /// Returns `true` if the program contains any parsing errors.
    #[inline]
    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Returns `true` if the program contains any non-inline script statements.
    #[must_use]
    pub fn has_script(&self) -> bool {
        for statement in &self.statements {
            if !matches!(statement, Statement::Inline(_)) {
                return true;
            }
        }

        false
    }
}

impl HasSpan for Program<'_> {
    fn span(&self) -> Span {
        let start = self.statements.first().map_or_else(Position::zero, |stmt| stmt.span().start);
        let end = self.statements.last().map_or_else(Position::zero, |stmt| stmt.span().end);

        Span::new(self.file_id, start, end)
    }
}
