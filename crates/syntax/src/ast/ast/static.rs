use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::expression::Expression;
use crate::ast::ast::keyword::Keyword;
use crate::ast::ast::terminator::Terminator;
use crate::ast::ast::variable::DirectVariable;
use crate::ast::sequence::TokenSeparatedSequence;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Static<'arena> {
    pub r#static: Keyword<'arena>,
    pub items: TokenSeparatedSequence<'arena, StaticItem<'arena>>,
    pub terminator: Terminator<'arena>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum StaticItem<'arena> {
    Abstract(StaticAbstractItem<'arena>),
    Concrete(StaticConcreteItem<'arena>),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct StaticAbstractItem<'arena> {
    pub variable: DirectVariable<'arena>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct StaticConcreteItem<'arena> {
    pub variable: DirectVariable<'arena>,
    pub equals: Span,
    pub value: Expression<'arena>,
}

impl<'arena> StaticItem<'arena> {
    pub fn variable(&self) -> &DirectVariable<'arena> {
        match self {
            StaticItem::Abstract(item) => &item.variable,
            StaticItem::Concrete(item) => &item.variable,
        }
    }

    pub fn value(&self) -> Option<&Expression<'arena>> {
        match self {
            StaticItem::Abstract(_) => None,
            StaticItem::Concrete(item) => Some(&item.value),
        }
    }
}

impl HasSpan for Static<'_> {
    fn span(&self) -> Span {
        self.r#static.span().join(self.terminator.span())
    }
}

impl HasSpan for StaticItem<'_> {
    fn span(&self) -> Span {
        match self {
            StaticItem::Abstract(item) => item.span(),
            StaticItem::Concrete(item) => item.span(),
        }
    }
}

impl HasSpan for StaticAbstractItem<'_> {
    fn span(&self) -> Span {
        self.variable.span()
    }
}

impl HasSpan for StaticConcreteItem<'_> {
    fn span(&self) -> Span {
        self.variable.span().join(self.value.span())
    }
}
