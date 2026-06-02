use bumpalo::collections::Vec as BVec;
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax_core::cst::Sequence;

use crate::error::ParseError;

pub use crate::cst::element::*;
pub use crate::cst::expression::*;
pub use crate::cst::identifier::*;
pub use crate::cst::inherit_doc::*;
pub use crate::cst::keyword::*;
pub use crate::cst::tag::*;
pub use crate::cst::text::*;
pub use crate::cst::trivia::*;
pub use crate::cst::variable::*;

pub mod element;
pub mod expression;
pub mod identifier;
pub mod inherit_doc;
pub mod keyword;
pub mod tag;
pub mod text;
pub mod trivia;
pub mod r#type;
pub mod variable;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Document<'arena> {
    pub span: Span,
    pub trivia: Sequence<'arena, Trivia<'arena>>,
    pub elements: Sequence<'arena, Element<'arena>>,
    pub inherit_docs: Sequence<'arena, InheritDoc>,
    pub errors: BVec<'arena, ParseError>,
}

impl Document<'_> {
    #[inline]
    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    #[inline]
    #[must_use]
    pub fn has_inherit_doc(&self) -> bool {
        !self.inherit_docs.is_empty()
    }
}

impl HasSpan for Document<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
