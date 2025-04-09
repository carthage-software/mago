use serde::Deserialize;
use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::class_like::member::ClassLikeConstantSelector;
use crate::ast::ast::class_like::member::ClassLikeMemberSelector;
use crate::ast::ast::expression::Expression;
use crate::ast::ast::identifier::Identifier;
use crate::ast::ast::variable::Variable;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct ConstantAccess {
    pub name: Identifier,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum Access {
    Property(PropertyAccess),
    NullSafeProperty(NullSafePropertyAccess),
    StaticProperty(StaticPropertyAccess),
    ClassConstant(ClassConstantAccess),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct PropertyAccess {
    pub object: Box<Expression>,
    pub arrow: Span,
    pub property: ClassLikeMemberSelector,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct NullSafePropertyAccess {
    pub object: Box<Expression>,
    pub question_mark_arrow: Span,
    pub property: ClassLikeMemberSelector,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct StaticPropertyAccess {
    pub class: Box<Expression>,
    pub double_colon: Span,
    pub property: Variable,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct ClassConstantAccess {
    pub class: Box<Expression>,
    pub double_colon: Span,
    pub constant: ClassLikeConstantSelector,
}

impl HasSpan for ConstantAccess {
    fn span(&self) -> Span {
        self.name.span()
    }
}

impl HasSpan for Access {
    fn span(&self) -> Span {
        match self {
            Access::Property(p) => p.span(),
            Access::NullSafeProperty(n) => n.span(),
            Access::StaticProperty(s) => s.span(),
            Access::ClassConstant(c) => c.span(),
        }
    }
}

impl HasSpan for PropertyAccess {
    fn span(&self) -> Span {
        self.object.span().join(self.property.span())
    }
}

impl HasSpan for NullSafePropertyAccess {
    fn span(&self) -> Span {
        self.object.span().join(self.property.span())
    }
}

impl HasSpan for StaticPropertyAccess {
    fn span(&self) -> Span {
        self.class.span().join(self.property.span())
    }
}

impl HasSpan for ClassConstantAccess {
    fn span(&self) -> Span {
        self.class.span().join(self.constant.span())
    }
}
