use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::identifier::Identifier;
use crate::ast::ast::identifier::LocalIdentifier;
use crate::ast::ast::keyword::Keyword;
use crate::ast::ast::modifier::Modifier;
use crate::ast::ast::terminator::Terminator;
use crate::ast::sequence::Sequence;
use crate::ast::sequence::TokenSeparatedSequence;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct TraitUse<'arena> {
    pub r#use: Keyword<'arena>,
    pub trait_names: TokenSeparatedSequence<'arena, Identifier<'arena>>,
    pub specification: TraitUseSpecification<'arena>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum TraitUseSpecification<'arena> {
    Abstract(TraitUseAbstractSpecification<'arena>),
    Concrete(TraitUseConcreteSpecification<'arena>),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct TraitUseAbstractSpecification<'arena>(pub Terminator<'arena>);

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct TraitUseConcreteSpecification<'arena> {
    pub left_brace: Span,
    pub adaptations: Sequence<'arena, TraitUseAdaptation<'arena>>,
    pub right_brace: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum TraitUseAdaptation<'arena> {
    Precedence(TraitUsePrecedenceAdaptation<'arena>),
    Alias(TraitUseAliasAdaptation<'arena>),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct TraitUsePrecedenceAdaptation<'arena> {
    pub method_reference: TraitUseAbsoluteMethodReference<'arena>,
    pub insteadof: Keyword<'arena>,
    pub trait_names: TokenSeparatedSequence<'arena, Identifier<'arena>>,
    pub terminator: Terminator<'arena>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct TraitUseAliasAdaptation<'arena> {
    pub method_reference: TraitUseMethodReference<'arena>,
    pub r#as: Keyword<'arena>,
    pub visibility: Option<Modifier<'arena>>,
    pub alias: Option<LocalIdentifier<'arena>>,
    pub terminator: Terminator<'arena>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum TraitUseMethodReference<'arena> {
    Identifier(LocalIdentifier<'arena>),
    Absolute(TraitUseAbsoluteMethodReference<'arena>),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct TraitUseAbsoluteMethodReference<'arena> {
    pub trait_name: Identifier<'arena>,
    pub double_colon: Span,
    pub method_name: LocalIdentifier<'arena>,
}

impl HasSpan for TraitUse<'_> {
    fn span(&self) -> Span {
        Span::between(self.r#use.span(), self.specification.span())
    }
}

impl HasSpan for TraitUseSpecification<'_> {
    fn span(&self) -> Span {
        match self {
            TraitUseSpecification::Abstract(specification) => specification.span(),
            TraitUseSpecification::Concrete(specification) => specification.span(),
        }
    }
}

impl HasSpan for TraitUseAbstractSpecification<'_> {
    fn span(&self) -> Span {
        self.0.span()
    }
}

impl HasSpan for TraitUseConcreteSpecification<'_> {
    fn span(&self) -> Span {
        Span::between(self.left_brace, self.right_brace)
    }
}

impl HasSpan for TraitUseAdaptation<'_> {
    fn span(&self) -> Span {
        match self {
            TraitUseAdaptation::Precedence(adaptation) => adaptation.span(),
            TraitUseAdaptation::Alias(adaptation) => adaptation.span(),
        }
    }
}

impl HasSpan for TraitUsePrecedenceAdaptation<'_> {
    fn span(&self) -> Span {
        Span::between(self.method_reference.span(), self.terminator.span())
    }
}

impl HasSpan for TraitUseAliasAdaptation<'_> {
    fn span(&self) -> Span {
        self.method_reference.span().join(self.terminator.span())
    }
}

impl HasSpan for TraitUseMethodReference<'_> {
    fn span(&self) -> Span {
        match self {
            TraitUseMethodReference::Identifier(identifier) => identifier.span(),
            TraitUseMethodReference::Absolute(absolute) => absolute.span(),
        }
    }
}

impl HasSpan for TraitUseAbsoluteMethodReference<'_> {
    fn span(&self) -> Span {
        Span::between(self.trait_name.span(), self.method_name.span())
    }
}
