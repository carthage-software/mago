#[cfg(feature = "serde")]
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::argument::Argument;
use crate::ir::delimited::Delimited;
use crate::ir::item::Item;
use crate::ir::item::annotation::ItemAnnotation;
use crate::ir::item::attribute::Attribute;
use crate::ir::item::inheritance::Extends;
use crate::ir::item::inheritance::Implements;
use crate::ir::item::member::MemberItem;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct AnonymousClass<'arena, I, S, E> {
    pub span: Span,
    pub annotation: Option<&'arena ItemAnnotation<'arena, I, S, E>>,
    pub attributes: &'arena [Attribute<'arena, I, S, E>],
    pub arguments: Option<Delimited<'arena, Argument<'arena, I, S, E>>>,
    pub extends: Option<&'arena Extends<'arena>>,
    pub implements: Option<&'arena Implements<'arena>>,
    pub members: Delimited<'arena, MemberItem<'arena, I, S, E>>,
}

impl<I, S, E> HasSpan for AnonymousClass<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<'arena, I, S, E> Item<'arena, I, S, E> for AnonymousClass<'arena, I, S, E> {
    fn attributes(&self) -> &'arena [Attribute<'arena, I, S, E>] {
        self.attributes
    }

    fn annotation(&self) -> Option<&'arena ItemAnnotation<'arena, I, S, E>> {
        self.annotation
    }
}
