use mago_span::HasSpan;
use mago_span::Span;

use crate::cst::identifier::Identifier;

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ConstantAccessExpression<'arena> {
    pub name: Identifier<'arena>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ClassLikeConstantAccessExpression<'arena> {
    pub class: Identifier<'arena>,
    pub double_colon: Span,
    pub constant: Identifier<'arena>,
}

impl HasSpan for ConstantAccessExpression<'_> {
    fn span(&self) -> Span {
        self.name.span()
    }
}

impl HasSpan for ClassLikeConstantAccessExpression<'_> {
    fn span(&self) -> Span {
        self.class.span().join(self.constant.span())
    }
}
