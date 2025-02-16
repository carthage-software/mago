use std::fmt::Debug;

use serde::Serialize;

use mago_source::SourceIdentifier;
use mago_span::HasSpan;
use mago_span::Position;
use mago_span::Span;

pub use crate::ast::*;
pub use crate::node::*;
pub use crate::sequence::Sequence;
pub use crate::trivia::Trivia;
pub use crate::trivia::TriviaKind;

pub mod ast;
pub mod node;
pub mod sequence;
pub mod trivia;

#[derive(Debug, Hash, Serialize)]
pub struct Program<'a> {
    pub source: SourceIdentifier,
    pub trivia: Sequence<'a, Trivia>,
    pub statements: Sequence<'a, Statement<'a>>,
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
        let start =
            self.statements.first().map(|stmt| stmt.span().start).unwrap_or_else(|| Position::start_of(self.source));

        let end = self.statements.last().map(|stmt| stmt.span().end).unwrap_or_else(|| Position::start_of(self.source));

        Span::new(start, end)
    }
}
