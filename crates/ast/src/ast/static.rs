use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::expression::Expression;
use crate::ast::keyword::Keyword;
use crate::ast::terminator::Terminator;
use crate::ast::variable::DirectVariable;
use crate::sequence::TokenSeparatedSequence;

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct Static<'a> {
    pub r#static: Keyword,
    pub items: TokenSeparatedSequence<'a, StaticItem<'a>>,
    pub terminator: Terminator,
}

#[derive(Debug, Hash, Serialize, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum StaticItem<'a> {
    Abstract(StaticAbstractItem),
    Concrete(StaticConcreteItem<'a>),
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct StaticAbstractItem {
    pub variable: DirectVariable,
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct StaticConcreteItem<'a> {
    pub variable: DirectVariable,
    pub equals: Span,
    pub value: Expression<'a>,
}

impl StaticItem<'_> {
    pub fn variable(&self) -> &DirectVariable {
        match self {
            StaticItem::Abstract(item) => &item.variable,
            StaticItem::Concrete(item) => &item.variable,
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

impl HasSpan for StaticAbstractItem {
    fn span(&self) -> Span {
        self.variable.span()
    }
}

impl HasSpan for StaticConcreteItem<'_> {
    fn span(&self) -> Span {
        self.variable.span().join(self.value.span())
    }
}
