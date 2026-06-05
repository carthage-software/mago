use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::cst::Keyword;
use crate::cst::identifier::Identifier;
use crate::cst::r#type::generics::GenericParameters;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[serde(tag = "type", content = "value")]
pub enum ReferenceKind<'arena> {
    Identifier(Identifier<'arena>),
    Self_(Keyword<'arena>),
    Static(Keyword<'arena>),
    Parent(Keyword<'arena>),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ReferenceType<'arena> {
    pub kind: ReferenceKind<'arena>,
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
    pub kind: ReferenceKind<'arena>,
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

impl HasSpan for ReferenceKind<'_> {
    fn span(&self) -> Span {
        match self {
            ReferenceKind::Identifier(identifier) => identifier.span,
            ReferenceKind::Self_(keyword) | ReferenceKind::Static(keyword) | ReferenceKind::Parent(keyword) => {
                keyword.span
            }
        }
    }
}

impl HasSpan for ReferenceType<'_> {
    fn span(&self) -> Span {
        match &self.parameters {
            Some(parameters) => self.kind.span().join(parameters.span()),
            None => self.kind.span(),
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
        self.kind.span().join(self.member.span())
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

impl std::fmt::Display for ReferenceKind<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReferenceKind::Identifier(identifier) => write!(f, "{identifier}"),
            ReferenceKind::Self_(keyword) | ReferenceKind::Static(keyword) | ReferenceKind::Parent(keyword) => {
                write!(f, "{keyword}")
            }
        }
    }
}

impl std::fmt::Display for ReferenceType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(parameters) = &self.parameters {
            write!(f, "{}{}", self.kind, parameters)
        } else {
            write!(f, "{}", self.kind)
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
        write!(f, "{}::{}", self.kind, self.member)
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
