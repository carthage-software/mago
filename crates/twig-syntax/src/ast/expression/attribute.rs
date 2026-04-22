use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ArgumentList;
use crate::ast::Identifier;
use crate::ast::expression::Expression;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct GetAttribute<'arena> {
    pub object: &'arena Expression<'arena>,
    pub dot: Span,
    pub null_safe: bool,
    pub attribute: &'arena Expression<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct GetItem<'arena> {
    pub object: &'arena Expression<'arena>,
    pub left_bracket: Span,
    pub index: &'arena Expression<'arena>,
    pub right_bracket: Span,
}

/// `a[start:length]` slice access. `start` and `length` are both optional -
/// `a[:3]`, `a[1:]`, and `a[:]` are all valid Twig.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Slice<'arena> {
    pub object: &'arena Expression<'arena>,
    pub left_bracket: Span,
    pub start: Option<&'arena Expression<'arena>>,
    pub colon: Span,
    pub length: Option<&'arena Expression<'arena>>,
    pub right_bracket: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct MethodCall<'arena> {
    pub object: &'arena Expression<'arena>,
    pub dot: Span,
    pub null_safe: bool,
    pub method: Identifier<'arena>,
    pub argument_list: ArgumentList<'arena>,
}

impl HasSpan for GetAttribute<'_> {
    fn span(&self) -> Span {
        self.object.span().join(self.attribute.span())
    }
}

impl HasSpan for GetItem<'_> {
    fn span(&self) -> Span {
        self.object.span().join(self.right_bracket)
    }
}

impl HasSpan for Slice<'_> {
    fn span(&self) -> Span {
        self.object.span().join(self.right_bracket)
    }
}

impl HasSpan for MethodCall<'_> {
    fn span(&self) -> Span {
        self.object.span().join(self.argument_list.right_parenthesis)
    }
}
