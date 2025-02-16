use bumpalo::boxed::Box;
use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::class_like::member::ClassLikeConstantSelector;
use crate::ast::class_like::member::ClassLikeMemberSelector;
use crate::ast::expression::Expression;
use crate::ast::identifier::Identifier;
use crate::ast::variable::Variable;

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct ConstantAccess {
    pub name: Identifier,
}

#[derive(Debug, Hash, Serialize, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum Access<'a> {
    Property(PropertyAccess<'a>),
    NullSafeProperty(NullSafePropertyAccess<'a>),
    StaticProperty(StaticPropertyAccess<'a>),
    ClassConstant(ClassConstantAccess<'a>),
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct PropertyAccess<'a> {
    pub object: Box<'a, Expression<'a>>,
    pub arrow: Span,
    pub property: ClassLikeMemberSelector<'a>,
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct NullSafePropertyAccess<'a> {
    pub object: Box<'a, Expression<'a>>,
    pub question_mark_arrow: Span,
    pub property: ClassLikeMemberSelector<'a>,
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct StaticPropertyAccess<'a> {
    pub class: Box<'a, Expression<'a>>,
    pub double_colon: Span,
    pub property: Variable<'a>,
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct ClassConstantAccess<'a> {
    pub class: Box<'a, Expression<'a>>,
    pub double_colon: Span,
    pub constant: ClassLikeConstantSelector<'a>,
}

impl HasSpan for ConstantAccess {
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
