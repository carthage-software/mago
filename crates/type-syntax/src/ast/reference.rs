use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::generics::GenericParameters;
use crate::ast::identifier::Identifier;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[repr(C)]
pub struct ReferenceType<'input> {
    pub identifier: Identifier<'input>,
    pub parameters: Option<GenericParameters<'input>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[repr(C)]
pub enum MemberReferenceSelector<'input> {
    Wildcard(Span),
    Identifier(Identifier<'input>),
    StartsWith(Identifier<'input>, Span),
    EndsWith(Span, Identifier<'input>),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[repr(C)]
pub struct MemberReferenceType<'input> {
    pub class: Identifier<'input>,
    pub double_colon: Span,
    pub member: MemberReferenceSelector<'input>,
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
