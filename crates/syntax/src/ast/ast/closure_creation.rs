use serde::Deserialize;
use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::class_like::member::ClassLikeMemberSelector;
use crate::ast::ast::expression::Expression;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum ClosureCreation {
    Function(FunctionClosureCreation),
    Method(MethodClosureCreation),
    StaticMethod(StaticMethodClosureCreation),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct FunctionClosureCreation {
    pub function: Box<Expression>,
    pub left_parenthesis: Span,
    pub ellipsis: Span,
    pub right_parenthesis: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct MethodClosureCreation {
    pub object: Box<Expression>,
    pub arrow: Span,
    pub method: ClassLikeMemberSelector,
    pub left_parenthesis: Span,
    pub ellipsis: Span,
    pub right_parenthesis: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct StaticMethodClosureCreation {
    pub class: Box<Expression>,
    pub double_colon: Span,
    pub method: ClassLikeMemberSelector,
    pub left_parenthesis: Span,
    pub ellipsis: Span,
    pub right_parenthesis: Span,
}

impl HasSpan for ClosureCreation {
    fn span(&self) -> Span {
        match self {
            ClosureCreation::Function(f) => f.span(),
            ClosureCreation::Method(m) => m.span(),
            ClosureCreation::StaticMethod(s) => s.span(),
        }
    }
}

impl HasSpan for FunctionClosureCreation {
    fn span(&self) -> Span {
        self.function.span().join(self.right_parenthesis)
    }
}

impl HasSpan for MethodClosureCreation {
    fn span(&self) -> Span {
        self.object.span().join(self.right_parenthesis)
    }
}

impl HasSpan for StaticMethodClosureCreation {
    fn span(&self) -> Span {
        self.class.span().join(self.right_parenthesis)
    }
}
