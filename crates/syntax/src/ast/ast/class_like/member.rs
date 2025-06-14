use serde::Deserialize;
use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::Sequence;
use crate::ast::ast::class_like::constant::ClassLikeConstant;
use crate::ast::ast::class_like::enum_case::EnumCase;
use crate::ast::ast::class_like::method::Method;
use crate::ast::ast::class_like::property::Property;
use crate::ast::ast::class_like::trait_use::TraitUse;
use crate::ast::ast::expression::Expression;
use crate::ast::ast::identifier::LocalIdentifier;
use crate::ast::ast::variable::Variable;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum ClassLikeMember {
    TraitUse(TraitUse),
    Constant(ClassLikeConstant),
    Property(Property),
    EnumCase(EnumCase),
    Method(Method),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum ClassLikeMemberSelector {
    Identifier(LocalIdentifier),
    Variable(Variable),
    Expression(ClassLikeMemberExpressionSelector),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum ClassLikeConstantSelector {
    Identifier(LocalIdentifier),
    Expression(ClassLikeMemberExpressionSelector),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct ClassLikeMemberExpressionSelector {
    pub left_brace: Span,
    pub expression: Box<Expression>,
    pub right_brace: Span,
}

impl ClassLikeMember {
    #[inline]
    pub const fn is_trait_use(&self) -> bool {
        matches!(self, ClassLikeMember::TraitUse(_))
    }

    #[inline]
    pub const fn is_constant(&self) -> bool {
        matches!(self, ClassLikeMember::Constant(_))
    }

    #[inline]
    pub const fn is_property(&self) -> bool {
        matches!(self, ClassLikeMember::Property(_))
    }

    #[inline]
    pub const fn is_enum_case(&self) -> bool {
        matches!(self, ClassLikeMember::EnumCase(_))
    }

    #[inline]
    pub const fn is_method(&self) -> bool {
        matches!(self, ClassLikeMember::Method(_))
    }
}

impl Sequence<ClassLikeMember> {
    pub fn contains_trait_uses(&self) -> bool {
        self.iter().any(|member| matches!(member, ClassLikeMember::TraitUse(_)))
    }

    pub fn contains_constants(&self) -> bool {
        self.iter().any(|member| matches!(member, ClassLikeMember::Constant(_)))
    }

    pub fn contains_properties(&self) -> bool {
        self.iter().any(|member| matches!(member, ClassLikeMember::Property(_)))
    }

    pub fn contains_enum_cases(&self) -> bool {
        self.iter().any(|member| matches!(member, ClassLikeMember::EnumCase(_)))
    }

    pub fn contains_methods(&self) -> bool {
        self.iter().any(|member| matches!(member, ClassLikeMember::Method(_)))
    }
}

impl HasSpan for ClassLikeMember {
    fn span(&self) -> Span {
        match self {
            ClassLikeMember::TraitUse(trait_use) => trait_use.span(),
            ClassLikeMember::Constant(constant) => constant.span(),
            ClassLikeMember::Property(property) => property.span(),
            ClassLikeMember::EnumCase(enum_case) => enum_case.span(),
            ClassLikeMember::Method(method) => method.span(),
        }
    }
}

impl HasSpan for ClassLikeMemberSelector {
    fn span(&self) -> Span {
        match self {
            ClassLikeMemberSelector::Identifier(i) => i.span(),
            ClassLikeMemberSelector::Variable(v) => v.span(),
            ClassLikeMemberSelector::Expression(e) => e.span(),
        }
    }
}

impl HasSpan for ClassLikeConstantSelector {
    fn span(&self) -> Span {
        match self {
            ClassLikeConstantSelector::Identifier(i) => i.span(),
            ClassLikeConstantSelector::Expression(e) => e.span(),
        }
    }
}

impl HasSpan for ClassLikeMemberExpressionSelector {
    fn span(&self) -> Span {
        self.left_brace.join(self.right_brace)
    }
}
