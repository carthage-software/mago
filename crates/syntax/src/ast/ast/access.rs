use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::class_like::member::ClassLikeConstantSelector;
use crate::ast::ast::class_like::member::ClassLikeMemberSelector;
use crate::ast::ast::expression::Expression;
use crate::ast::ast::identifier::Identifier;
use crate::ast::ast::variable::Variable;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ConstantAccess<'arena> {
    pub name: Identifier<'arena>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum Access<'arena> {
    Property(PropertyAccess<'arena>),
    NullSafeProperty(NullSafePropertyAccess<'arena>),
    StaticProperty(StaticPropertyAccess<'arena>),
    ClassConstant(ClassConstantAccess<'arena>),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct PropertyAccess<'arena> {
    pub object: &'arena Expression<'arena>,
    pub arrow: Span,
    pub property: ClassLikeMemberSelector<'arena>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct NullSafePropertyAccess<'arena> {
    pub object: &'arena Expression<'arena>,
    pub question_mark_arrow: Span,
    pub property: ClassLikeMemberSelector<'arena>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct StaticPropertyAccess<'arena> {
    pub class: &'arena Expression<'arena>,
    pub double_colon: Span,
    pub property: Variable<'arena>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ClassConstantAccess<'arena> {
    pub class: &'arena Expression<'arena>,
    pub double_colon: Span,
    pub constant: ClassLikeConstantSelector<'arena>,
}

impl HasSpan for ConstantAccess<'_> {
    fn span(&self) -> Span {
        self.name.span()
    }
}

impl HasSpan for Access<'_> {
    fn span(&self) -> Span {
        match self {
            Access::Property(p) => p.span(),
            Access::NullSafeProperty(n) => n.span(),
            Access::StaticProperty(s) => s.span(),
            Access::ClassConstant(c) => c.span(),
        }
    }
}

impl HasSpan for PropertyAccess<'_> {
    fn span(&self) -> Span {
        self.object.span().join(self.property.span())
    }
}

impl HasSpan for NullSafePropertyAccess<'_> {
    fn span(&self) -> Span {
        self.object.span().join(self.property.span())
    }
}

impl HasSpan for StaticPropertyAccess<'_> {
    fn span(&self) -> Span {
        self.class.span().join(self.property.span())
    }
}

impl HasSpan for ClassConstantAccess<'_> {
    fn span(&self) -> Span {
        self.class.span().join(self.constant.span())
    }
}
