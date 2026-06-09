use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

/// Represents an identifier.
///
/// An identifier can be a local, qualified, or fully qualified identifier.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, Display)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "value"))]
pub enum Identifier<'arena> {
    Local(LocalIdentifier<'arena>),
    Qualified(QualifiedIdentifier<'arena>),
    FullyQualified(FullyQualifiedIdentifier<'arena>),
}

/// Represents a local, unqualified identifier.
///
/// Example: `foo`, `Bar`, `BAZ`
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct LocalIdentifier<'arena> {
    pub span: Span,
    pub value: &'arena [u8],
}

/// Represents a qualified identifier.
///
/// Example: `Foo\bar`, `Bar\Baz`, `Baz\QUX`
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct QualifiedIdentifier<'arena> {
    pub span: Span,
    pub value: &'arena [u8],
}

/// Represents a fully qualified identifier.
///
/// Example: `\Foo\bar`, `\Bar\Baz`, `\Baz\QUX`
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FullyQualifiedIdentifier<'arena> {
    pub span: Span,
    pub value: &'arena [u8],
}

impl<'arena> Identifier<'arena> {
    #[inline]
    #[must_use]
    pub const fn is_local(&self) -> bool {
        matches!(self, Identifier::Local(_))
    }

    #[inline]
    #[must_use]
    pub const fn is_qualified(&self) -> bool {
        matches!(self, Identifier::Qualified(_))
    }

    #[inline]
    #[must_use]
    pub const fn is_fully_qualified(&self) -> bool {
        matches!(self, Identifier::FullyQualified(_))
    }

    #[inline]
    #[must_use]
    pub const fn value(&self) -> &'arena [u8] {
        match &self {
            Identifier::Local(local_identifier) => local_identifier.value,
            Identifier::Qualified(qualified_identifier) => qualified_identifier.value,
            Identifier::FullyQualified(fully_qualified_identifier) => fully_qualified_identifier.value,
        }
    }

    #[inline]
    #[must_use]
    pub fn last_segment(&self) -> &'arena [u8] {
        let value = self.value();

        match memchr::memrchr(b'\\', value) {
            Some(pos) => &value[pos + 1..],
            None => value,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn identifier(value: &'static [u8]) -> Identifier<'static> {
        Identifier::FullyQualified(FullyQualifiedIdentifier { span: Span::dummy(0, value.len() as u32), value })
    }

    #[test]
    fn last_segment_returns_the_final_namespace_segment() {
        assert_eq!(identifier(b"\\Illuminate\\Database\\Eloquent\\Attributes\\Scope").last_segment(), b"Scope");
        assert_eq!(identifier(b"Foo\\Bar").last_segment(), b"Bar");
        assert_eq!(identifier(b"\\Foo").last_segment(), b"Foo");
    }

    #[test]
    fn last_segment_returns_the_whole_value_when_unqualified() {
        assert_eq!(identifier(b"Scope").last_segment(), b"Scope");
    }
}
