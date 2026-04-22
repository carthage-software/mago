use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::Type;
use crate::ast::keyword::Keyword;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ConditionalType<'arena> {
    pub subject: &'arena Type<'arena>,
    pub is: Keyword<'arena>,
    pub not: Option<Keyword<'arena>>,
    pub target: &'arena Type<'arena>,
    pub question_mark: Span,
    pub then: &'arena Type<'arena>,
    pub colon: Span,
    pub otherwise: &'arena Type<'arena>,
}

impl ConditionalType<'_> {
    #[must_use]
    pub fn is_negated(&self) -> bool {
        self.not.is_some()
    }
}

impl HasSpan for ConditionalType<'_> {
    fn span(&self) -> Span {
        self.subject.span().join(self.otherwise.span())
    }
}

impl std::fmt::Display for ConditionalType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}{} {} ? {} : {}",
            self.subject,
            self.is,
            self.not.as_ref().map(|k| format!(" {k}")).unwrap_or_default(),
            self.target,
            self.then,
            self.otherwise
        )
    }
}
