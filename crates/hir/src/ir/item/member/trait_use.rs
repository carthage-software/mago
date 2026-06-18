#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::delimited::Delimited;
use crate::ir::identifier::Identifier;
use crate::ir::item::annotation::ItemAnnotation;
use crate::ir::item::modifier::Modifier;
use crate::ir::name::Name;
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_ref_into;
use mago_allocator::copy::copy_slice_into;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct TraitUse<'arena, I, S, E> {
    pub span: Span,
    pub annotation: Option<&'arena ItemAnnotation<'arena, I, S, E>>,
    pub traits: &'arena [Identifier<'arena>],
    pub adaptations: Option<Delimited<'arena, TraitUseAdaptation<'arena>>>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum TraitUseAdaptation<'arena> {
    Precedence(TraitUsePrecedenceAdaptation<'arena>),
    Alias(TraitUseAliasAdaptation<'arena>),
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct TraitUsePrecedenceAdaptation<'arena> {
    pub span: Span,
    pub r#trait: Identifier<'arena>,
    pub method: Name<'arena>,
    pub instead_of: &'arena [Identifier<'arena>],
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct TraitUseAliasAdaptation<'arena> {
    pub span: Span,
    pub r#trait: Option<Identifier<'arena>>,
    pub method: Name<'arena>,
    pub modifier: Option<Modifier>,
    pub alias: Name<'arena>,
}

impl<I, S, E> CopyInto for TraitUse<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = TraitUse<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        TraitUse {
            span: self.span,
            annotation: self.annotation.map(|node| copy_ref_into(node, arena)),
            traits: copy_slice_into(self.traits, arena),
            adaptations: self.adaptations.map(|node| node.copy_into(arena)),
        }
    }
}

impl CopyInto for TraitUseAdaptation<'_> {
    type Output<'arena> = TraitUseAdaptation<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        match *self {
            TraitUseAdaptation::Precedence(adaptation) => TraitUseAdaptation::Precedence(adaptation.copy_into(arena)),
            TraitUseAdaptation::Alias(adaptation) => TraitUseAdaptation::Alias(adaptation.copy_into(arena)),
        }
    }
}

impl CopyInto for TraitUsePrecedenceAdaptation<'_> {
    type Output<'arena> = TraitUsePrecedenceAdaptation<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        TraitUsePrecedenceAdaptation {
            span: self.span,
            r#trait: self.r#trait.copy_into(arena),
            method: self.method.copy_into(arena),
            instead_of: copy_slice_into(self.instead_of, arena),
        }
    }
}

impl CopyInto for TraitUseAliasAdaptation<'_> {
    type Output<'arena> = TraitUseAliasAdaptation<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        TraitUseAliasAdaptation {
            span: self.span,
            r#trait: self.r#trait.map(|r#trait| r#trait.copy_into(arena)),
            method: self.method.copy_into(arena),
            modifier: self.modifier,
            alias: self.alias.copy_into(arena),
        }
    }
}

impl<I, S, E> HasSpan for TraitUse<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for TraitUsePrecedenceAdaptation<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for TraitUseAliasAdaptation<'_> {
    fn span(&self) -> Span {
        self.span
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
