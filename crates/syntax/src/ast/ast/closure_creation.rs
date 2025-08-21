use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::class_like::member::ClassLikeMemberSelector;
use crate::ast::ast::expression::Expression;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum ClosureCreation<'arena> {
    Function(FunctionClosureCreation<'arena>),
    Method(MethodClosureCreation<'arena>),
    StaticMethod(StaticMethodClosureCreation<'arena>),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct FunctionClosureCreation<'arena> {
    pub function: &'arena Expression<'arena>,
    pub left_parenthesis: Span,
    pub ellipsis: Span,
    pub right_parenthesis: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct MethodClosureCreation<'arena> {
    pub object: &'arena Expression<'arena>,
    pub arrow: Span,
    pub method: ClassLikeMemberSelector<'arena>,
    pub left_parenthesis: Span,
    pub ellipsis: Span,
    pub right_parenthesis: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct StaticMethodClosureCreation<'arena> {
    pub class: &'arena Expression<'arena>,
    pub double_colon: Span,
    pub method: ClassLikeMemberSelector<'arena>,
    pub left_parenthesis: Span,
    pub ellipsis: Span,
    pub right_parenthesis: Span,
}

impl HasSpan for ClosureCreation<'_> {
    fn span(&self) -> Span {
        match self {
            ClosureCreation::Function(f) => f.span(),
            ClosureCreation::Method(m) => m.span(),
            ClosureCreation::StaticMethod(s) => s.span(),
        }
    }
}

impl HasSpan for FunctionClosureCreation<'_> {
    fn span(&self) -> Span {
        self.function.span().join(self.right_parenthesis)
    }
}

impl HasSpan for MethodClosureCreation<'_> {
    fn span(&self) -> Span {
        self.object.span().join(self.right_parenthesis)
    }
}

impl HasSpan for StaticMethodClosureCreation<'_> {
    fn span(&self) -> Span {
        self.class.span().join(self.right_parenthesis)
    }
}
