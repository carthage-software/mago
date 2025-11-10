use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::identifier::Identifier;
use crate::ast::keyword::Keyword;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub enum AliasName<'input> {
    Identifier(Identifier<'input>),
    Keyword(Keyword<'input>),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct AliasReferenceType<'input> {
    pub exclamation: Span,
    pub class: Identifier<'input>,
    pub double_colon: Span,
    pub alias: AliasName<'input>,
}

impl HasSpan for AliasName<'_> {
    fn span(&self) -> Span {
        match self {
            AliasName::Identifier(identifier) => identifier.span,
            AliasName::Keyword(keyword) => keyword.span,
        }
    }
}

impl HasSpan for AliasReferenceType<'_> {
    fn span(&self) -> Span {
        self.exclamation.join(self.alias.span())
    }
}

impl std::fmt::Display for AliasName<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AliasName::Identifier(identifier) => write!(f, "{identifier}"),
            AliasName::Keyword(keyword) => write!(f, "{keyword}"),
        }
    }
}

impl std::fmt::Display for AliasReferenceType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "!{}::{}", self.class, self.alias)
    }
}
