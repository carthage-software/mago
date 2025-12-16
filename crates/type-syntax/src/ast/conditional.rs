use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::Type;
use crate::ast::keyword::Keyword;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ConditionalType<'input> {
    pub subject: Box<Type<'input>>,
    pub is: Keyword<'input>,
    pub not: Option<Keyword<'input>>,
    pub target: Box<Type<'input>>,
    pub question_mark: Span,
    pub then: Box<Type<'input>>,
    pub colon: Span,
    pub otherwise: Box<Type<'input>>,
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
