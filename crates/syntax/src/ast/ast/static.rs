use serde::Deserialize;
use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::expression::Expression;
use crate::ast::ast::keyword::Keyword;
use crate::ast::ast::terminator::Terminator;
use crate::ast::ast::variable::DirectVariable;
use crate::ast::sequence::TokenSeparatedSequence;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct Static {
    pub r#static: Keyword,
    pub items: TokenSeparatedSequence<StaticItem>,
    pub terminator: Terminator,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum StaticItem {
    Abstract(StaticAbstractItem),
    Concrete(StaticConcreteItem),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct StaticAbstractItem {
    pub variable: DirectVariable,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct StaticConcreteItem {
    pub variable: DirectVariable,
    pub equals: Span,
    pub value: Expression,
}

impl StaticItem {
    pub fn variable(&self) -> &DirectVariable {
        match self {
            StaticItem::Abstract(item) => &item.variable,
            StaticItem::Concrete(item) => &item.variable,
        }
    }
}

impl HasSpan for Static {
    fn span(&self) -> Span {
        self.r#static.span().join(self.terminator.span())
    }
}

impl HasSpan for StaticItem {
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

impl HasSpan for StaticConcreteItem {
    fn span(&self) -> Span {
        self.variable.span().join(self.value.span())
    }
}
