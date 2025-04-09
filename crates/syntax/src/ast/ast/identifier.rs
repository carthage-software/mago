use serde::Deserialize;
use serde::Serialize;
use strum::Display;

use mago_interner::StringIdentifier;
use mago_span::HasSpan;
use mago_span::Span;

/// Represents an identifier.
///
/// An identifier can be a local, qualified, or fully qualified identifier.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum Identifier {
    Local(LocalIdentifier),
    Qualified(QualifiedIdentifier),
    FullyQualified(FullyQualifiedIdentifier),
}

/// Represents a local, unqualified identifier.
///
/// Example: `foo`, `Bar`, `BAZ`
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct LocalIdentifier {
    pub span: Span,
    pub value: StringIdentifier,
}

/// Represents a qualified identifier.
///
/// Example: `Foo\bar`, `Bar\Baz`, `Baz\QUX`
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct QualifiedIdentifier {
    pub span: Span,
    pub value: StringIdentifier,
}

/// Represents a fully qualified identifier.
///
/// Example: `\Foo\bar`, `\Bar\Baz`, `\Baz\QUX`
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct FullyQualifiedIdentifier {
    pub span: Span,
    pub value: StringIdentifier,
}

impl Identifier {
    pub fn value(&self) -> &StringIdentifier {
        match &self {
            Identifier::Local(local_identifier) => &local_identifier.value,
            Identifier::Qualified(qualified_identifier) => &qualified_identifier.value,
            Identifier::FullyQualified(fully_qualified_identifier) => &fully_qualified_identifier.value,
        }
    }
}

impl HasSpan for Identifier {
    fn span(&self) -> Span {
        match self {
            Identifier::Local(local) => local.span(),
            Identifier::Qualified(qualified) => qualified.span(),
            Identifier::FullyQualified(fully_qualified) => fully_qualified.span(),
        }
    }
}

impl HasSpan for LocalIdentifier {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for QualifiedIdentifier {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for FullyQualifiedIdentifier {
    fn span(&self) -> Span {
        self.span
    }
}
