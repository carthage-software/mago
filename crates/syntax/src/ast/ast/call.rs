use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::argument::ArgumentList;
use crate::ast::ast::class_like::member::ClassLikeMemberSelector;
use crate::ast::ast::expression::Expression;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum Call<'arena> {
    Function(FunctionCall<'arena>),
    Method(MethodCall<'arena>),
    NullSafeMethod(NullSafeMethodCall<'arena>),
    StaticMethod(StaticMethodCall<'arena>),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct FunctionCall<'arena> {
    pub function: &'arena Expression<'arena>,
    pub argument_list: ArgumentList<'arena>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct MethodCall<'arena> {
    pub object: &'arena Expression<'arena>,
    pub arrow: Span,
    pub method: ClassLikeMemberSelector<'arena>,
    pub argument_list: ArgumentList<'arena>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct NullSafeMethodCall<'arena> {
    pub object: &'arena Expression<'arena>,
    pub question_mark_arrow: Span,
    pub method: ClassLikeMemberSelector<'arena>,
    pub argument_list: ArgumentList<'arena>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct StaticMethodCall<'arena> {
    pub class: &'arena Expression<'arena>,
    pub double_colon: Span,
    pub method: ClassLikeMemberSelector<'arena>,
    pub argument_list: ArgumentList<'arena>,
}

impl<'arena> Call<'arena> {
    #[inline]
    pub const fn is_null_safe(&self) -> bool {
        matches!(self, Call::NullSafeMethod(_))
    }

    #[inline]
    pub fn get_argument_list(&self) -> &ArgumentList<'arena> {
        match self {
            Call::Function(f) => &f.argument_list,
            Call::Method(m) => &m.argument_list,
            Call::NullSafeMethod(n) => &n.argument_list,
            Call::StaticMethod(s) => &s.argument_list,
        }
    }
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
