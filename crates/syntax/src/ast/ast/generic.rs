use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::Identifier;
use crate::ast::ast::identifier::LocalIdentifier;
use crate::ast::ast::type_hint::Hint;
use crate::ast::sequence::TokenSeparatedSequence;

/// Variance marker on a generic type parameter.
///
/// `+T` is covariant, `-T` is contravariant. An absent marker means invariant.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
pub enum GenericVariance {
    Covariant(Span),
    Contravariant(Span),
}

impl HasSpan for GenericVariance {
    fn span(&self) -> Span {
        match self {
            GenericVariance::Covariant(span) | GenericVariance::Contravariant(span) => *span,
        }
    }
}

/// The upper-bound clause of a generic parameter: `: <type>`.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct GenericParameterBound<'arena> {
    pub colon: Span,
    pub hint: Hint<'arena>,
}

impl HasSpan for GenericParameterBound<'_> {
    fn span(&self) -> Span {
        self.colon.join(self.hint.span())
    }
}

/// The default-argument clause of a generic parameter: `= <type>`.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct GenericParameterDefault<'arena> {
    pub equals: Span,
    pub hint: Hint<'arena>,
}

impl HasSpan for GenericParameterDefault<'_> {
    fn span(&self) -> Span {
        self.equals.join(self.hint.span())
    }
}

/// A single generic type parameter on a declaration.
///
/// Example: `+T : Stringable = string`
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct GenericParameter<'arena> {
    pub variance: Option<GenericVariance>,
    pub name: LocalIdentifier<'arena>,
    pub bound: Option<GenericParameterBound<'arena>>,
    pub default: Option<GenericParameterDefault<'arena>>,
}

impl HasSpan for GenericParameter<'_> {
    fn span(&self) -> Span {
        let start = self.variance.as_ref().map(HasSpan::span).unwrap_or_else(|| self.name.span());
        let end = self
            .default
            .as_ref()
            .map(HasSpan::span)
            .or_else(|| self.bound.as_ref().map(HasSpan::span))
            .unwrap_or_else(|| self.name.span());

        start.join(end)
    }
}

/// The comma-separated generic parameter list on a declaration.
///
/// Example: `<K, V : array-key = string>`
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct GenericParameterList<'arena> {
    pub less_than: Span,
    pub parameters: TokenSeparatedSequence<'arena, GenericParameter<'arena>>,
    pub greater_than: Span,
}

impl HasSpan for GenericParameterList<'_> {
    fn span(&self) -> Span {
        self.less_than.join(self.greater_than)
    }
}

/// The comma-separated generic argument list at a type-use site.
///
/// Example: `<int, string>` in `Map<int, string>`.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct GenericArgumentList<'arena> {
    pub less_than: Span,
    pub arguments: TokenSeparatedSequence<'arena, Hint<'arena>>,
    pub greater_than: Span,
}

impl HasSpan for GenericArgumentList<'_> {
    fn span(&self) -> Span {
        self.less_than.join(self.greater_than)
    }
}

/// A named type with type arguments: `Box<T>`, `self<T>`, `static<T>`, `parent<T>`.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct GenericHint<'arena> {
    pub base: &'arena Hint<'arena>,
    pub arguments: GenericArgumentList<'arena>,
}

impl HasSpan for GenericHint<'_> {
    fn span(&self) -> Span {
        self.base.span().join(self.arguments.span())
    }
}

/// A reference to a named class-like (class, interface, trait), optionally
/// carrying a generic argument list.
///
/// Used in `extends`, `implements`, and `use Trait;` clauses.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ClassLikeReference<'arena> {
    pub name: Identifier<'arena>,
    pub generic_arguments: Option<GenericArgumentList<'arena>>,
}

impl HasSpan for ClassLikeReference<'_> {
    fn span(&self) -> Span {
        match &self.generic_arguments {
            Some(args) => self.name.span().join(args.span()),
            None => self.name.span(),
        }
    }
}

/// Turbofish: a call-site type-argument list, introduced by the `::<` token.
///
/// Example: `Box::<int>` in `new Box::<int>($v)`.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Turbofish<'arena> {
    pub turbofish: Span,
    pub arguments: TokenSeparatedSequence<'arena, Hint<'arena>>,
    pub greater_than: Span,
}

impl HasSpan for Turbofish<'_> {
    fn span(&self) -> Span {
        self.turbofish.join(self.greater_than)
    }
}
