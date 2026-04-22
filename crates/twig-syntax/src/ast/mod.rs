//! AST types for Twig templates.
//!
//! * [`Template`] is the root. It carries the source slice, a trivia channel,
//!   a statement sequence, and any deferred parse errors.
//! * [`Statement`] is the enum of significant body constructs - raw text,
//!   `{{ expr }}` prints, `{% verbatim %}` blocks, and every `{% tag %}`
//!   form.
//! * [`Trivia`] covers comments (`{# ... #}`, inline `# ...`) and every
//!   whitespace run inside `{% %}` / `{{ }}` blocks. Trivia are surfaced on
//!   the root so consumers can recover the exact source by interleaving
//!   statements and trivia by span.
//!
//! All nodes are arena-allocated in a [`bumpalo::Bump`]; string fragments
//! borrow directly from the source template wherever possible.

pub use crate::ast::expression::*;
pub use crate::ast::keyword::*;
pub use crate::ast::node::*;
pub use crate::ast::sequence::*;
pub use crate::ast::statement::*;
pub use crate::ast::trivia::*;

pub mod expression;
pub mod keyword;
pub mod node;
pub mod sequence;
pub mod statement;
pub mod trivia;

use bumpalo::collections::Vec as BVec;
use serde::Serialize;

use mago_database::file::FileId;
use mago_database::file::HasFileId;
use mago_span::HasSpan;
use mago_span::Position;
use mago_span::Span;

use crate::error::ParseError;

/// A fully parsed Twig template.
///
/// The template preserves its source text and every piece of trivia encountered
/// by the lexer, so that `(statements, trivia)` can be re-serialised back to
/// the original bytes (though consumers must interleave the two by span -
/// trivia are not embedded in the statement tree).
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Template<'arena> {
    pub file_id: FileId,
    pub source_text: &'arena str,
    pub trivia: Sequence<'arena, Trivia<'arena>>,
    pub statements: Sequence<'arena, Statement<'arena>>,
    pub errors: BVec<'arena, ParseError>,
}

impl Template<'_> {
    /// Returns `true` if parsing produced any deferred errors.
    #[inline]
    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Returns `true` if the template contains any statement other than
    /// raw [`Text`] - i.e. at least one template construct (print, tag,
    /// verbatim).
    #[inline]
    #[must_use]
    pub fn has_script(&self) -> bool {
        self.statements.iter().any(|s| !matches!(s, Statement::Text(_)))
    }
}

impl HasFileId for Template<'_> {
    fn file_id(&self) -> FileId {
        self.file_id
    }
}

impl HasSpan for Template<'_> {
    fn span(&self) -> Span {
        let start = self.statements.first().map_or_else(Position::zero, |s| s.span().start);
        let end = self.statements.last().map_or_else(Position::zero, |s| s.span().end);

        Span::new(self.file_id, start, end)
    }
}
