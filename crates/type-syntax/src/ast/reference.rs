use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::generics::GenericParameters;
use crate::ast::identifier::Identifier;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ReferenceType<'arena> {
    pub identifier: Identifier<'arena>,
    pub parameters: Option<GenericParameters<'arena>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub enum MemberReferenceSelector<'arena> {
    Wildcard(Span),
    Identifier(Identifier<'arena>),
    StartsWith(Identifier<'arena>, Span),
    EndsWith(Span, Identifier<'arena>),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct MemberReferenceType<'arena> {
    pub class: Identifier<'arena>,
    pub double_colon: Span,
    pub member: MemberReferenceSelector<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct GlobalWildcardType<'arena> {
    pub selector: GlobalWildcardSelector<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub enum GlobalWildcardSelector<'arena> {
    StartsWith(Identifier<'arena>, Span),
    EndsWith(Span, Identifier<'arena>),
}

impl HasSpan for ReferenceType<'_> {
    fn span(&self) -> Span {
        match &self.parameters {
            Some(parameters) => self.identifier.span.join(parameters.span()),
            None => self.identifier.span,
        }
    }
}

impl HasSpan for MemberReferenceSelector<'_> {
    fn span(&self) -> Span {
        match self {
            MemberReferenceSelector::Wildcard(span) => *span,
            MemberReferenceSelector::Identifier(identifier) => identifier.span,
            MemberReferenceSelector::StartsWith(identifier, span) => identifier.span.join(*span),
            MemberReferenceSelector::EndsWith(span, identifier) => span.join(identifier.span),
        }
    }
}

impl HasSpan for MemberReferenceType<'_> {
    fn span(&self) -> Span {
        self.class.span.join(self.member.span())
    }
}

impl HasSpan for GlobalWildcardSelector<'_> {
    fn span(&self) -> Span {
        match self {
            GlobalWildcardSelector::StartsWith(identifier, span) => identifier.span.join(*span),
            GlobalWildcardSelector::EndsWith(span, identifier) => span.join(identifier.span),
        }
    }
}

impl HasSpan for GlobalWildcardType<'_> {
    fn span(&self) -> Span {
        self.selector.span()
    }
}

impl std::fmt::Display for ReferenceType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(parameters) = &self.parameters {
            write!(f, "{}{}", self.identifier, parameters)
        } else {
            write!(f, "{}", self.identifier)
        }
    }
}

impl std::fmt::Display for MemberReferenceSelector<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemberReferenceSelector::Wildcard(_) => write!(f, "*"),
            MemberReferenceSelector::Identifier(identifier) => write!(f, "{identifier}"),
            MemberReferenceSelector::StartsWith(identifier, _) => write!(f, "{identifier}*"),
            MemberReferenceSelector::EndsWith(_, identifier) => write!(f, "*{identifier}"),
        }
    }
}

impl std::fmt::Display for MemberReferenceType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{}", self.class, self.member)
    }
}

impl std::fmt::Display for GlobalWildcardSelector<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GlobalWildcardSelector::StartsWith(identifier, _) => write!(f, "{identifier}*"),
            GlobalWildcardSelector::EndsWith(_, identifier) => write!(f, "*{identifier}"),
        }
    }
}

impl std::fmt::Display for GlobalWildcardType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.selector)
    }
}
