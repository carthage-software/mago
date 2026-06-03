use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::attribute::Attribute;
use crate::ir::effect::annotation::AssertAnnotation;
use crate::ir::effect::annotation::SelfOutAnnotation;
use crate::ir::effect::annotation::ThrowsAnnotation;
use crate::ir::expression::Expression;
use crate::ir::flags::Flags;
use crate::ir::generics::annotation::TypeParameterAnnotation;
use crate::ir::generics::annotation::WhereConstraintAnnotation;
use crate::ir::hook::Hook;
use crate::ir::identifier::Identifier;
use crate::ir::inheritance::annotation::UseAnnotation;
use crate::ir::modifier::Modifier;
use crate::ir::name::Name;
use crate::ir::parameter::Parameter;
use crate::ir::statement::Statement;
use crate::ir::r#type::Type;
use crate::ir::r#type::annotation::TypeAnnotation;
use crate::ir::variable::DirectVariable;

pub mod annotation;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct EnumCase<'arena, S, D, E> {
    pub span: Span,
    pub attributes: &'arena [Attribute<'arena, S, D, E>],
    pub name: Name<'arena>,
    pub value: Option<&'arena Expression<'arena, S, D, E>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub enum PropertyFlags {
    Deprecated = 1 << 0,
    Experimental = 1 << 2,
    Final = 1 << 3,
    Internal = 1 << 4,
    API = 1 << 5,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Property<'arena, S, D, E> {
    pub span: Span,
    pub attributes: &'arena [Attribute<'arena, S, D, E>],
    pub flags: Flags<PropertyFlags>,
    pub modifiers: &'arena [Modifier],
    pub r#type: Option<&'arena Type<'arena>>,
    pub type_annotation: Option<&'arena TypeAnnotation<'arena>>,
    pub items: &'arena [PropertyItem<'arena, S, D, E>],
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct PropertyItem<'arena, S, D, E> {
    pub variable: DirectVariable<'arena>,
    pub default_value: Option<&'arena Expression<'arena, S, D, E>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct HookedProperty<'arena, S, D, E> {
    pub span: Span,
    pub attributes: &'arena [Attribute<'arena, S, D, E>],
    pub flags: Flags<PropertyFlags>,
    pub modifiers: &'arena [Modifier],
    pub r#type: Option<&'arena Type<'arena>>,
    pub type_annotation: Option<&'arena TypeAnnotation<'arena>>,
    pub item: PropertyItem<'arena, S, D, E>,
    pub hooks: &'arena [Hook<'arena, S, D, E>],
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub enum MethodFlags {
    Deprecated = 1 << 0,
    Experimental = 1 << 2,
    Final = 1 << 3,
    Internal = 1 << 4,
    API = 1 << 5,
    Pure = 1 << 6,
    ExternalMutationFree = 1 << 7,
    MutationFree = 1 << 8,
    SuspendsFiber = 1 << 9,
    IgnoreNullableReturnType = 1 << 10,
    IgnoreFalsableReturnType = 1 << 11,
    InheritDoc = 1 << 12,
    NoNamedArguments = 1 << 13,
    MustUse = 1 << 14,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Method<'arena, S, D, E> {
    pub span: Span,
    pub attributes: &'arena [Attribute<'arena, S, D, E>],
    pub flags: Flags<MethodFlags>,
    pub modifiers: &'arena [Modifier],
    pub name: Name<'arena>,
    pub type_parameter_annotations: &'arena [TypeParameterAnnotation<'arena>],
    pub parameters: &'arena [Parameter<'arena, S, D, E>],
    pub where_constraint_annotations: &'arena [WhereConstraintAnnotation<'arena>],
    pub return_by_reference: bool,
    pub return_type: Option<&'arena Type<'arena>>,
    pub return_type_annotation: Option<&'arena TypeAnnotation<'arena>>,
    pub throws: &'arena [ThrowsAnnotation<'arena>],
    pub asserts: &'arena [AssertAnnotation<'arena>],
    pub asserts_if_true: &'arena [AssertAnnotation<'arena>],
    pub asserts_if_false: &'arena [AssertAnnotation<'arena>],
    pub self_out_annotation: Option<&'arena SelfOutAnnotation<'arena>>,
    pub body: Option<&'arena Statement<'arena, S, D, E>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ClassLikeConstant<'arena, S, D, E> {
    pub span: Span,
    pub attributes: &'arena [Attribute<'arena, S, D, E>],
    pub modifiers: &'arena [Modifier],
    pub r#type: Option<&'arena Type<'arena>>,
    pub type_annotation: Option<&'arena TypeAnnotation<'arena>>,
    pub items: &'arena [ClassLikeConstantItem<'arena, S, D, E>],
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ClassLikeConstantItem<'arena, S, D, E> {
    pub name: Name<'arena>,
    pub value: &'arena Expression<'arena, S, D, E>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct TraitUse<'arena> {
    pub span: Span,
    pub use_annotation: &'arena [UseAnnotation<'arena>],
    pub traits: &'arena [Identifier<'arena>],
    pub adaptations: &'arena [TraitUseAdaptation<'arena>],
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[serde(tag = "type", content = "value")]
pub enum TraitUseAdaptation<'arena> {
    Precedence(TraitUsePrecedenceAdaptation<'arena>),
    Alias(TraitUseAliasAdaptation<'arena>),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct TraitUsePrecedenceAdaptation<'arena> {
    pub r#trait: Identifier<'arena>,
    pub method: Name<'arena>,
    pub instead_of: &'arena [Identifier<'arena>],
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct TraitUseAliasAdaptation<'arena> {
    pub r#trait: Option<Identifier<'arena>>,
    pub method: Name<'arena>,
    pub visibility: Option<Modifier>,
    pub alias: Name<'arena>,
}

impl From<PropertyFlags> for u32 {
    fn from(flags: PropertyFlags) -> Self {
        flags as u32
    }
}

impl From<MethodFlags> for u32 {
    fn from(flags: MethodFlags) -> Self {
        flags as u32
    }
}

impl<S, D, E> HasSpan for EnumCase<'_, S, D, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<S, D, E> HasSpan for Property<'_, S, D, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<S, D, E> HasSpan for HookedProperty<'_, S, D, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<S, D, E> HasSpan for Method<'_, S, D, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<S, D, E> HasSpan for ClassLikeConstant<'_, S, D, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for TraitUse<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
