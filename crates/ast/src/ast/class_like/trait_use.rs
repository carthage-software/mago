use serde::Deserialize;
use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::identifier::Identifier;
use crate::ast::identifier::LocalIdentifier;
use crate::ast::keyword::Keyword;
use crate::ast::modifier::Modifier;
use crate::ast::terminator::Terminator;
use crate::sequence::Sequence;
use crate::sequence::TokenSeparatedSequence;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct TraitUse {
    pub r#use: Keyword,
    pub trait_names: TokenSeparatedSequence<Identifier>,
    pub specification: TraitUseSpecification,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum TraitUseSpecification {
    Abstract(TraitUseAbstractSpecification),
    Concrete(TraitUseConcreteSpecification),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct TraitUseAbstractSpecification(pub Terminator);

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct TraitUseConcreteSpecification {
    pub left_brace: Span,
    pub adaptations: Sequence<TraitUseAdaptation>,
    pub right_brace: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum TraitUseAdaptation {
    Precedence(TraitUsePrecedenceAdaptation),
    Alias(TraitUseAliasAdaptation),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct TraitUsePrecedenceAdaptation {
    pub method_reference: TraitUseAbsoluteMethodReference,
    pub insteadof: Keyword,
    pub trait_names: TokenSeparatedSequence<Identifier>,
    pub terminator: Terminator,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct TraitUseAliasAdaptation {
    pub method_reference: TraitUseMethodReference,
    pub r#as: Keyword,
    pub visibility: Option<Modifier>,
    pub alias: Option<LocalIdentifier>,
    pub terminator: Terminator,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum TraitUseMethodReference {
    Identifier(LocalIdentifier),
    Absolute(TraitUseAbsoluteMethodReference),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct TraitUseAbsoluteMethodReference {
    pub trait_name: Identifier,
    pub double_colon: Span,
    pub method_name: LocalIdentifier,
}

impl HasSpan for TraitUse {
    fn span(&self) -> Span {
        Span::between(self.r#use.span(), self.specification.span())
    }
}

impl HasSpan for TraitUseSpecification {
    fn span(&self) -> Span {
        match self {
            TraitUseSpecification::Abstract(specification) => specification.span(),
            TraitUseSpecification::Concrete(specification) => specification.span(),
        }
    }
}

impl HasSpan for TraitUseAbstractSpecification {
    fn span(&self) -> Span {
        self.0.span()
    }
}

impl HasSpan for TraitUseConcreteSpecification {
    fn span(&self) -> Span {
        Span::between(self.left_brace, self.right_brace)
    }
}

impl HasSpan for TraitUseAdaptation {
    fn span(&self) -> Span {
        match self {
            TraitUseAdaptation::Precedence(adaptation) => adaptation.span(),
            TraitUseAdaptation::Alias(adaptation) => adaptation.span(),
        }
    }
}

impl HasSpan for TraitUsePrecedenceAdaptation {
    fn span(&self) -> Span {
        Span::between(self.method_reference.span(), self.terminator.span())
    }
}

impl HasSpan for TraitUseAliasAdaptation {
    fn span(&self) -> Span {
        self.method_reference.span().join(self.terminator.span())
    }
}

impl HasSpan for TraitUseMethodReference {
    fn span(&self) -> Span {
        match self {
            TraitUseMethodReference::Identifier(identifier) => identifier.span(),
            TraitUseMethodReference::Absolute(absolute) => absolute.span(),
        }
    }
}

impl HasSpan for TraitUseAbsoluteMethodReference {
    fn span(&self) -> Span {
        Span::between(self.trait_name.span(), self.method_name.span())
    }
}
