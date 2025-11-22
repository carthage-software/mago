use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::argument::PartialArgumentList;
use crate::ast::ast::class_like::member::ClassLikeMemberSelector;
use crate::ast::ast::expression::Expression;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum PartialApplication<'arena> {
    Function(FunctionPartialApplication<'arena>),
    Method(MethodPartialApplication<'arena>),
    StaticMethod(StaticMethodPartialApplication<'arena>),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct FunctionPartialApplication<'arena> {
    pub function: &'arena Expression<'arena>,
    pub argument_list: PartialArgumentList<'arena>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct MethodPartialApplication<'arena> {
    pub object: &'arena Expression<'arena>,
    pub arrow: Span,
    pub method: ClassLikeMemberSelector<'arena>,
    pub argument_list: PartialArgumentList<'arena>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct StaticMethodPartialApplication<'arena> {
    pub class: &'arena Expression<'arena>,
    pub double_colon: Span,
    pub method: ClassLikeMemberSelector<'arena>,
    pub argument_list: PartialArgumentList<'arena>,
}

impl<'arena> PartialApplication<'arena> {
    #[inline]
    pub fn is_first_class_callable(&self) -> bool {
        self.get_argument_list().is_first_class_callable()
    }

    #[inline]
    pub fn get_argument_list(&self) -> &PartialArgumentList<'arena> {
        match self {
            PartialApplication::Function(f) => &f.argument_list,
            PartialApplication::Method(m) => &m.argument_list,
            PartialApplication::StaticMethod(s) => &s.argument_list,
        }
    }
}

impl HasSpan for PartialApplication<'_> {
    fn span(&self) -> Span {
        match self {
            PartialApplication::Function(f) => f.span(),
            PartialApplication::Method(m) => m.span(),
            PartialApplication::StaticMethod(s) => s.span(),
        }
    }
}

impl HasSpan for FunctionPartialApplication<'_> {
    fn span(&self) -> Span {
        self.function.span().join(self.argument_list.span())
    }
}

impl HasSpan for MethodPartialApplication<'_> {
    fn span(&self) -> Span {
        self.object.span().join(self.argument_list.span())
    }
}

impl HasSpan for StaticMethodPartialApplication<'_> {
    fn span(&self) -> Span {
        self.class.span().join(self.argument_list.span())
    }
}
