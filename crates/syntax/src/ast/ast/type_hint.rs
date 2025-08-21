use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::identifier::Identifier;
use crate::ast::ast::identifier::LocalIdentifier;
use crate::ast::ast::keyword::Keyword;

/// Represents a type statement.
///
/// A type statement specifies the type of a parameter, property, constant, or return value.
///
/// # Examples
///
/// ```php
/// int
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum Hint<'arena> {
    Identifier(Identifier<'arena>),
    Parenthesized(ParenthesizedHint<'arena>),
    Nullable(NullableHint<'arena>),
    Union(UnionHint<'arena>),
    Intersection(IntersectionHint<'arena>),
    Null(Keyword<'arena>),
    True(Keyword<'arena>),
    False(Keyword<'arena>),
    Array(Keyword<'arena>),
    Callable(Keyword<'arena>),
    Static(Keyword<'arena>),
    Self_(Keyword<'arena>),
    Parent(Keyword<'arena>),
    Void(LocalIdentifier<'arena>),
    Never(LocalIdentifier<'arena>),
    Float(LocalIdentifier<'arena>),
    Bool(LocalIdentifier<'arena>),
    Integer(LocalIdentifier<'arena>),
    String(LocalIdentifier<'arena>),
    Object(LocalIdentifier<'arena>),
    Mixed(LocalIdentifier<'arena>),
    Iterable(LocalIdentifier<'arena>),
}

/// Represents a parenthesized type hint.
///
/// # Examples
///
/// ```php
/// <?php
///
/// function(): string|(Foo&Bar) {
///    return 'hello';
/// }
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ParenthesizedHint<'arena> {
    pub left_parenthesis: Span,
    pub hint: &'arena Hint<'arena>,
    pub right_parenthesis: Span,
}

/// Represents a union type statement
///
/// A union type is a type that is a union of multiple type hints separated by a pipe (`|`) character.
///
/// # Examples
///
/// ```php
/// int|string
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct UnionHint<'arena> {
    pub left: &'arena Hint<'arena>,
    pub pipe: Span,
    pub right: &'arena Hint<'arena>,
}

/// Represents an intersection type.
///
/// An intersection type is a type that is an intersection of multiple type hints separated by an ampersand (`&`) character.
///
/// # Examples
///
/// ```php
/// ArrayAccess&Countable
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct IntersectionHint<'arena> {
    pub left: &'arena Hint<'arena>,
    pub ampersand: Span,
    pub right: &'arena Hint<'arena>,
}

/// Represents a nullable type.
///
/// A nullable type is a type that is preceded by a question mark (`?`) character.
///
/// # Examples
///
/// ```php
/// ?string
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct NullableHint<'arena> {
    pub question_mark: Span,
    pub hint: &'arena Hint<'arena>,
}

impl Hint<'_> {
    /// Returns `true` if the type hint is a standalone type hint.
    ///
    /// Standalone type hints are type hints that cannot be wrapped inside another type hint.
    #[inline]
    pub const fn is_standalone(&self) -> bool {
        matches!(self, Self::Mixed(_) | Self::Never(_) | Self::Void(_) | Self::Nullable(_))
    }

    #[inline]
    pub const fn is_complex(&self) -> bool {
        matches!(self, Self::Union(_) | Self::Intersection(_) | Self::Parenthesized(_) | Self::Nullable(_))
    }

    /// Returns `true` if the type hint is a nullable type hint.
    ///
    /// A nullable type hint is a type hint that is preceded by a question mark (`?`) character.
    #[inline]
    pub const fn is_nullable(&self) -> bool {
        matches!(self, Self::Nullable(_))
    }

    #[inline]
    pub fn contains_null(&self) -> bool {
        match self {
            Hint::Mixed(_) => true,
            Hint::Nullable(_) => true,
            Hint::Null(_) => true,
            Hint::Union(union) => union.left.contains_null() || union.right.contains_null(),
            _ => false,
        }
    }

    /// Returns `true` if the type is a bottom type.
    ///
    /// A bottom type is a type that has no instances.
    #[inline]
    pub const fn is_bottom(&self) -> bool {
        matches!(self, Self::Never(_) | Self::Void(_))
    }

    /// Returns `true` if the type can be intersected with another type.
    #[inline]
    pub const fn is_intersectable(&self) -> bool {
        matches!(self, Self::Identifier(_) | Self::Parenthesized(_) | Self::Intersection(_))
    }

    /// Returns `true` if the type can be unioned with another type.
    #[inline]
    pub const fn is_unionable(&self) -> bool {
        if let Hint::Intersection(_) = self {
            return false;
        }

        !self.is_standalone()
    }

    /// Returns `true` if the type can be wrapped in parentheses.
    #[inline]
    pub const fn is_parenthesizable(&self) -> bool {
        matches!(self, Self::Union(_) | Self::Intersection(_))
    }

    /// Returns `true` if the type is a scalar type.
    ///
    /// A scalar type is a type that represents a single value.
    #[inline]
    pub fn is_scalar(&self) -> bool {
        if let Hint::Union(union) = self {
            return union.left.is_scalar() && union.right.is_scalar();
        }

        matches!(self, Self::Bool(_) | Self::Float(_) | Self::Integer(_) | Self::String(_))
    }

    /// Returns `true` if the type is a union type.
    ///
    /// A union type is a type that is a union of multiple type hints separated by a pipe (`|`) character.
    ///
    /// If the type is wrapped in parentheses, this method will unwrap the parentheses and
    ///  check if the unwrapped type is a union type.
    #[inline]
    pub fn is_union(&self) -> bool {
        match self {
            Hint::Union(_) => true,
            Hint::Parenthesized(parenthesized) => parenthesized.hint.is_union(),
            _ => false,
        }
    }

    /// Returns `true` if the type is an intersection type.
    ///
    /// An intersection type is a type that is an intersection of multiple type hints separated by an ampersand (`&`)
    ///  character.
    ///
    /// If the type is wrapped in parentheses, this method will unwrap the parentheses and
    ///  check if the unwrapped type is an intersection type.
    #[inline]
    pub fn is_intersection(&self) -> bool {
        match self {
            Hint::Intersection(_) => true,
            Hint::Parenthesized(parenthesized) => parenthesized.hint.is_intersection(),
            _ => false,
        }
    }
}

impl HasSpan for Hint<'_> {
    fn span(&self) -> Span {
        match &self {
            Hint::Identifier(identifier) => identifier.span(),
            Hint::Parenthesized(parenthesized) => parenthesized.span(),
            Hint::Nullable(nullable) => nullable.span(),
            Hint::Union(union) => union.span(),
            Hint::Intersection(intersection) => intersection.span(),
            Hint::Null(keyword)
            | Hint::True(keyword)
            | Hint::Static(keyword)
            | Hint::Callable(keyword)
            | Hint::Self_(keyword)
            | Hint::Parent(keyword)
            | Hint::Array(keyword)
            | Hint::False(keyword) => keyword.span(),
            Hint::Void(identifier)
            | Hint::Never(identifier)
            | Hint::Float(identifier)
            | Hint::Bool(identifier)
            | Hint::Integer(identifier)
            | Hint::String(identifier)
            | Hint::Object(identifier)
            | Hint::Mixed(identifier)
            | Hint::Iterable(identifier) => identifier.span(),
        }
    }
}

impl HasSpan for ParenthesizedHint<'_> {
    fn span(&self) -> Span {
        self.left_parenthesis.join(self.right_parenthesis)
    }
}

impl HasSpan for UnionHint<'_> {
    fn span(&self) -> Span {
        self.left.span().join(self.right.span())
    }
}

impl HasSpan for IntersectionHint<'_> {
    fn span(&self) -> Span {
        self.left.span().join(self.right.span())
    }
}

impl HasSpan for NullableHint<'_> {
    fn span(&self) -> Span {
        Span::between(self.question_mark, self.hint.span())
    }
}
