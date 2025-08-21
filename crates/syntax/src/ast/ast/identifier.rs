use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

/// Represents an identifier.
///
/// An identifier can be a local, qualified, or fully qualified identifier.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum Identifier<'arena> {
    Local(LocalIdentifier<'arena>),
    Qualified(QualifiedIdentifier<'arena>),
    FullyQualified(FullyQualifiedIdentifier<'arena>),
}

/// Represents a local, unqualified identifier.
///
/// Example: `foo`, `Bar`, `BAZ`
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct LocalIdentifier<'arena> {
    pub span: Span,
    pub value: &'arena str,
}

/// Represents a qualified identifier.
///
/// Example: `Foo\bar`, `Bar\Baz`, `Baz\QUX`
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct QualifiedIdentifier<'arena> {
    pub span: Span,
    pub value: &'arena str,
}

/// Represents a fully qualified identifier.
///
/// Example: `\Foo\bar`, `\Bar\Baz`, `\Baz\QUX`
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct FullyQualifiedIdentifier<'arena> {
    pub span: Span,
    pub value: &'arena str,
}

impl<'arena> Identifier<'arena> {
    #[inline]
    pub const fn is_local(&self) -> bool {
        matches!(self, Identifier::Local(_))
    }

    #[inline]
    pub const fn is_qualified(&self) -> bool {
        matches!(self, Identifier::Qualified(_))
    }

    #[inline]
    pub const fn is_fully_qualified(&self) -> bool {
        matches!(self, Identifier::FullyQualified(_))
    }

    #[inline]
    pub const fn value(&self) -> &'arena str {
        match &self {
            Identifier::Local(local_identifier) => local_identifier.value,
            Identifier::Qualified(qualified_identifier) => qualified_identifier.value,
            Identifier::FullyQualified(fully_qualified_identifier) => fully_qualified_identifier.value,
        }
    }
}

impl HasSpan for Identifier<'_> {
    fn span(&self) -> Span {
        match self {
            Identifier::Local(local) => local.span(),
            Identifier::Qualified(qualified) => qualified.span(),
            Identifier::FullyQualified(fully_qualified) => fully_qualified.span(),
        }
    }
}

impl HasSpan for LocalIdentifier<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for QualifiedIdentifier<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for FullyQualifiedIdentifier<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
