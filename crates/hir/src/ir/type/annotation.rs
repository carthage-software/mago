#![allow(clippy::ref_option_ref)]

use ordered_float::OrderedFloat;
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::generics::TypeParameterDefiningEntity;
use crate::ir::identifier::Identifier;
use crate::ir::name::Name;
use crate::ir::variable::DirectVariable;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct TypeAnnotation<'arena> {
    pub span: Span,
    pub kind: TypeAnnotationKind<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[serde(tag = "kind", content = "value")]
pub enum TypeAnnotationKind<'arena> {
    Named(NamedTypeAnnotation<'arena>),
    GenericParameter(GenericParameterAnnotation<'arena>),
    Union(&'arena [TypeAnnotationKind<'arena>]),
    Intersection(&'arena [TypeAnnotationKind<'arena>]),
    Array(bool, &'arena TypeAnnotationKind<'arena>, &'arena TypeAnnotationKind<'arena>),
    List(bool, &'arena TypeAnnotationKind<'arena>),
    Iterable(&'arena TypeAnnotationKind<'arena>, &'arena TypeAnnotationKind<'arena>),
    ClassLikeString(&'arena TypeAnnotationKind<'arena>),
    ClassString(&'arena TypeAnnotationKind<'arena>),
    InterfaceString(&'arena TypeAnnotationKind<'arena>),
    EnumString(&'arena TypeAnnotationKind<'arena>),
    TraitString(&'arena TypeAnnotationKind<'arena>),
    Mixed(bool),
    Null,
    Void,
    Never,
    Resource(Option<bool>),
    Bool(Option<bool>),
    Float(Option<FloatLiteral>),
    Int(Option<IntLiteral>),
    String(StringTypeAnnotation<'arena>),
    StringableObject,
    Object,
    ObjectShape(ObjectShapeTypeAnnotation<'arena>),
    Numeric,
    MemberReference(Identifier<'arena>, MemberReferenceSelector<'arena>),
    AliasReference(Identifier<'arena>, Name<'arena>),
    Shape(ShapeTypeAnnotation<'arena>),
    Callable(CallableTypeAnnotation<'arena>),
    Variable(DirectVariable<'arena>),
    Conditional(ConditionalTypeAnnotation<'arena>),
    KeyOf(&'arena TypeAnnotationKind<'arena>),
    ValueOf(&'arena TypeAnnotationKind<'arena>),
    IntMask(&'arena [TypeAnnotationKind<'arena>]),
    IntMaskOf(&'arena TypeAnnotationKind<'arena>),
    New(&'arena TypeAnnotationKind<'arena>),
    TemplateType(&'arena [TypeAnnotationKind<'arena>]),
    IndexAccess(&'arena TypeAnnotationKind<'arena>, &'arena TypeAnnotationKind<'arena>),
    Negated(&'arena TypeAnnotationKind<'arena>),
    Posited(&'arena TypeAnnotationKind<'arena>),
    IntRange(Option<i64>, Option<i64>),
    PropertiesOf(PropertiesOfFilter, &'arena TypeAnnotationKind<'arena>),
    Slice(&'arena TypeAnnotationKind<'arena>),
    Wildcard,
    GlobalSelector(GlobalSelector<'arena>),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct NamedTypeAnnotation<'arena> {
    pub name: Identifier<'arena>,
    pub type_arguments: &'arena [TypeAnnotationKind<'arena>],
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct GenericParameterAnnotation<'arena> {
    pub name: Name<'arena>,
    pub defining_entity: TypeParameterDefiningEntity<'arena>,
    pub bound: Option<&'arena TypeAnnotation<'arena>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[serde(tag = "type", content = "value")]
pub enum MemberReferenceSelector<'arena> {
    Wildcard,
    Exact(Name<'arena>),
    StartsWith(Name<'arena>),
    EndsWith(Name<'arena>),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[serde(tag = "type", content = "value")]
pub enum GlobalSelector<'arena> {
    StartsWith(Identifier<'arena>),
    EndsWith(Identifier<'arena>),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[serde(tag = "type", content = "value")]
pub enum IntLiteral {
    Specific(i64),
    Unspecified,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[serde(tag = "type", content = "value")]
pub enum FloatLiteral {
    Specific(OrderedFloat<f64>),
    Unspecified,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[serde(tag = "type", content = "value")]
pub enum StringCasing {
    Uppercase,
    Lowercase,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[serde(tag = "type", content = "value")]
pub enum StringLiteral<'arena> {
    Specific(&'arena [u8]),
    Unspecified,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct StringTypeAnnotation<'arena> {
    pub casing: Option<StringCasing>,
    pub literal: Option<StringLiteral<'arena>>,
    pub non_empty: bool,
    pub numeric: bool,
    pub truthy: bool,
    pub callable: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[serde(tag = "type", content = "value")]
pub enum PropertiesOfFilter {
    All,
    Public,
    Protected,
    Private,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[serde(tag = "type", content = "value")]
pub enum CallableTypeKind {
    Callable,
    PureCallable,
    Closure,
    PureClosure,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct CallableParameter<'arena> {
    pub r#type: Option<&'arena TypeAnnotation<'arena>>,
    pub variadic: bool,
    pub by_reference: bool,
    pub variable: Option<DirectVariable<'arena>>,
    pub has_default: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct CallableTypeAnnotation<'arena> {
    pub kind: CallableTypeKind,
    pub parameters: &'arena [CallableParameter<'arena>],
    pub r#return: Option<&'arena TypeAnnotationKind<'arena>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ConditionalTypeAnnotation<'arena> {
    pub target: &'arena TypeAnnotationKind<'arena>,
    pub subject: &'arena TypeAnnotationKind<'arena>,
    pub is_negated: bool,
    pub then: &'arena TypeAnnotationKind<'arena>,
    pub r#else: &'arena TypeAnnotationKind<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[serde(tag = "kind", content = "value")]
pub enum ShapeTypeAnnotationKey<'arena> {
    String(&'arena [u8]),
    Integer(i64),
    ClassLikeConstant(Identifier<'arena>, Name<'arena>),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ShapeTypeAnnotationField<'arena> {
    pub key: ShapeTypeAnnotationKey<'arena>,
    pub optional: bool,
    pub value: TypeAnnotationKind<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ShapeTypeAnnotation<'arena> {
    pub fields: &'arena [ShapeTypeAnnotationField<'arena>],
    pub additional_fields: Option<ShapeTypeAnnotationAdditionalFields<'arena>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ShapeTypeAnnotationAdditionalFields<'arena> {
    pub key_type: &'arena TypeAnnotationKind<'arena>,
    pub value_type: &'arena TypeAnnotationKind<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ObjectShapeTypeAnnotation<'arena> {
    pub fields: &'arena [ShapeTypeAnnotationField<'arena>],
    pub sealed: bool,
}

impl HasSpan for TypeAnnotation<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
