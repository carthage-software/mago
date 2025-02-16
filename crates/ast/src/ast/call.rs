use bumpalo::boxed::Box;
use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::argument::ArgumentList;
use crate::ast::class_like::member::ClassLikeMemberSelector;
use crate::ast::expression::Expression;

#[derive(Debug, Hash, Serialize, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum Call<'a> {
    Function(FunctionCall<'a>),
    Method(MethodCall<'a>),
    NullSafeMethod(NullSafeMethodCall<'a>),
    StaticMethod(StaticMethodCall<'a>),
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct FunctionCall<'a> {
    pub function: Box<'a, Expression<'a>>,
    pub argument_list: ArgumentList<'a>,
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct MethodCall<'a> {
    pub object: Box<'a, Expression<'a>>,
    pub arrow: Span,
    pub method: ClassLikeMemberSelector<'a>,
    pub argument_list: ArgumentList<'a>,
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct NullSafeMethodCall<'a> {
    pub object: Box<'a, Expression<'a>>,
    pub question_mark_arrow: Span,
    pub method: ClassLikeMemberSelector<'a>,
    pub argument_list: ArgumentList<'a>,
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct StaticMethodCall<'a> {
    pub class: Box<'a, Expression<'a>>,
    pub double_colon: Span,
    pub method: ClassLikeMemberSelector<'a>,
    pub argument_list: ArgumentList<'a>,
}

impl HasSpan for Call<'_> {
    fn span(&self) -> Span {
        match self {
            Call::Function(f) => f.span(),
            Call::Method(m) => m.span(),
            Call::NullSafeMethod(n) => n.span(),
            Call::StaticMethod(s) => s.span(),
        }
    }
}

impl HasSpan for FunctionCall<'_> {
    fn span(&self) -> Span {
        self.function.span().join(self.argument_list.span())
    }
}

impl HasSpan for MethodCall<'_> {
    fn span(&self) -> Span {
        self.object.span().join(self.argument_list.span())
    }
}

impl HasSpan for NullSafeMethodCall<'_> {
    fn span(&self) -> Span {
        self.object.span().join(self.argument_list.span())
    }
}

impl HasSpan for StaticMethodCall<'_> {
    fn span(&self) -> Span {
        self.class.span().join(self.argument_list.span())
    }
}
