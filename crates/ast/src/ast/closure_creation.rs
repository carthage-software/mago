use bumpalo::boxed::Box;
use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::class_like::member::ClassLikeMemberSelector;
use crate::ast::expression::Expression;

#[derive(Debug, Hash, Serialize, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum ClosureCreation<'a> {
    Function(FunctionClosureCreation<'a>),
    Method(MethodClosureCreation<'a>),
    StaticMethod(StaticMethodClosureCreation<'a>),
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct FunctionClosureCreation<'a> {
    pub function: Box<'a, Expression<'a>>,
    pub left_parenthesis: Span,
    pub ellipsis: Span,
    pub right_parenthesis: Span,
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct MethodClosureCreation<'a> {
    pub object: Box<'a, Expression<'a>>,
    pub arrow: Span,
    pub method: ClassLikeMemberSelector<'a>,
    pub left_parenthesis: Span,
    pub ellipsis: Span,
    pub right_parenthesis: Span,
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct StaticMethodClosureCreation<'a> {
    pub class: Box<'a, Expression<'a>>,
    pub double_colon: Span,
    pub method: ClassLikeMemberSelector<'a>,
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
