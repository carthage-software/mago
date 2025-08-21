#![expect(clippy::module_inception)]

use std::fmt::Debug;

use serde::Serialize;

use mago_database::file::FileId;
use mago_span::HasSpan;
use mago_span::Position;
use mago_span::Span;

pub use crate::ast::ast::*;
pub use crate::ast::node::*;
pub use crate::ast::sequence::Sequence;
pub use crate::ast::trivia::Trivia;
pub use crate::ast::trivia::TriviaKind;

pub mod ast;
pub mod node;
pub mod sequence;
pub mod trivia;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Program<'arena> {
    pub file_id: FileId,
    pub source_text: &'arena str,
    pub trivia: Sequence<'arena, Trivia<'arena>>,
    pub statements: Sequence<'arena, Statement<'arena>>,
}

impl Program<'_> {
    pub fn has_script(&self) -> bool {
        for statement in self.statements.iter() {
            if !matches!(statement, Statement::Inline(_)) {
                return true;
            }
        }

        false
    }
}

impl HasSpan for Program<'_> {
    fn span(&self) -> Span {
        let start = self.statements.first().map(|stmt| stmt.span().start).unwrap_or_else(Position::zero);
        let end = self.statements.last().map(|stmt| stmt.span().end).unwrap_or_else(Position::zero);

        Span::new(self.file_id, start, end)
    }
}
